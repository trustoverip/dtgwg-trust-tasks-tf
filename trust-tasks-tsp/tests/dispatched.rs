//! A consumer with no fixed payload type (#969): it opens a Trust Task of
//! **any** type, routes it on `type`, and answers a type it does not implement
//! with `unsupportedType` — over TSP, with the duplicate-execution record
//! (SPEC §7.2 item 11) and the freshness bound still applied.
//!
//! Runs in-process with `PrivateVid::generate` — no intermediary, no network.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use affinidi_tsp::PrivateVid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, Dispatcher, ErrorPayload, ErrorResponse, InMemoryReplayGuard,
    NoValidator, Payload, PayloadPolicy, Proof, ProofPolicy, ProofVerifier, ResolvedParties,
    StandardCode, TrustTask, TrustTaskCode, VerificationError,
};
use trust_tasks_tsp::{pack_trust_task, unpack_trust_task, TspConsumer};

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

/// A document of `type_uri` with `payload`, addressed alice → bob.
fn doc(p: &Pair, id: &str, type_uri: &str, payload: Value) -> TrustTask<Value> {
    let mut doc = TrustTask::new(id, type_uri.parse().unwrap(), payload);
    doc.issuer = Some(p.alice.id.clone());
    doc.recipient = Some(p.bob.id.clone());
    doc.issued_at = Some(Utc::now());
    doc
}

fn grant(p: &Pair, id: &str) -> TrustTask<Value> {
    doc(
        p,
        id,
        GrantPayload::TYPE_URI,
        json!({ "subject": "did:example:carol", "role": "admin" }),
    )
}

type Answer = Result<TrustTask<Value>, ErrorResponse>;

