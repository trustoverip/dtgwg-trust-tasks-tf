//! Round-trip integration tests for the `affinidi` backend's [`Verifier`].
//!
//! Each test:
//! 1. Generates a fresh Ed25519 keypair.
//! 2. Builds a Trust Task document.
//! 3. Signs it with `affinidi-data-integrity`'s `DataIntegrityProof::sign`.
//! 4. Attaches the resulting proof onto the framework's typed
//!    [`Proof`](trust_tasks_rs::Proof) struct.
//! 5. Verifies via [`trust_tasks_proof::affinidi::Verifier`] using a
//!    local `MapResolver` (test stand-in for a real `did:web` /
//!    `did:webvh` resolver).
//!
//! Covers: happy path, payload tampering, proof tampering, missing-proof.

#![cfg(feature = "affinidi")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use affinidi_data_integrity::{
    DataIntegrityError, DataIntegrityProof, DidKeyResolver, ResolvedKey, SignOptions,
    VerificationMethodResolver,
};
use affinidi_secrets_resolver::secrets::{KeyType, Secret};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use trust_tasks_proof::affinidi::Verifier;
use trust_tasks_rs::{Payload, Proof, ProofVerifier, TrustTask, TypeUri, VerificationError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DemoPayload {
    subject: String,
    claim: String,
}

impl Payload for DemoPayload {
    const TYPE_URI: &'static str = "https://example.com/spec/demo-attestation/0.1";
}

#[derive(Default)]
struct MapResolver {
    entries: Mutex<HashMap<String, ResolvedKey>>,
}

impl MapResolver {
    fn insert(&self, vm: &str, key: ResolvedKey) {
        self.entries.lock().unwrap().insert(vm.to_string(), key);
    }
}

#[async_trait]
impl VerificationMethodResolver for MapResolver {
    async fn resolve_vm(&self, vm: &str) -> Result<ResolvedKey, DataIntegrityError> {
        if let Some(hit) = self.entries.lock().unwrap().get(vm) {
            return Ok(hit.clone());
        }
        DidKeyResolver.resolve_vm(vm).await
    }
}

/// Mint a fresh Ed25519 secret, register its public key under `vm` in a
/// fresh `MapResolver`, return both. The caller signs with the secret
/// and verifies through the resolver — exactly the producer/consumer
/// split a real deployment uses.
fn keypair_with_resolver(vm: &str) -> (Secret, Arc<MapResolver>) {
    let mut secret = Secret::generate_ed25519(None, Some(&[7u8; 32]));
    secret.id = vm.to_string();

    let resolver = MapResolver::default();
    resolver.insert(
        vm,
        ResolvedKey::new(KeyType::Ed25519, secret.get_public_bytes().to_vec()),
    );
    (secret, Arc::new(resolver))
}

/// Produce a TrustTask, sign it via affinidi-data-integrity, attach the
/// proof onto our typed [`Proof`] struct (which serialises wire-compatible
/// with `DataIntegrityProof`).
async fn signed_doc(vm: &str, secret: &Secret) -> TrustTask<DemoPayload> {
    let mut doc = TrustTask::for_payload(
        "urn:uuid:test-1",
        DemoPayload {
            subject: "did:example:alice".into(),
            claim: "loa2".into(),
        },
    );
    doc.issuer = Some(vm.split('#').next().unwrap().to_string());
    doc.recipient = Some("did:example:bank".into());
    doc.issued_at = Some(chrono::Utc::now());

    // Serialise the doc (sans proof), sign, attach.
    let body = serde_json::to_value(&doc).unwrap();
    let proof = DataIntegrityProof::sign(&body, secret, SignOptions::new())
        .await
        .expect("sign");
    let proof_value = serde_json::to_value(&proof).unwrap();
    doc.proof = Some(serde_json::from_value::<Proof>(proof_value).unwrap());

    doc
}

#[tokio::test]
async fn happy_path_signs_and_verifies() {
    let vm = "did:web:org.example#key-0";
    let (secret, resolver) = keypair_with_resolver(vm);
    let doc = signed_doc(vm, &secret).await;

    let verifier = Verifier::with_resolver(resolver);
    verifier.verify(&doc).await.expect("valid proof verifies");
}

#[tokio::test]
async fn tampered_payload_fails_verification() {
    let vm = "did:web:org.example#key-0";
    let (secret, resolver) = keypair_with_resolver(vm);
    let mut doc = signed_doc(vm, &secret).await;

    // Flip a payload byte — proof is now over the *old* bytes.
    doc.payload.claim = "loa3".into();

    let err = Verifier::with_resolver(resolver)
        .verify(&doc)
        .await
        .unwrap_err();
    assert!(
        matches!(err, VerificationError::SignatureInvalid),
        "expected SignatureInvalid, got {err:?}"
    );
}

#[tokio::test]
async fn missing_proof_is_malformed() {
    let vm = "did:web:org.example#key-0";
    let (secret, resolver) = keypair_with_resolver(vm);
    let mut doc = signed_doc(vm, &secret).await;
    doc.proof = None;

    let err = Verifier::with_resolver(resolver)
        .verify(&doc)
        .await
        .unwrap_err();
    assert!(matches!(err, VerificationError::MalformedProof(_)));
}

#[tokio::test]
async fn unknown_verification_method_surfaces_resolver_error() {
    let vm = "did:web:org.example#key-0";
    let (secret, _resolver) = keypair_with_resolver(vm);
    let doc = signed_doc(vm, &secret).await;

    // Different verifier with an empty resolver — falls through to
    // DidKeyResolver, which doesn't recognise this did:web URI.
    let empty_resolver = Arc::new(MapResolver::default());
    let err = Verifier::with_resolver(empty_resolver)
        .verify(&doc)
        .await
        .unwrap_err();
    // Any of MalformedProof / Other / UnsupportedCryptosuite is acceptable;
    // the important thing is we don't return Ok on an unresolvable VM.
    assert!(
        !matches!(err, VerificationError::SignatureInvalid),
        "verifier returned SignatureInvalid for an unresolvable VM (got {err:?})"
    );
}

#[tokio::test]
async fn issuer_not_controlling_verification_method_is_rejected() {
    // The impersonation attack: an attacker signs with their OWN key, under
    // their OWN DID, but sets `issuer` to a different (trusted) party. The
    // signature is cryptographically valid over the spoofed-issuer document,
    // yet the verificationMethod is not controlled by the claimed issuer.
    // SPEC §4.7/§4.8/§7.2-item-7 require this to be rejected — otherwise every
    // downstream authorization keyed on `issuer` runs for a spoofed identity.
    let vm = "did:web:attacker.example#key-0";
    let (secret, resolver) = keypair_with_resolver(vm);

    let mut doc = TrustTask::for_payload(
        "urn:uuid:test-spoof",
        DemoPayload {
            subject: "did:example:alice".into(),
            claim: "loa2".into(),
        },
    );
    doc.issuer = Some("did:web:victim.example".into()); // ≠ controller of `vm`
    doc.recipient = Some("did:example:bank".into());
    doc.issued_at = Some(chrono::Utc::now());
    let body = serde_json::to_value(&doc).unwrap();
    let proof = DataIntegrityProof::sign(&body, &secret, SignOptions::new())
        .await
        .expect("sign");
    doc.proof =
        Some(serde_json::from_value::<Proof>(serde_json::to_value(&proof).unwrap()).unwrap());

    let err = Verifier::with_resolver(resolver)
        .verify(&doc)
        .await
        .unwrap_err();
    assert!(
        matches!(err, VerificationError::IssuerMismatch(_)),
        "expected IssuerMismatch for issuer≠verificationMethod, got {err:?}"
    );
}

#[tokio::test]
async fn proof_present_without_issuer_is_rejected() {
    // A proof with no in-band `issuer` has nothing to bind to; it must not
    // verify (the issuer binding is checked before the signature, so this
    // holds regardless of the signature's own validity).
    let vm = "did:web:org.example#key-0";
    let (secret, resolver) = keypair_with_resolver(vm);
    let mut doc = signed_doc(vm, &secret).await;
    doc.issuer = None;

    let err = Verifier::with_resolver(resolver)
        .verify(&doc)
        .await
        .unwrap_err();
    assert!(
        matches!(err, VerificationError::IssuerMismatch(_)),
        "expected IssuerMismatch for proof-without-issuer, got {err:?}"
    );
}

#[tokio::test]
async fn type_uri_constant_is_well_formed() {
    // Sanity: the demo Payload's Type URI parses as a TypeUri.
    let _: TypeUri = DemoPayload::TYPE_URI.parse().expect("Type URI parses");
}
