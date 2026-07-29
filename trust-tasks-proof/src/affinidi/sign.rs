//! [`sign_trust_task`] — the sign-side counterpart to the
//! [`Verifier`](super::Verifier).
//!
//! Produces documents that the stock [`Verifier`](super::Verifier)
//! accepts by construction: the proof is computed over the document with
//! the `proof` member removed (the same canonicalisation contract the
//! verify side applies), and the in-band `issuer` is checked *before*
//! signing to equal the DID of the signer's `verificationMethod` — the
//! §4.7/§4.8 issuer binding the verify side enforces. A document that
//! would fail its own round-trip is rejected at sign time rather than at
//! the consumer.
//!
//! Defaults match the reference ecosystem's signing profile:
//! `proofPurpose: assertionMethod` (the upstream default) and the
//! `eddsa-jcs-2022` cryptosuite (applied here whenever the caller does
//! not pick a suite explicitly, overriding any signer-declared default so
//! the emitted suite is deterministic). Override either via
//! [`SignOptions`].
//!
//! ```rust,ignore
//! use trust_tasks_proof::affinidi::{sign_trust_task, SignOptions};
//!
//! // `doc` is the Trust Task document as serde_json::Value, `issuer`
//! // already set to the DID the secret's verification method belongs to.
//! let signed = sign_trust_task(&doc, &secret, SignOptions::new()).await?;
//! assert!(signed.get("proof").is_some());
//! ```

use affinidi_data_integrity::crypto_suites::CryptoSuite;
use affinidi_data_integrity::signer::Signer;
use affinidi_data_integrity::{DataIntegrityError, DataIntegrityProof, SignOptions};
use serde_json::Value;

/// Errors surfaced by [`sign_trust_task`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SignError {
    /// The supplied document is not a JSON object; a Trust Task document
    /// is always a top-level object (SPEC.md §4.1).
    #[error("Trust Task document must be a JSON object")]
    NotAnObject,

    /// The document carries no in-band `issuer` member. The stock
    /// verifier binds every proof to the in-band issuer (SPEC.md §4.7 /
    /// §4.8), so a proof minted without one could never verify; set
    /// `issuer` before signing.
    #[error("document carries no in-band `issuer` to bind the proof to")]
    MissingIssuer,

    /// The document's `issuer` is not the DID controlling the signer's
    /// `verificationMethod`. Signing would succeed cryptographically but
    /// the emitted document would be rejected by every conforming
    /// verifier as an issuer-spoofing attempt.
    #[error(
        "signer's verificationMethod is controlled by {vm_did}, not the document issuer {issuer}"
    )]
    IssuerMismatch {
        /// DID portion (before `#`) of the signer's `verificationMethod`.
        vm_did: String,
        /// The document's in-band `issuer`.
        issuer: String,
    },

    /// The produced proof failed to serialise back to JSON.
    #[error("serialise proof: {0}")]
    Serialize(#[from] serde_json::Error),

    /// The underlying Data Integrity signing operation failed.
    #[error(transparent)]
    DataIntegrity(#[from] DataIntegrityError),
}

/// Sign a Trust Task document and return it with an embedded `proof`.
///
/// The proof is computed over the document with the `proof` member
/// removed, exactly as the [`Verifier`](super::Verifier) canonicalises on
/// the verify side. **Any existing `proof` member is discarded and
/// replaced** — re-signing an already-signed document is treated as "mint
/// a fresh proof over the current content", never as appending a proof
/// set or signing over the old proof.
///
/// Defaults: `proofPurpose` falls back to `"assertionMethod"` (upstream
/// default) and the cryptosuite to [`CryptoSuite::EddsaJcs2022`] whenever
/// [`SignOptions::cryptosuite`] is unset — deliberately overriding the
/// signer's own declared default so the wire suite does not silently vary
/// with the signer implementation. Pass
/// [`SignOptions::with_cryptosuite`] / [`SignOptions::with_proof_purpose`]
/// to choose different values.
///
/// The document **must** already carry an in-band `issuer` equal to the
/// DID of the signer's `verificationMethod` (the portion before `#`,
/// compared by exact string equality per SPEC.md §4.8). This is the same
/// binding the stock verifier enforces; checking it here means a document
/// that could never verify is refused before a signature is produced.
///
/// `signer` is anything implementing the upstream
/// [`Signer`](affinidi_data_integrity::signer::Signer) trait —
/// an `affinidi_secrets_resolver::secrets::Secret` works directly, as do
/// KMS/HSM-backed remote signers.
pub async fn sign_trust_task(
    doc: &Value,
    signer: &dyn Signer,
    options: SignOptions,
) -> Result<Value, SignError> {
    let Some(obj) = doc.as_object() else {
        return Err(SignError::NotAnObject);
    };

    // ─── 1. Strip any existing proof: the signature is over the document
    //        minus `proof`, and a re-sign replaces rather than nests.
    let mut unsigned = obj.clone();
    unsigned.remove("proof");

    // ─── 2. Pre-flight the issuer binding (SPEC §4.7/§4.8): the verify
    //        side rejects any proof whose verificationMethod DID differs
    //        from the in-band issuer, so refuse to mint one.
    let issuer = unsigned
        .get("issuer")
        .and_then(|v| v.as_str())
        .ok_or(SignError::MissingIssuer)?;
    let vm = signer.verification_method();
    let vm_did = vm.split('#').next().unwrap_or(vm);
    if vm_did != issuer {
        return Err(SignError::IssuerMismatch {
            vm_did: vm_did.to_string(),
            issuer: issuer.to_string(),
        });
    }

    // ─── 3. Apply the ecosystem default suite when the caller picked
    //        none. Done here (not left to the signer's declared default)
    //        so the emitted suite is deterministic across signer
    //        implementations.
    let mut options = options;
    if options.cryptosuite.is_none() {
        options.cryptosuite = Some(CryptoSuite::EddsaJcs2022);
    }

    // ─── 4. Sign the proof-less document and embed the result.
    let unsigned = Value::Object(unsigned);
    let proof = DataIntegrityProof::sign(&unsigned, signer, options).await?;

    let Value::Object(mut signed) = unsigned else {
        unreachable!("constructed as an object above");
    };
    signed.insert("proof".to_string(), serde_json::to_value(&proof)?);
    Ok(Value::Object(signed))
}
