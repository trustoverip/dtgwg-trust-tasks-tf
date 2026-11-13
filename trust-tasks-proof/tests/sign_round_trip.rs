//! Round-trip tests for the `affinidi` backend's [`sign_trust_task`]:
//! documents it signs must verify with the crate's own stock
//! [`Verifier`], with no coordination beyond both sides using this crate.
//!
//! Covers: did:key sign → stock-verifier verify, option pass-through
//! (proofPurpose, cryptosuite), the deterministic `eddsa-jcs-2022`
//! default (even against a signer declaring a different suite), replace
//! semantics for an already-signed document, and the sign-time issuer
//! binding pre-flight.

#![cfg(feature = "affinidi")]

use affinidi_data_integrity::crypto_suites::CryptoSuite;
use affinidi_data_integrity::signer::Signer;
use affinidi_data_integrity::DataIntegrityError;
use affinidi_secrets_resolver::secrets::{KeyType, Secret};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_proof::affinidi::{sign_trust_task, SignError, SignOptions, Verifier};
use trust_tasks_rs::{Payload, ProofVerifier, TrustTask};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DemoPayload {
    subject: String,
    claim: String,
}

impl Payload for DemoPayload {
    const TYPE_URI: &'static str = "https://example.com/spec/demo-attestation/0.1";
}

/// Mint a fresh Ed25519 `did:key` secret whose `id` is the canonical
/// did:key verification method (`did:key:z6Mk...#z6Mk...`), so the stock
/// `Verifier::for_did_key()` can resolve it offline.
fn fresh_did_key(seed: u8) -> (Secret, String) {
    let throwaway = Secret::generate_ed25519(None, Some(&[seed; 32]));
    let pk_mb = throwaway.get_public_keymultibase().expect("multikey");
    let vm = format!("did:key:{pk_mb}#{pk_mb}");
    let mut secret = Secret::generate_ed25519(Some(&vm), Some(&[seed; 32]));
    secret.id = vm.clone();
    let did = vm.split('#').next().unwrap().to_string();
    (secret, did)
}

/// An unsigned Trust Task document (wire shape) issued by `issuer`.
fn unsigned_doc(issuer: &str) -> Value {
    json!({
        "id": "urn:uuid:sign-round-trip-1",
        "type": DemoPayload::TYPE_URI,
        "issuer": issuer,
        "recipient": "did:example:bank",
        "issuedAt": "2026-07-29T09:00:00Z",
        "payload": {
            "subject": "did:example:alice",
            "claim": "loa2"
        }
    })
}

#[tokio::test]
async fn signed_doc_verifies_with_stock_did_key_verifier() {
    let (secret, did) = fresh_did_key(11);
    let doc = unsigned_doc(&did);

    let signed = sign_trust_task(&doc, &secret, SignOptions::new())
        .await
        .expect("sign");

    // Defaults: the reference ecosystem's profile.
    let proof = signed.get("proof").expect("proof member inserted");
    assert_eq!(proof["type"], "DataIntegrityProof");
    assert_eq!(proof["cryptosuite"], "eddsa-jcs-2022");
    assert_eq!(proof["proofPurpose"], "authentication");
    assert_eq!(proof["verificationMethod"], secret.id);

    // Round-trip through the framework's typed document and the crate's
    // own stock verifier — the consumer side of the contract.
    let typed: TrustTask<DemoPayload> = serde_json::from_value(signed).expect("wire shape");
    Verifier::for_did_key()
        .verify(&typed)
        .await
        .expect("stock verifier accepts what sign_trust_task emits");
}

#[tokio::test]
async fn proof_purpose_option_is_honored() {
    let (secret, did) = fresh_did_key(12);
    let doc = unsigned_doc(&did);

    let signed = sign_trust_task(
        &doc,
        &secret,
        SignOptions::new().with_proof_purpose("authentication"),
    )
    .await
    .expect("sign");

    assert_eq!(signed["proof"]["proofPurpose"], "authentication");
    // Cryptosuite still defaults independently of the purpose override.
    assert_eq!(signed["proof"]["cryptosuite"], "eddsa-jcs-2022");
}

