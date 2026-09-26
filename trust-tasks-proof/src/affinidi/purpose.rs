//! [`ProofPurpose`] and [`ProofPurposeResolver`] — resolving a proof's
//! `verificationMethod` only when the issuer authorised it for the purpose
//! the proof declares.
//!
//! W3C Data Integrity verifies a proof against the verification relationship
//! its `proofPurpose` names (Controlled Identifiers v1.0 §3.3, *Retrieve
//! Verification Method*, steps 10 and 11): the method's `controller` must be
//! the controller document's id, and the document must list the method under
//! that relationship. A key the issuer published for key agreement, or
//! authorised only to authenticate, has not been authorised to make
//! assertions. The upstream [`VerificationMethodResolver`] trait carries no
//! purpose, so it cannot apply this rule; [`ProofPurposeResolver`] does.
//!
//! [`Verifier`](super::Verifier) resolves every proof through a
//! [`ProofPurposeResolver`]. A caller driving
//! [`DataIntegrityProof::verify`](affinidi_data_integrity::DataIntegrityProof::verify)
//! directly wraps its resolver in [`PurposeBound`] to get the same check.

use affinidi_crypto::KeyType;
use affinidi_data_integrity::{
    DataIntegrityError, DidKeyResolver, ResolvedKey, VerificationMethodResolver,
};
use async_trait::async_trait;

/// A verification relationship a signature can be made under — the values a
/// proof's `proofPurpose` may take.
///
/// `keyAgreement` is deliberately absent: a key-agreement key never
/// authorises a signature, so [`ProofPurpose::parse`] refuses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ProofPurpose {
    /// `assertionMethod` — the issuer asserts the document's content.
    AssertionMethod,
    /// `authentication` — the signer proves control of the identifier.
    Authentication,
    /// `capabilityInvocation` — the signer invokes a capability.
    CapabilityInvocation,
    /// `capabilityDelegation` — the signer delegates a capability.
    CapabilityDelegation,
}

impl ProofPurpose {
    /// Every purpose a signature can carry.
    pub const ALL: [ProofPurpose; 4] = [
        ProofPurpose::AssertionMethod,
        ProofPurpose::Authentication,
        ProofPurpose::CapabilityInvocation,
        ProofPurpose::CapabilityDelegation,
    ];

    /// The `proofPurpose` value, which is also the DID-document property
    /// naming the relationship.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProofPurpose::AssertionMethod => "assertionMethod",
            ProofPurpose::Authentication => "authentication",
            ProofPurpose::CapabilityInvocation => "capabilityInvocation",
            ProofPurpose::CapabilityDelegation => "capabilityDelegation",
        }
    }

    /// Parse a proof's `proofPurpose`. `keyAgreement`, an empty value, and
    /// any value naming no signing relationship are refused.
    pub fn parse(value: &str) -> Result<Self, DataIntegrityError> {
        match value {
            "assertionMethod" => Ok(ProofPurpose::AssertionMethod),
            "authentication" => Ok(ProofPurpose::Authentication),
            "capabilityInvocation" => Ok(ProofPurpose::CapabilityInvocation),
            "capabilityDelegation" => Ok(ProofPurpose::CapabilityDelegation),
            "keyAgreement" => Err(DataIntegrityError::MalformedProof(
                "proofPurpose keyAgreement never authorises a signature".to_string(),
            )),
            _ => Err(DataIntegrityError::MalformedProof(
                "proofPurpose names no signing verification relationship".to_string(),
            )),
        }
    }
}

