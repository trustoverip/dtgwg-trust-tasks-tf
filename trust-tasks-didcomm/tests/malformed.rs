//! Binding `didcomm/0.2` §4: a `body` that fails to deserialise, and a §3.1
//! thread disagreement, are `malformedRequest` — and the sender is
//! authenticated, so [`DidcommConsumer`] answers them as
//! [`ConsumeOutcome::Rejected`], addressed to the verified sender, rather than
//! returning an error with nothing to answer (#1096).

use affinidi_messaging_didcomm::{identity::PrivateIdentity, DIDCommAgent, Message};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_didcomm::{DidcommConsumer, DidcommError, ENVELOPE_TYPE};
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorResponse, NoValidator, Payload, PayloadPolicy,
    ProofPolicy, ProofVerifier, ResolvedParties, StandardCode, TrustTask, TrustTaskCode,
    VerificationError,
};

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
    alice_did: String,
    bob_did: String,
    alice_agent: DIDCommAgent,
    bob_agent: DIDCommAgent,
}

fn setup_agents() -> Pair {
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

/// Authcrypt `body` from alice to bob in this binding's envelope, with `thid`.
fn seal(p: &Pair, body: Value, thid: Option<&str>) -> String {
    let mut msg = Message::new(ENVELOPE_TYPE, body)
        .from(p.alice_did.clone())
        .to(vec![p.bob_did.clone()]);
    if let Some(thid) = thid {
        msg = msg.thid(thid.to_string());
    }
    p.alice_agent
        .pack_authcrypt(&msg, &p.alice_did, &p.bob_did)
        .expect("pack")
}

fn body(p: &Pair, payload: Value) -> Value {
    json!({
        "id": "urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b",
        "type": GrantPayload::TYPE_URI,
        "threadId": "thread-1",
        // An in-band issuer nothing has checked — never the addressee.
        "issuer": "did:web:somebody-else.example",
        "recipient": p.bob_did,
        "issuedAt": Utc::now().to_rfc3339(),
        "payload": payload,
    })
}

async fn receive(
    inbound: &DidcommConsumer,
    p: &Pair,
    wire: &str,
) -> Result<ConsumeOutcome<GrantResponse>, DidcommError> {
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
            |_doc, _parties| async move { panic!("a refused body must not reach the handler") },
        )
        .await
}

/// A malformed request answered to the authenticated sender, from bob.
fn assert_answered(p: &Pair, error: &ErrorResponse) {
    assert_eq!(
        error.payload.code,
        TrustTaskCode::Standard(StandardCode::MalformedRequest)
    );
    assert_eq!(error.recipient.as_deref(), Some(p.alice_did.as_str()));
    assert_eq!(error.issuer.as_deref(), Some(p.bob_did.as_str()));
    assert_eq!(error.thread_id.as_deref(), Some("thread-1"));
    let in_response_to = error.payload.in_response_to.as_ref().expect("inResponseTo");
    assert_eq!(
        in_response_to.id.as_deref(),
        Some("urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b")
    );
}

#[tokio::test]
async fn a_payload_that_does_not_deserialise_is_answered() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let wire = seal(
        &p,
        body(&p, json!({ "subject": "did:web:carol.example" })),
        None,
    );

    match receive(&inbound, &p, &wire)
        .await
        .expect("answered, not Err")
    {
        ConsumeOutcome::Rejected(error) => assert_answered(&p, &error),
        other => panic!("expected Rejected, got {other:?}"),
    }

    // Refused before the record: the id is not burned, so the corrected
    // document under the same id executes.
    let mut fixed = body(
        &p,
        json!({ "subject": "did:web:carol.example", "role": "reader" }),
    );
    fixed["issuer"] = json!(p.alice_did);
    let fixed = seal(&p, fixed, None);
    let outcome = inbound
        .receive::<GrantPayload, GrantResponse, NoVerifier, NoValidator, _, _>(
            &fixed,
            &p.bob_agent,
            Some(&p.alice_did),
            &p.bob_did,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            Utc::now(),
            || "urn:uuid:error-2".to_string(),
            |doc, _parties| async move {
                Ok(doc.respond_with("urn:uuid:resp-1", GrantResponse { granted: true }))
            },
        )
        .await
        .expect("opens");
    assert!(matches!(outcome, ConsumeOutcome::Handled(_)), "{outcome:?}");
}

#[tokio::test]
async fn a_framework_member_that_does_not_deserialise_is_answered() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let mut doc = body(
        &p,
        json!({ "subject": "did:web:carol.example", "role": "reader" }),
    );
    doc["issuedAt"] = json!(42);
    let wire = seal(&p, doc, None);

    match receive(&inbound, &p, &wire)
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
async fn a_thread_disagreement_is_answered() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let wire = seal(
        &p,
        body(
            &p,
            json!({ "subject": "did:web:carol.example", "role": "reader" }),
        ),
        Some("some-other-thread"),
    );

    match receive(&inbound, &p, &wire)
        .await
        .expect("answered, not Err")
    {
        ConsumeOutcome::Rejected(error) => assert_answered(&p, &error),
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[tokio::test]
async fn a_body_with_no_id_to_answer_is_still_an_error() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let wire = seal(&p, json!({ "payload": {} }), None);

    assert!(matches!(
        receive(&inbound, &p, &wire).await,
        Err(DidcommError::InvalidBody(_))
    ));
}

#[tokio::test]
async fn the_dispatched_path_answers_an_unparsable_body_too() {
    let p = setup_agents();
    let inbound = DidcommConsumer::new();
    let dispatcher: AsyncDispatcher<ResolvedParties, Result<TrustTask<Value>, ErrorResponse>> =
        AsyncDispatcher::new().on_async::<GrantPayload, _, _>(|_doc, _parties| async move {
            panic!("a refused body must not reach the handler")
        });
    let mut doc = body(
        &p,
        json!({ "subject": "did:web:carol.example", "role": "reader" }),
    );
    doc["threadId"] = json!(7);
    let wire = seal(&p, doc, None);

    let outcome = inbound
        .receive_dispatched::<ResolvedParties, Value, NoVerifier, NoValidator>(
            &wire,
            &p.bob_agent,
            Some(&p.alice_did),
            &p.bob_did,
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
        ConsumeOutcome::Rejected(error) => {
            assert_eq!(
                error.payload.code,
                TrustTaskCode::Standard(StandardCode::MalformedRequest)
            );
            assert_eq!(error.recipient.as_deref(), Some(p.alice_did.as_str()));
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}
