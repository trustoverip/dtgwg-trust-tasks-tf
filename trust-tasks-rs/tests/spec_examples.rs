//! End-to-end tests against the example documents in SPEC.md.
//!
//! Each test names the section of SPEC.md it covers so a reader can cross-
//! check the wire form against the normative source.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use trust_tasks_rs::{
    handlers::{InMemoryHandler, NoopHandler},
    ErrorPayload, JsonLdContext, StandardCode, TransportHandler, TrustTask, TrustTaskCode, TypeUri,
};

/// The payload of `acl/change-role/0.1`, the registered specification SPEC.md's
/// examples are drawn from. Modelled by hand rather than taken from the
/// generated bindings so this file stays a test of the *framework* envelope.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AclChangeRole {
    subject: String,
    from_role: String,
    to_role: String,
}

/// SPEC.md §4.2 Example 1 — a complete Trust Task document, `proof` included
/// because `acl/change-role` declares one REQUIRED.
#[test]
fn spec_example_1_round_trips() {
    let json = r#"{
        "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
        "type": "https://trusttasks.org/spec/acl/change-role/0.1",
        "issuer": "did:web:org.example",
        "recipient": "did:web:maintainer.example",
        "issuedAt": "2026-06-10T14:00:00Z",
        "expiresAt": "2026-06-11T14:00:00Z",
        "payload": {
            "subject": "did:web:bob.example",
            "fromRole": "member",
            "toRole": "moderator"
        },
        "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-rdfc-2022",
            "verificationMethod": "did:web:org.example#key-1",
            "created": "2026-06-10T14:00:00Z",
            "proofPurpose": "assertionMethod",
            "proofValue": "z5xy..."
        }
    }"#;

    let doc: TrustTask<AclChangeRole> = serde_json::from_str(json).unwrap();
    assert_eq!(
        doc.type_uri,
        "https://trusttasks.org/spec/acl/change-role/0.1"
            .parse()
            .unwrap()
    );
    assert_eq!(doc.type_uri.slug(), "acl/change-role");
    assert!(doc.proof.is_some());

    let rendered: serde_json::Value = serde_json::to_value(&doc).unwrap();
    let expected: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(rendered, expected);
}

/// SPEC.md §4.6 Example 2 — JSON-LD `@context` is carried as-is.
#[test]
fn spec_example_2_preserves_jsonld_context() {
    let json = r#"{
        "@context": [
            "https://www.w3.org/ns/credentials/v2",
            "https://trusttasks.org/spec/acl/change-role/0.1"
        ],
        "id": "urn:uuid:7d8b1e3a-9a72-4f86-9d04-2a4b6c2c5e10",
        "type": "https://trusttasks.org/spec/acl/change-role/0.1",
        "issuer": "did:web:org.example",
        "recipient": "did:web:maintainer.example",
        "issuedAt": "2026-06-10T14:00:00Z",
        "payload": {
            "subject": "did:web:bob.example",
            "fromRole": "member",
            "toRole": "moderator"
        }
    }"#;

    let doc: TrustTask<AclChangeRole> = serde_json::from_str(json).unwrap();
    match doc.context.as_ref().unwrap() {
        JsonLdContext::Multiple(items) => assert_eq!(items.len(), 2),
        other => panic!("expected JsonLdContext::Multiple, got {other:?}"),
    }
}

/// SPEC.md §8.2 Example 5 — an error-response document.
#[test]
fn spec_example_5_error_response() {
    let json = r#"{
        "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
        "type": "https://trusttasks.org/spec/trust-task-error/0.2",
        "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
        "issuer": "did:web:maintainer.example",
        "recipient": "did:web:org.example",
        "issuedAt": "2026-06-11T14:05:00Z",
        "payload": {
            "code": "expired",
            "message": "Task expired at 2026-06-11T14:00:00Z.",
            "retryable": false
        },
        "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-rdfc-2022",
            "verificationMethod": "did:web:maintainer.example#key-1",
            "created": "2026-06-11T14:05:00Z",
            "proofPurpose": "assertionMethod",
            "proofValue": "z58D..."
        }
    }"#;

    let doc: TrustTask<ErrorPayload> = serde_json::from_str(json).unwrap();
    assert_eq!(doc.type_uri.slug(), "trust-task-error");
    assert_eq!(
        doc.payload.code,
        TrustTaskCode::Standard(StandardCode::Expired)
    );
    assert!(!doc.payload.retryable);
    assert!(doc.proof.is_some());
    assert_eq!(
        doc.thread_id.as_deref(),
        Some("4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2")
    );
}

