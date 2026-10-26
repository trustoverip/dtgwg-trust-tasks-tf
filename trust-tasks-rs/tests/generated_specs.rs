//! Round-trip tests that exercise the `trust-tasks-codegen`-produced types
//! against the example documents in each spec's `spec.md`.
//!
//! These tests are the seam between the generator and the framework crate:
//! they prove that the generated `Payload` / `Response` structs implement the
//! `Payload` trait correctly, deserialize and serialize the wire form, and
//! compose with `TrustTask::for_payload`.
//!
//! Its specimens are `acl` payloads, so a build without that family skips it.
#![cfg(feature = "acl")]

use trust_tasks_rs::{specs::acl::grant::v0_2 as grant, Payload, TrustTask, TypeUri};

/// Sample taken from `specs/acl/grant/0.2/spec.md` — "A context
/// administrator at a key-holding maintainer" (its placeholder `proof`
/// omitted).
#[test]
fn acl_grant_request_round_trips() {
    let json = r#"{
        "id": "urn:uuid:4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
        "type": "https://trusttasks.org/spec/acl/grant/0.2#request",
        "issuer": "did:web:root-admin.example",
        "recipient": "did:web:vta.example",
        "issuedAt": "2026-10-02T10:00:00Z",
        "payload": {
            "entry": {
                "subject": "did:web:alice.example",
                "role": "admin",
                "act": { "scope": "contexts", "contexts": ["ctx/payments"] },
                "keys": { "scope": "all" },
                "capabilities": { "scope": "ceiling" },
                "approve": { "scope": "none" },
                "label": "Alice — payments context admin"
            },
            "reason": "Owns the payments integration."
        }
    }"#;

    let doc: TrustTask<grant::Payload> = serde_json::from_str(json).unwrap();
    assert_eq!(doc.payload.entry.subject.as_str(), "did:web:alice.example");
    assert_eq!(&*doc.payload.entry.role, "admin");
    // `label` declares a `maxLength` (SPEC §7.3), so it deserializes into a
    // validating newtype over `String` rather than a bare `String`.
    assert_eq!(
        doc.payload.entry.label.as_deref().map(String::as_str),
        Some("Alice — payments context admin")
    );
    assert!(matches!(
        doc.payload.entry.act,
        grant::AuthorityScope::Contexts(_)
    ));

    // The generated `Payload` trait impl pins the right Type URI.
    assert_eq!(doc.type_uri.bare(), grant::Payload::type_uri().bare());
    assert_eq!(
        grant::Payload::TYPE_URI,
        "https://trusttasks.org/spec/acl/grant/0.2"
    );

    // And it round-trips bit-for-bit through the wire form.
    let rendered = serde_json::to_value(&doc).unwrap();
    let expected: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(rendered, expected);
}

/// Sample from `specs/acl/grant/0.2/spec.md` — "Successful grant" response
/// (its placeholder `proof` omitted).
#[test]
fn acl_grant_response_round_trips() {
    let json = r#"{
        "id": "urn:uuid:5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
        "type": "https://trusttasks.org/spec/acl/grant/0.2#response",
        "threadId": "urn:uuid:4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
        "issuer": "did:web:vta.example",
        "recipient": "did:web:root-admin.example",
        "issuedAt": "2026-10-02T10:00:01Z",
        "payload": {
            "entry": {
                "subject": "did:web:alice.example",
                "role": "admin",
                "act": { "scope": "contexts", "contexts": ["ctx/payments"] },
                "keys": { "scope": "all" },
                "capabilities": { "scope": "ceiling" },
                "approve": { "scope": "none" },
                "label": "Alice — payments context admin",
                "createdAt": "2026-10-02T10:00:01Z",
                "createdBy": "did:web:root-admin.example",
                "delegatedBy": "did:web:root-admin.example"
            }
        }
    }"#;

    let doc: TrustTask<grant::Response> = serde_json::from_str(json).unwrap();
    assert_eq!(&*doc.payload.entry.role, "admin");
    assert_eq!(
        doc.payload.entry.created_by.as_deref(),
        Some("did:web:root-admin.example")
    );
    assert!(doc.type_uri.is_response());
    assert_eq!(doc.type_uri, grant::Response::type_uri());
    assert_eq!(
        grant::Response::TYPE_URI,
        "https://trusttasks.org/spec/acl/grant/0.2#response"
    );
}

/// An entry builder carrying every member `acl/grant/0.2` requires except
/// `subject`, built only through the generated builders — the path a consumer
/// outside `trust-tasks-rs` has. 0.2 requires `act`, `keys` and
/// `capabilities` beside `subject` and `role`; there is no default for any of
/// them.
fn entry_without_subject() -> grant::builder::AclEntry {
    let act: grant::AuthorityScopeAll = grant::AuthorityScopeAll::builder()
        .scope("all")
        .try_into()
        .expect("act scope builder");
    let keys: grant::KeyScopeNone = grant::KeyScopeNone::builder()
        .scope("none")
        .try_into()
        .expect("key scope builder");
    let capabilities: grant::CapabilityScopeCeiling = grant::CapabilityScopeCeiling::builder()
        .scope("ceiling")
        .try_into()
        .expect("capability scope builder");
    grant::AclEntry::builder()
        .role("admin")
        .act(act)
        .keys(keys)
        .capabilities(capabilities)
}

/// The minimal entry.
fn minimal_entry() -> grant::AclEntry {
    entry_without_subject()
        .subject("did:web:alice.example")
        .try_into()
        .expect("acl entry builder")
}

/// `TrustTask::for_payload` auto-fills the Type URI from the trait — the
/// per-spec ergonomics the codegen exists to enable.
#[test]
fn for_payload_pulls_type_uri_from_trait() {
    let payload: grant::Payload = grant::Payload::builder()
        .entry(minimal_entry())
        .try_into()
        .expect("acl grant payload builder");

    let doc = TrustTask::for_payload("req-1", payload);
    assert_eq!(doc.type_uri, TypeUri::canonical("acl/grant", 0, 2).unwrap());
}

/// The generated types are `#[non_exhaustive]`, so this file — a separate
/// crate from `trust-tasks-rs` — is the only place in the workspace that can
/// assert the property a consumer sees: the builder is a *complete*
/// construction path, naming only the members the specification requires.
///
/// This is the whole trade of 0.14.0. `acl/grant/0.1`'s minimal request needed
/// a struct literal spelling 13 members that carried no information (12
/// `None`s and an empty `vec![]`); every member added to `acl/grant` since has
/// been a source break for every one of those literals. The builder calls in
/// `minimal_entry` name the five members 0.2 requires, and a member added
/// tomorrow leaves them alone.
#[test]
fn the_builder_names_only_the_required_members() {
    let payload: grant::Payload = grant::Payload::builder()
        .entry(minimal_entry())
        .try_into()
        .expect("acl grant payload builder");

    // Absent optional members stay absent on the wire — the builder fills
    // them with `None`, not with an empty object.
    let json = serde_json::to_value(&payload).expect("serialise");
    assert_eq!(
        json,
        serde_json::json!({
            "entry": {
                "subject": "did:web:alice.example",
                "role": "admin",
                "act": { "scope": "all" },
                "keys": { "scope": "none" },
                "capabilities": { "scope": "ceiling" }
            }
        })
    );
}

/// A missing required member is a `Result`, not a panic and not a document
/// that fails validation at the far end.
#[test]
fn the_builder_reports_a_missing_required_member() {
    let err = grant::AclEntry::try_from(entry_without_subject()).expect_err("subject is required");
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
        "https://trusttasks.org/spec/acl/grant/0.2#response"
    );
}
