//! [`Envelope`] is the binding's envelope object without the TSP seal, for an
//! application whose own TSP stack seals and opens the message. Its bytes are
//! pinned here, and held equal to what `pack_trust_task` seals and what
//! `unpack_trust_task` opens, so the public type and the pack/unpack functions
//! cannot drift apart.

use affinidi_tsp::message::direct;
use affinidi_tsp::{MessageType, PrivateVid};
use serde_json::{json, Value};
use trust_tasks_rs::TrustTask;
use trust_tasks_tsp::{pack_trust_task, unpack_trust_task, Envelope, TspError, ENVELOPE_TYPE};

/// The envelope plaintext for [`document`], exactly as this binding has
/// sealed it since `tsp/0.1`: compact JSON, members in the order
/// `serde_json` writes them (sorted; serde_json without `preserve_order`,
/// as this workspace builds it).
const GOLDEN: &str = concat!(
    r#"{"document":{"id":"urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b","#,
    r#""issuedAt":"2026-01-01T00:00:00Z","#,
    r#""payload":{"role":"editor","subject":"did:example:carol"},"#,
    r#""type":"https://example.com/spec/grant/0.1"},"#,
    r#""type":"https://trusttasks.org/binding/tsp/0.1/envelope"}"#
);

/// A delivery id as the receipt rules want one: opaque, not derived from the
/// content.
const DELIVERY_ID: &str = "dlv-7c1d9e2a-4b6f-4f0e-8a35-0d2b9c6e1f47";

/// [`GOLDEN`] with a `deliveryId` (binding `tsp/0.2` §2.1): the same object
/// plus one member, ordered as [`GOLDEN`] is.
const GOLDEN_WITH_DELIVERY_ID: &str = concat!(
    r#"{"deliveryId":"dlv-7c1d9e2a-4b6f-4f0e-8a35-0d2b9c6e1f47","#,
    r#""document":{"id":"urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b","#,
    r#""issuedAt":"2026-01-01T00:00:00Z","#,
    r#""payload":{"role":"editor","subject":"did:example:carol"},"#,
    r#""type":"https://example.com/spec/grant/0.1"},"#,
    r#""type":"https://trusttasks.org/binding/tsp/0.1/envelope"}"#
);

fn document() -> TrustTask<Value> {
    serde_json::from_value(json!({
        "id": "urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b",
        "type": "https://example.com/spec/grant/0.1",
        "issuedAt": "2026-01-01T00:00:00Z",
        "payload": { "subject": "did:example:carol", "role": "editor" }
    }))
    .expect("document")
}

#[test]
fn the_envelope_type_is_the_binding_s() {
    assert_eq!(
        ENVELOPE_TYPE,
        "https://trusttasks.org/binding/tsp/0.1/envelope"
    );
}

#[test]
fn to_bytes_matches_the_golden_envelope() {
    let bytes = Envelope::new(&document()).expect("envelope").to_bytes();
    assert_eq!(String::from_utf8(bytes).expect("utf-8"), GOLDEN);
}

#[test]
fn to_bytes_writes_no_delivery_id_unless_one_is_set() {
    let envelope = Envelope::new(&document()).expect("envelope");
    assert_eq!(envelope.delivery_id(), None);
    assert!(!String::from_utf8(envelope.to_bytes())
        .unwrap()
        .contains("deliveryId"));
}

#[test]
fn to_bytes_matches_the_golden_envelope_with_a_delivery_id() {
    let envelope = Envelope::new(&document())
        .expect("envelope")
        .with_delivery_id(DELIVERY_ID)
        .expect("delivery id");
    assert_eq!(envelope.delivery_id(), Some(DELIVERY_ID));
    let bytes = envelope.to_bytes();
    assert_eq!(
        String::from_utf8(bytes).expect("utf-8"),
        GOLDEN_WITH_DELIVERY_ID
    );
}

#[test]
fn from_bytes_keeps_the_delivery_id() {
    let envelope = Envelope::from_bytes(GOLDEN_WITH_DELIVERY_ID.as_bytes()).expect("envelope");
    assert_eq!(envelope.delivery_id(), Some(DELIVERY_ID));
    assert_eq!(
        envelope.document(),
        &serde_json::to_value(document()).unwrap()
    );
    // Read and written back, it is the same envelope.
    assert_eq!(envelope.to_bytes(), GOLDEN_WITH_DELIVERY_ID.as_bytes());
}

