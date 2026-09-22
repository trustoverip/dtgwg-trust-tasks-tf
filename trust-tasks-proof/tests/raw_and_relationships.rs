//! `Verifier::verify_raw` checks the document as it was received, and
//! `CachedDidResolver` accepts only keys the DID authorises for signing, in
//! either `publicKeyMultibase` or `publicKeyJwk` form.
//!
//! The issuer's DID document is seeded straight into the resolver's cache, so
//! each test states exactly the document shape it is about.

#![cfg(feature = "affinidi")]

use std::sync::Arc;

use affinidi_data_integrity::{DataIntegrityProof, SignOptions};
use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
use affinidi_secrets_resolver::secrets::Secret;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_proof::affinidi::{CachedDidResolver, Verifier};
use trust_tasks_rs::{Payload, ProofVerifier, TrustTask};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Heartbeat {
    nonce: u64,
}

impl Payload for Heartbeat {
    const TYPE_URI: &'static str = "https://example.com/spec/heartbeat/0.1";
}

const DID: &str = "did:web:issuer.example";

/// A verifier whose resolver already holds `DID`'s document: one Ed25519 key,
/// published as `publicKeyJwk` and listed under `relationship` only.
async fn verifier_for(relationship: &str) -> (Verifier, Secret) {
    let seed = [7u8; 32];
    let public = Secret::generate_ed25519(None, Some(&seed))
        .get_public_bytes()
        .to_vec();
    let x = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public);
    let vm = format!("{DID}#key-1");
    let doc: affinidi_did_common::Document = serde_json::from_value(json!({
        "id": DID,
        "verificationMethod": [{
            "id": vm,
            "type": "JsonWebKey2020",
            "controller": DID,
            "publicKeyJwk": { "kty": "OKP", "crv": "Ed25519", "x": x },
        }],
        relationship: [vm],
    }))
    .expect("DID document");
    let mut client = DIDCacheClient::new(DIDCacheConfigBuilder::default().build())
        .await
        .expect("local DIDCacheClient");
    client.add_did_document(DID, doc).await;
    let verifier = Verifier::with_resolver(Arc::new(CachedDidResolver::new(Arc::new(client))));
    let secret = Secret::generate_ed25519(Some(&vm), Some(&seed));
    (verifier, secret)
}

/// Sign `doc` (as JSON) with `secret` and return it with its proof.
async fn signed(doc: Value, secret: &Secret) -> Value {
    let proof = DataIntegrityProof::sign(&doc, secret, SignOptions::new())
        .await
        .expect("sign");
    let mut doc = doc;
    doc["proof"] = serde_json::to_value(proof).unwrap();
    doc
}

fn heartbeat(issued_at: &str) -> Value {
    json!({
        "id": "urn:uuid:raw-1",
        "type": Heartbeat::TYPE_URI,
        "issuer": DID,
        "recipient": "did:web:auditor.example",
        "issuedAt": issued_at,
        "payload": { "nonce": 1 },
    })
}

#[tokio::test]
async fn a_jwk_signing_key_verifies() {
    let (verifier, secret) = verifier_for("authentication").await;
    let doc = signed(heartbeat("2026-01-01T00:00:00Z"), &secret).await;
    verifier
        .verify_raw(&doc)
        .await
        .expect("an authentication key published as publicKeyJwk verifies");
}

#[tokio::test]
async fn a_key_agreement_key_cannot_sign() {
    // The same key, listed for key agreement only: a valid signature by it is
    // still not a signature the DID authorised.
    let (verifier, secret) = verifier_for("keyAgreement").await;
    let doc = signed(heartbeat("2026-01-01T00:00:00Z"), &secret).await;
    let err = verifier.verify_raw(&doc).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("not an authentication or assertionMethod key"),
        "{err}"
    );
}

/// The typed path re-serialises the document, which changes a timestamp
/// written with `+00:00` to `Z` and breaks a genuine signature; the raw path
/// checks the bytes that were signed.
#[tokio::test]
async fn the_document_as_received_verifies_where_a_reserialised_one_does_not() {
    let (verifier, secret) = verifier_for("assertionMethod").await;
    let doc = signed(heartbeat("2026-01-01T00:00:00+00:00"), &secret).await;

    verifier
        .verify_raw(&doc)
        .await
        .expect("the document as signed verifies");

    let typed: TrustTask<Heartbeat> = serde_json::from_value(doc).unwrap();
    assert!(
        verifier.verify(&typed).await.is_err(),
        "re-serialising the typed document changes what was signed"
    );
}
