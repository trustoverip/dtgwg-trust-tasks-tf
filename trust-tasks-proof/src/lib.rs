//! Pluggable [`ProofVerifier`](trust_tasks_rs::ProofVerifier)
//! implementations for the Trust Tasks framework.
//!
//! The framework's [`ProofVerifier`](trust_tasks_rs::ProofVerifier) trait is
//! the seam where cryptosuite implementations plug in; this crate is the
//! umbrella that hosts those implementations behind Cargo features so a
//! single dependency line opts in to a specific backend without dragging
//! in the others.
//!
//! ## Backends
//!
//! | Cargo feature | Module                    | Backed by                              |
//! |---------------|---------------------------|----------------------------------------|
//! | `affinidi` ✱  | [`affinidi`]              | `affinidi-data-integrity` (EdDSA suites) |
//!
//! ✱ = enabled by default. Disable default features and opt in to the
//! backends you want via `default-features = false` + an explicit
//! `features = [...]` list.
//!
//! ## Quickstart (`affinidi` backend)
//!
//! ```rust,ignore
//! use trust_tasks_proof::affinidi::Verifier;
//! use trust_tasks_rs::ProofVerifier;
//!
//! // did:key only — offline, no I/O. Good for tests and self-issued docs.
//! let verifier = Verifier::for_did_key();
//! verifier.verify(&inbound_doc).await?;
//! ```
//!
//! Producers sign with the same backend's
//! [`affinidi::sign_trust_task`] — defaults to `eddsa-jcs-2022` /
//! `assertionMethod` and enforces the issuer↔verificationMethod binding
//! at sign time, so its output verifies with the stock
//! [`affinidi::Verifier`] by construction:
//!
//! ```rust,ignore
//! use trust_tasks_proof::affinidi::{sign_trust_task, SignOptions};
//!
//! let signed = sign_trust_task(&doc_value, &secret, SignOptions::new()).await?;
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

#[cfg(feature = "affinidi")]
pub mod affinidi;
