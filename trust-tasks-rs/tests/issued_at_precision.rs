//! #970: every instant this library stamps is written in whole seconds, and
//! every RFC 3339 precision is still read.
//!
//! SPEC §13.1 item 3 calls a full-precision timestamp a fingerprint and lets a
//! producer round `issuedAt` down. `chrono` writes `Utc::now()` at the clock's
//! precision — nine fractional digits on Linux — which is what every error
//! response, response and next-step document carried before.

use serde_json::{json, Value};
use trust_tasks_rs::{issued_at_now, Dispatcher, TrustTask};

/// The form written: `YYYY-MM-DDThh:mm:ssZ`, nothing after the seconds.
fn assert_whole_seconds(written: &Value) {
    let s = written.as_str().expect("an RFC 3339 string");
    let bytes = s.as_bytes();
    assert_eq!(s.len(), 20, "{s} is not whole seconds in UTC");
    assert_eq!((bytes[10], bytes[19]), (b'T', b'Z'), "{s}");
    assert!(
        s.chars()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7 | 10 | 13 | 16 | 19) || c.is_ascii_digit()),
        "{s}"
    );
}

fn request() -> TrustTask<Value> {
    serde_json::from_value(json!({
        "id": "5b0e8f3a-91c2-8d4e-a6f0-2c7d93b1e4a5",
        "threadId": "5b0e8f3a-91c2-8d4e-a6f0-2c7d93b1e4a5",
        "type": "https://real-life.org/trust-tasks/spec/sealed-envelope/0.1",
        "issuer": "did:key:z6MkProducer",
        "recipient": "did:key:z6MkConsumer",
        "issuedAt": "2026-10-27T18:04:11Z",
        "payload": {}
    }))
    .unwrap()
}

/// The issue's reproduction: the error document `dispatch_or_reject` builds.
#[test]
fn a_dispatcher_error_response_is_stamped_in_whole_seconds() {
    let dispatcher: Dispatcher<()> = Dispatcher::new();
    let err = dispatcher
        .dispatch_or_reject(request(), "urn:uuid:1")
        .unwrap_err();
    assert_whole_seconds(&serde_json::to_value(&err).unwrap()["issuedAt"]);
}

#[test]
fn reject_with_and_respond_with_stamp_whole_seconds() {
    let req = request();
    let error = req.reject_with("urn:uuid:2", trust_tasks_rs::StandardCode::TaskFailed);
    assert_whole_seconds(&serde_json::to_value(&error).unwrap()["issuedAt"]);
    let response = req.respond_with("urn:uuid:3", json!({}));
    assert_whole_seconds(&serde_json::to_value(&response).unwrap()["issuedAt"]);
}

#[test]
fn issued_at_now_rounds_down_never_up() {
    let before = chrono::Utc::now();
    let stamped = issued_at_now();
    assert!(stamped <= before + chrono::TimeDelta::seconds(1));
    assert!(stamped > before - chrono::TimeDelta::seconds(1));
    assert_eq!(chrono::Timelike::nanosecond(&stamped), 0);
}

/// Readers keep taking every precision RFC 3339 allows, and an offset: a
/// received document is read as written, never narrowed to what we write.
#[test]
fn every_rfc3339_precision_is_read() {
    for (issued_at, expires_at) in [
        ("2026-10-27T18:04:11Z", "2026-10-27T18:09:11Z"),
        ("2026-10-27T18:04:11.5Z", "2026-10-27T18:09:11.25Z"),
        ("2026-10-27T18:04:11.720Z", "2026-10-27T18:09:11.000Z"),
        ("2026-10-27T18:04:11.720263Z", "2026-10-27T18:09:11.720263Z"),
        (
            "2026-10-27T18:04:11.720263396Z",
            "2026-10-27T18:09:11.720263396Z",
        ),
        ("2026-10-27T20:04:11.720+02:00", "2026-10-27T18:09:11+00:00"),
    ] {
        let mut value = serde_json::to_value(request()).unwrap();
        value["issuedAt"] = json!(issued_at);
        value["expiresAt"] = json!(expires_at);
        let doc: TrustTask<Value> = serde_json::from_value(value)
            .unwrap_or_else(|e| panic!("{issued_at} / {expires_at} must parse: {e}"));
        assert_eq!(
            doc.issued_at.unwrap().timestamp(),
            chrono::DateTime::parse_from_rfc3339("2026-10-27T18:04:11Z")
                .unwrap()
                .timestamp()
        );
    }
}
