//! W3C Data Integrity proof attached to a Trust Task document.
//!
//! Implements the shape required by SPEC.md §4.7. The proof object is
//! deliberately minimal — the framework does not constrain the cryptographic
//! suite, so unknown members are preserved via `extra` for forward
//! compatibility with future suites.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A W3C Data Integrity proof object as required by SPEC.md §4.7.
///
/// When present, the proof binds the document's content to the party
/// identified by the document's `issuer` member. Verification is the
/// responsibility of the caller — this crate models the structure but does not
/// implement any specific cryptosuite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    /// The proof type, typically `"DataIntegrityProof"`.
    #[serde(rename = "type")]
    pub proof_type: String,

    /// The cryptographic suite identifier, e.g. `"eddsa-rdfc-2022"`.
    pub cryptosuite: String,

    /// A URL identifying the verification material. Per SPEC.md §4.7, the
    /// `verificationMethod` MUST resolve to material controlled by the party
    /// identified by the document's `issuer` member.
    #[serde(rename = "verificationMethod")]
    pub verification_method: String,

    /// When the proof was created.
    pub created: DateTime<Utc>,

    /// The relationship between the issuer and the assertion in the proof.
    #[serde(rename = "proofPurpose")]
    pub proof_purpose: String,

    /// The proof value, encoded according to the chosen cryptosuite.
    #[serde(rename = "proofValue")]
    pub proof_value: String,

    /// Any additional members carried by the proof (suite-specific or
    /// future-spec). Preserved on round-trip.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
