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
pub mod confirm;
pub mod trust_task_discovery;
// trust-tasks-codegen:end
