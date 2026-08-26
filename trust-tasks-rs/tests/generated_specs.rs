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
    // `label` declares a `maxLength` (SPEC §7.3), so it deserializes into a
    // validating newtype over `String` rather than a bare `String`.
    assert_eq!(
        doc.payload.entry.label.as_deref().map(String::as_str),
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
    let entry: grant::AclEntry = grant::AclEntry::builder()
        .subject("did:web:alice.example")
        .role("admin")
        .try_into()
        .expect("acl entry builder");
    let payload: grant::Payload = grant::Payload::builder()
        .entry(entry)
        .try_into()
        .expect("acl grant payload builder");

    let doc = TrustTask::for_payload("req-1", payload);
    assert_eq!(doc.type_uri, TypeUri::canonical("acl/grant", 0, 1).unwrap());
}

/// The generated types are `#[non_exhaustive]`, so this file — a separate
/// crate from `trust-tasks-rs` — is the only place in the workspace that can
/// assert the property a consumer sees: the builder is a *complete*
/// construction path, naming only the members the specification requires.
///
/// This is the whole trade of 0.14.0. `acl/grant`'s minimal request needed a
/// struct literal spelling 13 members that carry no information (12 `None`s
/// and an empty `vec![]`); every member added to `acl/grant` since has been a
/// source break for every one of those literals. The two calls below name two
/// members, and a thirteenth member added tomorrow leaves them alone.
#[test]
fn the_builder_names_only_the_required_members() {
    let entry: grant::AclEntry = grant::AclEntry::builder()
        .subject("did:web:alice.example")
        .role("admin")
        .try_into()
        .expect("acl entry builder");
    let payload: grant::Payload = grant::Payload::builder()
        .entry(entry)
        .try_into()
        .expect("acl grant payload builder");

    // Absent optional members stay absent on the wire — the builder fills
    // them with `None`, not with an empty object.
    let json = serde_json::to_value(&payload).expect("serialise");
    assert_eq!(
        json,
        serde_json::json!({
            "entry": { "subject": "did:web:alice.example", "role": "admin" }
        })
    );
}

/// A missing required member is a `Result`, not a panic and not a document
/// that fails validation at the far end.
#[test]
fn the_builder_reports_a_missing_required_member() {
    let err = grant::AclEntry::try_from(grant::AclEntry::builder().role("admin"))
        .expect_err("subject is required");
    assert!(
        err.to_string().contains("subject"),
        "error should name the missing member, got: {err}"
    );
}

/// SPEC §4.4.1 pairing, on the type. `RequestPayload::Response` is what
/// removes the second type parameter from `HttpsClient::send`, and a
/// mismatched pair can no longer be written.
#[test]
fn request_payload_names_the_response_type() {
    fn response_uri<P: trust_tasks_rs::RequestPayload>() -> &'static str {
        <P::Response as Payload>::TYPE_URI
    }

    assert_eq!(
        response_uri::<grant::Payload>(),
        "https://trusttasks.org/spec/acl/grant/0.1#response"
    );
}