/// One handler, for `grant/0.1`, counting its executions.
fn dispatcher(count: Arc<AtomicUsize>) -> AsyncDispatcher<ResolvedParties, Answer> {
    AsyncDispatcher::new().on_async::<GrantPayload, _, _>(move |doc, _parties| {
        let count = count.clone();
        async move {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(doc.respond_with("urn:uuid:resp-1", json!(GrantResponse { granted: true })))
        }
    })
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

async fn deliver(
    inbound: &TspConsumer,
    p: &Pair,
    wire: &[u8],
    dispatcher: &AsyncDispatcher<ResolvedParties, Answer>,
) -> ConsumeOutcome<Value> {
    inbound
        .receive_dispatched(
            wire,
            &p.bob,
            &p.alice.to_resolved(),
            &p.bob.id,
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

fn rejected_with(outcome: ConsumeOutcome<Value>) -> ErrorResponse {
    match outcome {
        ConsumeOutcome::Rejected(error) => error,
        other => panic!("expected a rejection, got {other:?}"),
    }
}

/// The issue's reproduction, as it was written: unpack untyped, dispatch, and
/// seal the error document `dispatch_or_reject` returns.
#[test]
fn the_reproduction_compiles_and_seals_the_error() {
    let p = setup();
    let wire = pack_trust_task(
        &doc(
            &p,
            "urn:uuid:1",
            "https://real-life.org/trust-tasks/spec/sealed-envelope/0.1",
            json!({}),
        ),
        &p.alice,
        &p.bob.to_resolved(),
    )
    .unwrap();

    let (inbound, _handler) =
        unpack_trust_task::<Value>(&wire, &p.bob, &p.alice.to_resolved()).unwrap();
    let dispatcher: Dispatcher<()> = Dispatcher::new();
    let err: ErrorResponse = dispatcher
        .dispatch_or_reject(inbound, "urn:uuid:err-1")
        .unwrap_err();
    let sealed = pack_trust_task(&err, &p.bob, &p.alice.to_resolved()).unwrap();

    // …and the producer reads it back as a typed error response.
    let (reply, _) =
        unpack_trust_task::<ErrorPayload>(&sealed, &p.alice, &p.bob.to_resolved()).unwrap();
    assert_eq!(reply.type_uri.to_string(), ErrorPayload::TYPE_URI);
    assert_eq!(
        reply.payload.code,
        TrustTaskCode::Standard(StandardCode::UnsupportedType)
    );
}

/// The record holds on the dispatched path: the same sealed message delivered
/// twice executes once.
#[tokio::test]
async fn a_redelivered_document_executes_once() {
    let p = setup();
    let count = Arc::new(AtomicUsize::new(0));
    let dispatcher = dispatcher(count.clone());
    let inbound = TspConsumer::new();
    let wire = pack_trust_task(&grant(&p, "urn:uuid:g-1"), &p.alice, &p.bob.to_resolved()).unwrap();

    match deliver(&inbound, &p, &wire, &dispatcher).await {
        ConsumeOutcome::Handled(response) => {
            assert_eq!(response.payload, json!({ "granted": true }));
            assert_eq!(
                response.type_uri.to_string(),
                format!("{}#response", GrantPayload::TYPE_URI)
            );
        }
        other => panic!("expected Handled, got {other:?}"),
    }
    match deliver(&inbound, &p, &wire, &dispatcher).await {
        ConsumeOutcome::Duplicate {
            prior_response: Some(prior),
            in_flight: false,
        } => assert_eq!(prior["payload"], json!({ "granted": true })),
        other => panic!("expected Duplicate, got {other:?}"),
    }
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

/// A type the consumer has no handler for is `unsupportedType`, and is refused
/// before the record: the `id` is not burned.
#[tokio::test]
async fn an_unknown_type_is_unsupported_type_and_writes_no_record() {
    let p = setup();
    let guard = Arc::new(InMemoryReplayGuard::default());
    let inbound = TspConsumer::with_replay_guard(guard.clone());
    let dispatcher = dispatcher(Arc::new(AtomicUsize::new(0)));
    let unknown = doc(
        &p,
        "urn:uuid:u-1",
        "https://real-life.org/trust-tasks/spec/sealed-envelope/0.1",
        json!({}),
    );
    let wire = pack_trust_task(&unknown, &p.alice, &p.bob.to_resolved()).unwrap();

    let error = rejected_with(deliver(&inbound, &p, &wire, &dispatcher).await);
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::UnsupportedType)
    );
    // §8.1: addressed back to the producer, and answers the document by id.
    assert_eq!(error.recipient.as_deref(), Some(p.alice.id.as_str()));
    assert_eq!(
        error.payload.in_response_to.as_ref().unwrap().id.as_deref(),
        Some("urn:uuid:u-1")
    );
    assert_eq!(guard.len(), 0, "an unrouteable document is never accepted");

    // The error document seals as it is.
    pack_trust_task(&error, &p.bob, &p.alice.to_resolved()).unwrap();
}

/// A registered slug at an unregistered version is `unsupportedVersion`.
#[tokio::test]
async fn a_known_slug_at_another_version_is_unsupported_version() {
    let p = setup();
    let inbound = TspConsumer::new();
    let dispatcher = dispatcher(Arc::new(AtomicUsize::new(0)));
    let later = doc(
        &p,
        "urn:uuid:v-1",
        "https://example.com/spec/grant/0.9",
        json!({ "subject": "did:example:carol", "role": "admin" }),
    );
    let wire = pack_trust_task(&later, &p.alice, &p.bob.to_resolved()).unwrap();

    let error = rejected_with(deliver(&inbound, &p, &wire, &dispatcher).await);
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::UnsupportedVersion)
    );
}

/// A payload the handler's type cannot hold is `malformedRequest`, before the
/// record and without running the handler.
#[tokio::test]
async fn a_malformed_payload_is_refused_before_the_record() {
    let p = setup();
    let count = Arc::new(AtomicUsize::new(0));
    let guard = Arc::new(InMemoryReplayGuard::default());
    let inbound = TspConsumer::with_replay_guard(guard.clone());
    let dispatcher = dispatcher(count.clone());
    let bad = doc(
        &p,
        "urn:uuid:m-1",
        GrantPayload::TYPE_URI,
        json!({ "subject": 7 }),
    );
    let wire = pack_trust_task(&bad, &p.alice, &p.bob.to_resolved()).unwrap();

    let error = rejected_with(deliver(&inbound, &p, &wire, &dispatcher).await);
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::MalformedRequest)
    );
    assert_eq!(guard.len(), 0);
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