#[tokio::test]
async fn explicit_cryptosuite_option_is_honored() {
    let (secret, did) = fresh_did_key(13);
    let mut doc = unsigned_doc(&did);
    // RDFC canonicalisation processes the document as JSON-LD and
    // requires an @context (the JCS default needs none). It expands in
    // safe mode (affinidi-data-integrity 0.8.2+), so every member must be
    // defined by the context: the inline `@vocab` defines the Trust Task
    // members the credentials context does not.
    doc.as_object_mut().unwrap().insert(
        "@context".into(),
        json!([
            "https://www.w3.org/ns/credentials/v2",
            { "@vocab": "https://trusttasks.org/vocab#" }
        ]),
    );

    let signed = sign_trust_task(
        &doc,
        &secret,
        SignOptions::new().with_cryptosuite(CryptoSuite::EddsaRdfc2022),
    )
    .await
    .expect("sign with explicit rdfc suite");

    assert_eq!(signed["proof"]["cryptosuite"], "eddsa-rdfc-2022");

    let typed: TrustTask<DemoPayload> = serde_json::from_value(signed).expect("wire shape");
    Verifier::for_did_key()
        .verify(&typed)
        .await
        .expect("rdfc-signed doc verifies");
}

/// A signer that declares a non-JCS default suite. `sign_trust_task`'s
/// deterministic default must override it — the wire suite must not vary
/// with the signer implementation unless the caller opts in.
struct RdfcDefaultSigner(Secret);

#[async_trait]
impl Signer for RdfcDefaultSigner {
    fn key_type(&self) -> KeyType {
        self.0.key_type()
    }
    fn verification_method(&self) -> &str {
        self.0.verification_method()
    }
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, DataIntegrityError> {
        self.0.sign(data).await
    }
    fn cryptosuite(&self) -> CryptoSuite {
        CryptoSuite::EddsaRdfc2022
    }
}

#[tokio::test]
async fn default_suite_is_jcs_even_when_signer_declares_otherwise() {
    let (secret, did) = fresh_did_key(14);
    let doc = unsigned_doc(&did);

    let signed = sign_trust_task(&doc, &RdfcDefaultSigner(secret), SignOptions::new())
        .await
        .expect("sign");

    assert_eq!(signed["proof"]["cryptosuite"], "eddsa-jcs-2022");
}

#[tokio::test]
async fn existing_proof_is_replaced_not_nested() {
    let (secret, did) = fresh_did_key(15);
    let doc = unsigned_doc(&did);

    let first = sign_trust_task(&doc, &secret, SignOptions::new())
        .await
        .expect("first sign");
    let first_value = first["proof"]["proofValue"].clone();

    // Re-sign the already-signed document with a different purpose: the
    // old proof must be stripped before signing (never signed over) and
    // replaced by exactly one fresh proof.
    let second = sign_trust_task(
        &first,
        &secret,
        SignOptions::new().with_proof_purpose("assertionMethod"),
    )
    .await
    .expect("re-sign");

    let proof = &second["proof"];
    assert!(proof.is_object(), "single proof object, not a proof set");
    assert_eq!(proof["proofPurpose"], "assertionMethod");
    assert_ne!(proof["proofValue"], first_value, "fresh signature minted");

    let typed: TrustTask<DemoPayload> = serde_json::from_value(second).expect("wire shape");
    Verifier::for_did_key()
        .verify(&typed)
        .await
        .expect("re-signed doc verifies — proof was not signed over");
}

#[tokio::test]
async fn issuer_mismatch_is_rejected_at_sign_time() {
    let (secret, _did) = fresh_did_key(16);
    // Claim a different issuer than the DID controlling the signer's VM —
    // the doc could never pass the stock verifier's binding check.
    let doc = unsigned_doc("did:key:z6MkSomebodyElseEntirely");

    let err = sign_trust_task(&doc, &secret, SignOptions::new())
        .await
        .unwrap_err();
    assert!(
        matches!(err, SignError::IssuerMismatch { .. }),
        "expected IssuerMismatch, got {err:?}"
    );
}

#[tokio::test]
async fn missing_issuer_is_rejected_at_sign_time() {
    let (secret, did) = fresh_did_key(17);
    let mut doc = unsigned_doc(&did);
    doc.as_object_mut().unwrap().remove("issuer");

    let err = sign_trust_task(&doc, &secret, SignOptions::new())
        .await
        .unwrap_err();
    assert!(
        matches!(err, SignError::MissingIssuer),
        "expected MissingIssuer, got {err:?}"
    );
}

