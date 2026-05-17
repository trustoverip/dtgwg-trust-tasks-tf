//! W3C Data Integrity proof attached to a Trust Task document.
//!
//! Implements the shape required by SPEC.md §4.7. The proof object is
//! deliberately minimal — the framework does not constrain the cryptographic
//! suite, so unknown members are preserved via `extra` for forward
//! compatibility with future suites.
//!
//! # ⚠ This module models proof *structure* only
//!
//! This crate **does not** implement any Data Integrity cryptosuite. A
//! [`Proof`] that round-trips through serde tells you nothing about whether
//! the proof is valid — both a legitimately-signed proof and a fabricated
//! one parse to the same `Proof { … }`. Consumers that act on `proof`
//! presence without verifying it (the [`ProofVerifier`] seam below, or an
//! equivalent in a cryptosuite crate) are non-conforming per SPEC.md §7.2
//! item 7 and trivially vulnerable to forged-issuer attacks.
//!
//! The framework's `validate_basic`, `reject_with`, and `respond_with`
//! helpers deliberately do not touch `proof`. Verification belongs in a
//! cryptosuite-specific companion crate that plugs in through
//! [`ProofVerifier`].

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::document::TrustTask;

/// A W3C Data Integrity proof object as required by SPEC.md §4.7.
///
/// When present, the proof binds the document's content to the party
/// identified by the document's `issuer` member. Verification is the
/// responsibility of the caller — this crate models the structure but does not
/// implement any specific cryptosuite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    /// The proof type, typically `"DataIntegrityProof"`.
    #[serde(rename = "type")]
    pub proof_type: String,

    /// The cryptographic suite identifier, e.g. `"eddsa-rdfc-2022"`.
    pub cryptosuite: String,

    /// A URL identifying the verification material. Per SPEC.md §4.7, the
    /// `verificationMethod` MUST resolve to material controlled by the party
    /// identified by the document's `issuer` member.
    #[serde(rename = "verificationMethod")]
    pub verification_method: String,

    /// When the proof was created.
    pub created: DateTime<Utc>,

    /// The relationship between the issuer and the assertion in the proof.
    #[serde(rename = "proofPurpose")]
    pub proof_purpose: String,

    /// The proof value, encoded according to the chosen cryptosuite.
    #[serde(rename = "proofValue")]
    pub proof_value: String,

    /// Any additional members carried by the proof (suite-specific or
    /// future-spec). Preserved on round-trip.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Plug-in seam for verifying a Trust Task document's `proof` member.
///
/// This crate intentionally implements **no** cryptosuites; verification
/// lives in companion crates (e.g. `trust-tasks-proof` with its `affinidi`
/// feature for the Affinidi Data Integrity stack). Consumer pipelines pick a verifier and
/// invoke it as part of their §7.2 validation step, alongside
/// [`crate::TrustTask::validate_basic`].
///
/// The trait is **async** — most verifiers need to resolve a
/// `verificationMethod` URI to verification material, which is naturally
/// I/O (DID resolution, JWKS fetch, …). Implementations that hold all
/// keys locally can simply `async`-no-op around their sync path.
///
/// A verifier MUST:
///
/// * Confirm the proof's `cryptosuite` matches an algorithm it implements;
///   reject otherwise with [`VerificationError::UnsupportedCryptosuite`].
/// * Resolve `verificationMethod` to verification material controlled by
///   the document's in-band `issuer` (SPEC.md §4.7, §4.8 paragraph 1).
/// * Validate the signature over the document with `proof` excluded per
///   the chosen Data Integrity suite's canonicalisation rules.
#[async_trait::async_trait]
pub trait ProofVerifier: Send + Sync {
    /// Verify `doc.proof` against `doc`'s content and `doc.issuer`.
    ///
    /// Returns `Ok(())` when the proof is valid. Implementations should
    /// match the spec's failure-mode taxonomy — see [`VerificationError`]
    /// variants — so the consumer pipeline can map to the right
    /// [`crate::StandardCode`] in its outbound `trust-task-error/0.1`
    /// response.
    async fn verify<P>(&self, doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: serde::Serialize + Send + Sync;
}

/// Reasons a [`ProofVerifier`] rejects a proof.
///
/// Mapping to [`crate::StandardCode`] for the wire:
/// `UnsupportedCryptosuite` / `MalformedProof` / `IssuerMismatch` →
/// [`crate::StandardCode::ProofInvalid`]; the consumer composes the
/// `trust-task-error/0.1` response with that code per SPEC.md §7.2 item 7.
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    /// The proof's `cryptosuite` is not implemented by this verifier.
    #[error("unsupported cryptosuite: {0}")]
    UnsupportedCryptosuite(String),

    /// The proof object is structurally invalid for the declared suite
    /// (missing required members, badly-encoded `proofValue`, etc.).
    #[error("malformed proof: {0}")]
    MalformedProof(String),

    /// The `verificationMethod` does not resolve to material controlled
    /// by the document's `issuer` — per SPEC.md §4.8 paragraph 1, the two
    /// MUST identify the same entity.
    #[error("verification method does not bind to document issuer: {0}")]
    IssuerMismatch(String),

    /// The signature did not verify over the canonicalised document.
    #[error("signature verification failed")]
    SignatureInvalid,

    /// Any other failure surfaced by the underlying cryptosuite
    /// implementation.
    #[error("proof verification failed: {0}")]
    Other(String),
}
