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
//! ## Where resolution may go
//!
//! Resolving `did:web`, `did:webvh`, `did:webs` or `did:scid` fetches from a
//! host the DID names — and the DID comes from the document being verified,
//! so whoever sent it chooses where this resolver sends requests. Two limits
//! apply to every resolution:
//!
//! * **A deadline** — [`DEFAULT_RESOLUTION_TIMEOUT`] unless
//!   [`CachedDidResolver::with_timeout`] sets another. A resolution that has
//!   not finished by then fails, however many fetches it involves.
//! * **A host policy** — a DID whose host is `localhost`, a loopback,
//!   private-network, link-local, shared-address or unspecified IP literal is
//!   refused before the client is called.
//!   [`CachedDidResolver::allow_private_hosts`] lifts that (local development,
//!   or DID hosts on a trusted private network), and
//!   [`CachedDidResolver::with_host_filter`] adds an allow-list of your own.
//!   This screens what the DID *names*; a public hostname that resolves to a
//!   private address is the client's to refuse, which the SDK's
//!   `DIDCacheConfigBuilder::with_host_policy` does by default.
//!
//! ## What isn't (yet)
//!
//! * Anything beyond Ed25519 / X25519 / P-256 / P-384 / secp256k1 —
//!   post-quantum multicodecs (ML-DSA, SLH-DSA) would slot in here
//!   under the same feature flags `affinidi-crypto` exposes.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::time::Duration;

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

/// How long one resolution may take, unless
/// [`CachedDidResolver::with_timeout`] says otherwise.
pub const DEFAULT_RESOLUTION_TIMEOUT: Duration = Duration::from_secs(10);

/// A caller-supplied predicate over the host a DID names; see
/// [`CachedDidResolver::with_host_filter`].
type HostFilter = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// A [`ProofPurposeResolver`] that delegates DID resolution to the Affinidi DID resolver cache, then walks
/// the resulting DID document to pull out the named verification method's
/// key material.
///
/// Every resolution has a deadline ([`DEFAULT_RESOLUTION_TIMEOUT`], or
/// [`Self::with_timeout`]) and a host policy: a DID naming `localhost` or a
/// loopback, private-network or link-local address is refused before the
/// client is called ([`Self::allow_private_hosts`] opts out), and
/// [`Self::with_host_filter`] restricts resolution to the hosts it accepts.
#[derive(Clone)]
pub struct CachedDidResolver {
    client: Arc<DIDCacheClient>,
    timeout: Duration,
    allow_private_hosts: bool,
    host_filter: Option<HostFilter>,
}

impl CachedDidResolver {
    /// Wrap a configured [`DIDCacheClient`], with
    /// [`DEFAULT_RESOLUTION_TIMEOUT`] and the default host policy (DIDs
    /// naming a loopback or private host are refused).
    pub fn new(client: Arc<DIDCacheClient>) -> Self {
        Self {
            client,
            timeout: DEFAULT_RESOLUTION_TIMEOUT,
            allow_private_hosts: false,
            host_filter: None,
        }
    }

