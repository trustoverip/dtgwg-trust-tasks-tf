//! [`Verifier`] — a [`ProofVerifier`] implementation backed by
//! `affinidi-data-integrity`.
//!
//! Supports the W3C Data Integrity cryptosuites `eddsa-rdfc-2022` and
//! `eddsa-jcs-2022` out of the box; `bbs-2023` and post-quantum variants
//! follow whatever feature flags the upstream crate exposes (see its
//! changelog).
//!
//! ```rust,ignore
//! use trust_tasks_proof::affinidi::Verifier;
//!
//! // For did:key — purely local, no I/O.
//! let verifier = Verifier::for_did_key();
//! verifier.verify(&inbound_doc).await?;
//! ```
//!
//! For `did:web` or other resolvers, supply a
//! [`affinidi_data_integrity::VerificationMethodResolver`] via
//! [`Verifier::with_resolver`].
//!
//! The implementation removes the `proof` member from the document before
//! handing it to the Affinidi `verify` call, as required by the W3C
//! Data Integrity canonicalisation rules — the proof is over the doc
//! *plus* the proof's own configuration (everything except `proofValue`),
//! not over the embedded proof object itself.

mod resolver;
pub use resolver::CachedDidResolver;

use std::sync::Arc;

use affinidi_data_integrity::{
    DataIntegrityError, DataIntegrityProof, DidKeyResolver, SignatureFailure,
    VerificationMethodResolver, VerifyOptions,
};
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use trust_tasks_rs::{ProofVerifier, TrustTask, VerificationError};

/// Re-export the upstream resolver trait so callers can implement custom
/// `did:web` / `did:webvh` resolvers without adding a direct dep on the
/// upstream crate.
pub use affinidi_data_integrity::DidKeyResolver as AffinidiDidKeyResolver;

/// [`ProofVerifier`] implementation backed by the Affinidi Data Integrity
/// crate.
///
/// Construct with [`Self::for_did_key`] for `did:key`-only verification
/// (no I/O, suitable for tests and self-issued documents), or with
/// [`Self::with_resolver`] when you need to resolve `did:web` /
/// `did:webvh` / other DID methods.
pub struct Verifier {
    resolver: Arc<dyn VerificationMethodResolver>,
    options: VerifyOptions,
}

impl Verifier {
    /// Verifier that resolves `did:key:` URIs locally; rejects every other
    /// DID method.
    pub fn for_did_key() -> Self {
        Self::with_resolver(Arc::new(DidKeyResolver))
    }

    /// Verifier with a caller-supplied resolver. Use this with the
    /// Affinidi DID-resolver cache SDK, an in-process `did:web` lookup,
    /// a custom HSM bridge, etc.
    pub fn with_resolver(resolver: Arc<dyn VerificationMethodResolver>) -> Self {
        Self {
            resolver,
            options: VerifyOptions::default(),
        }
    }

    /// Override the [`VerifyOptions`] (expected proof purpose, expected
    /// domain/challenge, etc.). Defaults are equivalent to
    /// `VerifyOptions::default()`.
    pub fn with_options(mut self, options: VerifyOptions) -> Self {
        self.options = options;
        self
    }
}

#[async_trait]
impl ProofVerifier for Verifier {
    async fn verify<P>(&self, doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: Serialize + Send + Sync,
    {
        // ─── 1. Extract the proof.
        let Some(proof) = &doc.proof else {
            return Err(VerificationError::MalformedProof(
                "document carries no proof member".to_string(),
            ));
        };

        // ─── 2. Round-trip our typed Proof into the Affinidi
        //        DataIntegrityProof. The two structs are members-equivalent
        //        but use slightly different field names (proof_type vs type_,
        //        camelCase vs snake_case via serde).
        let proof_value = serde_json::to_value(proof)
            .map_err(|e| VerificationError::MalformedProof(format!("serialise proof: {e}")))?;
        let parsed_proof: DataIntegrityProof = serde_json::from_value(proof_value)
            .map_err(|e| VerificationError::MalformedProof(format!("parse proof: {e}")))?;

        // ─── 3. Serialise the document minus the proof member. We can't
        //        avoid the JSON round-trip because TrustTask is generic
        //        over P; a manual "skip this field" path would force
        //        re-deriving the serializer.
        let mut doc_value = serde_json::to_value(doc).map_err(|e| {
            VerificationError::Other(format!("serialise TrustTask for verification: {e}"))
        })?;
        if let Some(obj) = doc_value.as_object_mut() {
            obj.remove("proof");
        }

        // ─── 4. Hand to Affinidi.
        parsed_proof
            .verify(&doc_value, &*self.resolver, self.options.clone())
            .await
            .map_err(map_error)?;
        Ok(())
    }
}

/// Map [`DataIntegrityError`] variants into the framework's
/// [`VerificationError`] taxonomy. The mapping aligns with SPEC.md §8.3:
/// every failure surfaces as `proof_invalid` to the wire, distinguished
/// from `proof_required` (which our caller raises) — `VerificationError`
/// is what the framework returns when a proof IS present but fails to
/// verify.
fn map_error(err: DataIntegrityError) -> VerificationError {
    match err {
        DataIntegrityError::UnsupportedCryptoSuite { name } => {
            VerificationError::UnsupportedCryptosuite(name)
        }
        DataIntegrityError::KeyTypeMismatch {
            expected,
            actual,
            suite,
        } => VerificationError::IssuerMismatch(format!(
            "key type {actual:?} does not match cryptosuite {suite:?} (expected {expected:?})"
        )),
        DataIntegrityError::InvalidSignature { reason, .. } => match reason {
            SignatureFailure::Malformed | SignatureFailure::Invalid => {
                VerificationError::SignatureInvalid
            }
            _ => VerificationError::SignatureInvalid,
        },
        DataIntegrityError::InvalidPublicKey { reason, .. } => {
            VerificationError::MalformedProof(format!("public key: {reason}"))
        }
        DataIntegrityError::Canonicalization(reason) => {
            VerificationError::Other(format!("canonicalisation: {reason}"))
        }
        DataIntegrityError::MalformedProof(reason) => VerificationError::MalformedProof(reason),
        other => VerificationError::Other(other.to_string()),
    }
}

/// Convenience: parse the framework `Proof` JSON-equivalent into an
/// [`affinidi_data_integrity::DataIntegrityProof`]. Exposed for callers
/// who want to verify by passing the doc body and proof separately
/// (e.g. when the proof was carried out-of-band).
pub fn parse_data_integrity_proof(value: &Value) -> Result<DataIntegrityProof, VerificationError> {
    serde_json::from_value(value.clone())
        .map_err(|e| VerificationError::MalformedProof(format!("parse DataIntegrityProof: {e}")))
}
