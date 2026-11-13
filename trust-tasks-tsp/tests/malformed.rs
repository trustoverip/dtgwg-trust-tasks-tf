//! Binding `tsp/0.1` §4: a `document` that fails to deserialise is
//! `malformedRequest` — and the sender is authenticated by the seal, so
//! [`TspConsumer`] answers it as [`ConsumeOutcome::Rejected`], addressed to
//! the verified sender, rather than returning an error with nothing to answer
//! (#1096). The Go consumer does the same since #1226.

use affinidi_tsp::message::direct;
use affinidi_tsp::{MessageType, PrivateVid};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorResponse, NoValidator, Payload, PayloadPolicy,
    ProofPolicy, ProofVerifier, ResolvedParties, StandardCode, TrustTask, TrustTaskCode,
    VerificationError,
};
use trust_tasks_tsp::{TspConsumer, TspError, ENVELOPE_TYPE};

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

/// Seal `document` from alice to bob in this binding's envelope, whatever it
/// holds — `pack_trust_task` only seals well-formed documents.
fn seal(p: &Pair, document: Value) -> Vec<u8> {
    let bob = p.bob.to_resolved();
    let payload = serde_json::to_vec(&json!({ "type": ENVELOPE_TYPE, "document": document }))
        .expect("envelope");
    direct::pack(
        &payload,
        MessageType::Direct,
        &p.alice.id,
        &bob.id,
        &p.alice.signing_key,
        &bob.encryption_key,
    )
    .expect("seal")
    .bytes
}

fn document(p: &Pair, payload: Value) -> Value {
    json!({
        "id": "urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b",
        "type": GrantPayload::TYPE_URI,
        "threadId": "thread-1",
        // An in-band issuer nothing has checked — never the addressee.
        "issuer": "did:example:somebody-else",
        "recipient": p.bob.id,
        "issuedAt": Utc::now().to_rfc3339(),
        "payload": payload,
    })
}

async fn receive(
    inbound: &TspConsumer,
    p: &Pair,
    wire: &[u8],
) -> Result<ConsumeOutcome<GrantResponse>, TspError> {
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
            |doc, _parties| async move {
                Ok(doc.respond_with("urn:uuid:resp-1", GrantResponse { granted: true }))
            },
        )
        .await
}

fn assert_answered(p: &Pair, error: &ErrorResponse) {
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::MalformedRequest)
    );
    assert_eq!(error.recipient.as_deref(), Some(p.alice.id.as_str()));
    assert_eq!(error.issuer.as_deref(), Some(p.bob.id.as_str()));
    assert_eq!(error.thread_id.as_deref(), Some("thread-1"));
    let in_response_to = error.payload.in_response_to.as_ref().expect("inResponseTo");
    assert_eq!(
        in_response_to.id.as_deref(),
        Some("urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b")
    );
}

#[tokio::test]
async fn a_payload_that_does_not_deserialise_is_answered_before_the_record() {
    let p = setup();
    let inbound = TspConsumer::new();
    let wire = seal(&p, document(&p, json!({ "subject": "did:example:carol" })));
    match receive(&inbound, &p, &wire)
        .await
        .expect("answered, not Err")
    {
        ConsumeOutcome::Rejected(error) => assert_answered(&p, &error),
        other => panic!("expected Rejected, got {other:?}"),
    }

    // Not recorded: the corrected document under the same id executes rather
    // than being refused `idConflict`.
    let mut fixed = document(
        &p,
        json!({ "subject": "did:example:carol", "role": "reader" }),
    );
    fixed["issuer"] = json!(p.alice.id);
    let fixed = seal(&p, fixed);
    let outcome = receive(&inbound, &p, &fixed).await.expect("opens");
    assert!(matches!(outcome, ConsumeOutcome::Handled(_)), "{outcome:?}");
}

#[tokio::test]
async fn a_framework_member_that_does_not_deserialise_is_answered() {
    let p = setup();
    let inbound = TspConsumer::new();
    let mut doc = document(
        &p,
        json!({ "subject": "did:example:carol", "role": "reader" }),
    );
    doc["issuedAt"] = json!(42);
    match receive(&inbound, &p, &seal(&p, doc))
        .await
        .expect("answered, not Err")
    {
        ConsumeOutcome::Rejected(error) => {
            assert_answered(&p, &error);
            let message = error.payload.message.unwrap_or_default();
            assert!(!message.contains("line"), "deserializer detail: {message}");
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[tokio::test]
async fn a_document_with_no_id_to_answer_is_still_an_error() {
    let p = setup();
    let inbound = TspConsumer::new();
    let wire = seal(&p, json!({ "payload": {} }));
    assert!(matches!(
        receive(&inbound, &p, &wire).await,
        Err(TspError::InvalidBody(_))
    ));
}

#[tokio::test]
async fn the_dispatched_path_answers_an_unparsable_document_too() {
    let p = setup();
    let inbound = TspConsumer::new();
    let dispatcher: AsyncDispatcher<ResolvedParties, Result<TrustTask<Value>, ErrorResponse>> =
        AsyncDispatcher::new().on_async::<GrantPayload, _, _>(|_doc, _parties| async move {
            panic!("a refused document must not reach the handler")
        });
    let mut doc = document(
        &p,
        json!({ "subject": "did:example:carol", "role": "reader" }),
    );
    doc["recipient"] = json!(["not", "a", "string"]);

    let outcome = inbound
        .receive_dispatched::<ResolvedParties, Value, NoVerifier, NoValidator>(
            &seal(&p, doc),
            &p.bob,
            &p.alice.to_resolved(),
            &p.bob.id,
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
        ConsumeOutcome::Rejected(error) => assert_answered(&p, &error),
        other => panic!("expected Rejected, got {other:?}"),
    }
}
