//! `git-ns/bridge/event/0.6`: `bindCompleted` may carry `supersedes`.

#![cfg(all(feature = "validate", feature = "git-ns"))]
#![allow(deprecated)]
use trust_tasks_rs::{specs::git_ns::bridge::event::v0_6 as event, validate::ValidatedPayload};

fn payload(supersedes: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "namespace": "ns_new",
        "event": {
            "type": "bindCompleted",
            "jobId": "job_1",
            "ownerId": "91827364",
            "kind": "organization",
            "supersedes": supersedes
        }
    })
}

#[test]
fn supersedes_round_trips() {
    let value = payload(serde_json::json!(["ns_old1", "ns_old2"]));
    let parsed: event::Payload = serde_json::from_value(value.clone()).expect("deserialize");
    assert_eq!(serde_json::to_value(&parsed).expect("serialize"), value);
    event::Payload::validate_value(&value).expect("schema accepts it");
}

#[test]
fn bind_completed_without_supersedes_still_valid() {
    let mut value = payload(serde_json::json!(["x"]));
    value["event"].as_object_mut().unwrap().remove("supersedes");
    serde_json::from_value::<event::Payload>(value.clone()).expect("deserialize");
    event::Payload::validate_value(&value).expect("schema accepts it");
}

#[test]
fn empty_or_duplicate_supersedes_is_rejected() {
    for bad in [serde_json::json!([]), serde_json::json!(["a", "a"])] {
        assert!(
            event::Payload::validate_value(&payload(bad)).is_err(),
            "invalid supersedes was accepted"
        );
    }
}
