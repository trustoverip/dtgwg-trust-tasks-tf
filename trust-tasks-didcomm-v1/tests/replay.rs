//! SPEC §7.2 item 11 over the DIDComm v1 binding: the duplicate-execution
//! record [`DidcommV1Consumer`] keeps, and what it does with each verdict.
//!
//! The scenario every test here is about is a **mediator redelivery**. Binding
//! `didcomm-v1/0.2` §6 records this transport's freshness guarantee as
//! "**None**", and a mediator "can drop, delay, reorder, and re-deliver". When
//! it does, the same Trust Task document arrives again inside a *new* v1
//! message with a freshly generated `@id`. Nothing at the transport layer
//! repeats; the only thing that does is the document, and the only key that
//! identifies it is its `id`.
//!
//! The messages here are built and handed to the binding directly rather than
//! packed and unpacked: v1 packing needs connection state (a verkey pair per
//! side) that this crate deliberately does not model, and none of it bears on
//! item 11.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use affinidi_messaging_didcomm_v1::{Did, MessageV1, UnpackResult, Verkey};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use trust_tasks_didcomm_v1::{build_message, DidcommV1Consumer};
use trust_tasks_rs::{
    ConsumeOutcome, DocumentDigest, InMemoryReplayGuard, NoValidator, Payload, PayloadPolicy,
    ProofPolicy, ProofVerifier, ReplayGuard, ReplayGuardError, ReplayVerdict, StandardCode,
    TrustTask, VerificationError,
};

const ALICE: &str = "did:sov:alice";
const BOB: &str = "did:sov:bob";

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

fn grant_doc(role: &str) -> TrustTask<GrantPayload> {
    let mut doc = TrustTask::for_payload(
        "urn:uuid:8b0f6e1c-0d9a-4d0e-9f6c-3f2a1b4c5d6e",
        GrantPayload {
            subject: "did:sov:carol".into(),
            role: role.into(),
        },
    );
    doc.issuer = Some(ALICE.to_string());
    doc.recipient = Some(BOB.to_string());
    doc.issued_at = Some(Utc::now());
    doc
}

/// Wrap `msg` as a v1 authcrypt envelope alice sent bob — the shape the
/// binding's inbound gate accepts.
fn delivered(msg: MessageV1) -> UnpackResult {
    UnpackResult::Authcrypt {
        message: msg,
        sender: Did::parse(ALICE).expect("sender DID"),
        sender_verkey: Verkey::from_bytes([7u8; 32]),
        recipient: Did::parse(BOB).expect("recipient DID"),
        recipient_verkey: Verkey::from_bytes([9u8; 32]),
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
        Err(ReplayGuardError("postgres: connection refused".into()))
    }
}

