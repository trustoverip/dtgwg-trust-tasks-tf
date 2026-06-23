//! End-to-end roundtrip: alice TSP-seals a Trust Task document for bob, bob
//! unpacks it, and the framework's TransportHandler pipeline applies SPEC.md
//! §4.8.1 precedence on the authenticated peer VID.
//!
//! Runs entirely in-process using `affinidi-tsp`'s `PrivateVid::generate` — no
//! mediator, no network, no configuration file.

use affinidi_tsp::{MessageType, PrivateVid, message::direct};
use serde::{Deserialize, Serialize};
use serde_json::json;
use trust_tasks_rs::{Payload, TransportHandler, TrustTask};
use trust_tasks_tsp::{BINDING_URI, ENVELOPE_TYPE, TspError, pack_trust_task, unpack_trust_task};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GrantPayload {
    subject: String,
    role: String,
}

impl Payload for GrantPayload {
    const TYPE_URI: &'static str = "https://example.com/spec/grant/0.1";
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
fn direct_roundtrip_with_verified_sender() {
    let p = setup();

    let doc = grant_doc(&p.alice.id, &p.bob.id);
    let wire = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

    let (received, handler) =
        unpack_trust_task::<GrantPayload>(&wire, &p.bob, &p.alice.to_resolved()).expect("unpack");

    assert_eq!(received.payload, doc.payload);
    assert_eq!(received.id, doc.id);
    assert_eq!(received.issuer.as_deref(), Some(p.alice.id.as_str()));
    assert_eq!(received.recipient.as_deref(), Some(p.bob.id.as_str()));

    assert_eq!(handler.binding_uri(), BINDING_URI);
    assert_eq!(handler.peer(), Some(p.alice.id.as_str()));
    assert_eq!(handler.local(), Some(p.bob.id.as_str()));

    let resolved = handler
        .resolve_parties(&received)
        .expect("identity consistent");
    assert_eq!(resolved.issuer.as_deref(), Some(p.alice.id.as_str()));
    assert_eq!(resolved.recipient.as_deref(), Some(p.bob.id.as_str()));
}

#[test]
fn forged_in_band_issuer_triggers_identity_mismatch() {
    let p = setup();

    // The document claims an issuer that is not the VID that actually sealed it.
    let doc = grant_doc("did:web:attacker.example", &p.bob.id);
    let wire = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

    let (received, handler) =
        unpack_trust_task::<GrantPayload>(&wire, &p.bob, &p.alice.to_resolved()).expect("unpack");
    assert_eq!(received.issuer.as_deref(), Some("did:web:attacker.example"));
    assert_eq!(handler.peer(), Some(p.alice.id.as_str()));

    // §4.8.1: in-band issuer must match the transport-authenticated sender.
    let err = handler.resolve_parties(&received).unwrap_err();
    assert!(
        err.to_string().contains("issuer"),
        "expected an issuer mismatch, got: {err}"
    );
}

#[test]
fn envelope_is_sealed_and_carries_the_framework_type() {
    assert_eq!(ENVELOPE_TYPE, "https://trusttasks.org/binding/tsp/0.1/envelope");

    let p = setup();
    let doc = grant_doc(&p.alice.id, &p.bob.id);
    let wire = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

    assert!(affinidi_tsp::is_tsp(&wire), "wire is a TSP message");
    // The envelope is encrypted — its plaintext type must not appear on the wire.
    assert!(
        !String::from_utf8_lossy(&wire).contains(ENVELOPE_TYPE),
        "the envelope must be sealed, not in cleartext"
    );
}

#[test]
fn rejects_a_non_trust_task_envelope() {
    let p = setup();

    // A valid TSP message whose payload is not a Trust Tasks envelope.
    let payload = serde_json::to_vec(&json!({
        "type": "https://example.com/some-other-protocol",
        "document": { "hello": "world" }
    }))
    .unwrap();
    let wire = direct::pack(
        &payload,
        MessageType::Direct,
        &p.alice.id,
        &p.bob.id,
        &p.alice.signing_key,
        &p.alice.decryption_key,
        &p.bob.to_resolved().encryption_key,
    )
    .unwrap()
    .bytes;

    let err = unpack_trust_task::<GrantPayload>(&wire, &p.bob, &p.alice.to_resolved()).unwrap_err();
    assert!(
        matches!(err, TspError::WrongEnvelopeType(_)),
        "expected WrongEnvelopeType, got: {err}"
    );
}

#[test]
fn verifying_against_the_wrong_sender_fails() {
    let p = setup();
    let carol = PrivateVid::generate("did:example:carol");

    let doc = grant_doc(&p.alice.id, &p.bob.id);
    let wire = pack_trust_task(&doc, &p.alice, &p.bob.to_resolved()).expect("pack");

    // bob tries to verify alice's message against carol's keys — the authenticated
    // seal does not check out.
    let err =
        unpack_trust_task::<GrantPayload>(&wire, &p.bob, &carol.to_resolved()).unwrap_err();
    assert!(
        matches!(err, TspError::Upstream(_) | TspError::SenderMismatch { .. }),
        "expected an authentication failure, got: {err}"
    );
}
