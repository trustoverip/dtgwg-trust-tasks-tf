//! The Type URI → schema lookup a dispatching consumer needs.

#![cfg(feature = "validate")]

use serde_json::json;
use trust_tasks_rs::schema_index::schema_for;
use trust_tasks_rs::validate::against_schema;

const WEBVH_UPDATE: &str = "https://trusttasks.org/spec/vta/webvh/dids/update/1.0";

#[test]
fn a_dispatching_consumer_can_find_a_schema_by_type_uri() {
    assert!(
        schema_for(WEBVH_UPDATE).is_some(),
        "a consumer that dispatches on the URI must be able to validate by it"
    );
    assert!(schema_for("https://trusttasks.org/spec/not/a/task/9.9").is_none());
}

#[test]
fn a_valid_payload_passes() {
    let schema = schema_for(WEBVH_UPDATE).unwrap();
    let payload = json!({
        "did": "did:webvh:QmScid:example.com:acme",
        "document": { "id": "did:webvh:QmScid:example.com:acme" },
        "expectedVersionId": "3-QmPrior"
    });
    assert!(against_schema(schema, &payload).is_ok());
}

/// The defect this whole exercise came from.
///
/// `expected_version_id` is the optimistic-concurrency precondition in the wrong
/// case. A permissive consumer drops it without a word, publishes with no
/// lost-update protection, and the caller's own source still reads as though the
/// danger were handled. The schema is closed precisely so that the member is
/// *rejected* rather than *ignored*.
#[test]
fn a_safety_precondition_in_the_wrong_case_is_rejected_not_ignored() {
    let schema = schema_for(WEBVH_UPDATE).unwrap();
    let payload = json!({
        "did": "did:webvh:QmScid:example.com:acme",
        "expected_version_id": "3-QmPrior"
    });
    assert!(
        against_schema(schema, &payload).is_err(),
        "an unrecognised member must be refused — a member the consumer silently \
         ignores is a member the caller believes it sent"
    );
}

#[test]
fn an_unknown_member_is_refused() {
    let schema = schema_for(WEBVH_UPDATE).unwrap();
    let payload = json!({ "did": "did:webvh:x", "skipApproval": true });
    assert!(against_schema(schema, &payload).is_err());
}

/// `ext` is the framework's sanctioned extension slot (SPEC §4.5.1). A closed
/// payload that rejected it would break the one mechanism the spec provides for
/// carrying anything it does not define — including the origin an enrolled device
/// stamps on a relayed task.
#[test]
fn the_framework_ext_slot_is_permitted() {
    let schema = schema_for(WEBVH_UPDATE).unwrap();
    let payload = json!({
        "did": "did:webvh:QmScid:example.com:acme",
        "ext": { "openvtc.origin": "https://control.example.com" }
    });
    assert!(
        against_schema(schema, &payload).is_ok(),
        "closed payloads must still admit `ext`, or the relay cannot stamp an origin"
    );
}

#[test]
fn a_missing_subject_is_refused() {
    let schema = schema_for(WEBVH_UPDATE).unwrap();
    assert!(against_schema(schema, &json!({ "label": "no subject" })).is_err());
}

// ---------------------------------------------------------------------------
// The response side
// ---------------------------------------------------------------------------

const VTC_MEMBERS_SHOW: &str = "https://trusttasks.org/spec/vtc/members/show/0.1";

/// A producer can validate what it is about to *send back*.
///
/// Indexing only requests left every implementation able to check what it
/// received and not what it emitted — and its own output is the half it can
/// actually fix. The `#response` suffix is not a convention invented here: it
/// is what each generated `Response` type already declares as its `TYPE_URI`.
#[test]
fn a_producer_can_find_the_response_schema_for_a_task() {
    let uri = format!("{VTC_MEMBERS_SHOW}#response");
    assert!(
        schema_for(&uri).is_some(),
        "a producer that dispatches on the URI must be able to validate its own response"
    );
}

/// The request and response schemas for one task are different documents.
///
/// Pinned because the cheap implementation — returning the payload schema for
/// both — would pass every "is it Some" check while validating responses
/// against the wrong shape, which is worse than not validating them.
#[test]
fn the_response_schema_is_not_the_request_schema() {
    let req = schema_for(VTC_MEMBERS_SHOW).unwrap();
    let resp = schema_for(&format!("{VTC_MEMBERS_SHOW}#response")).unwrap();
    assert_ne!(req, resp, "a task's two sides must not share one schema");
}

/// `members/show`'s response is `{member: …}` and its request is `{did}`.
/// Each must refuse the other, or the index is not distinguishing them.
#[test]
fn each_side_refuses_the_other_side_document() {
    let req = schema_for(VTC_MEMBERS_SHOW).unwrap();
    let resp = schema_for(&format!("{VTC_MEMBERS_SHOW}#response")).unwrap();
    let request_doc = json!({ "did": "did:key:z6Mk" });
    assert!(against_schema(req, &request_doc).is_ok());
    assert!(
        against_schema(resp, &request_doc).is_err(),
        "the response schema must refuse a request document"
    );
}

/// A task with no response side has no `#response` entry, and `None` stays a
/// real answer rather than a silent pass.
#[test]
fn an_unknown_response_uri_is_none() {
    assert!(schema_for("https://trusttasks.org/spec/not/a/task/9.9#response").is_none());
}
