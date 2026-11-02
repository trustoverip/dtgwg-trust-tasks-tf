//! A consumer with no fixed payload type (#969) over DIDComm v2.1: it routes
//! on `type`, answers an unknown one `unsupportedType` before writing any
//! record, and keeps the duplicate-execution record (SPEC §7.2 item 11) for
//! the types it does handle.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use affinidi_messaging_didcomm::{identity::PrivateIdentity, DIDCommAgent};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_didcomm::{pack_trust_task, unpack_trust_task, DidcommConsumer, SenderAllowlist};
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorPayload, ErrorResponse, InMemoryReplayGuard, NoValidator,
    Payload, PayloadPolicy, ProofPolicy, ProofVerifier, ResolvedParties, StandardCode, TrustTask,
    TrustTaskCode, VerificationError,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GrantPayload {
    subject: String,
    role: String,
}

impl Payload for GrantPayload {
    const TYPE_URI: &'static str = "https://example.com/spec/grant/0.1";
}

struct Pair {
    alice_did: String,
    bob_did: String,
    alice_agent: DIDCommAgent,
    bob_agent: DIDCommAgent,
}

fn setup() -> Pair {
    let alice = PrivateIdentity::generate("did:peer:alice");
    let bob = PrivateIdentity::generate("did:peer:bob");
    let (alice_did, bob_did) = (alice.did.clone(), bob.did.clone());
    let (alice_resolved, bob_resolved) = (alice.to_resolved(), bob.to_resolved());
    let mut alice_agent = DIDCommAgent::new();
    alice_agent.add_identity(alice);
    alice_agent.add_peer(bob_resolved);
    let mut bob_agent = DIDCommAgent::new();
    bob_agent.add_identity(bob);
    bob_agent.add_peer(alice_resolved);
    Pair {
        alice_did,
        bob_did,
        alice_agent,
        bob_agent,
    }
}

fn doc(p: &Pair, id: &str, type_uri: &str, payload: Value) -> TrustTask<Value> {
    let mut doc = TrustTask::new(id, type_uri.parse().unwrap(), payload);
    doc.issuer = Some(p.alice_did.clone());
    doc.recipient = Some(p.bob_did.clone());
    doc.issued_at = Some(Utc::now());
    doc
}

type Answer = Result<TrustTask<Value>, ErrorResponse>;

fn dispatcher(count: Arc<AtomicUsize>) -> AsyncDispatcher<ResolvedParties, Answer> {
    AsyncDispatcher::new().on_async::<GrantPayload, _, _>(move |doc, _parties| {
        let count = count.clone();
        async move {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(doc.respond_with("urn:uuid:resp-1", json!({ "granted": true })))
        }
    })
}

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

async fn deliver(
    inbound: &DidcommConsumer,
    p: &Pair,
    wire: &str,
    dispatcher: &AsyncDispatcher<ResolvedParties, Answer>,
) -> ConsumeOutcome<Value> {
    inbound
        .receive_dispatched(
            wire,
            &p.bob_agent,
            Some(&p.alice_did),
            &p.bob_did,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            dispatcher,
            Utc::now(),
            || "urn:uuid:error-1".to_string(),
            |parties| parties,
        )
        .await
        .expect("envelope opens")
}

#[tokio::test]
async fn a_redelivered_document_executes_once() {
    let p = setup();
    let count = Arc::new(AtomicUsize::new(0));
    let dispatcher = dispatcher(count.clone());
    let inbound = DidcommConsumer::new();
    let grant = doc(
        &p,
        "urn:uuid:g-1",
        GrantPayload::TYPE_URI,
        json!({ "subject": "did:web:carol.example", "role": "admin" }),
    );
    let wire = pack_trust_task(&grant, &p.alice_agent, &p.alice_did, &p.bob_did).unwrap();

    assert!(matches!(
        deliver(&inbound, &p, &wire, &dispatcher).await,
        ConsumeOutcome::Handled(_)
    ));
    // A re-send is a fresh envelope around the same document.
    let resent = pack_trust_task(&grant, &p.alice_agent, &p.alice_did, &p.bob_did).unwrap();
    assert!(matches!(
        deliver(&inbound, &p, &resent, &dispatcher).await,
        ConsumeOutcome::Duplicate { .. }
    ));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn an_unknown_type_is_unsupported_type_and_the_error_seals() {
    let p = setup();
    let guard = Arc::new(InMemoryReplayGuard::default());
    let inbound = DidcommConsumer::with_replay_guard(guard.clone());
    let dispatcher = dispatcher(Arc::new(AtomicUsize::new(0)));
    let unknown = doc(
        &p,
        "urn:uuid:u-1",
        "https://example.com/spec/sealed-envelope/0.1",
        json!({}),
    );
    let allowlist = SenderAllowlist::new([p.alice_did.clone()]);
    let wire = pack_trust_task(&unknown, &p.alice_agent, &p.alice_did, &p.bob_did).unwrap();

    let outcome = inbound
        .receive_from_dispatched(
            &wire,
            &p.bob_agent,
            &allowlist,
            &p.bob_did,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            &dispatcher,
            Utc::now(),
            || "urn:uuid:error-1".to_string(),
            |parties| parties,
        )
        .await
        .unwrap();
    let ConsumeOutcome::Rejected(error) = outcome else {
        panic!("expected a rejection, got {outcome:?}");
    };
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::UnsupportedType)
    );
    assert_eq!(guard.len(), 0);

    // The error document is sealed as it is, and read back typed.
    let reply = pack_trust_task(&error, &p.bob_agent, &p.bob_did, &p.alice_did).unwrap();
    let (reply, _) =
        unpack_trust_task::<ErrorPayload>(&reply, &p.alice_agent, Some(&p.bob_did)).unwrap();
    assert_eq!(reply.type_uri.to_string(), ErrorPayload::TYPE_URI);
}
