//! Runs the shared vectors in `fixtures/consume/received-payload.json` at the
//! repository root, which the TypeScript, Go and Dart runtimes also run: the
//! typed `consume_inbound::<P>`, with the received document in hand
//! (`ProofPolicy::with_received`), validates the payload **as received** —
//! where decoding into `P` reads a forbidden `null` as absent — and a later
//! version read forward along `extends` pruned, as `consume_dispatched` does
//! (SPEC §7.2 item 2).

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_rs::extension::{declare_extends, read_as};
use trust_tasks_rs::handlers::NoopHandler;
use trust_tasks_rs::{
    consume_inbound, ConsumeChecks, ConsumeOutcome, ErrorResponse, Payload, PayloadPolicy,
    PayloadValidator, ProofPolicy, ProofVerifier, TrustTask, VerificationError,
};

/// The implemented version, as a generated type reads it: an optional `role`
/// that is `None` whether absent or `null`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Grant {
    subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

impl Payload for Grant {
    const TYPE_URI: &'static str = "https://example.com/spec/vector/grant/0.1";
    const PAYLOAD_SCHEMA: Option<&'static str> = Some(
        r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"subject":{"type":"string"},"role":{"type":"string"}},"required":["subject"],"additionalProperties":false}"#,
    );
}

/// The documents carry no proof; this only names a verifier type.
struct NoVerifier;

#[async_trait::async_trait]
impl ProofVerifier for NoVerifier {
    async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: Serialize + Send + Sync,
    {
        Ok(())
    }
}

/// Records the payload it is handed and checks the vector schema by hand: an
/// object, `subject` present, every member named in `properties`, every
/// member a string. (The `validate` feature's engine is compared with it
/// below.)
#[derive(Default)]
struct Recording(Mutex<Option<Value>>);

impl PayloadValidator for Recording {
    fn validate(&self, _schema_json: &str, payload: &Value) -> Result<(), String> {
        *self.0.lock().unwrap() = Some(payload.clone());
        let Value::Object(members) = payload else {
            return Err("not an object".into());
        };
        if !members.contains_key("subject") {
            return Err("missing subject".into());
        }
        for (name, value) in members {
            if name != "subject" && name != "role" {
                return Err(format!("unknown member {name}"));
            }
            if !value.is_string() {
                return Err(format!("{name} is not a string"));
            }
        }
        Ok(())
    }
}

fn vectors() -> Value {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/consume/received-payload.json");
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[tokio::test]
async fn shared_received_payload_vectors() {
    let vectors = vectors();
    assert_eq!(
        vectors["implements"]["typeUri"].as_str().unwrap(),
        Grant::TYPE_URI
    );
    assert_eq!(
        vectors["implements"]["payloadSchema"].as_str().unwrap(),
        Grant::PAYLOAD_SCHEMA.unwrap()
    );
    let table: Vec<(&'static str, &'static str)> = vectors["extends"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(uri, target)| -> (&'static str, &'static str) {
            (
                Box::leak(uri.clone().into_boxed_str()),
                Box::leak(target.as_str().unwrap().to_string().into_boxed_str()),
            )
        })
        .collect();
    declare_extends(Box::leak(table.into_boxed_slice()));
    let consumer = vectors["consumer"].as_str().unwrap();
    let now = vectors["now"].as_str().unwrap().parse().unwrap();

    let cases = vectors["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let received: Value = serde_json::from_str(case["document"].as_str().unwrap()).unwrap();
        let untyped: TrustTask<Value> = serde_json::from_value(received.clone()).unwrap();
        let doc = read_as::<Grant>(untyped).unwrap_or_else(|e| panic!("{name}: read_as: {e:?}"));
        let validator = Recording::default();
        let outcome = consume_inbound(
            &NoopHandler::new(),
            ProofPolicy::<NoVerifier>::AcceptUnverified.with_received(&received),
            PayloadPolicy::Validate(&validator),
            ConsumeChecks::not_consequential(),
            doc,
            consumer,
            now,
            || "urn:uuid:error".to_string(),
            |req, _| async move {
                Ok::<_, ErrorResponse>(req.respond_with("urn:uuid:answer", json!({})))
            },
        )
        .await;
        let verdict = match &outcome {
            ConsumeOutcome::Handled(_) => "accept".to_string(),
            ConsumeOutcome::Rejected(err) => err.payload.code.to_string(),
            other => format!("{other:?}"),
        };
        assert_eq!(
            verdict,
            case["verdict"].as_str().unwrap(),
            "{name}: verdict"
        );
        let validated = validator.0.lock().unwrap().clone();
        assert_eq!(
            validated.as_ref(),
            Some(&case["validated"]),
            "{name}: validated"
        );

        // The bundled engine reaches the same verdict on the same payload.
        #[cfg(feature = "validate")]
        {
            let engine = trust_tasks_rs::validate::SchemaValidator::new();
            let ok = engine
                .validate(Grant::PAYLOAD_SCHEMA.unwrap(), &case["validated"])
                .is_ok();
            assert_eq!(ok, case["verdict"] == "accept", "{name}: engine verdict");
        }
    }
}
