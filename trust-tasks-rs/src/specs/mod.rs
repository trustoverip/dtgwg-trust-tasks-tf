//! Generated payload types for the *Trust Task specifications* hosted in this
//! repo's `specs/` registry.
//!
//! ⚠ This module's contents are produced by `trust-tasks-codegen`. Do not edit
//! files under `specs/` by hand. To regenerate:
//!
//! ```sh
//! cargo run -p trust-tasks-codegen
//! cargo fmt
//! ```
//!
//! Each module corresponds to one (slug, version) pair from the registry and
//! contains:
//!
//! * A `Payload` struct representing the request payload, with a [`Payload`]
//!   trait impl pinning its Type URI.
//! * A `Response` struct (when the spec defines a success response, per
//!   SPEC.md §4.4.1), with a second `Payload` impl whose `TYPE_URI` carries
//!   the `#response` fragment.
//! * Any shared `$defs` types referenced by request or response.
//!
//! The framework-defined `trust-task-error/0.1` spec is *not* generated here —
//! its payload is modelled by hand in [`crate::ErrorPayload`] with a richer
//! [`crate::TrustTaskCode`] enum.
//!
//! [`Payload`]: crate::Payload

// Generated code uses typify's struct shape; we suppress the crate-level
// lints that fire on its output rather than fighting the generator.
#![allow(missing_docs)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::to_string_trait_impl)]
// `oneOf` enums whose variants reference structs with differing field
// counts (e.g. `sync/event/0.1`'s `VaultUpsertedEvent` carries a full
// VaultEntry; `VaultDeletedEvent` carries only ids) emit a Rust enum
// with a size disparity clippy flags. Boxing the large variant to
// shrink the disparity would mean the codegen output diverges from
// typify's emission, breaking the drift check. Suppress the lint on
// generated code instead — the size disparity reflects the spec's
// natural shape, not a bug we'd fix at this layer.
#![allow(clippy::large_enum_variant)]
// JSON-Schema `description` fields are copied verbatim into Rust doc
// comments. Schemas embed full URIs and angle-bracketed grammar
// placeholders (e.g. `<prefix>/*`), neither of which rustdoc parses as
// nicely-rendered markdown. Rather than scrub every schema, we silence
// the rustdoc warnings on the generated tree only.
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::invalid_html_tags)]
// The struct builders name one setter per schema member, so a member called
// `add`, `sub`, `not` or `next` produces an inherent method whose name
// collides with a std trait method. Renaming it would put the Rust API out of
// step with the schema member it sets, which is the one property the builder
// has to keep.
#![allow(clippy::should_implement_trait)]
// A builder keeps each field as `Result<FieldType, String>`, so an optional
// map member whose presence is kept (`Option<HashMap<K, Map<…>>>`, e.g.
// `vta/dids/get/0.1`'s `methodDetails`) nests deeply enough to trip this lint.
// The type is the member's own shape; naming it would add a public alias per
// such member for no reader's benefit.
#![allow(clippy::type_complexity)]

/// The member-presence helper every generated module here reads an optional
/// member that admits `null` through
/// (`#[serde(deserialize_with = "crate::specs::__presence::present")]`; see
/// `trust-tasks-build`'s `presence` pass). One crate-private copy instead of
/// one per module: nothing here is public API. An application's modules,
/// generated against `::trust_tasks_rs`, carry their own copy instead, so they
/// need nothing new from the runtime. The leading underscores keep the name
/// out of the generated family modules' namespace: a slug segment starts with
/// a letter, so no family can be called `__presence`.
///
/// There is no helper that refuses a `null`: on a member whose schema has no
/// null form, serde's own `Option` reads a `null` as absent, and refusing it
/// is payload-schema validation's job (`PayloadPolicy::Validate`).
// Unused when no specification family is enabled.
#[allow(dead_code)]
pub(crate) mod __presence {
    /// Reads a present member, `null` included, as `Some`, so an absent member
    /// (`None`, via `#[serde(default)]`) stays distinct. For a member whose
    /// schema admits `null`.
    pub(crate) fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de>,
    {
        T::deserialize(deserializer).map(Some)
    }

