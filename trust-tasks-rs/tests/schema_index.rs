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