/// A `tsp/0.2` producer's envelope opens with `unpack_trust_task`: the
/// envelope type is unchanged, and the delivery id stays out of the document.
#[test]
fn unpack_trust_task_opens_an_envelope_with_a_delivery_id() {
    let alice = PrivateVid::generate("did:example:alice");
    let bob = PrivateVid::generate("did:example:bob");
    let bob_resolved = bob.to_resolved();

    let wire = direct::pack(
        GOLDEN_WITH_DELIVERY_ID.as_bytes(),
        MessageType::Direct,
        &alice.id,
        &bob_resolved.id,
        &alice.signing_key,
        &bob_resolved.encryption_key,
    )
    .expect("seal")
    .bytes;
    let (doc, _) = unpack_trust_task::<Value>(&wire, &bob, &alice.to_resolved()).expect("unpack");
    assert_eq!(doc.id, document().id);
}

#[test]
fn from_bytes_reads_the_golden_envelope() {
    let envelope = Envelope::from_bytes(GOLDEN.as_bytes()).expect("envelope");
    assert_eq!(
        envelope.document(),
        &serde_json::to_value(document()).unwrap()
    );
    let doc: TrustTask<Value> = envelope.into_trust_task().expect("document");
    assert_eq!(doc.id, "urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b");
}

/// Member order and whitespace are not significant: another runtime's
/// envelope (`type` first, pretty-printed) reads the same.
#[test]
fn from_bytes_ignores_member_order_and_whitespace() {
    let other = serde_json::to_string_pretty(&json!({
        "type": ENVELOPE_TYPE,
        "document": serde_json::to_value(document()).unwrap(),
    }))
    .unwrap();
    let envelope = Envelope::from_bytes(other.as_bytes()).expect("envelope");
    assert_eq!(envelope.to_bytes(), GOLDEN.as_bytes());
}

/// The document is kept as received — what a proof is verified over — not
/// a re-serialisation of a typed copy.
#[test]
fn from_bytes_keeps_the_document_as_received() {
    let received = json!({ "id": "x", "type": "y", "payload": {}, "unknownMember": [1, 2] });
    let payload =
        serde_json::to_vec(&json!({ "type": ENVELOPE_TYPE, "document": received })).unwrap();
    let envelope = Envelope::from_bytes(&payload).expect("envelope");
    assert_eq!(envelope.into_document(), received);
}

#[test]
fn from_bytes_refuses_another_envelope_type() {
    let payload = serde_json::to_vec(&json!({
        "type": "https://trusttasks.org/binding/didcomm/0.1/envelope",
        "document": serde_json::to_value(document()).unwrap(),
    }))
    .unwrap();
    assert!(matches!(
        Envelope::from_bytes(&payload),
        Err(TspError::WrongEnvelopeType(t)) if t == "https://trusttasks.org/binding/didcomm/0.1/envelope"
    ));
}

#[test]
fn from_bytes_refuses_a_payload_that_is_not_an_envelope() {
    for payload in [
        &b"not json"[..],
        br#"{"type":"https://trusttasks.org/binding/tsp/0.1/envelope"}"#,
        br#"{"document":{}}"#,
    ] {
        assert!(matches!(
            Envelope::from_bytes(payload),
            Err(TspError::InvalidBody(_))
        ));
    }
}

/// `pack_trust_task` seals exactly `Envelope::to_bytes`.
#[test]
fn pack_trust_task_seals_the_envelope_bytes() {
    let alice = PrivateVid::generate("did:example:alice");
    let bob = PrivateVid::generate("did:example:bob");

    let wire = pack_trust_task(&document(), &alice, &bob.to_resolved()).expect("pack");
    let opened = direct::unpack(&wire, &bob.decryption_key, &alice.to_resolved().signing_key)
        .expect("unseal");

    assert_eq!(opened.payload, GOLDEN.as_bytes());
}

/// What an application's own stack seals from `Envelope::to_bytes` opens
/// with `unpack_trust_task`, and the reverse.
#[test]
fn envelope_bytes_interoperate_with_unpack_trust_task() {
    let alice = PrivateVid::generate("did:example:alice");
    let bob = PrivateVid::generate("did:example:bob");
    let bob_resolved = bob.to_resolved();

    let payload = Envelope::new(&document()).expect("envelope").to_bytes();
    let wire = direct::pack(
        &payload,
        MessageType::Direct,
        &alice.id,
        &bob_resolved.id,
        &alice.signing_key,
        &bob_resolved.encryption_key,
    )
    .expect("seal")
    .bytes;
    let (doc, _) = unpack_trust_task::<Value>(&wire, &bob, &alice.to_resolved()).expect("unpack");
    assert_eq!(doc.id, document().id);

    let wire = pack_trust_task(&document(), &alice, &bob_resolved).expect("pack");
    let opened = direct::unpack(&wire, &bob.decryption_key, &alice.to_resolved().signing_key)
        .expect("unseal");
    let doc: TrustTask<Value> = Envelope::from_bytes(&opened.payload)
        .expect("envelope")
        .into_trust_task()
        .expect("document");
    assert_eq!(doc.id, document().id);
}
