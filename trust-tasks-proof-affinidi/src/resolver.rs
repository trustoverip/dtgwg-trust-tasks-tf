//! [`CachedDidResolver`] — bridges the Affinidi DID resolver cache SDK to
//! the [`VerificationMethodResolver`] trait `affinidi-data-integrity`
//! expects, so the [`AffinidiProofVerifier`](crate::AffinidiProofVerifier)
//! can verify proofs against keys published in real DID documents
//! (`did:web`, `did:webvh`, `did:peer`, `did:key`, `did:jwk`, …) rather
//! than only against locally-derivable `did:key:`s.
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
//! use trust_tasks_proof_affinidi::{AffinidiProofVerifier, CachedDidResolver};
//!
//! let client = DIDCacheClient::new(DIDCacheConfigBuilder::default().build()).await?;
//! let resolver = Arc::new(CachedDidResolver::new(Arc::new(client)));
//! let verifier = AffinidiProofVerifier::with_resolver(resolver);
//! ```
//!
//! ## What's resolved
//!
//! The adapter handles every DID method the resolver cache supports in
//! its local mode (`did:key`, `did:peer`, `did:jwk`, `did:web`, …; see
//! [`affinidi_did_resolver_cache_sdk::DIDMethod`] for the canonical
//! list). For each resolved DID document it scans
//! [`verification_method`](affinidi_did_common::Document) for an entry
//! whose `id` matches the inbound verificationMethod URI exactly, then
//! decodes the `publicKeyMultibase` to determine both the key type and
//! raw key bytes.
//!
//! ## What isn't (yet)
//!
//! * Verification methods that publish their key via `publicKeyJwk`
//!   rather than `publicKeyMultibase` — `affinidi-did-common`'s
//!   `VerificationMethod::get_public_key_bytes` currently supports only
//!   the `Multikey` `type_`. The adapter surfaces a clean
//!   [`DataIntegrityError::Resolver`] error for those cases so the
//!   caller can fall back to a custom resolver.
//! * Anything beyond Ed25519 / X25519 / P-256 / P-384 / secp256k1 —
//!   post-quantum multicodecs (ML-DSA, SLH-DSA) would slot in here
//!   under the same feature flags `affinidi-crypto` exposes.

use std::sync::Arc;

use affinidi_crypto::KeyType;
use affinidi_data_integrity::{DataIntegrityError, ResolvedKey, VerificationMethodResolver};
use affinidi_did_resolver_cache_sdk::DIDCacheClient;
use affinidi_encoding::{
    decode_multikey_with_codec, ED25519_PUB, P256_PUB, P384_PUB, SECP256K1_PUB, X25519_PUB,
};
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
        let matching = doc
            .verification_method
            .iter()
            .find(|m| m.id.as_str() == vm)
            .ok_or_else(|| {
                DataIntegrityError::Resolver(format!(
                    "verificationMethod {vm} not present in DID document for {did}"
                ))
            })?;

        // Today we only handle Multikey verificationMethods. JWK-bearing
        // ones surface a typed error; callers wanting JWK support stack
        // a custom resolver in front of this one.
        let multibase = matching
            .property_set
            .get("publicKeyMultibase")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                DataIntegrityError::Resolver(format!(
                    "verificationMethod {vm} has no publicKeyMultibase (type {:?}); \
                     JWK-bearing methods need a custom resolver",
                    matching.type_
                ))
            })?;

        let (codec, public_key_bytes) = decode_multikey_with_codec(multibase)
            .map_err(|e| DataIntegrityError::Resolver(format!("decode multikey: {e}")))?;

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
