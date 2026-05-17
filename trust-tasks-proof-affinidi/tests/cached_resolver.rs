//! End-to-end test for [`CachedDidResolver`]: signs a Trust Task document
//! against a generated Ed25519 key, publishes a matching `did:key`
//! identifier, and verifies the proof through `affinidi-did-resolver-cache-sdk`
//! → `CachedDidResolver` → `AffinidiProofVerifier`.
//!
//! Runs entirely with the SDK's local mode (no network) so it stays a
//! self-contained unit test of the wiring.

use std::sync::Arc;

use affinidi_data_integrity::{DataIntegrityProof, SignOptions};
use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
use affinidi_secrets_resolver::secrets::Secret;
use serde::{Deserialize, Serialize};
use trust_tasks_proof_affinidi::{AffinidiProofVerifier, CachedDidResolver};
use trust_tasks_rs::{Payload, Proof, ProofVerifier, TrustTask, VerificationError};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Heartbeat {
    issued_by: String,
    nonce: u64,
}

impl Payload for Heartbeat {
    const TYPE_URI: &'static str = "https://example.com/spec/heartbeat/0.1";
}

/// Spin up a local-mode resolver cache. The default builder enables the
/// `local` feature on `affinidi-did-resolver-cache-sdk` which ships with
/// in-process resolvers for `did:key`, `did:peer`, `did:jwk`, etc.
async fn local_resolver() -> CachedDidResolver {
    let config = DIDCacheConfigBuilder::default().build();
    let client = DIDCacheClient::new(config)
        .await
        .expect("local DIDCacheClient");
    CachedDidResolver::new(Arc::new(client))
}

/// Build a `did:key` for a freshly-generated Ed25519 secret. The
/// returned `(secret, vm_uri)` round-trips through both `Secret::sign`
/// and the cache-backed resolver.
fn fresh_did_key() -> (Secret, String) {
    let secret_no_kid = Secret::generate_ed25519(None, Some(&[42u8; 32]));
    let pk_mb = secret_no_kid.get_public_keymultibase().expect("multikey");
    // did:key VMs put the multibase identifier in both the DID and the
    // fragment per the did:key spec.
    let vm = format!("did:key:{pk_mb}#{pk_mb}");

    // Re-mint with the correct kid so the proof's verificationMethod
    // matches what the resolver will produce when reading the DID doc.
    let mut secret = Secret::generate_ed25519(Some(&vm), Some(&[42u8; 32]));
    secret.id = vm.clone();
    (secret, vm)
}

#[tokio::test]
async fn cached_resolver_verifies_did_key_signed_document() {
    let resolver = Arc::new(local_resolver().await);
    let verifier = AffinidiProofVerifier::with_resolver(resolver);

    let (secret, vm) = fresh_did_key();
    let issuer_did = vm.split('#').next().unwrap().to_string();

    let mut doc = TrustTask::for_payload(
        "urn:uuid:cached-1",
        Heartbeat {
            issued_by: issuer_did.clone(),
            nonce: 1,
        },
    );
    doc.issuer = Some(issuer_did);
    doc.recipient = Some("did:web:auditor.example".into());
    doc.issued_at = Some(chrono::Utc::now());

    let body = serde_json::to_value(&doc).unwrap();
    let proof = DataIntegrityProof::sign(&body, &secret, SignOptions::new())
        .await
        .expect("sign");
    let proof_value = serde_json::to_value(&proof).unwrap();
    doc.proof = Some(serde_json::from_value::<Proof>(proof_value).unwrap());

    verifier.verify(&doc).await.expect("valid proof verifies");
}

#[tokio::test]
async fn cached_resolver_surfaces_resolver_error_for_unknown_method() {
    let resolver = Arc::new(local_resolver().await);
    let verifier = AffinidiProofVerifier::with_resolver(resolver);

    let (secret, _vm) = fresh_did_key();

    // Build a doc but lie about the verification method — point it at
    // a fictional did:web that the local-mode resolver can't reach.
    let mut doc = TrustTask::for_payload(
        "urn:uuid:cached-2",
        Heartbeat {
            issued_by: "did:web:nonexistent.example".into(),
            nonce: 2,
        },
    );
    doc.issuer = Some("did:web:nonexistent.example".into());
    doc.recipient = Some("did:web:auditor.example".into());
    doc.issued_at = Some(chrono::Utc::now());

    let body = serde_json::to_value(&doc).unwrap();
    let mut proof = DataIntegrityProof::sign(&body, &secret, SignOptions::new())
        .await
        .expect("sign");
    proof.verification_method = "did:web:nonexistent.example#key-0".into();
    let proof_value = serde_json::to_value(&proof).unwrap();
    doc.proof = Some(serde_json::from_value::<Proof>(proof_value).unwrap());

    let err = verifier.verify(&doc).await.unwrap_err();
    // Network mode is disabled in this test so the SDK can't reach the
    // did:web. The error should NOT be SignatureInvalid (which would
    // mis-signal a forgery).
    assert!(
        !matches!(err, VerificationError::SignatureInvalid),
        "unresolvable VM should not return SignatureInvalid, got {err:?}"
    );
}
