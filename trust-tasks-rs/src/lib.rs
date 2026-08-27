//! Reference Rust library for the [Trust Tasks] framework.
//!
//! Trust Tasks are self-contained, transport-agnostic, JSON-based descriptions
//! of the verifiable work that happens between two parties. This crate models
//! the framework-level document envelope and provides a [`TransportHandler`]
//! trait through which concrete transports (REST, DIDComm, ...) plug in their
//! identity, integrity, and freshness semantics.
//!
//! The crate tracks the `SPEC.md` framework at version `0.4`. The
//! `trust-task-error` version it emits has one definition —
//! [`trust_task_error_type_uri`] — and it still parses the `0.1` snake_case
//! error codes for backwards compatibility. See the spec sections referenced
//! from each item for the normative text.
//!
//! # The stateful §7.2 checks
//!
//! Most of §7.2 is a pure function of one document. Two items are not, and
//! both live behind seams the caller supplies:
//!
//! * [`replay`] — item 11's duplicate-execution record, so a §8.4 retry or a
//!   replayed envelope does not grant the same ACL entry twice.
//!   [`InMemoryReplayGuard`] is the default; [`ReplayGuard`] is the trait a
//!   replicated deployment implements over a shared store.
//! * [`freshness`] — the acceptance window over `issuedAt` / `expiresAt` that
//!   §7.2 makes the *same bound* as the replay record's retention.
//!
//! [`consume_inbound`] takes them together as [`ConsumeChecks`].
//!
//! [Trust Tasks]: https://trusttasks.org/
//!
//! # Versioning
//!
//! Every other crate in this workspace that depends on this one exposes its
//! types in their own public API. A breaking change here is therefore a
//! breaking change in all of them, so they are released as a single
//! compatibility unit sharing one version — see `version_group` in
//! `release-plz.toml`.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod async_dispatch;
mod canonical;
mod ceremony;
mod consume;
mod dispatcher;
mod document;
mod error;
mod payload;
mod proof;
mod transport;
mod type_uri;

pub mod discovery;
pub mod freshness;
pub mod guards;
pub mod handlers;
pub mod replay;
pub mod specs;

#[cfg(feature = "validate")]
#[cfg_attr(docsrs, doc(cfg(feature = "validate")))]
pub mod schema_index;

#[cfg(feature = "validate")]
#[cfg_attr(docsrs, doc(cfg(feature = "validate")))]
pub mod validate;

pub use async_dispatch::AsyncDispatcher;
pub use canonical::{canonical_json, sha256_hex};
pub use ceremony::{Ceremony, CeremonyPrev};
pub use consume::{
    consume_inbound, ConsumeChecks, ConsumeOutcome, NoValidator, PayloadPolicy, PayloadValidator,
    ProofPolicy, PROOF_NOT_ACCEPTED_BY_POLICY,
};
pub use dispatcher::Dispatcher;
pub use document::{trust_task_error_type_uri, ErrorResponse, JsonLdContext, TrustTask};
pub use error::{
    ErrorPayload, InResponseTo, ParseCodeError, RejectReason, StandardCode, TrustTaskCode,
    IDENTITY_MISMATCH_WIRE_MESSAGE, PROOF_INVALID_WIRE_MESSAGE, STALE_WIRE_MESSAGE,
    UNAVAILABLE_WIRE_MESSAGE, WRONG_RECIPIENT_WIRE_MESSAGE,
};
pub use freshness::{FreshnessPolicy, StaleReason, DEFAULT_MAX_AGE, DEFAULT_SKEW};
pub use payload::{Payload, RequestPayload, SpecPolicy};
pub use proof::{
    erase_verifier, DynProofVerifier, ErasedVerifier, Proof, ProofVerifier, VerificationError,
};
pub use replay::{
    document_digest, DocumentDigest, InMemoryReplayGuard, ReplayGuard, ReplayGuardError,
    ReplayPolicy, ReplayVerdict,
};
pub use transport::{ConsistencyError, ResolvedParties, TransportContext, TransportHandler};
pub use type_uri::{ParseTypeUriError, TypeUri, Variant};
