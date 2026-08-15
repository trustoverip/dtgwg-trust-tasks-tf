//! Reference Rust library for the [Trust Tasks] framework.
//!
//! Trust Tasks are self-contained, transport-agnostic, JSON-based descriptions
//! of the verifiable work that happens between two parties. This crate models
//! the framework-level document envelope and provides a [`TransportHandler`]
//! trait through which concrete transports (REST, DIDComm, ...) plug in their
//! identity, integrity, and freshness semantics.
//!
//! The crate tracks the `SPEC.md` framework at version `0.4`; it emits
//! `trust-task-error/0.4` and still parses the `0.1` snake_case error codes for
//! backwards compatibility. See the spec sections referenced from each item for
//! the normative text.
//!
//! [Trust Tasks]: https://trusttasks.org/

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

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
pub mod guards;
pub mod handlers;
pub mod specs;

#[cfg(feature = "validate")]
#[cfg_attr(docsrs, doc(cfg(feature = "validate")))]
pub mod schema_index;

#[cfg(feature = "validate")]
#[cfg_attr(docsrs, doc(cfg(feature = "validate")))]
pub mod validate;

pub use ceremony::{Ceremony, CeremonyPrev};
pub use consume::{consume_inbound, ConsumeOutcome, ProofPolicy, PROOF_NOT_ACCEPTED_BY_POLICY};
pub use dispatcher::Dispatcher;
pub use document::{ErrorResponse, JsonLdContext, TrustTask};
pub use error::{
    ErrorPayload, InResponseTo, ParseCodeError, RejectReason, StandardCode, TrustTaskCode,
};
pub use payload::Payload;
pub use proof::{
    erase_verifier, DynProofVerifier, ErasedVerifier, Proof, ProofVerifier, VerificationError,
};
pub use transport::{ConsistencyError, ResolvedParties, TransportContext, TransportHandler};
pub use type_uri::{ParseTypeUriError, TypeUri, Variant};