#[tokio::test]
async fn non_object_document_is_rejected() {
    let (secret, _did) = fresh_did_key(18);
    let err = sign_trust_task(&json!(["not", "an", "object"]), &secret, SignOptions::new())
        .await
        .unwrap_err();
    assert!(
        matches!(err, SignError::NotAnObject),
        "expected NotAnObject, got {err:?}"
    );
}

/// Re-sign a shared vector (`fixtures/proof`) with the seed-7 key and its own
/// `created` / `proofPurpose`, and return the vector and the result.
async fn resign_shared_vector(file: &str) -> (Value, Value) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/proof")
        .join(file);
    let vector: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let (secret, did) = fresh_did_key(7);
    assert_eq!(vector["issuer"], did.as_str());
    let created = vector["proof"]["created"]
        .as_str()
        .unwrap()
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap();
    let options = SignOptions::new()
        .with_created(created)
        .with_proof_purpose(vector["proof"]["proofPurpose"].as_str().unwrap());
    let signed = sign_trust_task(&vector, &secret, options)
        .await
        .expect("sign");
    (vector, signed)
}

#[tokio::test]
async fn reproduces_the_shared_baseline_vector_byte_for_byte() {
    let (vector, signed) = resign_shared_vector("eddsa-jcs-2022-baseline.json").await;
    assert_eq!(signed, vector);
    assert!(signed["proof"].get("@context").is_none());
}

/// eddsa-jcs-2022 §3.3.1 step 2 copies the document's `@context` into the
/// proof; every signer reproduces the shared `@context` vector byte for byte.
#[tokio::test]
async fn reproduces_the_shared_context_vector_byte_for_byte() {
    let (vector, signed) = resign_shared_vector("eddsa-jcs-2022-context.json").await;
    assert_eq!(signed, vector);
    assert_eq!(signed["proof"]["@context"], vector["@context"]);
}

/// A caller-supplied context is replaced by the document's: any other value
/// would make the stock verifier refuse the proof.
#[tokio::test]
async fn proof_context_follows_the_document() {
    let (secret, did) = fresh_did_key(21);
    let mut doc = unsigned_doc(&did);
    let stray = || SignOptions::new().with_context(vec!["https://example.org/stray".to_string()]);
    let verifier = Verifier::for_did_key();

    let signed = sign_trust_task(&doc, &secret, stray()).await.expect("sign");
    assert!(signed["proof"].get("@context").is_none());
    verifier.verify_raw(&signed).await.expect("verifies");

    doc["@context"] = json!(["https://www.w3.org/ns/credentials/v2"]);
    let signed = sign_trust_task(&doc, &secret, stray()).await.expect("sign");
    assert_eq!(signed["proof"]["@context"], doc["@context"]);
    verifier.verify_raw(&signed).await.expect("verifies");

    // A form the upstream proof type cannot hold gets no proof `@context`,
    // and verifies: the proof configuration is hashed as received.
    doc["@context"] = json!("https://www.w3.org/ns/credentials/v2");
    let signed = sign_trust_task(&doc, &secret, stray()).await.expect("sign");
    assert!(signed["proof"].get("@context").is_none());
    verifier.verify_raw(&signed).await.expect("verifies");
}

#[tokio::test]
async fn proof_context_must_equal_the_document_context() {
    let (secret, did) = fresh_did_key(22);
    let mut doc = unsigned_doc(&did);
    doc["@context"] = json!([
        "https://www.w3.org/ns/credentials/v2",
        "https://example.org/v1"
    ]);
    let signed = sign_trust_task(&doc, &secret, SignOptions::new())
        .await
        .expect("sign");
    Verifier::for_did_key()
        .verify_raw(&signed)
        .await
        .expect("verifies");

    type Mutation = (&'static str, fn(&mut Value));
    let mutations: [Mutation; 3] = [
        ("removed from the document", |d| {
            d.as_object_mut().unwrap().remove("@context");
        }),
        ("reordered in the document", |d| {
            d["@context"].as_array_mut().unwrap().reverse();
        }),
        ("changed in the proof", |d| {
            d["proof"]["@context"] = json!(["https://example.org/other"]);
        }),
    ];
    for (name, mutate) in mutations {
        let mut tampered = signed.clone();
        mutate(&mut tampered);
        let err = Verifier::for_did_key()
            .verify_raw(&tampered)
            .await
            .expect_err(name);
        assert!(
            matches!(err, trust_tasks_rs::VerificationError::MalformedProof(_)),
            "{name}: {err:?}"
        );
    }
}
