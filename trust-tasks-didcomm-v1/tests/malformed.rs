//! Binding `didcomm-v1/0.2` §4: an attachment that does not deserialise as a
//! document, and a `~thread` disagreement, are `malformedRequest` — and the
//! sender is authenticated, so [`DidcommV1Consumer`] answers them as
//! [`ConsumeOutcome::Rejected`], addressed to that sender, rather than
//! returning an error with nothing to answer (#1096).

use affinidi_messaging_didcomm_v1::{Did, MessageV1, UnpackResult, Verkey};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_didcomm_v1::{DidcommV1Consumer, DidcommV1Error, ATTACHMENT_ID, ENVELOPE_TYPE};
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorResponse, NoValidator, Payload, PayloadPolicy,
    ProofPolicy, ProofVerifier, ResolvedParties, StandardCode, TrustTask, TrustTaskCode,
    VerificationError,
};

const ALICE: &str = "did:sov:alice";
const BOB: &str = "did:sov:bob";
const DOC_ID: &str = "urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

fn document(payload: Value) -> Value {
    json!({
        "id": DOC_ID,
        "type": GrantPayload::TYPE_URI,
        "threadId": "thread-0001",
        // An in-band issuer nothing has checked — never the addressee.
        "issuer": "did:sov:somebody-else",
        "recipient": BOB,
        "issuedAt": Utc::now().to_rfc3339(),
        "payload": payload,
    })
}

/// A v1 message of the binding's type carrying `document`, delivered authcrypt
/// from alice to bob.
fn delivered(document: Value, thid: Option<&str>) -> UnpackResult {
    let mut msg = MessageV1::new(
        ENVELOPE_TYPE,
        json!({
            "~attach": [{
                "@id": ATTACHMENT_ID,
                "mime-type": "application/json",
                "data": { "json": document }
            }],
        }),
    )
    .expect("message");
    if let Some(thid) = thid {
        msg = msg.thid(thid);
    }
    UnpackResult::Authcrypt {
        message: msg,
        sender: Did::parse(ALICE).expect("sender DID"),
        sender_verkey: Verkey::from_bytes([7u8; 32]),
        recipient: Did::parse(BOB).expect("recipient DID"),
        recipient_verkey: Verkey::from_bytes([9u8; 32]),
    }
}

async fn receive(
    inbound: &DidcommV1Consumer,
    unpacked: UnpackResult,
) -> Result<ConsumeOutcome<GrantResponse>, DidcommV1Error> {
    inbound
        .receive::<GrantPayload, GrantResponse, NoVerifier, NoValidator, _, _>(
            unpacked,
            BOB,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            Utc::now(),
            || "urn:uuid:error-1".to_string(),
            |doc, _parties| async move {
                Ok(doc.respond_with("urn:uuid:resp-1", GrantResponse { granted: true }))
            },
        )
        .await
}

fn assert_answered(error: &ErrorResponse) {
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::MalformedRequest)
    );
    assert_eq!(error.recipient.as_deref(), Some(ALICE));
    assert_eq!(error.issuer.as_deref(), Some(BOB));
    assert_eq!(error.thread_id.as_deref(), Some("thread-0001"));
    let in_response_to = error.payload.in_response_to.as_ref().expect("inResponseTo");
    assert_eq!(in_response_to.id.as_deref(), Some(DOC_ID));
}

#[tokio::test]
async fn an_attachment_that_does_not_deserialise_is_answered_before_the_record() {
    let inbound = DidcommV1Consumer::new();
    let malformed = delivered(document(json!({ "subject": "did:sov:carol" })), None);
    match receive(&inbound, malformed)
        .await
        .expect("answered, not Err")
    {
        ConsumeOutcome::Rejected(error) => assert_answered(&error),
        other => panic!("expected Rejected, got {other:?}"),
    }

    // Not recorded: the corrected document under the same id executes rather
    // than being refused `idConflict`.
    let mut fixed = document(json!({ "subject": "did:sov:carol", "role": "reader" }));
    fixed["issuer"] = json!(ALICE);
    let fixed = delivered(fixed, None);
    let outcome = receive(&inbound, fixed).await.expect("opens");
    assert!(matches!(outcome, ConsumeOutcome::Handled(_)), "{outcome:?}");
}

#[tokio::test]
async fn a_framework_member_that_does_not_deserialise_is_answered() {
    let inbound = DidcommV1Consumer::new();
    let mut doc = document(json!({ "subject": "did:sov:carol", "role": "reader" }));
    doc["issuedAt"] = json!(42);
    match receive(&inbound, delivered(doc, None))
        .await
        .expect("answered, not Err")
    {
        ConsumeOutcome::Rejected(error) => {
            assert_answered(&error);
            let message = error.payload.message.unwrap_or_default();
            assert!(!message.contains("line"), "deserializer detail: {message}");
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[tokio::test]
async fn a_thread_disagreement_is_answered() {
    let inbound = DidcommV1Consumer::new();
    let unpacked = delivered(
        document(json!({ "subject": "did:sov:carol", "role": "reader" })),
        Some("thread-0002"),
    );
    match receive(&inbound, unpacked)
        .await
        .expect("answered, not Err")
    {
        ConsumeOutcome::Rejected(error) => assert_answered(&error),
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[tokio::test]
async fn an_attachment_with_no_id_to_answer_is_still_an_error() {
    let inbound = DidcommV1Consumer::new();
    assert!(matches!(
        receive(&inbound, delivered(json!({ "payload": {} }), None)).await,
        Err(DidcommV1Error::InvalidDocument(_))
    ));
}

#[tokio::test]
async fn the_dispatched_path_answers_an_unparsable_attachment_too() {
    let inbound = DidcommV1Consumer::new();
    let dispatcher: AsyncDispatcher<ResolvedParties, Result<TrustTask<Value>, ErrorResponse>> =
        AsyncDispatcher::new().on_async::<GrantPayload, _, _>(|_doc, _parties| async move {
            panic!("a refused document must not reach the handler")
        });
    let mut doc = document(json!({ "subject": "did:sov:carol", "role": "reader" }));
    doc["parentThreadId"] = json!(false);

    let outcome = inbound
        .receive_dispatched::<ResolvedParties, Value, NoVerifier, NoValidator>(
            delivered(doc, None),
            BOB,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            &dispatcher,
            Utc::now(),
            || "urn:uuid:error-1".to_string(),
            |parties| parties,
        )
        .await
        .expect("answered, not Err");
    match outcome {
        ConsumeOutcome::Rejected(error) => assert_answered(&error),
        other => panic!("expected Rejected, got {other:?}"),
    }
}
