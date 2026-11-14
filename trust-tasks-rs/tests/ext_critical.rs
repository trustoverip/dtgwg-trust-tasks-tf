//! SPEC §4.5.2 — a namespace marked critical is understood or refused.
//!
//! Runs the shared vectors in `fixtures/consume/ext-critical.json` at the
//! repository root, which the TypeScript, Go and Dart runtimes run too, and
//! drives the marking through `consume_inbound`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::Utc;
use serde_json::Value;
use trust_tasks_rs::handlers::NoopHandler;
use trust_tasks_rs::{
    check_critical_extensions, consume_inbound, ConsumeChecks, ConsumeOutcome, ErrorResponse,
    InMemoryReplayGuard, NoValidator, Payload, PayloadPolicy, ProofPolicy, ProofVerifier,
    StandardCode, TrustTask, VerificationError,
};

const SCHEMA: &str = include_str!("../../fixtures/consume/ext-critical.schema.json");
const MARKED: &str = "https://trusttasks.org/spec/conformance/marked/0.1";

/// Models only `subject`; serde drops anything else, as a consumer's
/// generated type drops what it does not declare.
#[derive(serde::Serialize, serde::Deserialize)]
struct Marked {
    subject: String,
}

impl Payload for Marked {
    const TYPE_URI: &'static str = MARKED;
    const PAYLOAD_SCHEMA: Option<&'static str> = Some(SCHEMA);
}

struct NoVerifier;

#[async_trait::async_trait]
impl ProofVerifier for NoVerifier {
    async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: serde::Serialize + Send + Sync,
    {
        Ok(())
    }
}

fn vectors() -> Vec<Value> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/consume/ext-critical.json");
    let raw: Value = serde_json::from_slice(&std::fs::read(path).expect("fixture")).unwrap();
    raw["cases"].as_array().expect("cases").clone()
}

#[test]
fn shared_vectors() {
    for case in vectors() {
        let name = case["name"].as_str().unwrap();
        let recognized: Vec<&str> = case["recognized"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let schema = if case["schemaless"] == Value::Bool(true) {
            None
        } else {
            Some(SCHEMA)
        };
        let got = match check_critical_extensions(&case["payload"], schema, &recognized) {
            Ok(()) => "ok".to_string(),
            Err(reason) => reason.code().as_str().to_string(),
        };
        assert_eq!(got, case["expect"].as_str().unwrap(), "{name}");
    }
}

fn marked_document(id: &str) -> Value {
    serde_json::json!({
        "id": id,
        "type": MARKED,
        "issuedAt": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "payload": {
            "subject": "s",
            "ext": { "org.example.a": 1 },
            "extCritical": ["org.example.a"]
        }
    })
}

async fn consume(
    guard: &InMemoryReplayGuard,
    received: &Value,
    recognized: &[&str],
    runs: &AtomicUsize,
) -> ConsumeOutcome<Value> {
    let doc: TrustTask<Marked> = serde_json::from_value(received.clone()).unwrap();
    consume_inbound(
        &NoopHandler::new(),
        ProofPolicy::<NoVerifier>::AcceptUnverified.with_received(received),
        PayloadPolicy::<NoValidator>::AcceptUnvalidated,
        ConsumeChecks::consequential(guard).with_recognized_extensions(recognized),
        doc,
        "did:web:consumer.example",
        Utc::now(),
        || "err-1".to_string(),
        |req, _parties| async move {
            runs.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ErrorResponse>(req.respond_with("resp-1", serde_json::json!({ "ok": true })))
        },
    )
    .await
}

#[tokio::test]
async fn an_unrecognized_critical_namespace_is_refused_before_the_handler_and_the_record() {
    let guard = InMemoryReplayGuard::default();
    let runs = AtomicUsize::new(0);
    let received = marked_document("urn:uuid:00000000-0000-4000-8000-000000000001");

    // The payload type drops `ext` and `extCritical`; the pipeline reads the
    // marking from the document as received.
    match consume(&guard, &received, &[], &runs).await {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::UnsupportedExtension.into());
            assert!(!err.payload.retryable);
        }
        other => panic!("expected unsupportedExtension, got {other:?}"),
    }
    assert_eq!(runs.load(Ordering::SeqCst), 0, "the handler must not run");

    // The refusal wrote no record: the same document, consumed by a consumer
    // that implements the namespace, runs.
    match consume(&guard, &received, &["org.example.a"], &runs).await {
        ConsumeOutcome::Handled(_) => {}
        other => panic!("expected Handled, got {other:?}"),
    }
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}