/// SPEC.md §8.5 Example 6 — extended error code with `details`.
#[test]
fn spec_example_6_extended_error_code() {
    let json = r#"{
        "id": "c4d2f713-9a8e-4d04-b29c-2f1b0b4cbe71",
        "type": "https://trusttasks.org/spec/trust-task-error/0.2",
        "threadId": "7b1e4d09-3c62-4a17-9f2d-51c8ab3e7d40",
        "issuer": "did:web:maintainer.example",
        "recipient": "did:web:org.example",
        "issuedAt": "2026-06-12T11:04:00Z",
        "payload": {
            "code": "acl/change-role:roleNotRecognized",
            "message": "Role 'principal' is not part of this ACL's role vocabulary.",
            "retryable": false,
            "details": {
                "offendingRole": "principal",
                "knownRoles": ["member", "moderator", "admin"]
            }
        }
    }"#;

    let doc: TrustTask<ErrorPayload> = serde_json::from_str(json).unwrap();
    match &doc.payload.code {
        TrustTaskCode::Extended { slug, local } => {
            // A hierarchical slug is the namespace here — the case SPEC.md §8.5
            // rule 1 covers and a single-segment example never exercised.
            assert_eq!(slug, "acl/change-role");
            assert_eq!(local, "roleNotRecognized");
        }
        other => panic!("expected extension code, got {other:?}"),
    }
    assert!(doc.payload.details.is_some());
}

/// SPEC.md §4.8.1 — in-band identity is authoritative; transport fills in
/// absent members. Verified end-to-end through [`TransportHandler::resolve_parties`].
#[test]
fn in_band_identity_is_authoritative() {
    let json = r#"{
        "id": "id-1",
        "type": "https://trusttasks.org/spec/acl/change-role/0.1",
        "issuer": "did:web:org.example",
        "payload": {"subject":"s","fromRole":"member","toRole":"moderator"}
    }"#;
    let doc: TrustTask<AclChangeRole> = serde_json::from_str(json).unwrap();

    let handler = InMemoryHandler::new()
        .with_local("did:web:maintainer.example")
        .with_peer("did:web:org.example");

    let resolved = handler.resolve_parties(&doc).unwrap();
    // Issuer from in-band; recipient filled from transport.
    assert_eq!(resolved.issuer.as_deref(), Some("did:web:org.example"));
    assert_eq!(
        resolved.recipient.as_deref(),
        Some("did:web:maintainer.example")
    );
}

/// SPEC.md §7.2 item 6 — an in-band issuer that contradicts the transport-
/// authenticated sender is rejected.
#[test]
fn identity_mismatch_is_rejected() {
    let json = r#"{
        "id": "id-2",
        "type": "https://trusttasks.org/spec/acl/change-role/0.1",
        "issuer": "did:web:attacker.example",
        "payload": {"subject":"s","fromRole":"member","toRole":"moderator"}
    }"#;
    let doc: TrustTask<AclChangeRole> = serde_json::from_str(json).unwrap();

    let handler = InMemoryHandler::new()
        .with_local("did:web:maintainer.example")
        .with_peer("did:web:org.example");

    assert!(handler.resolve_parties(&doc).is_err());
}

/// SPEC.md §9.2 — an unauthenticated transport falls back entirely to the
/// in-band members.
#[test]
fn noop_handler_falls_back_to_in_band() {
    let json = r#"{
        "id": "id-3",
        "type": "https://trusttasks.org/spec/acl/change-role/0.1",
        "issuer": "did:web:org.example",
        "recipient": "did:web:maintainer.example",
        "payload": {"subject":"s","fromRole":"member","toRole":"moderator"}
    }"#;
    let doc: TrustTask<AclChangeRole> = serde_json::from_str(json).unwrap();

    let resolved = NoopHandler::new().resolve_parties(&doc).unwrap();
    assert_eq!(resolved.issuer.as_deref(), Some("did:web:org.example"));
    assert_eq!(
        resolved.recipient.as_deref(),
        Some("did:web:maintainer.example")
    );
}

/// SPEC.md §7.2 item 4 — expired documents are flagged.
#[test]
fn expired_document_is_flagged() {
    let json = r#"{
        "id": "id-4",
        "type": "https://trusttasks.org/spec/acl/change-role/0.1",
        "expiresAt": "2026-04-12T09:31:00Z",
        "payload": {"subject":"s","fromRole":"member","toRole":"moderator"}
    }"#;
    let doc: TrustTask<AclChangeRole> = serde_json::from_str(json).unwrap();
    let now: DateTime<Utc> = "2026-05-01T00:00:00Z".parse().unwrap();
    assert!(doc.is_expired_at(now));
}

/// SPEC.md §4.4.1 — `#response` is preserved and routes separately from
/// the bare request URI.
#[test]
fn response_variant_is_distinct_from_request() {
    let request: TypeUri = "https://trusttasks.org/spec/acl/grant/0.1".parse().unwrap();
    let response: TypeUri = "https://trusttasks.org/spec/acl/grant/0.1#response"
        .parse()
        .unwrap();
    assert_ne!(request, response);
    assert_eq!(request, response.bare());
    assert!(response.is_response());
}
