//! SPEC.md §4.8.2 / §7.2 item 8 — audience binding enforcement.
//!
//! Covers the cross-recipient replay attack: a document signed without an
//! in-band `recipient` is replayable to any consumer who can verify the
//! proof, since the signature does not cover the audience identity.

use chrono::Utc;
use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, Payload, Proof, StandardCode, TrustTask};

fn entry() -> grant::AclEntry {
    grant::AclEntry {
        allowed_keys: None,
        subject: "did:web:alice.example".into(),
        role: "admin".parse().unwrap(),
        scopes: vec![],
        label: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        expires_at: None,
        approve: None,
        step_up: None,
        ext: None,
    }
}

fn dummy_proof() -> Proof {
    Proof {
        proof_type: "DataIntegrityProof".into(),
        cryptosuite: "eddsa-rdfc-2022".into(),
        verification_method: "did:web:org.example#key-1".into(),
        created: Utc::now(),
        proof_purpose: "assertionMethod".into(),
        proof_value: "z3kg".into(),
        extra: Default::default(),
    }
}

/// SPEC §4.8.2 — proof present, recipient absent, non-bearer spec ⇒
/// reject as malformed_request.
#[test]
fn proof_without_recipient_rejected_on_non_bearer_spec() {
    let mut doc = TrustTask::for_payload(
        "req-1",
        grant::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );
    doc.issuer = Some("did:web:org.example".into());
    doc.proof = Some(dummy_proof());
    // recipient deliberately omitted

    let err = doc.enforce_audience_binding().unwrap_err();
    assert_eq!(err.code(), StandardCode::MalformedRequest);
    // The message names the rule for diagnosis.
    let msg = err.to_string();
    assert!(msg.contains("audience binding") || msg.contains("§4.8.2"));
}

/// SPEC §4.8.2 — proof present *and* recipient in-band ⇒ accepted.
#[test]
fn proof_with_recipient_passes() {
    let mut doc = TrustTask::for_payload(
        "req-1",
        grant::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );
    doc.issuer = Some("did:web:org.example".into());
    doc.recipient = Some("did:web:maintainer.example".into());
    doc.proof = Some(dummy_proof());

    assert!(doc.enforce_audience_binding().is_ok());
}

/// No proof ⇒ rule does not engage. The transport may still bind audience
/// out-of-band, but §4.8.2 only governs proof-carrying documents.
#[test]
fn no_proof_passes_regardless_of_recipient() {
    let mut doc = TrustTask::for_payload(
        "req-1",
        grant::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );
    doc.issuer = Some("did:web:org.example".into());
    // No proof, no recipient.

    assert!(doc.enforce_audience_binding().is_ok());
}

/// Codegen-generated ACL specs are non-bearer (the meta-schema's default
/// applies; no `bearer: true` in front matter), so `IS_BEARER == false`.
#[test]
fn acl_grant_is_not_a_bearer_spec() {
    // The assertions are over compile-time constants — clippy would prefer
    // a `const _: () = assert!(!…)` form, but as test functions they are
    // load-bearing documentation of the spec's bearer status. Suppress
    // both candidate lints rather than reach for the const trick.
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(!<grant::Payload as Payload>::IS_BEARER);
        assert!(!<grant::Response as Payload>::IS_BEARER);
    }
}

/// A hand-rolled bearer payload demonstrates the trait default override:
/// proof + no recipient is accepted because IS_BEARER = true at the type
/// level (set by the spec's front matter, not by consumer policy).
#[test]
fn hand_rolled_bearer_payload_skips_audience_check() {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct PublicAttestation {
        subject: String,
        claim: String,
    }

    impl Payload for PublicAttestation {
        const TYPE_URI: &'static str = "https://example.com/spec/public-attestation/0.1";
        const IS_BEARER: bool = true;
    }

    let mut doc = TrustTask::for_payload(
        "att-1",
        PublicAttestation {
            subject: "did:web:alice.example".into(),
            claim: "is-over-18".into(),
        },
    );
    doc.issuer = Some("did:web:authority.example".into());
    doc.proof = Some(dummy_proof());
    // No recipient — fine for a bearer document.

    assert!(doc.enforce_audience_binding().is_ok());
}
