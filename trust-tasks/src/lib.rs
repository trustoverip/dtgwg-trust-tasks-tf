//! One dependency line for the [Trust Tasks] framework.
//!
//! The framework ships as eight independently-versioned crates, because each
//! transport binding drags in a different (and heavy) dependency tree and you
//! should only pay for the one you use. That split is right for a build; it is
//! a tax at the front door. This crate is the front door: it re-exports the
//! others behind Cargo features so that getting started is one line and one
//! version.
//!
//! ```toml
//! trust-tasks = { version = "0.1", features = ["https", "proof-affinidi"] }
//! ```
//!
//! Everything here is a `pub use`. There are no wrapper types, no shims, and
//! nothing to keep in step: `trust_tasks::TrustTask` **is**
//! [`trust_tasks_rs::TrustTask`], and `trust_tasks::https::HttpsClient` **is**
//! `trust_tasks_https::HttpsClient`. Reaching for the underlying crate
//! directly later is a find-and-replace, not a migration.
//!
//! # Which crates do I need?
//!
//! (Module names are written out rather than linked: a link to a module behind a
//! Cargo feature you have not enabled is a broken link in *your* `cargo doc`.)
//!
//! | I want to… | feature | you get | underlying crate |
//! |---|---|---|---|
//! | model a Trust Task document, run the SPEC §7.2 consumer checks | *(always on)* | crate root: [`TrustTask`], [`consume_inbound`], [`specs`], [`RejectReason`] | `trust-tasks-rs` |
//! | send/receive over HTTPS (typed client + axum server) | `https` | `trust_tasks::https` | `trust-tasks-https` |
//! | send/receive over DIDComm v2.1 | `didcomm` | `trust_tasks::didcomm` | `trust-tasks-didcomm` |
//! | talk to Aries-lineage agents (DIDComm v1) | `didcomm-v1` | `trust_tasks::didcomm_v1` | `trust-tasks-didcomm-v1` |
//! | send/receive over the ToIP Trust Spanning Protocol | `tsp` | `trust_tasks::tsp` | `trust-tasks-tsp` |
//! | **sign** a document, or **verify** an inbound proof | `proof-affinidi` | `trust_tasks::proof` | `trust-tasks-proof` |
//! | validate payloads against their JSON Schema at runtime | `validate` | `trust_tasks::validate`, `trust_tasks::schema_index` | `trust-tasks-rs` |
//! | verify Trust Ceremony receipts and step digests | `ceremony` | `trust_tasks::ceremony` | `trust-tasks-ceremony` |
//! | build/parse `governance/capability/*` + `git-trust/*` wire documents | `capability-client` | `trust_tasks::capability_client` | `trust-tasks-capability-client` |
//! | bridge two bindings in one process | `all-transports` | all four transport modules | — |
//! | use `JwtBearerAuth` on the HTTPS server | `https-jwt` | `trust_tasks::https::JwtBearerAuth` | `trust-tasks-https` |
//!
//! Almost every real deployment wants **one transport plus `proof-affinidi`**:
//! a task whose specification declares `proof` REQUIRED (`acl/grant/0.1` is
//! one) cannot be produced or consumed without a signer and a verifier.
//!
//! # What this crate deliberately does not forward
//!
//! `trust-tasks-rs` carries **26 per-spec-family features** (`vault`, `acl`,
//! `keys`, …) so a size-sensitive build can compile only the families it
//! speaks. Those are **not** forwarded here. Only the `all-specs` umbrella is,
//! and it is on by default.
//!
//! That is a deliberate limit on what a facade is for. Twenty-six more feature
//! names in front of a newcomer is the problem this crate exists to remove, and
//! forwarding them would not even work reliably: Cargo unifies features across
//! the whole dependency graph, so trimming spec families only pays off when
//! nothing else in the graph asks for them. **If you are trimming spec
//! families, depend on `trust-tasks-rs` directly and skip this crate.** That is
//! a supported answer, not a workaround — this crate is a convenience for
//! getting started, and you have outgrown it.
//!
//! The same applies to *subtracting* a transport crate's own defaults. Cargo
//! features are additive, so nothing here can turn `trust-tasks-https`'s
//! `server` off or close `trust-tasks-didcomm-v1`'s `legacy-basic-message`
//! gate. Those need `default-features = false` on the crate itself.
//!
//! # A first round trip
//!
//! See [`GETTING-STARTED.md`] at the repo root for a signed `acl/grant`
//! exchange with both ends written out, the TypeScript equivalent, and the
//! four traps that reliably cost an afternoon. The Rust in that document is
//! extracted from `examples/acl_grant_roundtrip.rs` in this crate and a test
//! fails if the two drift, so it is code that compiles and runs rather than
//! code that once did.
//!
//! ```sh
//! cargo run -p trust-tasks --features https,proof-affinidi \
//!     --example acl_grant_roundtrip
//! ```
//!
//! [Trust Tasks]: https://trusttasks.org/
//! [`GETTING-STARTED.md`]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/GETTING-STARTED.md
//!
//! # Versioning
//!
//! This crate exposes `trust-tasks-rs` types in its own public API, so a
//! breaking change there breaks this crate's callers even when nothing here
//! changes. `cargo-semver-checks` cannot catch that: it compares each crate's
//! rustdoc against that crate's own published baseline, and does not track
//! type identity across dependency versions. The crates that share
//! `trust-tasks-rs` in their public API are therefore released as one
//! compatibility unit with a single shared version — see `version_group` in
//! `release-plz.toml`.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// The core crate lands at the root, flattened: `trust_tasks::TrustTask`,
// `trust_tasks::specs::…`, `trust_tasks::consume_inbound`. A glob rather than
// an enumerated list on purpose — an item added to `trust-tasks-rs` should
// appear here without anyone remembering to add it, which is the failure mode
// a hand-maintained re-export list has.
pub use trust_tasks_rs::*;

