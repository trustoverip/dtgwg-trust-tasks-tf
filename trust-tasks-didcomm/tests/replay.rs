//! SPEC §7.2 item 11 over the DIDComm v2.1 binding: the duplicate-execution
//! record [`DidcommConsumer`] keeps, and what it does with each verdict.
//!
//! The scenario every test here is about is a **mediator redelivery**. Binding
//! `didcomm/0.2` §6: a mediator "can drop, delay, reorder, and re-deliver".
//! When it does, the same Trust Task document arrives a second time inside a
//! *fresh* DIDComm envelope — new `@id`, new JWE, and (where the document set
//! no `threadId`) potentially a new `thid`. Nothing about that message is a
//! replica of the first at the transport layer; the only thing that repeats is
//! the document, and the only key that identifies it is its `id`.
//!
//! Runs in-process with `PrivateIdentity::generate` — no mediator, no network.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use affinidi_messaging_didcomm::{identity::PrivateIdentity, DIDCommAgent, UnpackResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use trust_tasks_didcomm::{pack_trust_task, DidcommConsumer};
use trust_tasks_rs::{
    ConsumeOutcome, DocumentDigest, InMemoryReplayGuard, NoValidator, Payload, PayloadPolicy,
    ProofPolicy, ProofVerifier, ReplayGuard, ReplayGuardError, ReplayVerdict, StandardCode,
    TrustTask, VerificationError,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GrantPayload {
    subject: String,
    role: String,
}

impl Payload for GrantPayload {
    const TYPE_URI: &'static str = "https://example.com/spec/grant/0.1";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GrantResponse {
    granted: bool,
}

struct Pair {
    alice_did: String,
    bob_did: String,
    alice_agent: DIDCommAgent,
    bob_agent: DIDCommAgent,
}

fn setup_agents() -> Pair {
    let alice = PrivateIdentity::generate("did:peer:alice");
    let bob = PrivateIdentity::generate("did:peer:bob");

    let alice_did = alice.did.clone();
    let bob_did = bob.did.clone();
    let alice_resolved = alice.to_resolved();
    let bob_resolved = bob.to_resolved();

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

fn grant_doc(issuer: &str, recipient: &str, role: &str) -> TrustTask<GrantPayload> {
    let mut doc = TrustTask::for_payload(
        "urn:uuid:8b0f6e1c-0d9a-4d0e-9f6c-3f2a1b4c5d6e",
        GrantPayload {
            subject: "did:web:carol.example".into(),
            role: role.into(),
        },
    );
    doc.issuer = Some(issuer.to_string());
    doc.recipient = Some(recipient.to_string());
    doc.issued_at = Some(Utc::now());
    doc
}

/// The DIDComm `@id` of an envelope — the transport message identifier SPEC
/// §7.2 forbids substituting for the document `id`. Read here so the tests can
/// assert it differs across a redelivery.
fn transport_message_id(wire: &str, agent: &DIDCommAgent, sender: &str) -> String {
    match agent.unpack(wire, Some(sender)).expect("unpack") {
        UnpackResult::Encrypted { message, .. } => message.id,
        _ => panic!("expected an authcrypt envelope"),
    }
}

/// Pins `ProofPolicy`'s verifier type on the `AcceptUnverified` arm. These
/// tests are about item 11, not about proofs — the documents carry none.
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

/// Counts dispatches. The whole of item 11 is "this number stays at 1".
#[derive(Default)]
struct Dispatches(AtomicUsize);

impl Dispatches {
    fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

/// A guard whose store is unreachable — the `Err(ReplayGuardError)` path.
struct BrokenGuard;

#[async_trait::async_trait]
impl ReplayGuard for BrokenGuard {
    async fn claim(
        &self,
        _id: &str,
        _digest: &DocumentDigest,
        _retain_until: Option<DateTime<Utc>>,
        _now: DateTime<Utc>,
    ) -> Result<ReplayVerdict, ReplayGuardError> {
        Err(ReplayGuardError("redis: connection refused".into()))
    }
}

/// Run one inbound envelope through `inbound`, counting dispatches.
async fn deliver(
    inbound: &DidcommConsumer,
    p: &Pair,
    wire: &str,
    dispatches: &Dispatches,
) -> ConsumeOutcome<GrantResponse> {
    inbound
        .receive::<GrantPayload, GrantResponse, NoVerifier, NoValidator, _, _>(
            wire,
            &p.bob_agent,
            Some(&p.alice_did),
            &p.bob_did,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            Utc::now(),
            || "urn:uuid:error-1".to_string(),
            |doc, _parties| {
                let dispatches = &dispatches.0;
                async move {
                    dispatches.fetch_add(1, Ordering::SeqCst);
                    Ok(doc.respond_with("urn:uuid:resp-1", GrantResponse { granted: true }))
                }
            },
        )
        .await
        .expect("envelope opens")
}

/// The base case: the identical envelope arrives twice (a mediator that never
/// saw the acknowledgement), and the grant happens once.
#[tokio::test]
async fn a_redelivered_envelope_is_absorbed_and_dispatched_once() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p.alice_did, &p.bob_did, "moderator");
    let wire = pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");

    let first = deliver(&inbound, &p, &wire, &dispatches).await;
    assert!(matches!(first, ConsumeOutcome::Handled(_)));

    let second = deliver(&inbound, &p, &wire, &dispatches).await;
    match second {
        // §7.2 (*Disposition of a duplicate*): the consumer SHOULD return the
        // previously determined result. Never `taskFailed` — the task did not
        // fail, it already happened.
        ConsumeOutcome::Duplicate {
            prior_response: Some(prior),
            in_flight,
        } => {
            assert!(!in_flight, "the first execution finished");
            assert_eq!(prior["payload"]["granted"], Value::Bool(true));
            assert_eq!(prior["id"], "urn:uuid:resp-1");
        }
        other => panic!("expected an absorbed duplicate, got {other:?}"),
    }

    assert_eq!(
        dispatches.count(),
        1,
        "item 11: the consequential effect must not happen a second time"
    );
}

/// **The mediator case, and the one that proves the key.**
///
/// The redelivery is a *different DIDComm message* carrying the *same
/// document*: a fresh `@id`, a fresh JWE, a fresh ephemeral key. SPEC §7.2
/// forbids substituting a transport message identifier for the document `id`
/// precisely because of this — a record keyed on `@id` would see two distinct
/// messages, admit the second, and grant twice.
#[tokio::test]
async fn redelivery_under_a_fresh_transport_id_is_absorbed() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p.alice_did, &p.bob_did, "moderator");
    let first_wire = pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");
    // The same document, packed again — which is exactly what a re-queued
    // send, or a producer resending after a transport error, puts on the wire.
    let second_wire =
        pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");

    let first_id = transport_message_id(&first_wire, &p.bob_agent, &p.alice_did);
    let second_id = transport_message_id(&second_wire, &p.bob_agent, &p.alice_did);
    assert_ne!(
        first_id, second_id,
        "the premise of this test: a redelivery carries a fresh transport `@id`"
    );
    assert_ne!(
        first_wire, second_wire,
        "and fresh envelope bytes, so nothing at the transport layer repeats"
    );

    let first = deliver(&inbound, &p, &first_wire, &dispatches).await;
    assert!(matches!(first, ConsumeOutcome::Handled(_)));

    let second = deliver(&inbound, &p, &second_wire, &dispatches).await;
    assert!(
        matches!(second, ConsumeOutcome::Duplicate { .. }),
        "the record is keyed on the document `id`, which did not change"
    );

    assert_eq!(
        dispatches.count(),
        1,
        "keying on the DIDComm `@id` would have executed this twice"
    );
}