impl std::fmt::Display for ProofPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ProofPurpose {
    type Err = DataIntegrityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Resolves a verification method to its public key **only if** the DID
/// that controls it authorised it for `purpose`.
///
/// An implementation MUST refuse a method whose controller is not the DID
/// the method identifier names, and a method not listed under the
/// relationship `purpose` names (by reference, absolute or relative to the
/// document id, or embedded). [`CachedDidResolver`](super::CachedDidResolver)
/// does this against the resolved DID document; the impl for
/// [`DidKeyResolver`] applies `did:key`'s implicit relationships.
#[async_trait]
pub trait ProofPurposeResolver: Send + Sync {
    /// Resolve `vm` (an absolute DID URL) for a proof declaring `purpose`.
    async fn resolve_vm_for_purpose(
        &self,
        vm: &str,
        purpose: ProofPurpose,
    ) -> Result<ResolvedKey, DataIntegrityError>;
}

#[async_trait]
impl<R: ProofPurposeResolver + ?Sized> ProofPurposeResolver for &R {
    async fn resolve_vm_for_purpose(
        &self,
        vm: &str,
        purpose: ProofPurpose,
    ) -> Result<ResolvedKey, DataIntegrityError> {
        (**self).resolve_vm_for_purpose(vm, purpose).await
    }
}

#[async_trait]
impl<R: ProofPurposeResolver + ?Sized> ProofPurposeResolver for std::sync::Arc<R> {
    async fn resolve_vm_for_purpose(
        &self,
        vm: &str,
        purpose: ProofPurpose,
    ) -> Result<ResolvedKey, DataIntegrityError> {
        (**self).resolve_vm_for_purpose(vm, purpose).await
    }
}

/// `did:key` defines its relationships implicitly: the key the identifier
/// encodes is authorised for `authentication`, `assertionMethod`,
/// `capabilityInvocation` and `capabilityDelegation`, and its only
/// verification method is `did:key:<id>#<id>`. An X25519 `did:key` is a
/// key-agreement key and signs for nothing.
#[async_trait]
impl ProofPurposeResolver for DidKeyResolver {
    async fn resolve_vm_for_purpose(
        &self,
        vm: &str,
        _purpose: ProofPurpose,
    ) -> Result<ResolvedKey, DataIntegrityError> {
        let (did, fragment) = split_vm(vm)?;
        let id = did.strip_prefix("did:key:").ok_or_else(|| {
            DataIntegrityError::Resolver("verificationMethod is not a did:key".to_string())
        })?;
        if fragment != id {
            return Err(DataIntegrityError::Resolver(
                "verificationMethod is not the did:key's own key (fragment must repeat the key id)"
                    .to_string(),
            ));
        }
        let key = self.resolve_vm(vm).await?;
        if key.key_type == KeyType::X25519 {
            return Err(DataIntegrityError::Resolver(
                "a did:key X25519 key is authorised for keyAgreement only".to_string(),
            ));
        }
        Ok(key)
    }
}

/// An upstream [`VerificationMethodResolver`] bound to one proof's purpose,
/// for callers verifying through
/// [`DataIntegrityProof::verify`](affinidi_data_integrity::DataIntegrityProof::verify)
/// rather than through [`Verifier`](super::Verifier):
///
/// ```rust,ignore
/// let purpose = ProofPurpose::parse(&proof.proof_purpose)?;
/// proof.verify(&doc, &PurposeBound::new(&resolver, purpose), options).await?;
/// ```
pub struct PurposeBound<R> {
    resolver: R,
    purpose: ProofPurpose,
}

impl<R> PurposeBound<R> {
    /// Bind `resolver` to `purpose`.
    pub fn new(resolver: R, purpose: ProofPurpose) -> Self {
        Self { resolver, purpose }
    }

    /// The purpose every resolution is checked against.
    pub fn purpose(&self) -> ProofPurpose {
        self.purpose
    }
}

#[async_trait]
impl<R: ProofPurposeResolver> VerificationMethodResolver for PurposeBound<R> {
    async fn resolve_vm(&self, vm: &str) -> Result<ResolvedKey, DataIntegrityError> {
        self.resolver.resolve_vm_for_purpose(vm, self.purpose).await
    }
}

/// Split an absolute verification-method DID URL into its DID and fragment.
/// A relative reference, or a DID URL with no fragment, is not a
/// verification method a proof can name.
pub(crate) fn split_vm(vm: &str) -> Result<(&str, &str), DataIntegrityError> {
    match vm.split_once('#') {
        Some((did, fragment)) if did.starts_with("did:") && !fragment.is_empty() => {
            Ok((did, fragment))
        }
        _ => Err(DataIntegrityError::Resolver(
            "verificationMethod must be an absolute DID URL with a fragment".to_string(),
        )),
    }
}
