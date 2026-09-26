//! [`CachedDidResolver`] — bridges the Affinidi DID resolver cache SDK to
//! [`ProofPurposeResolver`], so [`Verifier`](crate::affinidi::Verifier) can verify proofs
//! against keys published in real DID documents (`did:web`, `did:webvh`,
//! `did:peer`, `did:key`, `did:jwk`, …) rather than only against
//! locally-derivable `did:key:`s.
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
//! use trust_tasks_proof::affinidi::{Verifier, CachedDidResolver};
//!
//! let client = DIDCacheClient::new(DIDCacheConfigBuilder::default().build()).await?;
//! let resolver = Arc::new(CachedDidResolver::new(Arc::new(client)));
//! let verifier = Verifier::with_resolver(resolver);
//! ```
//!
//! ## What's resolved
//!
//! The adapter handles every DID method the resolver cache supports in
//! its local mode (`did:key`, `did:peer`, `did:jwk`, `did:web`, …; see
//! [`affinidi_did_resolver_cache_sdk::DIDMethod`] for the canonical
//! list). For each resolved DID document it:
//!
//! 1. requires the document's `id` to be the DID the verification method
//!    names, and the method's `controller` to be that same DID;
//! 2. requires the method to be listed — by absolute DID URL, by a
//!    `#fragment` relative to the document id, or embedded — under the
//!    verification relationship the proof's `proofPurpose` names
//!    ([`ProofPurposeResolver`]). A key listed only under `keyAgreement`,
//!    under another relationship, or only under `verificationMethod` is
//!    refused;
//! 3. decodes its public key from `publicKeyMultibase` **or**
//!    `publicKeyJwk`, to determine both the key type and raw key bytes.
//!
//! `CachedDidResolver` deliberately does **not** implement the upstream
//! [`VerificationMethodResolver`] trait, which carries no purpose and so
//! could only accept a key listed under *any* relationship. A caller
//! verifying through `DataIntegrityProof::verify` passes
//! [`PurposeBound::new(&resolver, purpose)`](super::PurposeBound), which
//! implements that trait for one proof's purpose.
//!
//! ## What isn't (yet)
//!
//! * Anything beyond Ed25519 / X25519 / P-256 / P-384 / secp256k1 —
//!   post-quantum multicodecs (ML-DSA, SLH-DSA) would slot in here
//!   under the same feature flags `affinidi-crypto` exposes.

use std::sync::Arc;

use affinidi_crypto::KeyType;
#[allow(unused_imports)] // doc links
use affinidi_data_integrity::VerificationMethodResolver;
use affinidi_data_integrity::{DataIntegrityError, ResolvedKey};
use affinidi_did_common::verification_method::{VerificationMethod, VerificationRelationship};
use affinidi_did_common::Document;
use affinidi_did_resolver_cache_sdk::DIDCacheClient;
use affinidi_encoding::{ED25519_PUB, P256_PUB, P384_PUB, SECP256K1_PUB, X25519_PUB};
use async_trait::async_trait;

use super::purpose::{split_vm, ProofPurpose, ProofPurposeResolver};

/// A [`ProofPurposeResolver`] that delegates DID resolution to the Affinidi DID resolver cache, then walks
/// the resulting DID document to pull out the named verification method's
/// key material.
#[derive(Clone)]
pub struct CachedDidResolver {
    client: Arc<DIDCacheClient>,
}

impl CachedDidResolver {
    /// Wrap a configured [`DIDCacheClient`].
    pub fn new(client: Arc<DIDCacheClient>) -> Self {
        Self { client }
    }

    /// Borrow the underlying cache client (useful for sharing the same
    /// resolver between, say, the proof verifier and a DIDComm secrets
    /// resolver).
    pub fn client(&self) -> &Arc<DIDCacheClient> {
        &self.client
    }

