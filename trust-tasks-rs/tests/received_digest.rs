//! SPEC.md §7.2 item 11 compares the document **as received**.
//!
//! Runs the shared vectors in `fixtures/consume/item11-digest.json` at the
//! repository root, which the TypeScript, Go and Dart runtimes run too. The
//! payload type models only `payload.subject`, so a typed re-serialisation
//! cannot see the members the vectors differ in; the digest must be over the
//! received JSON for those to be an `idConflict`.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde_json::Value;
use trust_tasks_rs::handlers::NoopHandler;
use trust_tasks_rs::{
    consume_inbound, ConsumeChecks, ConsumeOutcome, ErrorResponse, InMemoryReplayGuard,
    NoValidator, Payload, PayloadPolicy, ProofPolicy, ProofVerifier, StandardCode, TrustTask,
    VerificationError,
};

const WIDGET: &str = "https://trusttasks.org/spec/conformance/widget/0.1";

/// Models only `subject`; serde drops anything else, as a consumer's
/// generated type drops what its schema version does not declare.
#[derive(serde::Serialize, serde::Deserialize)]
struct Widget {
    subject: String,
}

impl Payload for Widget {
    const TYPE_URI: &'static str = WIDGET;
}

/// The vectors carry no proof; this only gives `VerifyReceived` a verifier.
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

struct Vectors {
    consumer: String,
    now: DateTime<Utc>,
    /// `(name, first, second, expect)`.
    cases: Vec<(String, String, String, String)>,
}

fn vectors() -> Vectors {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/consume/item11-digest.json");
    let raw: Value = serde_json::from_slice(&std::fs::read(path).expect("fixture")).unwrap();
    assert_eq!(raw["spec"], WIDGET);
    let s = |v: &Value| v.as_str().expect("a string").to_string();
    Vectors {
        consumer: s(&raw["consumer"]),
        now: s(&raw["now"]).parse().unwrap(),
        cases: raw["cases"]
            .as_array()
            .expect("cases")
            .iter()
            .map(|c| {
                let first = if c["first"].is_string() {
                    s(&c["first"])
                } else {
                    s(&raw["first"])
                };
                (s(&c["name"]), first, s(&c["second"]), s(&c["expect"]))
            })
            .collect(),
    }
}

async fn consume(
    v: &Vectors,
    guard: &InMemoryReplayGuard,
    body: &str,
    with_received: bool,
) -> ConsumeOutcome<Value> {
    let received: Value = serde_json::from_str(body).unwrap();
    let doc: TrustTask<Widget> = serde_json::from_value(received.clone()).unwrap();
    let policy = if with_received {
        ProofPolicy::VerifyReceived(&NoVerifier, &received)
    } else {
        ProofPolicy::Verify(&NoVerifier)
    };
    consume_inbound(
        &NoopHandler::new(),
        policy,
        PayloadPolicy::<NoValidator>::AcceptUnvalidated,
        ConsumeChecks::consequential(guard),
        doc,
        &v.consumer,
        v.now,
        || "err-1".to_string(),
        |req, _parties| async move {
            Ok::<_, ErrorResponse>(req.respond_with("resp-1", serde_json::json!({ "ok": true })))
        },
    )
    .await
}

fn verdict(outcome: &ConsumeOutcome<Value>) -> String {
    match outcome {
        ConsumeOutcome::Duplicate { .. } => "duplicate".into(),
        ConsumeOutcome::Rejected(err) if err.payload.code == StandardCode::IdConflict.into() => {
            "idConflict".into()
        }
        other => format!("{other:?}"),
    }
}

#[tokio::test]
async fn shared_item11_vectors_compare_the_document_as_received() {
    let v = vectors();
    assert!(!v.cases.is_empty());
    for (name, first_body, second, expect) in &v.cases {
        let guard = InMemoryReplayGuard::new(16);
        let first = consume(&v, &guard, first_body, true).await;
        assert!(
            matches!(first, ConsumeOutcome::Handled(_)),
            "{name}: first {first:?}"
        );
        let got = verdict(&consume(&v, &guard, second, true).await);
        assert_eq!(&got, expect, "{name}");
    }
}

/// Without the received JSON the digest is of the typed serialisation, which
/// cannot see a member `P` does not model. Pinned so the documented limit is
/// the actual one.
#[tokio::test]
async fn without_the_received_json_the_digest_is_of_the_typed_view() {
    let v = vectors();
    let (_, first, second, _) = v
        .cases
        .iter()
        .find(|(name, _, _, _)| name == "extra payload member the consumer's type does not model")
        .expect("vector");
    let guard = InMemoryReplayGuard::new(16);
    assert!(matches!(
        consume(&v, &guard, first, false).await,
        ConsumeOutcome::Handled(_)
    ));
    assert_eq!(
        verdict(&consume(&v, &guard, second, false).await),
        "duplicate"
    );
}
