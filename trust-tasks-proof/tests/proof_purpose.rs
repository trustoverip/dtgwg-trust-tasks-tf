//! A proof verifies only when the issuer's DID document lists its
//! `verificationMethod` under the relationship its `proofPurpose` names
//! (W3C Data Integrity; Controlled Identifiers v1.0 §3.3), and the method
//! belongs to the issuer.
//!
//! Documents are seeded straight into the resolver's cache, so each test
//! states the exact DID document it is about. `did:key` and `did:peer` are
//! resolved by the cache's own local resolvers.

#![cfg(feature = "affinidi")]

use std::sync::Arc;

use affinidi_data_integrity::{DataIntegrityProof, SignOptions};
use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
use affinidi_secrets_resolver::secrets::Secret;
use serde_json::{json, Value};
use trust_tasks_proof::affinidi::{
    CachedDidResolver, ProofPurpose, ProofPurposeResolver, PurposeBound, Verifier,
};
use trust_tasks_rs::VerificationError;

const DID: &str = "did:web:issuer.example";
const OTHER: &str = "did:web:other.example";
const SEED: [u8; 32] = [9u8; 32];

fn public_multibase() -> String {
    Secret::generate_ed25519(None, Some(&SEED))
        .get_public_keymultibase()
        .expect("multikey")
}

/// A verification method for the seeded key.
fn method(id: &str, controller: &str) -> Value {
    json!({
        "id": id,
        "type": "Multikey",
        "controller": controller,
        "publicKeyMultibase": public_multibase(),
    })
}

async fn client_with(did: &str, doc: Value) -> DIDCacheClient {
    let doc: affinidi_did_common::Document = serde_json::from_value(doc).expect("DID document");
    let mut client = DIDCacheClient::new(DIDCacheConfigBuilder::default().build())
        .await
        .expect("local DIDCacheClient");
    client.add_did_document(did, doc).await;
    client
}

async fn verifier_with(did: &str, doc: Value) -> Verifier {
    let client = client_with(did, doc).await;
    Verifier::with_resolver(Arc::new(CachedDidResolver::new(Arc::new(client))))
}

/// A Trust Task document from `issuer`, signed by the seeded key as `vm`
/// with `proofPurpose` `purpose`.
async fn signed(issuer: &str, vm: &str, purpose: &str) -> Value {
    let doc = json!({
        "id": "urn:uuid:purpose-1",
        "type": "https://example.com/spec/heartbeat/0.1",
        "issuer": issuer,
        "recipient": "did:web:auditor.example",
        "issuedAt": "2026-01-01T00:00:00Z",
        "payload": { "nonce": 1 },
    });
    let secret = Secret::generate_ed25519(Some(vm), Some(&SEED));
    let proof = DataIntegrityProof::sign(
        &doc,
        &secret,
        SignOptions::new().with_proof_purpose(purpose),
    )
    .await
    .expect("sign");
    let mut doc = doc;
    doc["proof"] = serde_json::to_value(proof).unwrap();
    doc
}

fn refusal(err: VerificationError, rule: &str) {
    let msg = err.to_string();
    assert!(
        msg.contains(rule),
        "expected refusal naming {rule:?}, got: {msg}"
    );
    assert!(
        !msg.contains(&public_multibase()),
        "a refusal must not carry key material: {msg}"
    );
}

// ─── Wrong relationship ─────────────────────────────────────────────────

#[tokio::test]
async fn an_authentication_only_key_cannot_make_an_assertion() {
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({ "id": DID, "verificationMethod": [method(&vm, DID)], "authentication": [vm] }),
    )
    .await;

    let err = verifier
        .verify_raw(&signed(DID, &vm, "assertionMethod").await)
        .await
        .unwrap_err();
    refusal(err, "not listed under assertionMethod");

    verifier
        .verify_raw(&signed(DID, &vm, "authentication").await)
        .await
        .expect("the same key verifies for the purpose it is listed under");
}

#[tokio::test]
async fn an_assertion_only_key_cannot_authenticate() {
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({ "id": DID, "verificationMethod": [method(&vm, DID)], "assertionMethod": [vm] }),
    )
    .await;
    let err = verifier
        .verify_raw(&signed(DID, &vm, "authentication").await)
        .await
        .unwrap_err();
    refusal(err, "not listed under authentication");
}

#[tokio::test]
async fn a_key_agreement_key_signs_for_nothing() {
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({ "id": DID, "verificationMethod": [method(&vm, DID)], "keyAgreement": [vm] }),
    )
    .await;
    for purpose in ["assertionMethod", "authentication", "capabilityInvocation"] {
        assert!(verifier
            .verify_raw(&signed(DID, &vm, purpose).await)
            .await
            .is_err());
    }
    let err = verifier
        .verify_raw(&signed(DID, &vm, "keyAgreement").await)
        .await
        .unwrap_err();
    refusal(err, "keyAgreement never authorises a signature");
}

