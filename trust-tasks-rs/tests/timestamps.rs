//! Runs the shared RFC 3339 vectors in `fixtures/consume/timestamps.json` at
//! the repository root, which the TypeScript, Go and Dart runtimes also run:
//! every field of `issuedAt` / `expiresAt` must be in range. chrono's parser
//! alone refuses most out-of-range fields, but reads `:60` as a leap second.

use std::path::PathBuf;

use serde_json::{json, Value};
use trust_tasks_rs::TrustTask;

fn vectors() -> Value {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/consume/timestamps.json");
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn document(member: &str, value: &str) -> Value {
    let mut doc = json!({
        "id": "req-1",
        "type": "https://trusttasks.org/spec/acl/show/0.2",
        "payload": {}
    });
    doc[member] = json!(value);
    doc
}

#[test]
fn shared_timestamp_vectors() {
    let vectors = vectors();
    let valid = vectors["valid"].as_array().unwrap();
    let invalid = vectors["invalid"].as_array().unwrap();
    assert!(!valid.is_empty() && !invalid.is_empty());
    for member in ["issuedAt", "expiresAt"] {
        for v in valid {
            let value = v["value"].as_str().unwrap();
            let doc: TrustTask<Value> = serde_json::from_value(document(member, value))
                .unwrap_or_else(|e| panic!("{member} {value} ({}): {e}", v["why"]));
            let at = if member == "issuedAt" {
                doc.issued_at
            } else {
                doc.expires_at
            };
            assert_eq!(
                at.map(|t| t.timestamp_millis()),
                v["epochMs"].as_i64(),
                "{member} {value} ({})",
                v["why"]
            );
        }
        for v in invalid {
            let value = v["value"].as_str().unwrap();
            let parsed = serde_json::from_value::<TrustTask<Value>>(document(member, value));
            assert!(
                parsed.is_err(),
                "{member} {value} ({}) was accepted",
                v["why"]
            );
        }
    }
}

#[test]
fn absent_and_null_timestamps_are_still_none() {
    let doc: TrustTask<Value> = serde_json::from_value(json!({
        "id": "req-1",
        "type": "https://trusttasks.org/spec/acl/show/0.2",
        "issuedAt": null,
        "payload": {}
    }))
    .unwrap();
    assert!(doc.issued_at.is_none() && doc.expires_at.is_none());
}
