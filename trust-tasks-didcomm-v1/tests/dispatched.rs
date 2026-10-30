//! A consumer with no fixed payload type (#969) over DIDComm v1: it routes on
//! `type`, answers an unknown one `unsupportedType` before writing any record,
//! and keeps the duplicate-execution record (SPEC §7.2 item 11) for the types
//! it does handle.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use affinidi_messaging_didcomm_v1::{Did, MessageV1, UnpackResult, Verkey};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_didcomm_v1::{build_message, DidcommV1Consumer};
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorResponse, InMemoryReplayGuard, NoValidator, Payload,
    PayloadPolicy, ProofPolicy, ProofVerifier, ResolvedParties, StandardCode, TrustTask,
    TrustTaskCode, VerificationError,
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

fn doc(id: &str, type_uri: &str, payload: Value) -> TrustTask<Value> {
    let mut doc = TrustTask::new(id, type_uri.parse().unwrap(), payload);
    doc.issuer = Some(ALICE.to_string());
    doc.recipient = Some(BOB.to_string());
    doc.issued_at = Some(Utc::now());
    doc
}

fn delivered(msg: MessageV1) -> UnpackResult {
    UnpackResult::Authcrypt {
        message: msg,
        sender: Did::parse(ALICE).expect("sender DID"),
        sender_verkey: Verkey::from_bytes([7u8; 32]),
        recipient: Did::parse(BOB).expect("recipient DID"),
        recipient_verkey: Verkey::from_bytes([9u8; 32]),
    }
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
    inbound: &DidcommV1Consumer,
    msg: MessageV1,
    dispatcher: &AsyncDispatcher<ResolvedParties, Answer>,
) -> ConsumeOutcome<Value> {
    inbound
        .receive_dispatched(
            delivered(msg),
            BOB,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            dispatcher,
            Utc::now(),
            || "urn:uuid:error-1".to_string(),
            |parties| parties,
        )
        .await
        .expect("message is a well-formed carriage")
}

#[tokio::test]
async fn a_redelivered_document_executes_once() {
    let count = Arc::new(AtomicUsize::new(0));
    let dispatcher = dispatcher(count.clone());
    let inbound = DidcommV1Consumer::new();
    let msg = build_message(&doc(
        "urn:uuid:g-1",
        GrantPayload::TYPE_URI,
        json!({ "subject": "did:sov:carol", "role": "admin" }),
    ))
    .unwrap();

    assert!(matches!(
        deliver(&inbound, msg.clone(), &dispatcher).await,
        ConsumeOutcome::Handled(_)
    ));
    assert!(matches!(
        deliver(&inbound, msg, &dispatcher).await,
        ConsumeOutcome::Duplicate { .. }
    ));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn an_unknown_type_is_unsupported_type_and_the_error_is_carried() {
    let guard = Arc::new(InMemoryReplayGuard::default());
    let inbound = DidcommV1Consumer::with_replay_guard(guard.clone());
    let dispatcher = dispatcher(Arc::new(AtomicUsize::new(0)));
    let msg = build_message(&doc(
        "urn:uuid:u-1",
        "https://example.com/spec/sealed-envelope/0.1",
        json!({}),
    ))
    .unwrap();

    let ConsumeOutcome::Rejected(error) = deliver(&inbound, msg, &dispatcher).await else {
        panic!("expected a rejection");
    };
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::UnsupportedType)
    );
    assert_eq!(guard.len(), 0);
    // The error document goes into a v1 message as it is.
    build_message(&error).unwrap();
}