/// SPEC §8.4: a retry is a bit-for-bit identical resend. A *different*
/// document under a reused `id` is not a retry, and item 11 requires
/// `idConflict` rather than treating it as one.
#[tokio::test]
async fn differing_content_under_a_reused_id_is_an_id_conflict() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let dispatches = Dispatches::default();

    let first = grant_doc(&p.alice_did, &p.bob_did, "moderator");
    let mut altered = first.clone();
    altered.payload.role = "admin".into();
    assert_eq!(altered.id, first.id, "same `id`, different content");

    let first_wire =
        pack_trust_task(&first, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");
    let altered_wire =
        pack_trust_task(&altered, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");

    assert!(matches!(
        deliver(&inbound, &p, &first_wire, &dispatches).await,
        ConsumeOutcome::Handled(_)
    ));

    match deliver(&inbound, &p, &altered_wire, &dispatches).await {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::IdConflict.into());
        }
        other => panic!("expected idConflict, got {other:?}"),
    }

    assert_eq!(
        dispatches.count(),
        1,
        "the conflicting document must not be executed"
    );
}

/// A guard that cannot answer has not established anything, and executing
/// anyway is the double execution item 11 forbids. Fail closed: `unavailable`,
/// retryable, and no dispatch.
#[tokio::test]
async fn a_guard_error_fails_closed_without_dispatching() {
    let p = setup_agents();
    let inbound = DidcommConsumer::with_replay_guard(Arc::new(BrokenGuard));
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p.alice_did, &p.bob_did, "moderator");
    let wire = pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");

    match deliver(&inbound, &p, &wire, &dispatches).await {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::Unavailable.into());
            assert!(
                err.payload.retryable,
                "§8.4: the producer's bit-for-bit resend will be absorbed once the store is back"
            );
            // §10.4: the store's identity and failure mode stay in the logs.
            let message = err.payload.message.unwrap_or_default();
            assert!(!message.contains("redis"), "leaked the store: {message}");
        }
        other => panic!("expected a fail-closed rejection, got {other:?}"),
    }

    assert_eq!(
        dispatches.count(),
        0,
        "never execute on a guard error — that is the failure the rule forbids"
    );
}

