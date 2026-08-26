//! Tests for [`ProofExt`] — the typed `.sign()` / `.verify()` wrapper
//! over `sign_trust_task` + `Verifier`.
//!
//! The properties that matter are (a) a document signed through the
//! trait verifies through the trait, (b) mutating it afterwards makes
//! verification fail, and (c) the trait is a *wrapper*: the proof it
//! attaches is bit-for-bit the one the free function produces over the
//! same document.

#![cfg(feature = "affinidi")]

use affinidi_secrets_resolver::secrets::Secret;
use serde::{Deserialize, Serialize};
use trust_tasks_proof::affinidi::{sign_trust_task, SignError, SignOptions, Verifier};
use trust_tasks_proof::ProofExt;
use trust_tasks_rs::{Payload, TrustTask, VerificationError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DemoPayload {
    subject: String,
    claim: String,
}

impl Payload for DemoPayload {
    const TYPE_URI: &'static str = "https://example.com/spec/demo-attestation/0.1";
}

/// Mint a fresh Ed25519 `did:key` secret whose `id` is the canonical
/// did:key verification method, so `Verifier::for_did_key()` resolves it
/// offline.
fn fresh_did_key(seed: u8) -> (Secret, String) {
    let throwaway = Secret::generate_ed25519(None, Some(&[seed; 32]));
    let pk_mb = throwaway.get_public_keymultibase().expect("multikey");
    let vm = format!("did:key:{pk_mb}#{pk_mb}");
    let mut secret = Secret::generate_ed25519(Some(&vm), Some(&[seed; 32]));
    secret.id = vm.clone();
    let did = vm.split('#').next().unwrap().to_string();
    (secret, did)
}

fn unsigned(issuer: &str) -> TrustTask<DemoPayload> {
    let mut doc = TrustTask::for_payload(
        "urn:uuid:proof-ext-1",
        DemoPayload {
            subject: "did:example:alice".into(),
            claim: "loa2".into(),
        },
    );
    doc.issuer = Some(issuer.to_string());
    doc.recipient = Some("did:example:bank".into());
    doc.issued_at = Some("2026-07-29T09:00:00Z".parse().expect("timestamp"));
    doc
}

/// The headline: compose, `.sign()`, `.verify()`. No `serde_json::Value`
/// anywhere in the caller's code.
#[tokio::test]
async fn signed_typed_document_round_trips() {
    let (secret, did) = fresh_did_key(31);
    let mut doc = unsigned(&did);
    assert!(doc.proof.is_none());

    doc.sign(&secret, SignOptions::new()).await.expect("sign");

    let proof = doc.proof.as_ref().expect("proof attached in place");
    assert_eq!(proof.proof_type, "DataIntegrityProof");
    assert_eq!(proof.cryptosuite, "eddsa-jcs-2022");
    assert_eq!(proof.proof_purpose, "assertionMethod");
    assert_eq!(proof.verification_method, secret.id);

    doc.verify(&Verifier::for_did_key())
        .await
        .expect("stock verifier accepts what ProofExt::sign emits");
}

/// Tamper detection: the proof covers the payload, so editing it after
/// signing must fail verification rather than silently pass.
#[tokio::test]
async fn mutating_the_payload_after_signing_fails_verification() {
    let (secret, did) = fresh_did_key(32);
    let mut doc = unsigned(&did);
    doc.sign(&secret, SignOptions::new()).await.expect("sign");
    doc.verify(&Verifier::for_did_key())
        .await
        .expect("valid before mutation");

    // Escalate the claim, keep the proof.
    doc.payload.claim = "loa4".into();

    let err = doc
        .verify(&Verifier::for_did_key())
        .await
        .expect_err("mutated payload must not verify");
    assert!(
        matches!(err, VerificationError::SignatureInvalid),
        "expected SignatureInvalid, got {err:?}"
    );

    // Re-signing over the new content is the supported recovery.
    doc.sign(&secret, SignOptions::new())
        .await
        .expect("re-sign");
    doc.verify(&Verifier::for_did_key())
        .await
        .expect("re-signed document verifies");
}

/// `ProofExt::sign` is a wrapper, not a reimplementation: the proof it
/// attaches must equal the one `sign_trust_task` produces over the same
/// unsigned document. (`eddsa-jcs-2022` over Ed25519 is deterministic, so
/// the `proofValue` is comparable — the point of the assertion is that
/// the *bytes signed over* are identical.)
#[tokio::test]
async fn typed_sign_matches_the_free_function_byte_for_byte() {
    let (secret, did) = fresh_did_key(33);
    let mut doc = unsigned(&did);

    let via_free_fn = sign_trust_task(
        &serde_json::to_value(&doc).expect("serialise"),
        &secret,
        SignOptions::new(),
    )
    .await
    .expect("free function sign");

    doc.sign(&secret, SignOptions::new()).await.expect("sign");

    assert_eq!(
        serde_json::to_value(&doc).expect("serialise signed"),
        via_free_fn,
        "the typed wrapper and the free function must emit the same document"
    );
}

/// A sign that cannot succeed must not half-apply. The §4.7/§4.8 issuer
/// pre-flight runs before any signature exists, so `self` is unchanged.
#[tokio::test]
async fn a_failed_sign_leaves_the_document_untouched() {
    let (secret, _did) = fresh_did_key(34);
    let mut doc = unsigned("did:key:z6MkSomebodyElseEntirely");
    let before = doc.clone();

    let err = doc
        .sign(&secret, SignOptions::new())
        .await
        .expect_err("issuer does not control the signer's verificationMethod");
    assert!(
        matches!(err, SignError::IssuerMismatch { .. }),
        "expected IssuerMismatch, got {err:?}"
    );
    assert_eq!(doc, before, "no partial mutation on the failure path");
}

/// `verify` on a proofless document is a verification failure, not a
/// panic or a silent pass — the `proofRequired` policy check is a
/// separate, earlier concern.
#[tokio::test]
async fn verifying_a_proofless_document_fails() {
    let (_secret, did) = fresh_did_key(35);
    let doc = unsigned(&did);

    let err = doc
        .verify(&Verifier::for_did_key())
        .await
        .expect_err("no proof to verify");
    assert!(
        matches!(err, VerificationError::MalformedProof(_)),
        "expected MalformedProof, got {err:?}"
    );
}