    /// The deadline for one resolution. A zero duration is raised to one
    /// millisecond rather than taken as "no deadline".
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }

    /// Accept DIDs naming `localhost` or a loopback, private-network or
    /// link-local address. Off by default: those hosts are this machine and
    /// its network, which a sender should not be able to point the verifier
    /// at. Turn it on for local development, or where the DID hosts are
    /// trusted and private. The client's own host policy
    /// (`DIDCacheConfigBuilder::with_host_policy`) applies independently.
    pub fn allow_private_hosts(mut self, allow: bool) -> Self {
        self.allow_private_hosts = allow;
        self
    }

    /// Resolve only DIDs whose host `filter` accepts — an allow-list of the
    /// hosts this deployment expects to meet. The host is passed lowercased,
    /// without port, brackets or path. It applies to DIDs that name a host;
    /// `did:key`, `did:peer` and `did:jwk` resolve locally and are not
    /// screened. Runs after the private-host check.
    pub fn with_host_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.host_filter = Some(Arc::new(filter));
        self
    }

    /// The host policy, applied before any resolution.
    fn check_host(&self, did: &str) -> Result<(), DataIntegrityError> {
        let Some(host) = did_host(did) else {
            return Ok(());
        };
        if !self.allow_private_hosts && is_non_public_host(&host) {
            return Err(DataIntegrityError::Resolver(
                "the verificationMethod's DID names a host this resolver does not contact"
                    .to_string(),
            ));
        }
        if let Some(filter) = &self.host_filter {
            if !filter(&host) {
                return Err(DataIntegrityError::Resolver(
                    "the verificationMethod's DID names a host this resolver does not contact"
                        .to_string(),
                ));
            }
        }
        Ok(())
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
        self.check_host(did)?;
        let resolve = tokio::time::timeout(self.timeout, self.client.resolve(did))
            .await
            .map_err(|_| {
                DataIntegrityError::Resolver(
                    "resolving the verificationMethod's DID did not finish in time".to_string(),
                )
            })?
            .map_err(|e| {
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

/// The host a DID fetches its document from, lowercased and without port,
/// brackets or path: `did:web:<host>…`, `did:webs:<host>…`,
/// `did:webvh:<scid>:<host>…`, or a `src=` locator (`did:scid`). `None` for a
/// DID that names no host.
fn did_host(did: &str) -> Option<String> {
    let (base, query) = match did.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (did, None),
    };
    let from_query = query.and_then(|q| {
        q.split('&')
            .find_map(|kv| kv.strip_prefix("src="))
            .map(|v| v.split('/').next().unwrap_or(v))
    });
    let mut parts = base.split(':');
    let authority = match (parts.next(), parts.next()) {
        (Some("did"), Some("web" | "webs")) => parts.next(),
        (Some("did"), Some("webvh")) => parts.nth(1),
        _ => None,
    };
    let raw = authority.or(from_query)?;
    let decoded = percent_decode(raw).to_ascii_lowercase();
    let host = if let Some(v6) = decoded.strip_prefix('[') {
        v6.split_once(']').map_or(v6, |(h, _)| h).to_string()
    } else {
        decoded.split(':').next().unwrap_or("").to_string()
    };
    (!host.is_empty()).then_some(host)
}

/// `%XX` decoding, enough for the `%3A` (port) and `%5B`/`%5D` (IPv6
/// brackets) a DID uses in its host; an invalid escape is kept as written.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && bytes[i + 1].is_ascii_hexdigit()
            && bytes[i + 2].is_ascii_hexdigit()
        {
            let hex = |c: u8| (c as char).to_digit(16).unwrap_or(0) as u8;
            out.push(hex(bytes[i + 1]) << 4 | hex(bytes[i + 2]));
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `localhost` (and names under it), or an IP literal that is not a public
/// unicast address.
fn is_non_public_host(host: &str) -> bool {
    let host = host.trim_end_matches('.');
    if host == "localhost" || host.ends_with(".localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => is_non_public_v4(ip),
        Ok(IpAddr::V6(ip)) => is_non_public_v6(ip),
        Err(_) => false,
    }
}

fn is_non_public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, ..] = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_documentation()
        || a == 0
        // 100.64.0.0/10, shared address space (carrier-grade NAT).
        || (a == 100 && (b & 0xc0) == 64)
        // 198.18.0.0/15, benchmarking.
        || (a == 198 && (b & 0xfe) == 18)
        || a >= 240
}

fn is_non_public_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_non_public_v4(v4);
    }
    let first = ip.segments()[0];
    ip.is_loopback()
        || ip.is_unspecified()
        // fc00::/7, unique local.
        || (first & 0xfe00) == 0xfc00
        // fe80::/10, link-local.
        || (first & 0xffc0) == 0xfe80
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
    // A did:scid identity is the DID; a `?src=` locator on the reference is
    // only where to fetch its log, and its document lists keys under the bare
    // DID. Every other DID is compared exactly as given.
    let did = &*affinidi_did_common::scid::strip_scid_src(did);
    let vm = &*affinidi_did_common::scid::strip_scid_src(vm);
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

#[cfg(test)]
mod host_tests {
    use super::*;

    #[test]
    fn did_host_reads_each_method() {
        assert_eq!(
            did_host("did:web:example.com").as_deref(),
            Some("example.com")
        );
        assert_eq!(
            did_host("did:web:Example.COM%3A8443:users:alice").as_deref(),
            Some("example.com")
        );
        assert_eq!(
            did_host("did:webvh:QmScid:host.example:a").as_deref(),
            Some("host.example")
        );
        assert_eq!(
            did_host("did:webs:127.0.0.1%3A7676:Eaid").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(
            did_host("did:web:%5B%3A%3A1%5D%3A80").as_deref(),
            Some("::1")
        );
        assert_eq!(
            did_host("did:scid:vh:1:QmScid?src=10.0.0.5%3A8000").as_deref(),
            Some("10.0.0.5")
        );
        assert_eq!(did_host("did:key:z6MkExample"), None);
        assert_eq!(did_host("did:peer:2.Ez6L"), None);
    }

    #[test]
    fn non_public_hosts() {
        for host in [
            "localhost",
            "api.localhost",
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "::1",
            "::",
            "fd00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
        ] {
            assert!(is_non_public_host(host), "{host}");
        }
        for host in [
            "example.com",
            "8.8.8.8",
            "2001:4860::8888",
            "localhost.example",
        ] {
            assert!(!is_non_public_host(host), "{host}");
        }
    }

    #[test]
    fn percent_decode_keeps_invalid_escapes() {
        assert_eq!(percent_decode("a%3Ab"), "a:b");
        assert_eq!(percent_decode("a%3"), "a%3");
        assert_eq!(percent_decode("a%zzb"), "a%zzb");
        assert_eq!(percent_decode("é%"), "é%");
    }
}
