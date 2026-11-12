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

mod bounds;
mod purpose;
mod resolver;
mod sign;
pub use bounds::check_proof_bounds;
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
    clock: Clock,
}

/// The clock a [`Verifier`] reads `expires` against.
pub type Clock = Arc<dyn Fn() -> chrono::DateTime<chrono::Utc> + Send + Sync>;

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
            clock: Arc::new(chrono::Utc::now),
        }
    }

    /// Override the clock `expires` is checked against (default:
    /// `chrono::Utc::now`). For tests.
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
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

    async fn verify_received(&self, received: &Value) -> Result<(), VerificationError> {
        self.verify_raw(received).await
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
    /// Checks, in order: a `proof` member is present and well-formed, its
    /// `created`, if any, has every field in range, and its `expires`, if
    /// any, has not passed; the document names an `issuer` controlling the proof's verificationMethod
    /// (SPEC §4.8, exact string equality); the `proofPurpose` names a signing
    /// relationship and the issuer lists the verificationMethod under it
    /// (W3C Data Integrity / Controlled Identifiers §3.3); the signature
    /// verifies over the document minus its `proof`.
    pub async fn verify_raw(&self, doc: &Value) -> Result<(), VerificationError> {
        // ─── 1. Extract the proof.
        let proof_value = doc.get("proof").cloned().ok_or_else(|| {
            VerificationError::MalformedProof("document carries no proof member".to_string())
        })?;
        // `expires` is read from the proof as received: the upstream proof
        // type has no member for it and drops it when parsing. A malformed or
        // passed one is refused as such; any other is refused as unsupported,
        // because the signature checked below would not cover it.
        check_expires(&proof_value, (self.clock)())?;
        check_created(&proof_value)?;
        if proof_value.get("expires").is_some() {
            return Err(VerificationError::MalformedProof(
                UNSUPPORTED_EXPIRES.to_string(),
            ));
        }
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

        // ─── 3a. Bound the encoded values before the Affinidi crates decode
        //        them: base58 decoding costs the square of a value's length,
        //        and both arrive before any signature is checked.
        check_proof_bounds(&parsed_proof)?;

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

/// Why [`Verifier::verify_raw`] refuses a proof carrying an unexpired,
/// well-formed `expires`.
///
/// `affinidi-data-integrity` 0.8's proof type has no `expires` member and
/// drops it when parsing, so the signature this crate recomputes would leave
/// out a member the signer covered (or, if the member was added after
/// signing, accept a value nobody signed). Until the upstream type carries
/// it, such a proof is refused here — the Go, TypeScript and Dart verifiers
/// accept it. The shared vector `eddsa-jcs-2022-expires-future.json` records
/// the gap.
pub const UNSUPPORTED_EXPIRES: &str =
    "proof.expires is not supported by this verifier: affinidi-data-integrity's proof type does not carry it, so the signature check could not cover it";

/// Refuse a proof whose `expires` (VC Data Integrity §2.1) is not a
/// `dateTimeStamp` — a date-time ending in `Z` or an explicit `±hh:mm` offset,
/// every field in range — or has passed at `now`. No clock-skew allowance
/// applies: the signer chose the instant. The same rule as the Go, TypeScript
/// and Dart verifiers.
fn check_expires(
    proof: &Value,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), VerificationError> {
    let Some(expires) = proof.get("expires") else {
        return Ok(());
    };
    let at = expires
        .as_str()
        .and_then(parse_date_time_stamp)
        .ok_or_else(|| {
            VerificationError::MalformedProof(format!(
                "proof.expires must be a dateTimeStamp with Z or an offset: {expires}"
            ))
        })?;
    if now > at {
        return Err(VerificationError::MalformedProof(format!(
            "proof.expires {expires} has passed"
        )));
    }
    Ok(())
}

/// Refuse a proof whose `created` is present but is not a `dateTimeStamp`
/// with every field in range — the rule [`check_expires`] applies, and the
/// one the Go, TypeScript and Dart verifiers apply to `created`. chrono, which
/// the upstream proof type parses `created` with, would take a leap second
/// and a lower-case `t`/`z`. Only the form is checked here; `created` is not
/// compared with the clock.
fn check_created(proof: &Value) -> Result<(), VerificationError> {
    let Some(created) = proof.get("created") else {
        return Ok(());
    };
    match created.as_str().and_then(parse_date_time_stamp) {
        Some(_) => Ok(()),
        None => Err(VerificationError::MalformedProof(format!(
            "proof.created must be a dateTimeStamp with Z or an offset: {created}"
        ))),
    }
}

/// `YYYY-MM-DDThh:mm:ss[.f+](Z|±hh:mm)` with every field in range — month
/// 1-12, a day the month has (leap years included), hour 0-23, minute and
/// second 0-59 (no leap second), an offset of at most 14:00 with minutes 0-59.
/// The Go, TypeScript and Dart verifiers apply the same ranges. chrono's RFC
/// 3339 parser alone would also take a lower-case `t`/`z`, a space separator,
/// and a leap second.
fn parse_date_time_stamp(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::{NaiveDate, TimeZone};
    let b = s.as_bytes();
    let num = |r: std::ops::Range<usize>| -> Option<u32> {
        let d = b.get(r)?;
        d.iter()
            .all(u8::is_ascii_digit)
            .then(|| d.iter().fold(0u32, |n, c| n * 10 + u32::from(c - b'0')))
    };
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let (year, month, day) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (hour, minute, second) = (num(11..13)?, num(14..16)?, num(17..19)?);
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let date = NaiveDate::from_ymd_opt(i32::try_from(year).ok()?, month, day)?;
    let mut rest = &b[19..];
    let mut nanos = 0u32;
    if let Some(fraction) = rest.strip_prefix(b".") {
        let n = fraction.iter().take_while(|c| c.is_ascii_digit()).count();
        if n == 0 {
            return None;
        }
        for i in 0..9 {
            nanos = nanos * 10
                + fraction
                    .get(i)
                    .filter(|_| i < n)
                    .map_or(0, |c| u32::from(c - b'0'));
        }
        rest = &fraction[n..];
    }
    let offset_minutes: i64 = match rest {
        [b'Z'] => 0,
        [sign @ (b'+' | b'-'), ..] if rest.len() == 6 && rest[3] == b':' => {
            let oh = num(b.len() - 5..b.len() - 3)?;
            let om = num(b.len() - 2..b.len())?;
            if oh > 14 || om > 59 || (oh == 14 && om != 0) {
                return None;
            }
            let m = i64::from(oh * 60 + om);
            if *sign == b'+' {
                m
            } else {
                -m
            }
        }
        _ => return None,
    };
    let local = date.and_hms_nano_opt(hour, minute, second, nanos)?;
    let utc = chrono::Utc.from_utc_datetime(&local);
    utc.checked_sub_signed(chrono::TimeDelta::minutes(offset_minutes))
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

#[cfg(test)]
mod expires_tests {
    use super::*;
    use serde_json::json;

    fn at(s: &str) -> chrono::DateTime<chrono::Utc> {
        s.parse().unwrap()
    }

    #[test]
    fn offsets_are_applied() {
        assert_eq!(
            parse_date_time_stamp("2026-06-01T14:00:00+02:00"),
            Some(at("2026-06-01T12:00:00Z"))
        );
        assert_eq!(
            parse_date_time_stamp("2026-06-01T10:30:00.25-01:30"),
            Some(at("2026-06-01T12:00:00.25Z"))
        );
    }

    #[test]
    fn absent_expires_is_accepted() {
        assert!(check_expires(&json!({}), at("2026-06-01T12:00:00Z")).is_ok());
    }

    #[test]
    fn unexpired_and_boundary_expires_are_accepted() {
        let now = at("2026-06-01T12:00:00Z");
        for expires in [
            "2026-06-01T12:00:01Z",
            "2026-06-01T12:00:00Z",
            "2026-06-01T14:00:00.5+02:00",
            "2028-02-29T00:00:00Z",
            "2027-01-01T00:00:00+14:00",
            "2027-01-01T00:00:00-12:59",
            "2027-01-01T23:59:59.999999999Z",
        ] {
            assert!(
                check_expires(&json!({ "expires": expires }), now).is_ok(),
                "{expires}"
            );
        }
    }

    #[test]
    fn passed_or_malformed_expires_is_refused() {
        let now = at("2026-06-01T12:00:00Z");
        for expires in [
            json!("2026-06-01T11:59:59Z"),
            json!("2026-06-01T13:59:59+02:00"),
            json!("2027-01-01T00:00:00"),
            json!("2027-01-01t00:00:00z"),
            json!("2027-01-01 00:00:00Z"),
            json!(1767225600),
            json!("tomorrow"),
            json!("2027-01-01"),
            json!("2027-13-01T00:00:00Z"),
            json!("2027-02-30T00:00:00Z"),
            json!("2027-02-29T00:00:00Z"),
            json!("2027-01-01T24:00:00Z"),
            json!("2027-01-01T00:60:00Z"),
            json!("2027-01-01T00:00:60Z"),
            json!("2027-01-01T00:00:00+24:00"),
            json!("2027-01-01T00:00:00+23:60"),
            json!("2027-01-01T00:00:00+14:01"),
            json!("2027-01-01T00:00:00-99:00"),
            json!(""),
        ] {
            let err = check_expires(&json!({ "expires": expires }), now)
                .expect_err(&format!("{expires} should be refused"));
            assert!(
                matches!(err, VerificationError::MalformedProof(_)),
                "{expires}: {err:?}"
            );
        }
    }
}