/// §7.2: "Where the original execution is still in progress, the *consumer*
/// **SHOULD** return or expose the existing execution state rather than begin
/// another." The redelivery here arrives *while* the first is still running.
#[tokio::test]
async fn a_duplicate_arriving_mid_execution_is_reported_in_flight() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p.alice_did, &p.bob_did, "moderator");
    let wire = pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");
    // The mediator's copy: the very same envelope, held in its queue.
    let redelivered = wire.clone();

    let outcome: ConsumeOutcome<GrantResponse> = inbound
        .receive::<GrantPayload, GrantResponse, NoVerifier, NoValidator, _, _>(
            &wire,
            &p.bob_agent,
            Some(&p.alice_did),
            &p.bob_did,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            Utc::now(),
            || "urn:uuid:error-1".to_string(),
            |doc, _parties| {
                let inbound = &inbound;
                let p = &p;
                let dispatches = &dispatches;
                async move {
                    dispatches.0.fetch_add(1, Ordering::SeqCst);
                    // The mediator redelivers before this execution finishes.
                    let concurrent = deliver(inbound, p, &redelivered, dispatches).await;
                    match concurrent {
                        ConsumeOutcome::Duplicate {
                            prior_response,
                            in_flight,
                        } => {
                            assert!(in_flight, "the first execution has not finished");
                            assert!(
                                prior_response.is_none(),
                                "there is no result yet to hand back"
                            );
                        }
                        other => panic!("expected an in-flight duplicate, got {other:?}"),
                    }
                    Ok(doc.respond_with("urn:uuid:resp-1", GrantResponse { granted: true }))
                }
            },
        )
        .await
        .expect("envelope opens");

    assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
    assert_eq!(dispatches.count(), 1, "only the first arrival executed");
}

/// The opt-out does what its documentation says, which is why it is spelled
/// out loudly: with no record, the redelivery grants twice.
#[tokio::test]
async fn the_opt_out_re_opens_double_execution() {
    let p = setup_agents();
    let inbound = DidcommConsumer::without_replay_record();
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p.alice_did, &p.bob_did, "moderator");
    let wire = pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");

    deliver(&inbound, &p, &wire, &dispatches).await;
    deliver(&inbound, &p, &wire, &dispatches).await;

    assert_eq!(
        dispatches.count(),
        2,
        "this is what `without_replay_record` gives up, and why it is not the default"
    );
}

/// The default is on. Stated as a test because a regression here is invisible:
/// a consumer with no record behaves identically until something redelivers.
#[tokio::test]
async fn the_record_is_on_by_default() {
    assert!(
        DidcommConsumer::new().replay_guard().is_some(),
        "SPEC §7.2 item 11 is unconditional for a consequential task"
    );
    assert!(DidcommConsumer::default().replay_guard().is_some());
    assert!(DidcommConsumer::without_replay_record()
        .replay_guard()
        .is_none());
    assert!(
        DidcommConsumer::with_replay_guard(Arc::new(InMemoryReplayGuard::new(4)))
            .replay_guard()
            .is_some()
    );
}
