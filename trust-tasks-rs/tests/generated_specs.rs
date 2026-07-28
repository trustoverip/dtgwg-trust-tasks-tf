//! Round-trip tests that exercise the `trust-tasks-codegen`-produced types
//! against the example documents in each spec's `spec.md`.
//!
//! These tests are the seam between the generator and the framework crate:
//! they prove that the generated `Payload` / `Response` structs implement the
//! `Payload` trait correctly, deserialize and serialize the wire form, and
//! compose with `TrustTask::for_payload`.

use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, Payload, TrustTask, TypeUri};

/// Sample taken verbatim from `specs/acl/grant/0.1/spec.md` — "A new admin is
/// added".
#[test]
fn acl_grant_request_round_trips() {
    let json = r#"{
        "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
        "type": "https://trusttasks.org/spec/acl/grant/0.1",
        "issuer": "did:web:org.example",
        "recipient": "did:web:maintainer.example",
        "issuedAt": "2026-05-16T10:00:00Z",
        "payload": {
            "entry": {
                "subject": "did:web:alice.example",
                "role": "admin",
                "label": "Alice — primary admin"
            }
        }
    }"#;

    let doc: TrustTask<grant::Payload> = serde_json::from_str(json).unwrap();
    assert_eq!(doc.payload.entry.subject, "did:web:alice.example");
    assert_eq!(&*doc.payload.entry.role, "admin");
    assert_eq!(
        doc.payload.entry.label.as_deref(),
        Some("Alice — primary admin")
    );

    // The generated `Payload` trait impl pins the right Type URI.
    assert_eq!(doc.type_uri, grant::Payload::type_uri());
    assert_eq!(
        grant::Payload::TYPE_URI,
        "https://trusttasks.org/spec/acl/grant/0.1"
    );

    // And it round-trips bit-for-bit through the wire form.
    let rendered = serde_json::to_value(&doc).unwrap();
    let expected: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(rendered, expected);
}

/// Sample from `specs/acl/grant/0.1/spec.md` — "Successful grant" response.
#[test]
fn acl_grant_response_round_trips() {
    let json = r#"{
        "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
        "type": "https://trusttasks.org/spec/acl/grant/0.1#response",
        "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
        "issuer": "did:web:maintainer.example",
        "recipient": "did:web:org.example",
        "issuedAt": "2026-05-16T10:00:01Z",
        "payload": {
            "entry": {
                "subject": "did:web:alice.example",
                "role": "admin",
                "label": "Alice — primary admin",
                "createdAt": "2026-05-16T10:00:01Z",
                "createdBy": "did:web:org.example"
            }
        }
    }"#;

    let doc: TrustTask<grant::Response> = serde_json::from_str(json).unwrap();
    assert_eq!(&*doc.payload.entry.role, "admin");
    assert_eq!(
        doc.payload.entry.created_by.as_deref(),
        Some("did:web:org.example")
    );
    assert!(doc.type_uri.is_response());
    assert_eq!(doc.type_uri, grant::Response::type_uri());
    assert_eq!(
        grant::Response::TYPE_URI,
        "https://trusttasks.org/spec/acl/grant/0.1#response"
    );
}

/// `TrustTask::for_payload` auto-fills the Type URI from the trait — the
/// per-spec ergonomics the codegen exists to enable.
#[test]
fn for_payload_pulls_type_uri_from_trait() {
    let payload = grant::Payload {
        entry: grant::AclEntry {
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
        },
        reason: None,
        ext: None,
    };

    let doc = TrustTask::for_payload("req-1", payload);
    assert_eq!(doc.type_uri, TypeUri::canonical("acl/grant", 0, 1).unwrap());
}
