//! Size bounds on the encoded values of a proof, checked before
//! `affinidi-data-integrity` decodes them.
//!
//! The upstream crate decodes `proofValue` and a `did:key` id with
//! `multibase::decode`, which accepts every multibase base. Its base58 and
//! base36 decoders cost the square of the input length, and both values arrive
//! before any signature is checked, so a long value is refused here first.

use affinidi_data_integrity::DataIntegrityProof;
use trust_tasks_rs::VerificationError;

/// The largest values a cryptosuite reads, in decoded bytes.
struct SuiteBounds {
    /// The base `proofValue` is written in: `'z'` (base58btc) or `'u'`
    /// (base64url, no padding).
    proof_base: char,
    /// The most bytes `proofValue` may decode to.
    proof_bytes: usize,
    /// The most bytes a `did:key` multikey (multicodec header included) may
    /// decode to.
    key_bytes: usize,
}

/// The Ed25519 and NIST suites. The largest signature is P-384's 96 bytes, the
/// largest multikey a 2-byte header and a 49-byte compressed P-384 point. 128
/// leaves headroom, and is the bound the Go, TypeScript and Dart verifiers use.
const CLASSIC: SuiteBounds = SuiteBounds {
    proof_base: 'z',
    proof_bytes: 128,
    key_bytes: 128,
};

/// The largest ML-DSA public key: ML-DSA-87's 2592 bytes and a 2-byte
/// multicodec header. Shared by the post-quantum suites, whatever their
/// parameter set.
const PQ_KEY_BYTES: usize = 2594;

/// `mldsa44-*-2024`, with room for every ML-DSA parameter set: ML-DSA-87
/// signatures are 4627 bytes.
const ML_DSA: SuiteBounds = SuiteBounds {
    proof_base: 'z',
    proof_bytes: 4627,
    key_bytes: PQ_KEY_BYTES,
};

/// `slhdsa128-*-2024`: SLH-DSA-SHA2-128s signatures are 7856 bytes.
const SLH_DSA: SuiteBounds = SuiteBounds {
    proof_base: 'z',
    proof_bytes: 7856,
    key_bytes: PQ_KEY_BYTES,
};

/// `bbs-2023`: a base64url CBOR proof, which grows with the number of
/// statements, decoded in linear time. The bound only caps memory.
const BBS: SuiteBounds = SuiteBounds {
    proof_base: 'u',
    proof_bytes: 48 * 1024,
    key_bytes: 128,
};

fn bounds_for(cryptosuite: &str) -> &'static SuiteBounds {
    if cryptosuite.starts_with("mldsa") {
        &ML_DSA
    } else if cryptosuite.starts_with("slhdsa") {
        &SLH_DSA
    } else if cryptosuite.starts_with("bbs") {
        &BBS
    } else {
        &CLASSIC
    }
}

/// The longest base58 string that can encode `bytes` bytes:
/// `ceil(bytes * log(256) / log(58))` — 175 characters for 128 bytes.
fn max_base58_chars(bytes: usize) -> usize {
    // bytes * 8 / log2(58), rounded up; exact enough for the sizes above.
    (bytes as f64 * 256f64.ln() / 58f64.ln()).ceil() as usize
}

/// The longest unpadded base64url string encoding `bytes` bytes.
fn max_base64url_chars(bytes: usize) -> usize {
    (bytes * 4).div_ceil(3)
}

/// Refuse a `proofValue`, or a `did:key` verificationMethod's multikey, that
/// is in a base its cryptosuite does not use or is larger than any value that
/// cryptosuite reads.
///
/// Call this before [`DataIntegrityProof::verify`]:
/// [`Verifier::verify_raw`](super::Verifier::verify_raw) does, and a caller
/// verifying through [`PurposeBound`](super::PurposeBound) directly should
/// too. Every length is checked before anything is decoded. The bounds are
/// per cryptosuite: 128 bytes for the Ed25519 and NIST suites (the bound the
/// Go, TypeScript and Dart verifiers apply), and the largest signature and
/// public key for the post-quantum ones.
///
/// Returns [`VerificationError::MalformedProof`] naming the member.
pub fn check_proof_bounds(proof: &DataIntegrityProof) -> Result<(), VerificationError> {
    let suite = bounds_for(&proof.cryptosuite.to_string());
    if let Some(value) = &proof.proof_value {
        check_value(value, suite.proof_base, suite.proof_bytes)
            .map_err(|e| VerificationError::MalformedProof(format!("proofValue: {e}")))?;
    }
    let vm = &proof.verification_method;
    let did = vm.split('#').next().unwrap_or(vm);
    if let Some(multikey) = did.strip_prefix("did:key:") {
        check_value(multikey, 'z', suite.key_bytes)
            .map_err(|e| VerificationError::MalformedProof(format!("verificationMethod: {e}")))?;
    }
    Ok(())
}

fn check_value(value: &str, base: char, max_bytes: usize) -> Result<(), String> {
    let Some(encoded) = value.strip_prefix(base) else {
        let name = if base == 'z' {
            "base58btc"
        } else {
            "base64url"
        };
        return Err(format!(
            "not multibase {name} (expected the '{base}' prefix)"
        ));
    };
    let max_chars = if base == 'z' {
        max_base58_chars(max_bytes)
    } else {
        max_base64url_chars(max_bytes)
    };
    if encoded.len() > max_chars {
        return Err(format!(
            "encoded value is {} characters, longer than the {max_chars} this cryptosuite allows",
            encoded.len()
        ));
    }
    if base == 'z' {
        // Within the length bound a value can still name one byte too many.
        if let Ok(bytes) = affinidi_encoding::decode_base58btc(value) {
            if bytes.len() > max_bytes {
                return Err(format!(
                    "encoded value decodes to more than the {max_bytes} bytes this cryptosuite allows"
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base58_length_bounds() {
        assert_eq!(max_base58_chars(128), 175);
        // The longest encodings of each bound fit it.
        for n in [128, 2594, 4627, 7856] {
            let all_ff = vec![0xffu8; n];
            let enc = affinidi_encoding::encode_base58btc(&all_ff);
            assert!(enc.len() - 1 <= max_base58_chars(n), "{n} bytes");
            check_value(&enc, 'z', n).expect("largest value within the bound");
            let zeros = affinidi_encoding::encode_base58btc(&vec![0u8; n]);
            check_value(&zeros, 'z', n).expect("leading zeros within the bound");
        }
    }

    #[test]
    fn suites_map_to_their_bounds() {
        assert_eq!(bounds_for("eddsa-jcs-2022").proof_bytes, 128);
        assert_eq!(bounds_for("ecdsa-jcs-2019").proof_bytes, 128);
        assert_eq!(bounds_for("mldsa44-jcs-2024").proof_bytes, 4627);
        assert_eq!(bounds_for("slhdsa128-jcs-2024").proof_bytes, 7856);
        assert_eq!(bounds_for("bbs-2023").proof_base, 'u');
    }
}
