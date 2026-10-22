//! `Verifier::verify_raw` checks the document as it was received, and
//! `CachedDidResolver` accepts only keys the DID authorises for the proof's
//! purpose, in
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
    // `signed` uses the default proofPurpose, assertionMethod.
    let (verifier, secret) = verifier_for("assertionMethod").await;
    let doc = signed(heartbeat("2026-01-01T00:00:00Z"), &secret).await;
    verifier
        .verify_raw(&doc)
        .await
        .expect("an assertionMethod key published as publicKeyJwk verifies");
}

#[tokio::test]
async fn a_key_agreement_key_cannot_sign() {
    // The same key, listed for key agreement only: a valid signature by it is
    // still not a signature the DID authorised.
    let (verifier, secret) = verifier_for("keyAgreement").await;
    let doc = signed(heartbeat("2026-01-01T00:00:00Z"), &secret).await;
    let err = verifier.verify_raw(&doc).await.unwrap_err();
    assert!(
        err.to_string().contains("not listed under assertionMethod"),
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

// ─── The pipeline: `consume_inbound` and the type-erased verifier ─────────────

/// The shape typify generates for an optional boolean: `#[serde(default)]`
/// with no `skip_serializing_if`, so a member the producer left out comes
/// back as `false` when the typed document is re-serialised.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeleteContext {
    name: String,
    #[serde(default)]
    force: bool,
}

impl Payload for DeleteContext {
    const TYPE_URI: &'static str = "https://example.com/spec/delete-context/0.1";
}

/// Signed by the producer exactly as it would put it on the wire: `force`
/// omitted and `issuedAt` written with `+00:00`. Both re-encode differently.
async fn signed_delete(secret: &Secret) -> Value {
    signed(
        json!({
            "id": "urn:uuid:received-1",
            "type": DeleteContext::TYPE_URI,
            "issuer": DID,
            "recipient": "did:web:auditor.example",
            "issuedAt": "2026-01-01T00:00:00+00:00",
            "payload": { "name": "ctx" },
        }),
        secret,
    )
    .await
}

async fn consume(
    policy: trust_tasks_rs::ProofPolicy<'_, Verifier>,
    doc: TrustTask<DeleteContext>,
    ran: &std::sync::atomic::AtomicBool,
) -> trust_tasks_rs::ConsumeOutcome<Value> {
    trust_tasks_rs::consume_inbound(
        &trust_tasks_rs::handlers::NoopHandler::new(),
        policy,
        trust_tasks_rs::PayloadPolicy::<trust_tasks_rs::NoValidator>::AcceptUnvalidated,
        trust_tasks_rs::ConsumeChecks::not_consequential(),
        doc,
        "did:web:auditor.example",
        "2026-01-01T00:00:30Z".parse().unwrap(),
        || "err-1".to_string(),
        |req, _parties| async move {
            ran.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok::<_, trust_tasks_rs::ErrorResponse>(req.respond_with("resp-1", json!({})))
        },
    )
    .await
}

/// The defect this guards: every binding parsed the document into its typed
/// form and verified that, re-serialised. A genuine proof over a document
/// whose typed form re-encodes differently was refused `proofInvalid`.
#[tokio::test]
async fn consume_inbound_verifies_the_document_as_received() {
    let (verifier, secret) = verifier_for("assertionMethod").await;
    let received = signed_delete(&secret).await;
    let doc: TrustTask<DeleteContext> = serde_json::from_value(received.clone()).unwrap();
    assert_ne!(
        serde_json::to_value(&doc).unwrap(),
        received,
        "precondition: the typed document must re-encode differently, or this test proves nothing"
    );

    let ran = std::sync::atomic::AtomicBool::new(false);
    let policy = trust_tasks_rs::ProofPolicy::Verify(&verifier).with_received(&received);
    match consume(policy, doc, &ran).await {
        trust_tasks_rs::ConsumeOutcome::Handled(_) => {}
        other => panic!("a genuine proof over the received document must verify: {other:?}"),
    }
    assert!(ran.load(std::sync::atomic::Ordering::SeqCst));
}

/// A proof over one document vouches for nothing about another: received JSON
/// that does not parse to the document being consumed is refused, and the
/// handler never runs.
#[tokio::test]
async fn a_received_document_that_is_not_the_one_consumed_is_refused() {
    let (verifier, secret) = verifier_for("assertionMethod").await;
    let received = signed_delete(&secret).await;
    let mut doc: TrustTask<DeleteContext> = serde_json::from_value(received.clone()).unwrap();
    doc.payload.name = "a-different-context".into();

    let ran = std::sync::atomic::AtomicBool::new(false);
    let policy = trust_tasks_rs::ProofPolicy::VerifyReceived(&verifier, &received);
    match consume(policy, doc, &ran).await {
        trust_tasks_rs::ConsumeOutcome::Rejected(err) => assert_eq!(
            err.payload.code,
            trust_tasks_rs::StandardCode::ProofInvalid.into()
        ),
        other => panic!("expected proofInvalid, got {other:?}"),
    }
    assert!(!ran.load(std::sync::atomic::Ordering::SeqCst));
}

/// The HTTPS server stores its verifier type-erased; erasure must reach the
/// raw path rather than fall back to re-serialising.
#[tokio::test]
async fn the_type_erased_verifier_verifies_the_document_as_received() {
    let (verifier, secret) = verifier_for("assertionMethod").await;
    let received = signed_delete(&secret).await;
    trust_tasks_rs::erase_verifier(verifier)
        .verify_received(&received)
        .await
        .expect("the erased verifier forwards to verify_raw");
}
