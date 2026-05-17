//! End-to-end roundtrip: alice authcrypts a Trust Task document for bob,
//! bob unpacks it, and the framework's TransportHandler pipeline applies
//! SPEC.md §4.8.1 precedence on the verified peer DID.
//!
//! Runs entirely in-process using `affinidi-messaging-didcomm`'s
//! `PrivateIdentity::generate` — no mediator, no network, no
//! configuration file.

use affinidi_messaging_didcomm::{identity::PrivateIdentity, DIDCommAgent};
use serde::{Deserialize, Serialize};
use trust_tasks_didcomm::{
    pack_trust_task, unpack_trust_task, DidcommError, BINDING_URI, ENVELOPE_TYPE,
};
use trust_tasks_rs::{Payload, TransportHandler, TrustTask};

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

fn grant_doc(issuer: &str, recipient: &str) -> TrustTask<GrantPayload> {
    let mut doc = TrustTask::for_payload(
        "urn:uuid:test-grant",
        GrantPayload {
            subject: "did:web:carol.example".into(),
            role: "moderator".into(),
        },
    );
    doc.issuer = Some(issuer.to_string());
    doc.recipient = Some(recipient.to_string());
    doc.issued_at = Some(chrono::Utc::now());
    doc
}

#[test]
fn authcrypt_roundtrip_with_verified_sender() {
    let p = setup_agents();

    let doc = grant_doc(&p.alice_did, &p.bob_did);
    let wire = pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");

    let (received, handler) =
        unpack_trust_task::<GrantPayload>(&wire, &p.bob_agent, Some(&p.alice_did)).expect("unpack");

    assert_eq!(received.payload, doc.payload);
    assert_eq!(received.id, doc.id);
    assert_eq!(received.issuer.as_deref(), Some(p.alice_did.as_str()));
    assert_eq!(received.recipient.as_deref(), Some(p.bob_did.as_str()));

    assert_eq!(handler.binding_uri(), BINDING_URI);
    assert_eq!(handler.peer(), Some(p.alice_did.as_str()));
    assert_eq!(handler.local(), Some(p.bob_did.as_str()));

    let resolved = handler
        .resolve_parties(&received)
        .expect("identity consistent");
    assert_eq!(resolved.issuer.as_deref(), Some(p.alice_did.as_str()));
    assert_eq!(resolved.recipient.as_deref(), Some(p.bob_did.as_str()));
}

#[test]
fn forged_in_band_issuer_triggers_identity_mismatch() {
    let p = setup_agents();

    let doc = grant_doc("did:web:attacker.example", &p.bob_did);
    let wire = pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");

    let (received, handler) =
        unpack_trust_task::<GrantPayload>(&wire, &p.bob_agent, Some(&p.alice_did)).expect("unpack");
    assert_eq!(received.issuer.as_deref(), Some("did:web:attacker.example"));
    assert_eq!(handler.peer(), Some(p.alice_did.as_str()));

    let err = handler.resolve_parties(&received).unwrap_err();
    let display = err.to_string();
    assert!(
        display.contains("issuer"),
        "expected IssuerMismatch, got: {display}"
    );
}

#[test]
fn wire_envelope_carries_framework_type() {
    let p = setup_agents();
    let doc = grant_doc(&p.alice_did, &p.bob_did);
    let wire = pack_trust_task(&doc, &p.alice_agent, &p.alice_did, &p.bob_did).expect("pack");

    assert_eq!(
        ENVELOPE_TYPE,
        "https://trusttasks.org/binding/didcomm/0.1/envelope"
    );
    assert!(wire.contains("ciphertext"), "expected a JWE on the wire");
}

#[test]
fn rejects_envelopes_with_wrong_type() {
    use affinidi_messaging_didcomm::Message;
    use serde_json::json;

    let p = setup_agents();

    let msg = Message::new(
        "https://example.com/other-protocol/v1",
        json!({"hello": "world"}),
    )
    .from(p.alice_did.clone())
    .to(vec![p.bob_did.clone()]);
    let wire = p
        .alice_agent
        .pack_authcrypt(&msg, &p.alice_did, &p.bob_did)
        .expect("pack");

    let err =
        unpack_trust_task::<GrantPayload>(&wire, &p.bob_agent, Some(&p.alice_did)).unwrap_err();
    assert!(matches!(err, DidcommError::WrongEnvelopeType(_)));
}
