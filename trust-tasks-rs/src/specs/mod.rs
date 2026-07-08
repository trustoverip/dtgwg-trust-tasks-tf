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

// trust-tasks-codegen:begin
pub mod acl;
pub mod auth;
pub mod chat;
pub mod confirm;
pub mod consent;
pub mod device;
pub mod did_management;
pub mod messaging;
pub mod policy;
pub mod provision;
pub mod push;
pub mod registry;
pub mod sync;
pub mod trust_task_discovery;
pub mod vault;
pub mod vta;
pub mod webvh;
// trust-tasks-codegen:end
