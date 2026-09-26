//! [`Verifier`] — a [`ProofVerifier`] implementation backed by
//! `affinidi-data-integrity` — and [`sign_trust_task`], its sign-side
//! counterpart for producers.
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
//! For `did:web` or other resolvers, supply a [`ProofPurposeResolver`] —
//! normally [`CachedDidResolver`] — via [`Verifier::with_resolver`]. The
//! verifier accepts a proof only when the issuer's DID document lists the
//! proof's `verificationMethod` under the relationship its `proofPurpose`
//! names ([`ProofPurpose`]).
//!
//! The implementation removes the `proof` member from the document before
//! handing it to the Affinidi `verify` call, as required by the W3C
//! Data Integrity canonicalisation rules — the proof is over the doc
//! *plus* the proof's own configuration (everything except `proofValue`),
//! not over the embedded proof object itself. [`sign_trust_task`] applies
//! the identical document-minus-`proof` contract on the sign side, so
//! what it emits is what [`Verifier`] verifies.

mod purpose;
mod resolver;
mod sign;
pub use purpose::{ProofPurpose, ProofPurposeResolver, PurposeBound};
pub use resolver::CachedDidResolver;
pub use sign::{sign_trust_task, SignError};

use std::sync::Arc;

use affinidi_data_integrity::{
    DataIntegrityError, DataIntegrityProof, DidKeyResolver, SignatureFailure, VerifyOptions,
};
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use trust_tasks_rs::{ProofVerifier, TrustTask, VerificationError};

/// Re-export the upstream resolver trait so callers can implement custom
/// `did:web` / `did:webvh` resolvers without adding a direct dep on the
/// upstream crate.
pub use affinidi_data_integrity::DidKeyResolver as AffinidiDidKeyResolver;

/// Re-export the upstream signer trait so callers can drive
/// [`sign_trust_task`] from a KMS/HSM-backed signer without adding a
/// direct dep on the upstream crate.
pub use affinidi_data_integrity::signer::Signer as AffinidiSigner;

/// Re-export the upstream sign options + cryptosuite enum so callers can
/// build [`sign_trust_task`] options without a direct upstream dep.
pub use affinidi_data_integrity::{crypto_suites::CryptoSuite, SignOptions};

/// [`ProofVerifier`] implementation backed by the Affinidi Data Integrity
/// crate.
///
/// Construct with [`Self::for_did_key`] for `did:key`-only verification
/// (no I/O, suitable for tests and self-issued documents), or with
/// [`Self::with_resolver`] when you need to resolve `did:web` /
/// `did:webvh` / other DID methods.
pub struct Verifier {
    resolver: Arc<dyn ProofPurposeResolver>,
    options: VerifyOptions,
}

impl Verifier {
    /// Verifier that resolves `did:key:` URIs locally; rejects every other
    /// DID method. A `did:key`'s signing key is authorised for every
    /// signing purpose, as the `did:key` method defines.
    pub fn for_did_key() -> Self {
        Self::with_resolver(Arc::new(DidKeyResolver))
    }

    /// Verifier with a caller-supplied resolver: normally
    /// [`CachedDidResolver`], or any [`ProofPurposeResolver`] that checks a
    /// method against the relationship its proof's `proofPurpose` names.
    ///
    /// Earlier releases took an upstream `VerificationMethodResolver`, which
    /// cannot see the proof's purpose; a key-only resolver now implements
    /// [`ProofPurposeResolver`] and states how it applies the relationship.
    pub fn with_resolver(resolver: Arc<dyn ProofPurposeResolver>) -> Self {
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
        // The typed document, re-serialised. Exact only when `P` round-trips
        // byte for byte; a verifier holding the document as it arrived should
        // call [`Verifier::verify_raw`] with that instead.
        let doc_value = serde_json::to_value(doc).map_err(|e| {
            VerificationError::Other(format!("serialise TrustTask for verification: {e}"))
        })?;
        self.verify_raw(&doc_value).await
    }
}

impl Verifier {
    /// Verify the `proof` of a Trust Task document **as it was received**.
    ///
    /// The signature covers the JCS canonicalisation of the document the
    /// signer serialised. [`ProofVerifier::verify`] rebuilds that document
    /// from a parsed `TrustTask<P>`, which is exact only when every member
    /// survives a serde round-trip unchanged: a timestamp written as
    /// `+00:00` that re-serialises as `Z`, a number re-rendered, an unknown
    /// member dropped by the payload type — any of those makes a genuine
    /// proof fail. A receiver holding the raw JSON should verify that.
    ///
    /// Checks, in order: a `proof` member is present and well-formed; the
    /// document names an `issuer` controlling the proof's verificationMethod
    /// (SPEC §4.8, exact string equality); the `proofPurpose` names a signing
    /// relationship and the issuer lists the verificationMethod under it
    /// (W3C Data Integrity / Controlled Identifiers §3.3); the signature
    /// verifies over the document minus its `proof`.
    pub async fn verify_raw(&self, doc: &Value) -> Result<(), VerificationError> {
        // ─── 1. Extract the proof.
        let proof_value = doc.get("proof").cloned().ok_or_else(|| {
            VerificationError::MalformedProof("document carries no proof member".to_string())
        })?;
        let parsed_proof: DataIntegrityProof = serde_json::from_value(proof_value)
            .map_err(|e| VerificationError::MalformedProof(format!("parse proof: {e}")))?;

        // ─── 2. The document minus its proof, exactly as given.
        let mut doc_value = doc.clone();
        if let Some(obj) = doc_value.as_object_mut() {
            obj.remove("proof");
        }

        // ─── 3. Bind the proof to the in-band issuer (SPEC §4.7 / §4.8 /
        //        §7.2 item 7). A valid signature proves only that *some* key
        //        signed the document; authenticity additionally requires that
        //        key to be controlled by the document's declared `issuer`.
        //        Compare the verificationMethod's DID (the portion before `#`)
        //        to `issuer` by exact string equality — no normalization, per
        //        §4.8.
        let vm = &parsed_proof.verification_method;
        match doc_value.get("issuer").and_then(|v| v.as_str()) {
            None => {
                return Err(VerificationError::IssuerMismatch(
                    "document carries a proof but no in-band issuer to bind it to".to_string(),
                ));
            }
            Some(issuer) => {
                let vm_did = vm.split('#').next().unwrap_or(vm);
                if vm_did != issuer {
                    return Err(VerificationError::IssuerMismatch(format!(
                        "verificationMethod is controlled by {vm_did}, not the document issuer {issuer}"
                    )));
                }
            }
        }

        // ─── 4. The key must be one the issuer authorised for the purpose
        //        the proof declares: `keyAgreement` and unknown purposes are
        //        refused here, and the resolver checks the relationship.
        let purpose = ProofPurpose::parse(&parsed_proof.proof_purpose).map_err(map_error)?;
        let resolver = PurposeBound::new(&*self.resolver, purpose);

        // ─── 5. Hand to Affinidi.
        parsed_proof
            .verify(&doc_value, &resolver, self.options.clone())
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
