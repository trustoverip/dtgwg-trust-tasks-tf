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
//! `Option<Option<T>>`); its own tests cover the other shapes (maps, a
//! `{"type": "null"}` member) on a fixture schema.
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
    round_trips::<show::Response>(json!({ "entry": null, "redactedFields": [] }));
}

#[test]
fn an_absent_optional_array_stays_absent() {
    round_trips::<show::Response>(json!({ "entry": null }));
}

#[test]
fn an_explicit_null_stays_null() {
    // "Explicit `null` clears it." / "Explicit `null` makes the entry permanent."
    round_trips::<update::Payload>(json!({
        "subject": "did:example:alice",
        "label": null,
        "expiresAt": null
    }));
}

#[test]
fn an_absent_nullable_member_stays_absent() {
    // "Omitted leaves it unchanged."
    round_trips::<update::Payload>(json!({ "subject": "did:example:alice" }));
}