#[tokio::test]
async fn a_key_listed_under_no_relationship_signs_for_nothing() {
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({ "id": DID, "verificationMethod": [method(&vm, DID)] }),
    )
    .await;
    let err = verifier
        .verify_raw(&signed(DID, &vm, "assertionMethod").await)
        .await
        .unwrap_err();
    refusal(err, "not listed under assertionMethod");
}

#[tokio::test]
async fn an_unknown_purpose_is_refused() {
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({ "id": DID, "verificationMethod": [method(&vm, DID)], "assertionMethod": [vm] }),
    )
    .await;
    let err = verifier
        .verify_raw(&signed(DID, &vm, "somethingElse").await)
        .await
        .unwrap_err();
    refusal(err, "names no signing verification relationship");
}

// ─── Another DID's key ──────────────────────────────────────────────────

#[tokio::test]
async fn a_method_controlled_by_another_did_is_refused() {
    // The issuer's document embeds, under assertionMethod, a method named
    // under the issuer's DID but controlled by someone else.
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({ "id": DID, "assertionMethod": [method(&vm, OTHER)] }),
    )
    .await;
    let err = verifier
        .verify_raw(&signed(DID, &vm, "assertionMethod").await)
        .await
        .unwrap_err();
    refusal(err, "controller is not the DID that names it");
}

#[tokio::test]
async fn another_dids_key_listed_by_the_issuer_is_refused() {
    // The issuer lists a key of another DID under assertionMethod; a proof
    // naming that key is not the issuer's proof.
    let other_vm = format!("{OTHER}#key-1");
    let verifier = verifier_with(
        DID,
        json!({
            "id": DID,
            "verificationMethod": [method(&other_vm, OTHER)],
            "assertionMethod": [other_vm],
        }),
    )
    .await;
    let err = verifier
        .verify_raw(&signed(DID, &other_vm, "assertionMethod").await)
        .await
        .unwrap_err();
    assert!(
        matches!(err, VerificationError::IssuerMismatch(_)),
        "{err:?}"
    );
}

#[tokio::test]
async fn a_document_for_another_did_is_refused() {
    // The resolver returns a document whose id is not the DID resolved.
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({
            "id": OTHER,
            "verificationMethod": [method(&vm, DID)],
            "assertionMethod": [vm],
        }),
    )
    .await;
    let err = verifier
        .verify_raw(&signed(DID, &vm, "assertionMethod").await)
        .await
        .unwrap_err();
    refusal(err, "id is not the verificationMethod's DID");
}

// ─── References ─────────────────────────────────────────────────────────

#[tokio::test]
async fn a_relative_reference_resolves_against_the_document_id() {
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({
            "id": DID,
            "verificationMethod": [method(&vm, DID)],
            "assertionMethod": ["#key-1"],
        }),
    )
    .await;
    verifier
        .verify_raw(&signed(DID, &vm, "assertionMethod").await)
        .await
        .expect("#key-1 names the issuer's key-1");

    // …and a relative reference to a different fragment does not.
    let verifier = verifier_with(
        DID,
        json!({
            "id": DID,
            "verificationMethod": [method(&vm, DID)],
            "assertionMethod": ["#key-2"],
        }),
    )
    .await;
    assert!(verifier
        .verify_raw(&signed(DID, &vm, "assertionMethod").await)
        .await
        .is_err());
}

#[tokio::test]
async fn an_embedded_method_verifies_for_its_relationship_only() {
    let vm = format!("{DID}#key-1");
    let verifier = verifier_with(
        DID,
        json!({ "id": DID, "capabilityInvocation": [method(&vm, DID)] }),
    )
    .await;
    verifier
        .verify_raw(&signed(DID, &vm, "capabilityInvocation").await)
        .await
        .expect("embedded under capabilityInvocation");
    assert!(verifier
        .verify_raw(&signed(DID, &vm, "assertionMethod").await)
        .await
        .is_err());
}

// ─── did:key and did:peer ───────────────────────────────────────────────

fn did_key() -> (String, String) {
    let mb = public_multibase();
    (format!("did:key:{mb}"), format!("did:key:{mb}#{mb}"))
}

