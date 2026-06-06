//! End-to-end tests for [`Dispatcher`] against the generated ACL types.

use serde_json::Value;
use trust_tasks_rs::{
    specs::acl::{grant, revoke},
    Dispatcher, Payload, RejectReason, StandardCode, TrustTask, TypeUri,
};

#[derive(Debug, PartialEq)]
enum Outcome {
    Granted(String),
    Revoked(String),
}

fn make_dispatcher() -> Dispatcher<Outcome> {
    Dispatcher::new()
        .on::<grant::v0_1::Payload, _>(|doc| Outcome::Granted(doc.payload.entry.subject.clone()))
        .on::<revoke::v0_1::Payload, _>(|doc| Outcome::Revoked(doc.payload.subject.clone()))
}

fn doc_of<P: Payload>(payload: serde_json::Value) -> TrustTask<Value> {
    let mut d = TrustTask::new("req-1", P::type_uri(), payload);
    d.issuer = Some("did:web:org.example".into());
    d.recipient = Some("did:web:maintainer.example".into());
    d
}

#[test]
fn routes_to_matching_handler_per_type_uri() {
    let dispatcher = make_dispatcher();

    let grant_doc = doc_of::<grant::v0_1::Payload>(serde_json::json!({
        "entry": { "subject": "did:web:alice.example", "role": "admin" }
    }));
    assert_eq!(
        dispatcher.dispatch(grant_doc).unwrap(),
        Outcome::Granted("did:web:alice.example".to_string())
    );

    let revoke_doc = doc_of::<revoke::v0_1::Payload>(serde_json::json!({
        "subject": "did:web:bob.example"
    }));
    assert_eq!(
        dispatcher.dispatch(revoke_doc).unwrap(),
        Outcome::Revoked("did:web:bob.example".to_string())
    );
}

#[test]
fn unregistered_type_uri_rejects_as_unsupported_type() {
    let dispatcher = make_dispatcher();
    let mut doc = TrustTask::new(
        "req-1",
        TypeUri::canonical("acl/list", 0, 1).unwrap(),
        serde_json::json!({}),
    );
    doc.issuer = Some("did:web:x.example".into());

    let err = dispatcher.dispatch(doc).unwrap_err();
    match err {
        RejectReason::UnsupportedType { type_uri } => {
            assert_eq!(type_uri, "https://trusttasks.org/spec/acl/list/0.1");
        }
        other => panic!("expected UnsupportedType, got {other:?}"),
    }
}

#[test]
fn known_slug_at_unregistered_version_rejects_as_unsupported_version() {
    // `acl/grant` is registered at 0.1; a document for `acl/grant/0.2` has a
    // recognized slug but an unregistered MAJOR.MINOR → unsupportedVersion
    // (SPEC §5.2 / §8.3), distinct from the unknown-slug unsupportedType.
    let dispatcher = make_dispatcher();
    let mut doc = TrustTask::new(
        "req-ver",
        TypeUri::canonical("acl/grant", 0, 2).unwrap(),
        serde_json::json!({ "entry": { "subject": "did:web:alice.example", "role": "admin" } }),
    );
    doc.issuer = Some("did:web:org.example".into());

    let err = dispatcher.dispatch(doc).unwrap_err();
    match err {
        RejectReason::UnsupportedVersion { type_uri } => {
            assert_eq!(type_uri, "https://trusttasks.org/spec/acl/grant/0.2");
        }
        other => panic!("expected UnsupportedVersion, got {other:?}"),
    }
}

#[test]
fn payload_mismatch_rejects_as_malformed_request() {
    let dispatcher = make_dispatcher();

    // `entry` is required for grant payloads; this document is malformed.
    let bad = doc_of::<grant::v0_1::Payload>(serde_json::json!({ "not_a_field": 1 }));
    let err = dispatcher.dispatch(bad).unwrap_err();
    assert_eq!(err.code(), StandardCode::MalformedRequest);
}

#[test]
fn registered_uris_lists_what_consumer_implements() {
    let dispatcher = make_dispatcher();
    let uris = dispatcher.registered_uris();
    assert_eq!(uris.len(), 2);
    assert!(uris.contains(&"https://trusttasks.org/spec/acl/grant/0.1"));
    assert!(uris.contains(&"https://trusttasks.org/spec/acl/revoke/0.1"));
}

/// SPEC.md §4.4.1 item 1: the bare URI and the `#request`-fragment form
/// are semantically equivalent; consumers MUST accept both. The dispatcher
/// canonicalises both registration and dispatch keys.
#[test]
fn request_fragment_routes_to_same_handler_as_bare_uri() {
    let dispatcher = make_dispatcher();

    let mut explicit_request = doc_of::<grant::v0_1::Payload>(serde_json::json!({
        "entry": { "subject": "did:web:carol.example", "role": "admin" }
    }));
    explicit_request.type_uri = explicit_request.type_uri.with_request();
    assert!(explicit_request.type_uri.to_string().ends_with("#request"));

    let outcome = dispatcher.dispatch(explicit_request).unwrap();
    assert_eq!(
        outcome,
        Outcome::Granted("did:web:carol.example".to_string())
    );
}

#[test]
fn dispatcher_returns_handler_result_verbatim() {
    // Demonstrates the common pattern: handlers return Result<Resp, Error>,
    // the dispatcher wraps that in its own Result for routing failures.
    let dispatcher: Dispatcher<Result<String, &'static str>> =
        Dispatcher::new().on::<grant::v0_1::Payload, _>(|doc| {
            if doc.payload.entry.subject.starts_with("did:") {
                Ok(format!("granted {}", doc.payload.entry.subject))
            } else {
                Err("subject must be a DID")
            }
        });

    let ok = dispatcher
        .dispatch(doc_of::<grant::v0_1::Payload>(serde_json::json!({
            "entry": { "subject": "did:web:alice.example", "role": "admin" }
        })))
        .unwrap();
    assert_eq!(ok, Ok("granted did:web:alice.example".to_string()));

    let business_fail = dispatcher
        .dispatch(doc_of::<grant::v0_1::Payload>(serde_json::json!({
            "entry": { "subject": "alice", "role": "admin" }
        })))
        .unwrap();
    assert_eq!(business_fail, Err("subject must be a DID"));
}
