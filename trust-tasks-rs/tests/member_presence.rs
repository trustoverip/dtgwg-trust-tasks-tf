//! A generated payload type keeps a member's presence: absent, present and
//! empty, and an explicit `null` stay three different things on decode and
//! through a decode-then-encode round trip.
//!
//! typify models an optional array as a bare `Vec` that is skipped when empty
//! and an optional nullable member as an `Option` that is skipped when `None`,
//! so `[]` and `null` used to decode as, and re-encode to, an absent member.
//! Where a schema gives the states different meanings that changes the
//! document's meaning: `acl/update` reads an explicit `null` `label` as
//! "clear the label" and an absent one as "leave it unchanged", and an
//! `expiresAt: null` makes the entry permanent. `trust-tasks-build`'s
//! `presence` pass rewrites those fields (`Option<Vec<T>>`,
//! `Option<Option<T>>`), and reads every other optional member through a
//! helper that refuses a `null` the schema has no form for, instead of reading
//! it as absent. Its own tests cover the field shapes on a fixture schema
//! (a `{"type": "null"}` member, an intrinsic `default`, `true`), and
//! `specs::__presence`'s unit tests run the helpers on each shape.
#![cfg(feature = "acl")]
// Exercises the prebuilt specification modules on purpose; they are deprecated
// in favour of generating the tasks an application uses (trust-tasks-build).
#![allow(deprecated)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use trust_tasks_rs::specs::acl::show::v0_3 as show;
use trust_tasks_rs::specs::acl::update::v0_3 as update;

/// Decode `value` into `T`, re-encode it, and require the same JSON back.
fn round_trips<T: DeserializeOwned + Serialize>(value: Value) -> T {
    let parsed: T = serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("deserialize {value}: {e}"));
    let rendered = serde_json::to_value(&parsed).expect("re-serialize");
    assert_eq!(rendered, value, "round-trip changed the payload");
    parsed
}

#[test]
fn an_empty_optional_array_stays_present_and_empty() {
    let r: show::Response = round_trips(json!({ "entry": null, "redactedFields": [] }));
    assert_eq!(r.redacted_fields.as_deref().map(<[_]>::len), Some(0));
}

#[test]
fn an_absent_optional_array_stays_absent() {
    let r: show::Response = round_trips(json!({ "entry": null }));
    assert!(r.redacted_fields.is_none());
}

#[test]
fn a_non_empty_optional_array_round_trips() {
    let r: show::Response = round_trips(json!({ "entry": null, "redactedFields": ["label"] }));
    assert_eq!(r.redacted_fields.map(|v| v.len()), Some(1));
}

#[test]
fn an_explicit_null_stays_null() {
    // "Explicit `null` clears it." / "Explicit `null` makes the entry permanent."
    let p: update::Payload = round_trips(json!({
        "subject": "did:example:alice",
        "label": null,
        "expiresAt": null
    }));
    assert!(
        matches!(p.label, Some(None)),
        "label: null decoded as {:?}",
        p.label
    );
    assert!(matches!(p.expires_at, Some(None)));
}

#[test]
fn an_absent_nullable_member_stays_absent() {
    // "Omitted leaves it unchanged."
    let p: update::Payload = round_trips(json!({ "subject": "did:example:alice" }));
    assert!(p.label.is_none());
    assert!(p.expires_at.is_none());
}

#[test]
fn a_present_nullable_member_round_trips() {
    let p: update::Payload = round_trips(json!({
        "subject": "did:example:alice",
        "label": "Alice",
        "expiresAt": "2030-01-01T00:00:00Z"
    }));
    assert_eq!(
        p.label.flatten().as_deref().map(String::as_str),
        Some("Alice")
    );
}

#[test]
fn a_null_where_the_schema_forbids_it_is_still_refused() {
    // `redactedFields` admits no `null`; reading one as absent would accept a
    // document the schema refuses.
    let err = serde_json::from_value::<show::Response>(json!({
        "entry": null,
        "redactedFields": null
    }));
    assert!(err.is_err());
}

#[test]
fn a_null_on_an_optional_member_without_a_null_form_is_refused() {
    // acl/update 0.3 `purposes`: "There is no null form." Before, `null` was
    // read as absent ("leave unchanged") and re-encoded without the member.
    for member in ["purposes", "approve", "keys", "ext"] {
        let err = serde_json::from_value::<update::Payload>(json!({
            "subject": "did:example:alice",
            member: null
        }))
        .expect_err(member)
        .to_string();
        assert!(err.contains("no null form"), "{member}: {err}");
    }
}

#[test]
fn the_builder_writes_each_state() {
    let clear = update::Payload::builder()
        .subject("did:example:alice")
        .label(None::<update::PayloadLabel>)
        .expires_at(None::<chrono::DateTime<chrono::Utc>>);
    let clear: update::Payload = clear.try_into().expect("build");
    assert_eq!(
        serde_json::to_value(&clear).unwrap(),
        json!({ "subject": "did:example:alice", "label": null, "expiresAt": null })
    );

    let set: update::Payload = update::Payload::builder()
        .subject("did:example:alice")
        .label(Some(update::PayloadLabel::try_from("Alice").unwrap()))
        .try_into()
        .expect("build");
    assert_eq!(
        serde_json::to_value(&set).unwrap(),
        json!({ "subject": "did:example:alice", "label": "Alice" })
    );

    let untouched: update::Payload = update::Payload::builder()
        .subject("did:example:alice")
        .try_into()
        .expect("build");
    assert_eq!(
        serde_json::to_value(&untouched).unwrap(),
        json!({ "subject": "did:example:alice" })
    );

    let none_redacted: show::Response = show::Response::builder()
        // `entry` is *required* and nullable, so it is a single `Option`:
        // `None` here is the member's `null` (always written), not the
        // optional-member rule above where `None::<T>` sends `null` and leaving
        // the setter out omits the member.
        .entry(None::<show::AclEntry>)
        .redacted_fields(Vec::<show::ResponseRedactedFieldsItem>::new())
        .try_into()
        .expect("build");
    assert_eq!(
        serde_json::to_value(&none_redacted).unwrap(),
        json!({ "entry": null, "redactedFields": [] })
    );
}

/// An optional map member: `{}` stays present and empty, absent stays absent,
/// and `null` (no null form) is refused.
#[cfg(feature = "vta")]
mod map_member {
    use super::*;
    use trust_tasks_rs::specs::vta::did_templates::update::v3_2 as templates;

    fn payload(defaults: Option<Value>) -> Value {
        let mut template = json!({
            "schemaVersion": 1,
            "kind": "agent",
            "name": "example",
            "document": { "id": "did:webvh:{SCID}:example.com:{DID}" }
        });
        if let Some(d) = defaults {
            template["defaults"] = d;
        }
        json!({ "name": "example", "template": template })
    }

    #[test]
    fn an_empty_map_stays_present_and_empty() {
        let p: templates::Payload = round_trips(payload(Some(json!({}))));
        assert_eq!(p.template.defaults.map(|m| m.len()), Some(0));
    }

    #[test]
    fn an_absent_map_stays_absent() {
        let p: templates::Payload = round_trips(payload(None));
        assert!(p.template.defaults.is_none());
    }

    #[test]
    fn a_null_map_is_refused() {
        let err = serde_json::from_value::<templates::Payload>(payload(Some(Value::Null)));
        assert!(err.unwrap_err().to_string().contains("no null form"));
    }
}
