//! A handler may keep its response out of the SPEC §7.2 item 11 record
//! ([`Delivery::withhold_response`]); a duplicate of that document then comes
//! back to it as a re-delivery ([`Delivery::is_redelivery`]) instead of being
//! answered from the record.
//!
//! Runs the shared vectors in `fixtures/consume/withheld-response.json`, which
//! the TypeScript, Go and Dart runtimes run too.

use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use trust_tasks_rs::handlers::NoopHandler;
use trust_tasks_rs::{
    consume_dispatched, consume_inbound, AsyncDispatcher, ConsumeChecks, ConsumeOutcome, Delivery,
    ErrorResponse, InMemoryReplayGuard, NoValidator, Payload, PayloadPolicy, ProofPolicy,
    ProofVerifier, ResolvedParties, StandardCode, TrustTask, VerificationError,
};

const WIDGET: &str = "https://trusttasks.org/spec/conformance/widget/0.1";

#[derive(serde::Serialize, serde::Deserialize)]
struct Widget {
    subject: String,
}

impl Payload for Widget {
    const TYPE_URI: &'static str = WIDGET;
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

fn fixture() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/consume/withheld-response.json");
    let raw: Value = serde_json::from_slice(&std::fs::read(path).expect("fixture")).unwrap();
    assert_eq!(raw["spec"], WIDGET);
    raw
}

/// What the handler saw: `Some("first" | "redelivery")`, or `None` if it was
/// not called.
type Seen = Mutex<Option<String>>;

/// The handler's answer, the same on both paths.
fn answer<P>(request: TrustTask<P>, index: usize, withhold: bool, seen: &Seen) -> TrustTask<Value> {
    let delivery = Delivery::current().expect("the pipeline sets the delivery");
    let label = if delivery.is_redelivery() {
        "redelivery"
    } else {
        "first"
    };
    *seen.lock().unwrap() = Some(label.to_string());
    if withhold {
        delivery.withhold_response();
    }
    request.respond_with(format!("resp-{index}"), json!({ "delivery": label }))
}

fn verdict(outcome: &ConsumeOutcome<Value>) -> String {
    match outcome {
        ConsumeOutcome::Handled(_) => "handled".into(),
        ConsumeOutcome::Duplicate { .. } => "duplicate".into(),
        ConsumeOutcome::Rejected(err) if err.payload.code == StandardCode::IdConflict.into() => {
            "idConflict".into()
        }
        other => format!("{other:?}"),
    }
}

/// Check one delivery's outcome against the vector; `responses` collects each
/// handled response's `id` by delivery index.
fn check(
    name: &str,
    index: usize,
    delivery: &Value,
    outcome: &ConsumeOutcome<Value>,
    seen: Option<String>,
    responses: &mut Vec<Option<String>>,
) {
    assert_eq!(
        verdict(outcome),
        delivery["outcome"].as_str().unwrap(),
        "{name} #{index}"
    );
    assert_eq!(
        seen.as_deref(),
        delivery["handler"].as_str(),
        "{name} #{index}: handler"
    );
    match outcome {
        ConsumeOutcome::Handled(response) => {
            assert_eq!(response.payload["delivery"], delivery["handler"]);
            responses.push(Some(response.id.clone()));
        }
        ConsumeOutcome::Duplicate {
            prior_response,
            in_flight,
        } => {
            assert!(!in_flight);
            let of = delivery["priorResponseOf"]
                .as_u64()
                .expect("priorResponseOf") as usize;
            let prior = prior_response.as_ref().expect("a replayed response");
            assert_eq!(
                prior["id"].as_str(),
                responses[of].as_deref(),
                "{name} #{index}: replays"
            );
            responses.push(None);
        }
        _ => responses.push(None),
    }
}

fn consumer(raw: &Value) -> (String, DateTime<Utc>) {
    (
        raw["consumer"].as_str().unwrap().to_string(),
        raw["now"].as_str().unwrap().parse().unwrap(),
    )
}

#[tokio::test]
async fn shared_withheld_response_vectors_typed() {
    let raw = fixture();
    let (me, now) = consumer(&raw);
    for case in raw["cases"].as_array().expect("cases") {
        let name = case["name"].as_str().unwrap();
        let withhold = case["withhold"].as_bool().unwrap();
        let guard = InMemoryReplayGuard::new(16);
        let mut responses = Vec::new();
        for (index, delivery) in case["deliveries"].as_array().unwrap().iter().enumerate() {
            let body = raw["documents"][delivery["document"].as_str().unwrap()]
                .as_str()
                .unwrap();
            let received: Value = serde_json::from_str(body).unwrap();
            let doc: TrustTask<Widget> = serde_json::from_value(received.clone()).unwrap();
            let seen = Seen::default();
            let outcome = consume_inbound(
                &NoopHandler::new(),
                ProofPolicy::VerifyReceived(&NoVerifier, &received),
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                ConsumeChecks::consequential(&guard),
                doc,
                &me,
                now,
                || "err-1".to_string(),
                |request, _parties| {
                    let response = answer(request, index, withhold, &seen);
                    async move { Ok::<_, ErrorResponse>(response) }
                },
            )
            .await;
            let seen = seen.into_inner().unwrap();
            check(name, index, delivery, &outcome, seen, &mut responses);
        }
    }
}

/// The same vectors through the dispatched path, with the delivery read
/// inside the handler's future rather than when it is built.
#[tokio::test]
async fn shared_withheld_response_vectors_dispatched() {
    let raw = fixture();
    let (me, now) = consumer(&raw);
    for case in raw["cases"].as_array().expect("cases") {
        let name = case["name"].as_str().unwrap();
        let withhold = case["withhold"].as_bool().unwrap();
        let guard = InMemoryReplayGuard::new(16);
        let mut responses = Vec::new();
        for (index, delivery) in case["deliveries"].as_array().unwrap().iter().enumerate() {
            let body = raw["documents"][delivery["document"].as_str().unwrap()]
                .as_str()
                .unwrap();
            let received: Value = serde_json::from_str(body).unwrap();
            let doc: TrustTask<Value> = serde_json::from_value(received.clone()).unwrap();
            let seen = std::sync::Arc::new(Seen::default());
            let handler_seen = seen.clone();
            let dispatcher =
                AsyncDispatcher::<ResolvedParties, Result<TrustTask<Value>, ErrorResponse>>::new()
                    .on_async::<Widget, _, _>(move |request, _parties| {
                    let seen = handler_seen.clone();
                    async move {
                        tokio::task::yield_now().await;
                        Ok(answer(request, index, withhold, &seen))
                    }
                });
            let outcome = consume_dispatched(
                &NoopHandler::new(),
                ProofPolicy::VerifyReceived(&NoVerifier, &received),
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                ConsumeChecks::consequential(&guard),
                &dispatcher,
                doc,
                &me,
                now,
                || "err-1".to_string(),
                |parties| parties,
            )
            .await;
            let seen = seen.lock().unwrap().clone();
            check(name, index, delivery, &outcome, seen, &mut responses);
        }
    }
}

#[test]
fn outside_a_pipeline_there_is_no_delivery() {
    assert!(Delivery::current().is_none());
}
