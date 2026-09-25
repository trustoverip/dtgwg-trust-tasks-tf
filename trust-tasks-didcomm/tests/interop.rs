//! Cross-library DIDComm authcrypt interop: the **Go** binding
//! (`trust-tasks-go/didcomm`) rolls its own ECDH-1PU authcrypt on the Go
//! standard library. This test unpacks a JWE that Go produced, using the same
//! `affinidi-messaging-didcomm` this crate is built on, and checks the sender is
//! authenticated and the plaintext recovered — proving the Go binding is
//! wire-compatible with this one (Go → affinidi direction; the affinidi → Go
//! direction is asserted by a Go test over an affinidi-produced fixture).
//!
//! The fixture is a Go-packed JWE with fixed X25519 keys; regenerate it with the
//! generator described in `trust-tasks-go/didcomm/README.md`.

use affinidi_crypto::jose::key_agreement::{Curve, PrivateKeyAgreement, PublicKeyAgreement};
use affinidi_messaging_didcomm::jwe::decrypt::{decrypt_bound, SenderKey};

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn affinidi_unpacks_a_go_produced_jwe() {
    let raw = include_str!("fixtures/go-authcrypt-interop.json");
    let f: serde_json::Value = serde_json::from_str(raw).unwrap();

    let recipient_private = PrivateKeyAgreement::from_raw_bytes(
        Curve::X25519,
        &unhex(f["recipientPrivX25519"].as_str().unwrap()),
    )
    .unwrap();
    let sender_public = PublicKeyAgreement::from_raw_bytes(
        Curve::X25519,
        &unhex(f["senderPubX25519"].as_str().unwrap()),
    )
    .unwrap();
    let recipient_kid = f["recipientKid"].as_str().unwrap();
    let sender_kid = f["senderKid"].as_str().unwrap();
    let jwe = f["jwe"].as_str().unwrap();

    let out = decrypt_bound(
        jwe,
        recipient_kid,
        &recipient_private,
        Some(SenderKey::new(sender_kid, &sender_public)),
    )
    .expect("affinidi failed to decrypt the Go-produced JWE");

    assert!(out.authenticated, "authcrypt sender not authenticated");
    assert_eq!(
        out.sender_kid.as_deref(),
        Some(sender_kid),
        "authenticated sender kid mismatch"
    );

    let plaintext: serde_json::Value = serde_json::from_slice(&out.plaintext).unwrap();
    assert_eq!(
        plaintext["type"],
        "https://trusttasks.org/binding/didcomm/0.1/envelope"
    );
    assert_eq!(plaintext["body"]["payload"]["text"], "hello from go");
}
