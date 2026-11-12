//! Runs the shared acceptance-window vectors in `fixtures/consume/freshness.json`
//! at the repository root, which the TypeScript, Go and Dart runtimes also run:
//! item 4's expiry check, then the freshness bound, then the record expiry, so
//! that the four runtimes reach the same verdict on the same document and keep
//! its replay record for the same time (SPEC §7.2, *Bounding the record*).

use std::path::PathBuf;

use chrono::{DateTime, TimeDelta, Utc};
use serde_json::{json, Value};
use trust_tasks_rs::{
    FreshnessPolicy, TrustTask, DEFAULT_EXPIRY_HORIZON, DEFAULT_MAX_AGE, DEFAULT_SKEW,
};

fn vectors() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/consume/freshness.json");
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn instant(v: &Value) -> DateTime<Utc> {
    v.as_str().unwrap().parse().unwrap()
}

fn ms(v: &Value) -> Option<TimeDelta> {
    v.as_i64().map(TimeDelta::milliseconds)
}

fn policy(v: &Value) -> FreshnessPolicy {
    let mut policy = FreshnessPolicy::default();
    policy.skew = ms(&v["skewMs"]).unwrap();
    policy.max_age = ms(&v["maxAgeMs"]);
    policy.expiry_horizon = ms(&v["expiryHorizonMs"]);
    policy.require_issued_at = v["requireIssuedAt"].as_bool().unwrap_or(false);
    policy
}

#[test]
fn named_policies_are_the_library_defaults() {
    let vectors = vectors();
    let policies = &vectors["policies"];
    assert_eq!(policy(&policies["default"]), FreshnessPolicy::default());
    assert_eq!(
        policy(&policies["consequential"]),
        FreshnessPolicy::consequential()
    );
    let consequential = FreshnessPolicy::consequential();
    assert_eq!(consequential.skew, DEFAULT_SKEW);
    assert_eq!(consequential.max_age, Some(DEFAULT_MAX_AGE));
    assert_eq!(consequential.expiry_horizon, Some(DEFAULT_EXPIRY_HORIZON));
}

#[test]
fn shared_freshness_vectors() {
    let vectors = vectors();
    let cases = vectors["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let policy = policy(&vectors["policies"][case["policy"].as_str().unwrap()]);
        let now = instant(&case["now"]);

        let mut raw = json!({
            "id": "req-1",
            "type": "https://trusttasks.org/spec/acl/show/0.2",
            "payload": {}
        });
        for member in ["issuedAt", "expiresAt"] {
            if let Some(v) = case.get(member) {
                raw[member] = v.clone();
            }
        }

        let verdict = match serde_json::from_value::<TrustTask<Value>>(raw) {
            // A timestamp the reader refuses is a malformed document.
            Err(_) => "malformedRequest".to_string(),
            Ok(doc) => match doc
                .validate_basic(now, "")
                .and_then(|()| doc.validate_freshness(now, &policy))
            {
                Err(reason) => reason.code().as_str().to_string(),
                Ok(()) => {
                    let expected = match &case["recordExpiry"] {
                        Value::Null => None,
                        v => Some(instant(v)),
                    };
                    assert_eq!(
                        policy.record_expiry(&doc, now),
                        expected,
                        "{name}: record expiry"
                    );
                    "accept".to_string()
                }
            },
        };
        assert_eq!(verdict, case["verdict"].as_str().unwrap(), "{name}");
    }
}
