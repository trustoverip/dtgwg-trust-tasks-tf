//! `GuardedConsumer::consume_body` and `consume_body_dispatched` — the inbound
//! half of the DIDComm, DIDComm v1 and TSP bindings — hold a body to its
//! specification's declared `maxDocumentBytes` (SPEC §12.4): a larger one is
//! refused `malformedRequest`, to the transport-authenticated sender, before
//! the item-11 record is written and without running the handler; one at the
//! bound is accepted; and a later version read forward along `extends` is not
//! held to the implemented version's bound.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_rs::extension::declare_extends;
use trust_tasks_rs::handlers::InMemoryHandler;
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorResponse, GuardedConsumer, InMemoryReplayGuard,
    NoValidator, Payload, PayloadPolicy, ProofPolicy, ProofVerifier, ResolvedParties, StandardCode,
    TrustTask, TrustTaskCode, VerificationError,
};

const BOUND: usize = 600;
const ME: &str = "did:example:bob";
const PEER: &str = "did:example:alice";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct Bounded(Value);

impl Payload for Bounded {
    const TYPE_URI: &'static str = "https://example.com/spec/bounded/grant/0.1";
    const MAX_DOCUMENT_BYTES: Option<usize> = Some(BOUND);
}

/// A later version, declaring it extends `0.1`.
const LATER: &str = "https://example.com/spec/bounded/grant/0.2";

fn declare_later() {
    static DECLARED: &[(&str, &str)] = &[(LATER, "0.1")];
    declare_extends(DECLARED);
}

/// Pins `ProofPolicy`'s verifier type where no proof is verified.
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

/// A body of `type_uri` whose compact JSON text is exactly `size` bytes.
fn body_of_size(id: &str, type_uri: &str, size: usize) -> Value {
    let build = |note: String| {
        json!({
            "id": id,
            "type": type_uri,
            "issuer": PEER,
            "recipient": ME,
            "issuedAt": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "payload": { "subject": "did:example:carol", "note": note },
        })
    };
    let empty = serde_json::to_vec(&build(String::new())).unwrap().len();
    assert!(empty <= size, "the bare body is already {empty} bytes");
    let body = build("x".repeat(size - empty));
    assert_eq!(serde_json::to_vec(&body).unwrap().len(), size);
    body
}

fn transport() -> InMemoryHandler {
    InMemoryHandler::new().with_local(ME).with_peer(PEER)
}

async fn typed(
    consumer: &GuardedConsumer,
    body: Value,
    runs: &Arc<AtomicUsize>,
) -> ConsumeOutcome<Value> {
    let runs = runs.clone();
    consumer
        .consume_body::<Bounded, Value, _, _, _, _, _>(
            &transport(),
            body,
            |_| Ok(()),
            ME,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            Utc::now(),
            || "urn:uuid:error".to_string(),
            move |doc, _parties| async move {
                runs.fetch_add(1, Ordering::SeqCst);
                Ok(doc.respond_with("urn:uuid:answer", json!({})))
            },
        )
        .await
        .expect("answerable")
}

type Answer = Result<TrustTask<Value>, ErrorResponse>;

async fn dispatched(
    consumer: &GuardedConsumer,
    body: Value,
    runs: &Arc<AtomicUsize>,
) -> ConsumeOutcome<Value> {
    let runs = runs.clone();
    let dispatcher: AsyncDispatcher<ResolvedParties, Answer> = AsyncDispatcher::new()
        .on_async::<Bounded, _, _>(move |doc, _parties| {
            let runs = runs.clone();
            async move {
                runs.fetch_add(1, Ordering::SeqCst);
                Ok(doc.respond_with("urn:uuid:answer", json!({})))
            }
        });
    consumer
        .consume_body_dispatched(
            &transport(),
            body,
            |_| Ok(()),
            ME,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            &dispatcher,
            Utc::now(),
            || "urn:uuid:error".to_string(),
            |parties| parties,
        )
        .await
        .expect("answerable")
}

fn assert_too_large(outcome: ConsumeOutcome<Value>) {
    match outcome {
        ConsumeOutcome::Rejected(error) => {
            assert_eq!(
                error.payload.code,
                TrustTaskCode::Standard(StandardCode::MalformedRequest)
            );
            // Addressed to the transport-authenticated sender.
            assert_eq!(error.recipient.as_deref(), Some(PEER));
            assert_eq!(error.issuer.as_deref(), Some(ME));
        }
        other => panic!("expected a size refusal, got {other:?}"),
    }
}

#[tokio::test]
async fn a_body_over_its_declared_bound_is_refused_before_the_record() {
    for path in ["typed", "dispatched"] {
        let guard = Arc::new(InMemoryReplayGuard::default());
        let consumer = GuardedConsumer::with_replay_guard(guard.clone());
        let runs = Arc::new(AtomicUsize::new(0));
        let body = body_of_size("urn:uuid:over", Bounded::TYPE_URI, BOUND + 1);
        let outcome = match path {
            "typed" => typed(&consumer, body, &runs).await,
            _ => dispatched(&consumer, body, &runs).await,
        };
        assert_too_large(outcome);
        assert_eq!(runs.load(Ordering::SeqCst), 0, "{path}: handler ran");
        assert_eq!(guard.len(), 0, "{path}: a refused document burned its id");
    }
}

#[tokio::test]
async fn a_body_at_its_declared_bound_is_accepted() {
    for path in ["typed", "dispatched"] {
        let consumer = GuardedConsumer::new();
        let runs = Arc::new(AtomicUsize::new(0));
        let body = body_of_size("urn:uuid:at", Bounded::TYPE_URI, BOUND);
        let outcome = match path {
            "typed" => typed(&consumer, body, &runs).await,
            _ => dispatched(&consumer, body, &runs).await,
        };
        assert!(
            matches!(outcome, ConsumeOutcome::Handled(_)),
            "{path}: {outcome:?}"
        );
        assert_eq!(runs.load(Ordering::SeqCst), 1);
    }
}

/// `0.2` may declare a larger bound than `0.1`; a consumer of `0.1` reading a
/// `0.2` document forward must not refuse it on `0.1`'s.
#[tokio::test]
async fn a_forward_read_document_is_not_held_to_the_older_bound() {
    declare_later();
    for path in ["typed", "dispatched"] {
        let consumer = GuardedConsumer::new();
        let runs = Arc::new(AtomicUsize::new(0));
        let body = body_of_size("urn:uuid:forward", LATER, BOUND * 2);
        let outcome = match path {
            "typed" => typed(&consumer, body, &runs).await,
            _ => dispatched(&consumer, body, &runs).await,
        };
        match outcome {
            ConsumeOutcome::Handled(answer) => {
                assert_eq!(answer.type_uri.to_string(), format!("{LATER}#response"));
            }
            other => panic!("{path}: expected Handled, got {other:?}"),
        }
        assert_eq!(runs.load(Ordering::SeqCst), 1);
    }
}