    #[cfg(test)]
    mod tests {
        //! The helper on the field shapes the generator gives each kind of
        //! optional member, spelled as the generated code spells them.
        use serde::{Deserialize, Serialize};
        use serde_json::{json, Map, Value};

        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct Shapes {
            // An array or map: `Option<Vec<T>>` / `Option<Map>`, no helper.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            map: Option<Map<String, Value>>,
            // `{"type": "null"}`: `Option<()>`, `present`.
            #[serde(
                default,
                deserialize_with = "super::present",
                skip_serializing_if = "Option::is_none"
            )]
            unit: Option<()>,
            // Admits `null`: `Option<Option<T>>`, `present`.
            #[serde(
                default,
                deserialize_with = "super::present",
                skip_serializing_if = "Option::is_none"
            )]
            nullable: Option<Option<String>>,
            // Has no null form, any `T` (here one that would read `null`):
            // left as typify renders it.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            value: Option<Value>,
        }

        fn round_trip(value: Value) -> Shapes {
            let parsed: Shapes = serde_json::from_value(value.clone()).expect("decode");
            assert_eq!(serde_json::to_value(&parsed).unwrap(), value);
            parsed
        }

        #[test]
        fn absent_empty_and_null_stay_apart() {
            let absent = round_trip(json!({}));
            assert_eq!(
                (absent.map, absent.unit, absent.nullable, absent.value),
                (None, None, None, None)
            );
            let present = round_trip(json!({"map": {}, "unit": null, "nullable": null}));
            assert_eq!(present.map, Some(Map::new()));
            assert_eq!(present.unit, Some(()));
            assert_eq!(present.nullable, Some(None));
            let set = round_trip(json!({"nullable": "x", "value": false}));
            assert_eq!(set.nullable, Some(Some("x".into())));
            assert_eq!(set.value, Some(Value::Bool(false)));
        }

        #[test]
        fn a_null_without_a_null_form_reads_as_absent() {
            // Refusing it is payload-schema validation's job, not decoding's.
            for member in ["map", "value"] {
                let parsed: Shapes = serde_json::from_value(json!({ member: null })).expect(member);
                assert_eq!((parsed.map, parsed.value), (None, None), "{member}");
            }
            // `()` reads only `null`.
            assert!(serde_json::from_value::<Shapes>(json!({"unit": 1})).is_err());
        }
    }
}

// trust-tasks-codegen:begin
#[cfg(feature = "acl")]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod acl;
#[cfg(feature = "audit")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod audit;
#[cfg(feature = "auth")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod auth;
#[cfg(feature = "backup")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod backup;
#[cfg(feature = "chat")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod chat;
#[cfg(feature = "config")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod config;
#[cfg(feature = "confirm")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod confirm;
#[cfg(feature = "consent")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod consent;
#[cfg(feature = "credential-exchange")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod credential_exchange;
#[cfg(feature = "device")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod device;
#[cfg(feature = "did-management")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod did_management;
#[cfg(feature = "external")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod external;
#[cfg(feature = "git-ns")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod git_ns;
#[cfg(feature = "git-trust")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod git_trust;
#[cfg(feature = "governance")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod governance;
#[cfg(feature = "hcp")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod hcp;
#[cfg(feature = "keys")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod keys;
#[cfg(feature = "messaging")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod messaging;
#[cfg(feature = "notifications")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod notifications;
#[cfg(feature = "peer-check")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod peer_check;
#[cfg(feature = "persona")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod persona;
#[cfg(feature = "policy")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod policy;
#[cfg(feature = "process-attestation")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod process_attestation;
#[cfg(feature = "provision")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod provision;
#[cfg(feature = "push")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod push;
#[cfg(feature = "registry")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod registry;
#[cfg(feature = "rooms")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod rooms;
mod shared_defs;
#[cfg(feature = "sync")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod sync;
#[cfg(feature = "task-consent")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod task_consent;
pub mod trust_ceremony_receipt;
pub mod trust_task_control;
pub mod trust_task_discovery;
pub mod trust_task_next_step;
pub mod trust_task_ok;
#[cfg(feature = "vault")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod vault;
#[cfg(feature = "vetting")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod vetting;
#[cfg(feature = "voting")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod voting;
#[cfg(feature = "vrc")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod vrc;
#[cfg(feature = "vtn")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod vtn;
#[cfg(feature = "webvh")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod webvh;
#[cfg(feature = "witness")]
#[cfg(not(test))]
#[deprecated(
    note = "the prebuilt specification modules will be removed: pin the tasks you use with the `trust-tasks` CLI and generate them in build.rs with `trust-tasks-build` (GETTING-STARTED.md, \"Generate only the tasks you use\")"
)]
pub mod witness;
// trust-tasks-codegen:end
