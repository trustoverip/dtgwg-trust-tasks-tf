//! Tests for the `validate` feature — runtime JSON Schema validation of
//! payloads against the embedded `payload.schema.json` files.

#![cfg(feature = "validate")]

use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, validate::ValidatedPayload};

#[test]
fn valid_payload_passes_schema_check() {
    let payload = serde_json::json!({
        "entry": {
            "subject": "did:web:alice.example",
            "role": "admin",
            "label": "Alice — primary admin"
        }
    });
    grant::Payload::validate_value(&payload).expect("valid payload accepted");
}

#[test]
fn missing_required_field_fails() {
    // `entry` is required by acl/grant/0.1.
    let payload = serde_json::json!({ "reason": "spring cleaning" });
    let err = grant::Payload::validate_value(&payload).expect_err("missing entry should reject");
    let combined = err.messages().join(" | ");
    assert!(
        combined.contains("entry") || combined.contains("required"),
        "expected error to mention `entry` or `required`, got: {combined}"
    );
}

#[test]
fn unknown_field_fails_when_schema_forbids_it() {
    // The schema declares `additionalProperties: false`, so an unknown
    // member is a hard reject even though serde would also catch it via
    // `deny_unknown_fields`. This proves runtime validation matches the
    // structural deserializer.
    let payload = serde_json::json!({
        "entry": { "subject": "did:web:alice.example", "role": "admin" },
        "noSuchField": 1
    });
    grant::Payload::validate_value(&payload)
        .expect_err("additionalProperties: false should reject unknown member");
}

#[test]
fn schema_json_is_embedded_at_compile_time() {
    // The const is exposed via the trait — implementors can read it for
    // logging or to wire up custom validators.
    let schema = grant::Payload::SCHEMA_JSON;
    assert!(schema.contains("\"$id\""));
    assert!(schema.contains("acl/grant/0.1"));
}
