//! `Verifier::verify_raw` refuses a base58btc `proofValue` or `did:key`
//! multikey over the size bounds the Go, TypeScript and Dart verifiers share —
//! at most 175 characters, decoding to at most 128 bytes — before the Affinidi
//! crates decode it, so what such a value costs does not grow with its length.

#![cfg(feature = "affinidi")]

use std::time::{Duration, Instant};

use affinidi_secrets_resolver::secrets::Secret;
use serde_json::{json, Value};
use trust_tasks_proof::affinidi::{sign_trust_task, SignOptions, Verifier};
use trust_tasks_rs::VerificationError;

const MAX_CHARS: usize = 175;
const MAX_BYTES: usize = 128;

async fn signed_doc() -> Value {
    let throwaway = Secret::generate_ed25519(None, Some(&[5; 32]));
    let pk_mb = throwaway.get_public_keymultibase().expect("multikey");
    let vm = format!("did:key:{pk_mb}#{pk_mb}");
    let mut secret = Secret::generate_ed25519(Some(&vm), Some(&[5; 32]));
    secret.id = vm.clone();
    let did = vm.split('#').next().unwrap().to_string();
    let doc = json!({
        "id": "urn:uuid:base58-bounds-1",
        "type": "https://example.com/spec/demo/0.1",
        "issuer": did,
        "recipient": "did:example:bank",
        "issuedAt": "2026-07-29T09:00:00Z",
        "payload": { "subject": "did:example:alice" }
    });
    sign_trust_task(&doc, &secret, SignOptions::new())
        .await
        .expect("sign")
}

fn with_proof_value(doc: &Value, value: &str) -> Value {
    let mut doc = doc.clone();
    doc["proof"]["proofValue"] = json!(value);
    doc
}

fn with_did_key(doc: &Value, multibase: &str) -> Value {
    let mut doc = doc.clone();
    let did = format!("did:key:{multibase}");
    doc["issuer"] = json!(did);
    doc["proof"]["verificationMethod"] = json!(format!("{did}#{multibase}"));
    doc
}

fn assert_refused(result: Result<(), VerificationError>, member: &str) {
    match result {
        Err(VerificationError::MalformedProof(detail)) => assert!(
            detail.starts_with(&format!("{member}: base58 value")),
            "unexpected detail: {detail}"
        ),
        other => panic!("{member}: expected a bounds refusal, got {other:?}"),
    }
}

#[tokio::test]
async fn a_document_within_the_bounds_still_verifies() {
    let doc = signed_doc().await;
    Verifier::for_did_key()
        .verify_raw(&doc)
        .await
        .expect("verifies");
}

#[tokio::test]
async fn values_over_the_bounds_are_refused() {
    let doc = signed_doc().await;
    let verifier = Verifier::for_did_key();
    let cases = [
        // One character too long.
        format!("z{}", "2".repeat(MAX_CHARS + 1)),
        // Within the length bound, still one byte too large.
        format!("z{}", "z".repeat(MAX_CHARS)),
        // Too many leading zero bytes.
        format!("z{}", "1".repeat(MAX_BYTES + 1)),
    ];
    for value in &cases {
        assert_refused(
            verifier.verify_raw(&with_proof_value(&doc, value)).await,
            "proofValue",
        );
        assert_refused(
            verifier.verify_raw(&with_did_key(&doc, value)).await,
            "verificationMethod",
        );
    }
}

#[tokio::test]
async fn a_one_mib_value_is_refused_quickly() {
    let doc = signed_doc().await;
    let verifier = Verifier::for_did_key();
    let huge = format!("z{}", "2".repeat(1 << 20));
    let docs = [with_proof_value(&doc, &huge), with_did_key(&doc, &huge)];
    let start = Instant::now();
    for _ in 0..100 {
        assert_refused(verifier.verify_raw(&docs[0]).await, "proofValue");
        assert_refused(verifier.verify_raw(&docs[1]).await, "verificationMethod");
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(1),
        "refusing 200 oversized proofs took {elapsed:?}"
    );
}