#[tokio::test]
async fn a_did_key_signs_for_every_signing_purpose() {
    let (did, vm) = did_key();
    let cached = Verifier::with_resolver(Arc::new(CachedDidResolver::new(Arc::new(
        DIDCacheClient::new(DIDCacheConfigBuilder::default().build())
            .await
            .unwrap(),
    ))));
    let offline = Verifier::for_did_key();
    for purpose in ProofPurpose::ALL {
        let doc = signed(&did, &vm, purpose.as_str()).await;
        offline
            .verify_raw(&doc)
            .await
            .unwrap_or_else(|e| panic!("did:key {purpose} offline: {e}"));
        cached
            .verify_raw(&doc)
            .await
            .unwrap_or_else(|e| panic!("did:key {purpose} via the cache: {e}"));
    }
    assert!(offline
        .verify_raw(&signed(&did, &vm, "keyAgreement").await)
        .await
        .is_err());
}

#[tokio::test]
async fn a_did_key_names_only_its_own_key() {
    let (did, _) = did_key();
    let err = Verifier::for_did_key()
        .verify_raw(&signed(&did, &format!("{did}#key-0"), "assertionMethod").await)
        .await
        .unwrap_err();
    refusal(err, "fragment must repeat the key id");
}

#[tokio::test]
async fn a_did_peer_v_key_signs_for_authentication_and_assertion_only() {
    // did:peer:2 with one V (verification) key: the resolver lists it under
    // authentication and assertionMethod as `#key-1`.
    let did = format!("did:peer:2.V{}", public_multibase());
    let vm = format!("{did}#key-1");
    let verifier = Verifier::with_resolver(Arc::new(CachedDidResolver::new(Arc::new(
        DIDCacheClient::new(DIDCacheConfigBuilder::default().build())
            .await
            .unwrap(),
    ))));
    for purpose in ["assertionMethod", "authentication"] {
        verifier
            .verify_raw(&signed(&did, &vm, purpose).await)
            .await
            .unwrap_or_else(|e| panic!("did:peer V key for {purpose}: {e}"));
    }
    assert!(verifier
        .verify_raw(&signed(&did, &vm, "capabilityInvocation").await)
        .await
        .is_err());
}

// ─── PurposeBound, for callers of DataIntegrityProof::verify ─────────────

#[tokio::test]
async fn purpose_bound_applies_the_rule_to_the_upstream_verify() {
    let vm = format!("{DID}#key-1");
    let client = client_with(
        DID,
        json!({ "id": DID, "verificationMethod": [method(&vm, DID)], "authentication": [vm] }),
    )
    .await;
    let resolver = CachedDidResolver::new(Arc::new(client));
    let mut doc = signed(DID, &vm, "assertionMethod").await;
    let proof: DataIntegrityProof = serde_json::from_value(doc["proof"].take()).unwrap();
    doc.as_object_mut().unwrap().remove("proof");

    let purpose = ProofPurpose::parse(&proof.proof_purpose).unwrap();
    assert!(proof
        .verify(
            &doc,
            &PurposeBound::new(&resolver, purpose),
            Default::default()
        )
        .await
        .is_err());
    assert!(resolver
        .resolve_vm_for_purpose(&vm, ProofPurpose::Authentication)
        .await
        .is_ok());
}

// ─── Migration: a key listed under both relationships ───────────────────

#[tokio::test]
async fn a_key_listed_for_both_purposes_verifies_either() {
    // The shape VTC and VTA DID documents publish today: `#key-0` under both
    // authentication and assertionMethod. Proofs made before and after the
    // move to `authentication` both verify.
    let vm = format!("{DID}#key-0");
    let verifier = verifier_with(
        DID,
        json!({
            "id": DID,
            "verificationMethod": [method(&vm, DID)],
            "authentication": [vm],
            "assertionMethod": [vm],
        }),
    )
    .await;
    for purpose in ["assertionMethod", "authentication"] {
        verifier
            .verify_raw(&signed(DID, &vm, purpose).await)
            .await
            .unwrap_or_else(|e| panic!("{purpose}: {e}"));
    }
}

// ─── Sign side ──────────────────────────────────────────────────────────

#[tokio::test]
async fn signing_refuses_a_purpose_no_verifier_accepts() {
    use trust_tasks_proof::affinidi::{sign_trust_task, SignError};
    let (did, vm) = did_key();
    let secret = Secret::generate_ed25519(Some(&vm), Some(&SEED));
    let doc = json!({ "id": "urn:uuid:s", "type": "https://example.com/spec/heartbeat/0.1",
        "issuer": did, "payload": {} });
    for purpose in ["keyAgreement", "nonsense"] {
        let err = sign_trust_task(
            &doc,
            &secret,
            SignOptions::new().with_proof_purpose(purpose),
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, SignError::DataIntegrity(_)),
            "{purpose}: {err}"
        );
    }
}