    /// Resolve `vm` if the DID naming it lists it under `purpose`.
    async fn resolve_for(
        &self,
        vm: &str,
        purpose: ProofPurpose,
    ) -> Result<ResolvedKey, DataIntegrityError> {
        let (did, _) = split_vm(vm)?;
        let resolve = self.client.resolve(did).await.map_err(|e| {
            DataIntegrityError::Resolver(format!("resolve the verificationMethod's DID: {e}"))
        })?;
        let method = authorised_method(&resolve.doc, did, vm, purpose)?;

        // Multikey `publicKeyMultibase` or `publicKeyJwk`.
        let (codec, public_key_bytes) = method.decode_public_key().map_err(|e| {
            DataIntegrityError::Resolver(format!("decode the verificationMethod's key: {e}"))
        })?;
        let key_type = codec_to_key_type(codec).ok_or_else(|| {
            DataIntegrityError::Resolver(format!(
                "unsupported multicodec 0x{codec:x} on the verificationMethod"
            ))
        })?;
        Ok(ResolvedKey::new(key_type, public_key_bytes))
    }
}

#[async_trait]
impl ProofPurposeResolver for CachedDidResolver {
    async fn resolve_vm_for_purpose(
        &self,
        vm: &str,
        purpose: ProofPurpose,
    ) -> Result<ResolvedKey, DataIntegrityError> {
        self.resolve_for(vm, purpose).await
    }
}

/// The relationship list a purpose names.
fn relationship(doc: &Document, purpose: ProofPurpose) -> &[VerificationRelationship] {
    match purpose {
        ProofPurpose::AssertionMethod => &doc.assertion_method,
        ProofPurpose::Authentication => &doc.authentication,
        ProofPurpose::CapabilityInvocation => &doc.capability_invocation,
        ProofPurpose::CapabilityDelegation => &doc.capability_delegation,
    }
}

/// The verification method `vm` names in `doc`, provided `doc` is `did`'s
/// document, the relationship `purpose` names lists the method, and its
/// controller is `did`. Errors name the rule that failed, never the document or its keys.
fn authorised_method(
    doc: &Document,
    did: &str,
    vm: &str,
    purpose: ProofPurpose,
) -> Result<VerificationMethod, DataIntegrityError> {
    if doc.id.as_str() != did {
        return Err(DataIntegrityError::Resolver(
            "the resolved DID document's id is not the verificationMethod's DID".to_string(),
        ));
    }
    // A reference is an absolute DID URL or a fragment relative to `did`.
    let names_vm = |id: &str| match id.strip_prefix('#') {
        Some(fragment) => {
            vm.strip_prefix(did).and_then(|rest| rest.strip_prefix('#')) == Some(fragment)
        }
        None => id == vm,
    };
    let embedded = |r: &VerificationRelationship| match r {
        VerificationRelationship::VerificationMethod(m) if names_vm(m.id.as_str()) => {
            Some((**m).clone())
        }
        _ => None,
    };

    // `Some(None)`: listed by reference; `Some(Some(m))`: embedded.
    let listed = relationship(doc, purpose).iter().find_map(|r| match r {
        VerificationRelationship::Reference(id) if names_vm(id) => Some(None),
        other => embedded(other).map(Some),
    });
    let method = match listed {
        None => {
            return Err(DataIntegrityError::Resolver(format!(
                "verificationMethod is not listed under {purpose} in its DID document"
            )));
        }
        Some(Some(method)) => method,
        // A reference names a method defined in the document's
        // `verificationMethod` set. It is never resolved to a method embedded
        // under another relationship: that method is authorised for that
        // relationship only, and borrowing it here would let the purpose's
        // list vouch for a key defined for a different purpose.
        Some(None) => doc
            .verification_method
            .iter()
            .find(|m| names_vm(m.id.as_str()))
            .cloned()
            .ok_or_else(|| {
                DataIntegrityError::Resolver(
                    "verificationMethod is referenced but not defined under verificationMethod \
                     in its DID document"
                        .to_string(),
                )
            })?,
    };
    if method.controller.as_str() != did {
        return Err(DataIntegrityError::Resolver(
            "verificationMethod's controller is not the DID that names it".to_string(),
        ));
    }
    Ok(method)
}

/// Map an `affinidi-encoding` public-key multicodec value to the
/// corresponding [`KeyType`]. Returns `None` for unrecognised codecs so
/// the caller can surface a typed `Resolver` error.
fn codec_to_key_type(codec: u64) -> Option<KeyType> {
    match codec {
        c if c == ED25519_PUB => Some(KeyType::Ed25519),
        c if c == X25519_PUB => Some(KeyType::X25519),
        c if c == P256_PUB => Some(KeyType::P256),
        c if c == P384_PUB => Some(KeyType::P384),
        c if c == SECP256K1_PUB => Some(KeyType::Secp256k1),
        _ => None,
    }
}
