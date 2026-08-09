//! The default [`Digester`](crate::Digester) backend.
//!
//! Canonicalizes with `serde_json_canonicalizer` — the same RFC 8785
//! implementation `affinidi-data-integrity` uses for `eddsa-jcs-2022`. That
//! sameness is the point: a ceremony digest and a Data Integrity proof over one
//! document must agree on what that document *is*, and two conforming
//! canonicalizers that disagree on a corner (number formatting, escaping, key
//! ordering by UTF-16 code unit) would produce evidence that verifies under one
//! and not the other.

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{Digester, Error};

/// Multihash prefix for SHA-256: code `0x12`, digest length `0x20`.
///
/// Written out rather than pulled from a multihash crate — two bytes, and the
/// dependency would carry a table of codes this crate will never use.
const MULTIHASH_SHA2_256: [u8; 2] = [0x12, 0x20];

/// SHA-256 over the JCS canonicalization, salt-suffixed, multibase-encoded.
///
/// `H(JCS(document) ‖ salt)` — the salt is a **suffix**. `H(salt ‖ message)`
/// invites a length-extension against the salt where `H` is a Merkle–Damgård
/// construction, and SHA-256 is one; the suffix ordering does not.
#[derive(Debug, Default, Clone, Copy)]
pub struct JcsSha256Digester;

impl Digester for JcsSha256Digester {
    fn digest(&self, document: &Value, salt: &[u8]) -> Result<String, Error> {
        let canonical = serde_json_canonicalizer::to_string(document)
            .map_err(|e| Error::Canonicalization(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        hasher.update(salt);
        let digest = hasher.finalize();

        let mut multihash = Vec::with_capacity(MULTIHASH_SHA2_256.len() + digest.len());
        multihash.extend_from_slice(&MULTIHASH_SHA2_256);
        multihash.extend_from_slice(&digest);

        // base58btc, for consistency with `did:key` and `did:webvh`, which the
        // registry's digest convention recommends.
        Ok(multibase::encode(multibase::Base::Base58Btc, multihash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn digest_is_multibase_base58btc_multihash() {
        let d = JcsSha256Digester.digest(&json!({"a": 1}), b"salt").unwrap();
        assert!(d.starts_with('z'), "base58btc multibase prefix, got {d}");
        let (_, bytes) = multibase::decode(&d).unwrap();
        assert_eq!(
            &bytes[..2],
            &MULTIHASH_SHA2_256,
            "sha2-256 multihash prefix"
        );
        assert_eq!(bytes.len(), 34, "2-byte prefix + 32-byte digest");
    }

    #[test]
    fn key_order_does_not_change_the_digest() {
        // The whole reason to canonicalize. Two documents differing only in the
        // order serde_json happened to emit keys are the same document.
        let a: Value = serde_json::from_str(r#"{"b":2,"a":1}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"a":1,"b":2}"#).unwrap();
        assert_eq!(
            JcsSha256Digester.digest(&a, b"s").unwrap(),
            JcsSha256Digester.digest(&b, b"s").unwrap()
        );
    }

    #[test]
    fn a_different_salt_gives_a_different_digest() {
        let doc = json!({"decision": "approved"});
        assert_ne!(
            JcsSha256Digester.digest(&doc, b"salt-one").unwrap(),
            JcsSha256Digester.digest(&doc, b"salt-two").unwrap(),
            "an unsalted-equivalent digest would make a one-bit payload a confirmation oracle"
        );
    }

    #[test]
    fn the_proof_member_is_part_of_the_digest() {
        // The digest names the bytes a party received, not a re-derivable
        // signing input. Excluding `proof` would let one issuer produce two
        // equally valid documents sharing a digest.
        let without = json!({"id": "urn:uuid:1", "payload": {}});
        let with = json!({"id": "urn:uuid:1", "payload": {}, "proof": {"proofValue": "z58D"}});
        assert_ne!(
            JcsSha256Digester.digest(&without, b"s").unwrap(),
            JcsSha256Digester.digest(&with, b"s").unwrap()
        );
    }
}