/// The consumer's freshness policy applies: `TspConsumer::new` requires
/// `issuedAt` (it bounds the record), so a document without one is refused.
#[tokio::test]
async fn the_freshness_policy_applies_on_the_dispatched_path() {
    let p = setup();
    let count = Arc::new(AtomicUsize::new(0));
    let inbound = TspConsumer::new();
    let dispatcher = dispatcher(count.clone());
    let mut undated = grant(&p, "urn:uuid:f-1");
    undated.issued_at = None;
    let wire = pack_trust_task(&undated, &p.alice, &p.bob.to_resolved()).unwrap();

    let error = rejected_with(deliver(&inbound, &p, &wire, &dispatcher).await);
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::MalformedRequest)
    );
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

/// Accepts a proof only when the document it is handed carries a payload
/// member the handler's type does not model — i.e. only when it is the
/// document as received.
struct ReceivedOnly {
    pass: bool,
}

#[async_trait::async_trait]
impl ProofVerifier for ReceivedOnly {
    async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: Serialize + Send + Sync,
    {
        Err(VerificationError::Other("verified a re-encoding".into()))
    }

    async fn verify_received(&self, received: &Value) -> Result<(), VerificationError> {
        assert_eq!(received["payload"]["note"], json!("unmodelled"));
        if self.pass {
            Ok(())
        } else {
            Err(VerificationError::Other("bad signature".into()))
        }
    }
}

fn signed_grant(p: &Pair, id: &str) -> TrustTask<Value> {
    let mut doc = grant(p, id);
    doc.payload["note"] = json!("unmodelled");
    doc.proof = Some(Proof {
        proof_type: "DataIntegrityProof".into(),
        cryptosuite: "eddsa-jcs-2022".into(),
        verification_method: format!("{}#key-1", p.alice.id),
        created: Utc::now(),
        proof_purpose: "assertionMethod".into(),
        proof_value: "z3kg".into(),
        extra: Default::default(),
    });
    doc
}

/// The proof is verified over the document as received, and a failure is
/// `proofInvalid` before the record.
#[tokio::test]
async fn the_proof_is_verified_over_the_received_document() {
    let p = setup();
    for pass in [true, false] {
        let count = Arc::new(AtomicUsize::new(0));
        let guard = Arc::new(InMemoryReplayGuard::default());
        let inbound = TspConsumer::with_replay_guard(guard.clone());
        let dispatcher = dispatcher(count.clone());
        let wire = pack_trust_task(
            &signed_grant(&p, "urn:uuid:p-1"),
            &p.alice,
            &p.bob.to_resolved(),
        )
        .unwrap();
        let verifier = ReceivedOnly { pass };
        let outcome = inbound
            .receive_dispatched(
                &wire,
                &p.bob,
                &p.alice.to_resolved(),
                &p.bob.id,
                ProofPolicy::Verify(&verifier),
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                &dispatcher,
                Utc::now(),
                || "urn:uuid:error-1".to_string(),
                |parties| parties,
            )
            .await
            .unwrap();
        if pass {
            assert!(matches!(outcome, ConsumeOutcome::Handled(_)), "{outcome:?}");
            assert_eq!(count.load(Ordering::SeqCst), 1);
        } else {
            let error = rejected_with(outcome);
            assert_eq!(
                error.payload.code,
                TrustTaskCode::Standard(StandardCode::ProofInvalid)
            );
            assert_eq!(guard.len(), 0);
            assert_eq!(count.load(Ordering::SeqCst), 0);
        }
    }
}

/// A consumer serving many peers spawns each delivery on a runtime, so the
/// dispatched path must be `Send`.
#[tokio::test]
async fn the_dispatched_path_can_be_spawned() {
    let p = Arc::new(setup());
    let inbound = Arc::new(TspConsumer::new());
    let dispatcher = Arc::new(dispatcher(Arc::new(AtomicUsize::new(0))));
    let wire = pack_trust_task(&grant(&p, "urn:uuid:s-1"), &p.alice, &p.bob.to_resolved()).unwrap();
    let outcome = tokio::spawn(async move { deliver(&inbound, &p, &wire, &dispatcher).await })
        .await
        .unwrap();
    assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
}
