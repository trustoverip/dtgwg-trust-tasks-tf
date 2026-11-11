//! `Verifier::verify_raw` refuses a `proofValue` or `did:key` multikey that
//! is not base58btc, or is over its cryptosuite's size bound — for the Ed25519
//! and NIST suites the bound the Go, TypeScript and Dart verifiers share, at
//! most 175 characters decoding to at most 128 bytes — before the Affinidi
//! crates decode it, so what such a value costs does not grow with its length.

#![cfg(feature = "affinidi")]

use std::time::{Duration, Instant};

use affinidi_secrets_resolver::secrets::Secret;
use serde_json::{json, Value};
use trust_tasks_proof::affinidi::{check_proof_bounds, sign_trust_task, SignOptions, Verifier};
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
            detail.starts_with(&format!("{member}: encoded value"))
                || detail.starts_with(&format!("{member}: not multibase")),
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

/// `multibase::decode` upstream also reads base58flickr (`Z`), base10 (`9`)
/// and base36 (`k`, `K`), whose decoders are as slow as base58btc's. None of
/// them is a base any supported cryptosuite or `did:key` uses.
#[tokio::test]
async fn other_multibase_bases_are_refused() {
    let doc = signed_doc().await;
    let verifier = Verifier::for_did_key();
    for prefix in ["Z", "9", "k", "K", "u", "f"] {
        for len in [8, 64 * 1024] {
            let value = format!("{prefix}{}", "2".repeat(len));
            assert_refused(
                verifier.verify_raw(&with_proof_value(&doc, &value)).await,
                "proofValue",
            );
            assert_refused(
                verifier.verify_raw(&with_did_key(&doc, &value)).await,
                "verificationMethod",
            );
        }
    }
}

#[tokio::test]
async fn check_proof_bounds_is_usable_on_its_own() {
    let doc = signed_doc().await;
    let proof =
        trust_tasks_proof::affinidi::parse_data_integrity_proof(&doc["proof"]).expect("proof");
    check_proof_bounds(&proof).expect("a signed proof is within bounds");
    let long = with_proof_value(&doc, &format!("z{}", "2".repeat(MAX_CHARS + 1)));
    let proof =
        trust_tasks_proof::affinidi::parse_data_integrity_proof(&long["proof"]).expect("proof");
    assert_refused(check_proof_bounds(&proof), "proofValue");
}

#[tokio::test]
async fn a_one_mib_value_is_refused_quickly() {
    let doc = signed_doc().await;
    let verifier = Verifier::for_did_key();
    let mut docs = Vec::new();
    for prefix in ["z", "Z", "k"] {
        let huge = format!("{prefix}{}", "2".repeat(1 << 20));
        docs.push((with_proof_value(&doc, &huge), "proofValue"));
        docs.push((with_did_key(&doc, &huge), "verificationMethod"));
    }
    let start = Instant::now();
    for _ in 0..30 {
        for (doc, member) in &docs {
            assert_refused(verifier.verify_raw(doc).await, member);
        }
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(1),
        "refusing 180 oversized proofs took {elapsed:?}"
    );
}

/// The post-quantum suites read far larger values than 128 bytes; their bounds
/// are their own largest signature and key, so a genuine ML-DSA proof still
/// verifies, and one over those bounds is refused just as quickly.
#[cfg(feature = "ml-dsa")]
mod ml_dsa {
    use super::*;
    use trust_tasks_proof::affinidi::CryptoSuite;

    async fn signed_ml_dsa_doc() -> Value {
        let throwaway = Secret::generate_ml_dsa_44(None, Some(&[9; 32]));
        let pk_mb = throwaway.get_public_keymultibase().expect("multikey");
        let vm = format!("did:key:{pk_mb}#{pk_mb}");
        let mut secret = Secret::generate_ml_dsa_44(Some(&vm), Some(&[9; 32]));
        secret.id = vm.clone();
        let did = vm.split('#').next().unwrap().to_string();
        let doc = json!({
            "id": "urn:uuid:base58-bounds-ml-dsa",
            "type": "https://example.com/spec/demo/0.1",
            "issuer": did,
            "recipient": "did:example:bank",
            "issuedAt": "2026-07-29T09:00:00Z",
            "payload": { "subject": "did:example:alice" }
        });
        sign_trust_task(
            &doc,
            &secret,
            SignOptions::new().with_cryptosuite(CryptoSuite::MlDsa44Jcs2024),
        )
        .await
        .expect("sign")
    }

    #[tokio::test]
    async fn an_ml_dsa_44_proof_verifies() {
        let doc = signed_ml_dsa_doc().await;
        let proof_value = doc["proof"]["proofValue"].as_str().unwrap();
        assert!(
            proof_value.len() > 3000,
            "ML-DSA-44 signature is 2420 bytes"
        );
        Verifier::for_did_key()
            .verify_raw(&doc)
            .await
            .expect("ML-DSA-44 proof verifies");
    }

    #[tokio::test]
    async fn an_ml_dsa_value_over_its_bound_is_refused_quickly() {
        let doc = signed_ml_dsa_doc().await;
        let verifier = Verifier::for_did_key();
        // Over ML-DSA-87's 4627-byte signature, and ML-DSA-87's 2594-byte
        // multikey, by one character's worth.
        let long_sig = format!("z{}", "z".repeat(6320));
        let long_key = format!("z{}", "z".repeat(3544));
        assert_refused(
            verifier
                .verify_raw(&with_proof_value(&doc, &long_sig))
                .await,
            "proofValue",
        );
        let mut keyed = with_did_key(&doc, &long_key);
        keyed["proof"]["cryptosuite"] = json!("mldsa44-jcs-2024");
        assert_refused(verifier.verify_raw(&keyed).await, "verificationMethod");

        let huge = format!("z{}", "2".repeat(1 << 20));
        let start = Instant::now();
        for _ in 0..100 {
            assert_refused(
                verifier.verify_raw(&with_proof_value(&doc, &huge)).await,
                "proofValue",
            );
        }
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_secs(1), "took {elapsed:?}");
    }
}