async fn deliver(
    inbound: &DidcommV1Consumer,
    msg: MessageV1,
    dispatches: &Dispatches,
) -> ConsumeOutcome<GrantResponse> {
    inbound
        .receive::<GrantPayload, GrantResponse, NoVerifier, NoValidator, _, _>(
            delivered(msg),
            BOB,
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
        .expect("message is a well-formed carriage")
}

/// The base case: the identical message arrives twice, and the grant happens
/// once.
#[tokio::test]
async fn a_redelivered_message_is_absorbed_and_dispatched_once() {
    let inbound = DidcommV1Consumer::new();
    let dispatches = Dispatches::default();
    let msg = build_message(&grant_doc("moderator")).expect("build");

    assert!(matches!(
        deliver(&inbound, msg.clone(), &dispatches).await,
        ConsumeOutcome::Handled(_)
    ));

    match deliver(&inbound, msg, &dispatches).await {
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

/// **The mediator case, and the one that proves the key.**
///
/// The redelivery is a *different v1 message* carrying the *same document*:
/// `MessageV1::new` mints a fresh UUID `@id` every time, which is exactly what
/// a mediator forwarding a queued message produces. SPEC §7.2 forbids
/// substituting a transport message identifier for the document `id` precisely
/// because of this — a record keyed on `@id` would see two distinct messages,
/// admit the second, and grant twice.
#[tokio::test]
async fn redelivery_under_a_fresh_transport_id_is_absorbed() {
    let inbound = DidcommV1Consumer::new();
    let dispatches = Dispatches::default();

    let doc = grant_doc("moderator");
    let first = build_message(&doc).expect("build");
    let second = build_message(&doc).expect("build");

    assert_ne!(
        first.id, second.id,
        "the premise of this test: a redelivery carries a fresh transport `@id`"
    );
    assert_eq!(
        first.explicit_thid(),
        second.explicit_thid(),
        "the ~thread decorator is derived from the document, so it does repeat"
    );

    assert!(matches!(
        deliver(&inbound, first, &dispatches).await,
        ConsumeOutcome::Handled(_)
    ));
    assert!(
        matches!(
            deliver(&inbound, second, &dispatches).await,
            ConsumeOutcome::Duplicate { .. }
        ),
        "the record is keyed on the document `id`, which did not change"
    );

    assert_eq!(
        dispatches.count(),
        1,
        "keying on the v1 message `@id` would have executed this twice"
    );
}

/// SPEC §8.4: a retry is a bit-for-bit identical resend. A *different*
/// document under a reused `id` is not a retry, and item 11 requires
/// `idConflict` rather than treating it as one.
#[tokio::test]
async fn differing_content_under_a_reused_id_is_an_id_conflict() {
    let inbound = DidcommV1Consumer::new();
    let dispatches = Dispatches::default();

    let first = grant_doc("moderator");
    let mut altered = first.clone();
    altered.payload.role = "admin".into();
    assert_eq!(altered.id, first.id, "same `id`, different content");

    assert!(matches!(
        deliver(&inbound, build_message(&first).expect("build"), &dispatches).await,
        ConsumeOutcome::Handled(_)
    ));

    match deliver(
        &inbound,
        build_message(&altered).expect("build"),
        &dispatches,
    )
    .await
    {
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
    let inbound = DidcommV1Consumer::with_replay_guard(Arc::new(BrokenGuard));
    let dispatches = Dispatches::default();
    let msg = build_message(&grant_doc("moderator")).expect("build");

    match deliver(&inbound, msg, &dispatches).await {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::Unavailable.into());
            assert!(
                err.payload.retryable,
                "§8.4: the producer's bit-for-bit resend is absorbed once the store is back"
            );
            // §10.4: the store's identity and failure mode stay in the logs.
            let message = err.payload.message.unwrap_or_default();
            assert!(!message.contains("postgres"), "leaked the store: {message}");
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
    let inbound = DidcommV1Consumer::new();
    let dispatches = Dispatches::default();

    let doc = grant_doc("moderator");
    let first = build_message(&doc).expect("build");
    let redelivered = build_message(&doc).expect("build");

    let outcome: ConsumeOutcome<GrantResponse> = inbound
        .receive::<GrantPayload, GrantResponse, NoVerifier, NoValidator, _, _>(
            delivered(first),
            BOB,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            Utc::now(),
            || "urn:uuid:error-1".to_string(),
            |doc, _parties| {
                let inbound = &inbound;
                let dispatches = &dispatches;
                async move {
                    dispatches.0.fetch_add(1, Ordering::SeqCst);
                    // The mediator redelivers before this execution finishes.
                    match deliver(inbound, redelivered, dispatches).await {
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
        .expect("message is a well-formed carriage");

    assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
    assert_eq!(dispatches.count(), 1, "only the first arrival executed");
}

/// The opt-out does what its documentation says, which is why it is spelled
/// out loudly: with no record, the redelivery grants twice.
#[tokio::test]
async fn the_opt_out_re_opens_double_execution() {
    let inbound = DidcommV1Consumer::without_replay_record();
    let dispatches = Dispatches::default();
    let msg = build_message(&grant_doc("moderator")).expect("build");

    deliver(&inbound, msg.clone(), &dispatches).await;
    deliver(&inbound, msg, &dispatches).await;

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
        DidcommV1Consumer::new().replay_guard().is_some(),
        "SPEC §7.2 item 11 is unconditional for a consequential task"
    );
    assert!(DidcommV1Consumer::default().replay_guard().is_some());
    assert!(DidcommV1Consumer::without_replay_record()
        .replay_guard()
        .is_none());
    assert!(
        DidcommV1Consumer::with_replay_guard(Arc::new(InMemoryReplayGuard::new(4)))
            .replay_guard()
            .is_some()
    );
}
