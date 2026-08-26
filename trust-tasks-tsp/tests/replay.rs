//! SPEC §7.2 item 11 over the TSP binding: the duplicate-execution record
//! [`TspConsumer`] keeps, and what it does with each verdict.
//!
//! The scenario every test here is about is a **re-delivery**. Binding
//! `tsp/0.1` §7 records that TSP data messages "do not inherently prevent
//! replay", and §5.2's routed and nested carriage puts intermediaries on the
//! path, each of which may hold and re-forward the sealed inner message. When
//! that happens the same Trust Task document arrives again inside an envelope
//! that repeats *nothing*: a fresh seal, fresh ephemeral material, fresh
//! bytes. The only thing that repeats is the document, and the only key that
//! identifies it is its `id`.
//!
//! Runs in-process with `PrivateVid::generate` — no intermediary, no network.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use affinidi_tsp::PrivateVid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use trust_tasks_rs::{
    ConsumeOutcome, DocumentDigest, InMemoryReplayGuard, NoValidator, Payload, PayloadPolicy,
    ProofPolicy, ProofVerifier, ReplayGuard, ReplayGuardError, ReplayVerdict, StandardCode,
    TrustTask, VerificationError,
};
use trust_tasks_tsp::{pack_trust_task, TspConsumer};

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
    alice: PrivateVid,
    bob: PrivateVid,
}

fn setup() -> Pair {
    Pair {
        alice: PrivateVid::generate("did:example:alice"),
        bob: PrivateVid::generate("did:example:bob"),
    }
}

fn grant_doc(p: &Pair, role: &str) -> TrustTask<GrantPayload> {
    let mut doc = TrustTask::for_payload(
        "urn:uuid:8b0f6e1c-0d9a-4d0e-9f6c-3f2a1b4c5d6e",
        GrantPayload {
            subject: "did:example:carol".into(),
            role: role.into(),
        },
    );
    doc.issuer = Some(p.alice.id.clone());
    doc.recipient = Some(p.bob.id.clone());
    doc.issued_at = Some(Utc::now());
    doc
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
        Err(ReplayGuardError("dynamodb: request timed out".into()))
    }
}

async fn deliver(
    inbound: &TspConsumer,
    p: &Pair,
    wire: &[u8],
    dispatches: &Dispatches,
) -> ConsumeOutcome<GrantResponse> {
    inbound
        .receive::<GrantPayload, GrantResponse, NoVerifier, NoValidator, _, _>(
            wire,
            &p.bob,
            &p.alice.to_resolved(),
            &p.bob.id,
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

/// The base case: the identical sealed message arrives twice, and the grant
/// happens once.
#[tokio::test]
async fn a_redelivered_message_is_absorbed_and_dispatched_once() {
    let p = setup();
    let inbound = TspConsumer::new();
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p, "moderator");
    let wire = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

    assert!(matches!(
        deliver(&inbound, &p, &wire, &dispatches).await,
        ConsumeOutcome::Handled(_)
    ));

    match deliver(&inbound, &p, &wire, &dispatches).await {
        // §7.2 (*Disposition of a duplicate*): return the previously
        // determined result. Never `taskFailed` — the task did not fail, it
        // already happened.
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

/// **The re-delivery case, and the one that proves the key.**
///
/// Sealing the same document again produces an envelope that shares no bytes
/// with the first: TSP derives fresh ephemeral material per message, so a
/// re-send is a wholly distinct transport object carrying an identical
/// payload. SPEC §7.2 forbids substituting a transport identifier for the
/// document `id` precisely because of this — there is nothing about the
/// envelope that a record *could* key on and still absorb a re-send.
#[tokio::test]
async fn redelivery_under_a_fresh_transport_envelope_is_absorbed() {
    let p = setup();
    let inbound = TspConsumer::new();
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p, "moderator");
    let first = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");
    // The same document, sealed again — what an intermediary re-forwarding
    // from its queue, or a producer resending after a transport error, puts on
    // the wire.
    let second = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

    assert_ne!(
        first, second,
        "the premise of this test: a re-send is a fresh envelope, byte for byte"
    );

    assert!(matches!(
        deliver(&inbound, &p, &first, &dispatches).await,
        ConsumeOutcome::Handled(_)
    ));
    assert!(
        matches!(
            deliver(&inbound, &p, &second, &dispatches).await,
            ConsumeOutcome::Duplicate { .. }
        ),
        "the record is keyed on the document `id`, which did not change"
    );

    assert_eq!(
        dispatches.count(),
        1,
        "keying on anything the envelope carries would have executed this twice"
    );
}

/// SPEC §8.4: a retry is a bit-for-bit identical resend. A *different*
/// document under a reused `id` is not a retry, and item 11 requires
/// `idConflict` rather than treating it as one.
#[tokio::test]
async fn differing_content_under_a_reused_id_is_an_id_conflict() {
    let p = setup();
    let inbound = TspConsumer::new();
    let dispatches = Dispatches::default();

    let first = grant_doc(&p, "moderator");
    let mut altered = first.clone();
    altered.payload.role = "admin".into();
    assert_eq!(altered.id, first.id, "same `id`, different content");

    let first_wire = pack_trust_task(&first, &p.alice, &p.bob.to_resolved()).expect("pack");
    let altered_wire = pack_trust_task(&altered, &p.alice, &p.bob.to_resolved()).expect("pack");

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

/// A guard that cannot answer has established nothing, and executing anyway is
/// the double execution item 11 forbids. Fail closed: `unavailable`,
/// retryable, and no dispatch.
#[tokio::test]
async fn a_guard_error_fails_closed_without_dispatching() {
    let p = setup();
    let inbound = TspConsumer::with_replay_guard(Arc::new(BrokenGuard));
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p, "moderator");
    let wire = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

    match deliver(&inbound, &p, &wire, &dispatches).await {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::Unavailable.into());
            assert!(
                err.payload.retryable,
                "§8.4: the producer's bit-for-bit resend is absorbed once the store is back"
            );
            // §10.4: the store's identity and failure mode stay in the logs.
            let message = err.payload.message.unwrap_or_default();
            assert!(!message.contains("dynamodb"), "leaked the store: {message}");
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
/// another."
#[tokio::test]
async fn a_duplicate_arriving_mid_execution_is_reported_in_flight() {
    let p = setup();
    let inbound = TspConsumer::new();
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p, "moderator");
    let wire = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");
    // The intermediary's copy, sealed independently.
    let redelivered = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

    let outcome: ConsumeOutcome<GrantResponse> = inbound
        .receive::<GrantPayload, GrantResponse, NoVerifier, NoValidator, _, _>(
            &wire,
            &p.bob,
            &p.alice.to_resolved(),
            &p.bob.id,
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
                    // The intermediary re-forwards before this finishes.
                    match deliver(inbound, p, &redelivered, dispatches).await {
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
/// out loudly: with no record, the re-delivery grants twice.
#[tokio::test]
async fn the_opt_out_re_opens_double_execution() {
    let p = setup();
    let inbound = TspConsumer::without_replay_record();
    let dispatches = Dispatches::default();

    let doc = grant_doc(&p, "moderator");
    let wire = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

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
        TspConsumer::new().replay_guard().is_some(),
        "SPEC §7.2 item 11 is unconditional for a consequential task"
    );
    assert!(TspConsumer::default().replay_guard().is_some());
    assert!(TspConsumer::without_replay_record()
        .replay_guard()
        .is_none());
    assert!(
        TspConsumer::with_replay_guard(Arc::new(InMemoryReplayGuard::new(4)))
            .replay_guard()
            .is_some()
    );
}
