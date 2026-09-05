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

// trust-tasks-codegen:begin
#[cfg(feature = "acl")]
pub mod acl;
#[cfg(feature = "audit")]
pub mod audit;
#[cfg(feature = "auth")]
pub mod auth;
#[cfg(feature = "chat")]
pub mod chat;
#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "confirm")]
pub mod confirm;
#[cfg(feature = "consent")]
pub mod consent;
#[cfg(feature = "credential-exchange")]
pub mod credential_exchange;
#[cfg(feature = "device")]
pub mod device;
#[cfg(feature = "did-management")]
pub mod did_management;
#[cfg(feature = "git-trust")]
pub mod git_trust;
#[cfg(feature = "governance")]
pub mod governance;
#[cfg(feature = "keys")]
pub mod keys;
#[cfg(feature = "messaging")]
pub mod messaging;
#[cfg(feature = "persona")]
pub mod persona;
#[cfg(feature = "policy")]
pub mod policy;
#[cfg(feature = "provision")]
pub mod provision;
#[cfg(feature = "push")]
pub mod push;
#[cfg(feature = "registry")]
pub mod registry;
#[cfg(feature = "rooms")]
pub mod rooms;
#[cfg(feature = "sync")]
pub mod sync;
#[cfg(feature = "task-consent")]
pub mod task_consent;
pub mod trust_ceremony_receipt;
pub mod trust_task_control;
pub mod trust_task_discovery;
pub mod trust_task_next_step;
pub mod trust_task_ok;
#[cfg(feature = "vault")]
pub mod vault;
#[cfg(feature = "vrc")]
pub mod vrc;
#[cfg(feature = "vta")]
pub mod vta;
#[cfg(feature = "vtc")]
pub mod vtc;
#[cfg(feature = "webvh")]
pub mod webvh;
#[cfg(feature = "witness")]
pub mod witness;
// trust-tasks-codegen:end
