//! [`ProofExt`] — `.sign()` and `.verify()` as methods on a typed
//! `TrustTask<P>`. Private module; the trait is re-exported at the crate
//! root and carries the documentation.

use async_trait::async_trait;
use serde::Serialize;
use trust_tasks_rs::{ProofVerifier, TrustTask, VerificationError};

#[cfg(feature = "affinidi")]
use crate::affinidi::{sign_trust_task, AffinidiSigner, SignError, SignOptions};
#[cfg(feature = "affinidi")]
use trust_tasks_rs::Proof;

/// Extension trait adding [`sign`](Self::sign) and
/// [`verify`](Self::verify) to the framework's [`TrustTask<P>`], for
/// every payload type `P` a producer can serialise.
///
/// # Why this trait exists
///
/// [`sign_trust_task`](crate::affinidi::sign_trust_task) operates on a
/// [`serde_json::Value`], because a W3C Data Integrity proof is computed
/// over the *document's JSON form* and the framework's document type is
/// generic over its payload. That is the right shape for the primitive
/// and the wrong shape for a producer, who holds a `TrustTask<P>` and
/// wants a signed `TrustTask<P>` back. Without this trait, signing is a
/// five-step ritual — serialise, call, check, deserialise, reassign —
/// that every producer writes out by hand and can get subtly wrong (most
/// often by mutating the document *after* signing it).
///
/// `ProofExt` is a thin typed wrapper over the free functions, not a
/// replacement for them. It reuses
/// [`sign_trust_task`](crate::affinidi::sign_trust_task) verbatim, so the
/// canonicalisation contract, the deterministic `eddsa-jcs-2022` default,
/// the replace-don't-nest rule for an existing proof, and the SPEC.md
/// §4.7/§4.8 issuer↔`verificationMethod` pre-flight are all exactly what
/// that function already implements. A document signed through this trait
/// and one signed through the free function are byte-identical.
///
/// ```rust,ignore
/// use trust_tasks_proof::{affinidi::{SignOptions, Verifier}, ProofExt};
/// use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, TrustTask};
///
/// let mut req = TrustTask::for_payload(new_id(), grant::Payload { /* … */ });
/// req.issuer = Some(my_did.clone());       // set every member first …
/// req.recipient = Some(server_did.clone());
/// req.sign(&secret, SignOptions::new()).await?;   // … then sign.
///
/// // Consumer side, same trait:
/// req.verify(&Verifier::for_did_key()).await?;
/// ```
///
/// # ⚠ Sign last
///
/// The proof covers the document as it stands at the moment
/// [`sign`](Self::sign) is called. Mutating any member afterwards
/// invalidates the signature, and nothing in the type system stops you —
/// `sign` takes `&mut self` precisely so the call reads as the final step
/// of composing the document. Re-signing after a change is always safe:
/// the existing proof is discarded and a fresh one minted over the
/// current content.
#[async_trait]
pub trait ProofExt {
    /// Sign this document in place, attaching the resulting Data
    /// Integrity proof to its `proof` member.
    ///
    /// Equivalent to serialising the document, calling
    /// [`sign_trust_task`](crate::affinidi::sign_trust_task), and
    /// deserialising the result — the round-trip through
    /// [`serde_json::Value`] happens inside, over exactly the same
    /// unsigned bytes, so the emitted proof is identical to the free
    /// function's.
    ///
    /// Every rule the free function applies applies here:
    ///
    /// * The signature is computed over the document **minus** its
    ///   `proof` member. An existing proof is replaced, never nested and
    ///   never signed over.
    /// * The document MUST already carry an in-band `issuer` equal to
    ///   the DID of the signer's `verificationMethod` (the part before
    ///   `#`). Otherwise the call fails with
    ///   [`SignError::MissingIssuer`] / [`SignError::IssuerMismatch`]
    ///   *before* a signature is produced, rather than emitting a
    ///   document no conforming verifier could accept.
    /// * `proofPurpose` defaults to `assertionMethod` and the
    ///   cryptosuite to `eddsa-jcs-2022` unless
    ///   [`SignOptions`](crate::affinidi::SignOptions) says otherwise.
    ///
    /// On error, `self` is left untouched — a failed sign never leaves a
    /// half-signed document behind.
    ///
    /// Available with the `affinidi` feature (on by default).
    #[cfg(feature = "affinidi")]
    async fn sign(
        &mut self,
        signer: &dyn AffinidiSigner,
        options: SignOptions,
    ) -> Result<(), SignError>;

    /// Verify this document's `proof` with `verifier`.
    ///
    /// The mirror of [`sign`](Self::sign), and the argument-order flip of
    /// [`ProofVerifier::verify`] — the same check, spelled from the
    /// document's point of view so a consumer holding a `TrustTask<P>`
    /// does not have to reach for the verifier first.
    ///
    /// Returns [`VerificationError`] on every failure mode, including a
    /// document that carries no `proof` at all (the framework's
    /// `proofRequired` check is a separate, earlier concern — see
    /// [`TrustTask::enforce_spec_policy`]).
    async fn verify<V>(&self, verifier: &V) -> Result<(), VerificationError>
    where
        V: ProofVerifier + ?Sized;
}

#[async_trait]
impl<P> ProofExt for TrustTask<P>
where
    P: Serialize + Send + Sync,
{
    #[cfg(feature = "affinidi")]
    async fn sign(
        &mut self,
        signer: &dyn AffinidiSigner,
        options: SignOptions,
    ) -> Result<(), SignError> {
        // The document as the verifier will see it. `sign_trust_task`
        // strips any existing `proof` itself, so nothing is done to
        // `self` before the signature exists — which is what makes the
        // failure paths below non-destructive.
        let unsigned = serde_json::to_value(&*self)?;
        let signed = sign_trust_task(&unsigned, signer, options).await?;

        // Lift only the `proof` member back onto the typed document.
        // Every other member came from `self` unchanged, so re-parsing
        // the whole document into `TrustTask<P>` would add a lossy
        // deserialise step for no gain — and a payload type whose serde
        // round-trip is not exact would silently invalidate the proof it
        // was just given.
        let proof = signed
            .get("proof")
            .cloned()
            .ok_or_else(|| SignError::ProofRoundTrip("signed document carries no proof".into()))?;
        let proof: Proof =
            serde_json::from_value(proof).map_err(|e| SignError::ProofRoundTrip(e.to_string()))?;

        self.proof = Some(proof);
        Ok(())
    }

    async fn verify<V>(&self, verifier: &V) -> Result<(), VerificationError>
    where
        V: ProofVerifier + ?Sized,
    {
        verifier.verify(self).await
    }
}