/// The core crate itself, under its own name.
///
/// Useful when you want to be explicit about where a type comes from, or need
/// to name the crate in a path that a glob re-export cannot express.
pub use trust_tasks_rs as rs;

/// HTTPS transport binding — typed client, axum-based server, bearer-token
/// identity. Re-export of [`trust_tasks_https`].
#[cfg(feature = "https")]
#[cfg_attr(docsrs, doc(cfg(feature = "https")))]
pub use trust_tasks_https as https;

/// DIDComm v2.1 transport binding, built on `affinidi-messaging-didcomm`.
/// Re-export of [`trust_tasks_didcomm`].
#[cfg(feature = "didcomm")]
#[cfg_attr(docsrs, doc(cfg(feature = "didcomm")))]
pub use trust_tasks_didcomm as didcomm;

/// DIDComm **v1** transport binding, for Aries-lineage agents that speak v1
/// only. Re-export of [`trust_tasks_didcomm_v1`].
#[cfg(feature = "didcomm-v1")]
#[cfg_attr(docsrs, doc(cfg(feature = "didcomm-v1")))]
pub use trust_tasks_didcomm_v1 as didcomm_v1;

/// ToIP Trust Spanning Protocol transport binding, built on `affinidi-tsp`.
/// Re-export of [`trust_tasks_tsp`].
#[cfg(feature = "tsp")]
#[cfg_attr(docsrs, doc(cfg(feature = "tsp")))]
pub use trust_tasks_tsp as tsp;

/// Signing and proof verification. `proof::ProofExt` adds `.sign()` to a
/// [`TrustTask`]; `proof::affinidi::Verifier` is what a server hands to
/// `with_verifier`. Re-export of [`trust_tasks_proof`].
#[cfg(feature = "proof-affinidi")]
#[cfg_attr(docsrs, doc(cfg(feature = "proof-affinidi")))]
pub use trust_tasks_proof as proof;

/// Trust Ceremony verification — salted step digests, receipt checking,
/// completion rules. Re-export of [`trust_tasks_ceremony`].
///
/// Note this crate does **not** depend on `trust-tasks-rs`; it is a standalone
/// verifier over a ceremony definition.
#[cfg(feature = "ceremony")]
#[cfg_attr(docsrs, doc(cfg(feature = "ceremony")))]
pub use trust_tasks_ceremony as ceremony;

/// Wire helpers for the capability families — `governance/capability/*` and
/// `git-trust/*` document builders, envelope parsing, reply classification.
/// Re-export of [`trust_tasks_capability_client`].
#[cfg(feature = "capability-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "capability-client")))]
pub use trust_tasks_capability_client as capability_client;
