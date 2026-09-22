//! [`CachedDidResolver`] — bridges the Affinidi DID resolver cache SDK to
//! the [`VerificationMethodResolver`] trait `affinidi-data-integrity`
//! expects, so [`Verifier`](crate::affinidi::Verifier) can verify proofs
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
//! 1. requires the verification method to be one the DID controller
//!    authorised for signing: referenced (by absolute DID URL or relative
//!    `#fragment`) or embedded under `authentication` or `assertionMethod`.
//!    A key listed only under `keyAgreement`, or listed nowhere, is
//!    refused — otherwise a key never meant for signing could sign;
//! 2. decodes its public key from `publicKeyMultibase` **or**
//!    `publicKeyJwk`, to determine both the key type and raw key bytes.
//!
//! ## What isn't (yet)
//!
//! * Anything beyond Ed25519 / X25519 / P-256 / P-384 / secp256k1 —
//!   post-quantum multicodecs (ML-DSA, SLH-DSA) would slot in here
//!   under the same feature flags `affinidi-crypto` exposes.

use std::sync::Arc;

use affinidi_crypto::KeyType;
use affinidi_data_integrity::{DataIntegrityError, ResolvedKey, VerificationMethodResolver};
use affinidi_did_common::verification_method::VerificationRelationship;
use affinidi_did_resolver_cache_sdk::DIDCacheClient;
use affinidi_encoding::{ED25519_PUB, P256_PUB, P384_PUB, SECP256K1_PUB, X25519_PUB};
use async_trait::async_trait;

/// A [`VerificationMethodResolver`] that delegates DID resolution to
/// the Affinidi DID resolver cache, then walks the resulting DID
/// document to pull out the named verification method's key material.
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
}

#[async_trait]
impl VerificationMethodResolver for CachedDidResolver {
    async fn resolve_vm(&self, vm: &str) -> Result<ResolvedKey, DataIntegrityError> {
        let did = vm.split('#').next().unwrap_or(vm);
        let resolve = self
            .client
            .resolve(did)
            .await
            .map_err(|e| DataIntegrityError::Resolver(format!("resolve {did}: {e}")))?;

        let doc = resolve.doc;

        // The method may be named by absolute DID URL or relative fragment,
        // and may be embedded in a relationship rather than listed under
        // `verificationMethod`.
        let fragment = vm.find('#').map(|i| &vm[i..]);
        let refers = |id: &str| id == vm || fragment.is_some_and(|f| id == f);
        let authorised = |rels: &[VerificationRelationship]| {
            rels.iter().find_map(|r| match r {
                VerificationRelationship::Reference(id) if refers(id) => Some(None),
                VerificationRelationship::VerificationMethod(m) if refers(m.id.as_str()) => {
                    Some(Some((**m).clone()))
                }
                _ => None,
            })
        };
        let embedded = authorised(&doc.authentication)
            .or_else(|| authorised(&doc.assertion_method))
            .ok_or_else(|| {
                DataIntegrityError::Resolver(format!(
                    "verificationMethod {vm} is not an authentication or assertionMethod key of {did}"
                ))
            })?;
        let method = match embedded {
            Some(method) => method,
            None => doc
                .verification_method
                .iter()
                .find(|m| refers(m.id.as_str()))
                .cloned()
                .ok_or_else(|| {
                    DataIntegrityError::Resolver(format!(
                        "verificationMethod {vm} not present in DID document for {did}"
                    ))
                })?,
        };

        // Multikey `publicKeyMultibase` or `publicKeyJwk`.
        let (codec, public_key_bytes) = method
            .decode_public_key()
            .map_err(|e| DataIntegrityError::Resolver(format!("{vm}: {e}")))?;

        let key_type = codec_to_key_type(codec).ok_or_else(|| {
            DataIntegrityError::Resolver(format!("unsupported multicodec 0x{codec:x} on {vm}"))
        })?;

        Ok(ResolvedKey::new(key_type, public_key_bytes))
    }
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
