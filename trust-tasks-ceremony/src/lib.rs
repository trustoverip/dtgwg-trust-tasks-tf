//! Verification for **Trust Ceremonies** — the flow layer of SPEC.md §4.11.
//!
//! A ceremony composes several Trust Tasks into one flow. A
//! [`trust-ceremony-receipt`][receipt] attests that one *enactment* of it
//! completed, and this crate checks that attestation.
//!
//! # What verification establishes, and what it cannot
//!
//! A recorder attests **completeness and ordering, never step content**. This
//! crate therefore checks the shape of an enactment against its pinned
//! definition; it does not and cannot tell you that a step's payload meant what
//! you hope. Step content is attested by that step's own issuer through its own
//! `proof`, verified separately (see `trust-tasks-proof`).
//!
//! Three consequences are built into the API rather than left to callers:
//!
//! * [`Outcome::Unverifiable`] is distinct from a failure. A verifier that
//!   cannot resolve the definition has learned nothing, which is not the same as
//!   having learned the receipt is bad.
//! * A receipt whose steps do not include a `terminal` one is a **prefix**, and
//!   is reported as incomplete however `complete` is set — the recorder's own
//!   determination is never trusted (§7.4 of the design note).
//! * Holding none of the step documents is a supported, non-degraded case. You
//!   verify the recorder's attestation and the shape of the flow, and learn
//!   nothing about content — which is correct.
//!
//! # Digests
//!
//! `digestMultibase = multibase(multihash(H(JCS(document) ‖ salt)))`, over the
//! document **including its `proof`**, with the salt as a **suffix**. The
//! rationale for each of those choices is in the `trust-ceremony-receipt/0.1`
//! specification; the trap they avoid is two conforming implementations that
//! cannot verify each other.
//!
//! [receipt]: https://trusttasks.org/spec/trust-ceremony-receipt/0.1

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "affinidi")]
mod affinidi;
#[cfg(feature = "affinidi")]
pub use affinidi::JcsSha256Digester;

/// Computes the salted digest of a step document.
///
/// A trait rather than a function so the canonicalization and hash are a
/// deployment choice: a ceremony's digests only have to agree with *themselves*
/// and with whatever produced its receipt. The [`affinidi`](JcsSha256Digester)
/// backend is the one to use unless you have a reason not to — it canonicalizes
/// identically to `eddsa-jcs-2022`, so a ceremony digest and a Data Integrity
/// proof over the same document agree on what that document is.
pub trait Digester {
    /// Returns the multibase-encoded multihash for `document` under `salt`.
    fn digest(&self, document: &Value, salt: &[u8]) -> Result<String, Error>;
}

/// A completion predicate — the closed combinator set of the ceremony
/// definition format.
///
/// Deliberately not a general expression language. Completion is an *evidence*
/// question settled at verification time, possibly years later and offline, by a
/// party who must not have to execute anyone's code; and a governance body has
/// to be able to read every path that satisfies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Predicate {
    /// Satisfied when the named step is present.
    Step(String),
    /// Satisfied when every sub-predicate is.
    AllOf {
        #[serde(rename = "allOf")]
        all_of: Vec<Predicate>,
    },
    /// Satisfied when any sub-predicate is.
    AnyOf {
        #[serde(rename = "anyOf")]
        any_of: Vec<Predicate>,
    },
    /// Satisfied when at least `n` of something is.
    Threshold { threshold: Threshold },
}

/// The two threshold shapes, which are genuinely different questions.
///
/// `of` counts **distinct named steps** — three of these five endorsements,
/// each its own step. `ofStep` counts the **instances of one step** — two of
/// however many witnesses were bound at enactment, or an approver set's
/// `minApprovals` over N approvers who were not known when the definition was
/// written. The second is the more common governance shape, and an earlier
/// draft of this design had only the first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Threshold {
    /// How many must be satisfied.
    pub n: usize,
    /// Distinct sub-predicates to count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub of: Option<Vec<Predicate>>,
    /// A step name whose *instances* are counted.
    #[serde(rename = "ofStep", default, skip_serializing_if = "Option::is_none")]
    pub of_step: Option<String>,
}

impl Predicate {
    /// Evaluate against the steps an enactment actually contains.
    ///
    /// `instances` maps a step name to how many instances of it are present —
    /// more than one where a step is `perRole` (one per bound party) or repeats
    /// under a bounded-repetition rule.
    pub fn is_satisfied_by(&self, instances: &BTreeMap<String, usize>) -> bool {
        match self {
            Predicate::Step(name) => instances.get(name).copied().unwrap_or(0) > 0,
            Predicate::AllOf { all_of } => all_of.iter().all(|p| p.is_satisfied_by(instances)),
            Predicate::AnyOf { any_of } => any_of.iter().any(|p| p.is_satisfied_by(instances)),
            Predicate::Threshold { threshold } => {
                if let Some(step) = &threshold.of_step {
                    // Instances of ONE step: N approvers each performing it once.
                    instances.get(step).copied().unwrap_or(0) >= threshold.n
                } else if let Some(of) = &threshold.of {
                    // Distinct named steps, each counted at most once however
                    // many instances it has — `n of these five` means five
                    // different things happened, not one thing five times.
                    of.iter().filter(|p| p.is_satisfied_by(instances)).count() >= threshold.n
                } else {
                    // Neither shape given. The definition schema forbids this;
                    // a hand-built Predicate can still reach it, and an
                    // unanswerable predicate is not satisfied.
                    false
                }
            }
        }
    }

    /// Every step name this predicate mentions, for cross-checking against a
    /// definition's declared steps.
    pub fn referenced_steps(&self, out: &mut BTreeSet<String>) {
        match self {
            Predicate::Step(name) => {
                out.insert(name.clone());
            }
            Predicate::AllOf { all_of } => all_of.iter().for_each(|p| p.referenced_steps(out)),
            Predicate::AnyOf { any_of } => any_of.iter().for_each(|p| p.referenced_steps(out)),
            Predicate::Threshold { threshold } => {
                if let Some(step) = &threshold.of_step {
                    out.insert(step.clone());
                }
                if let Some(of) = &threshold.of {
                    of.iter().for_each(|p| p.referenced_steps(out));
                }
            }
        }
    }
}

/// The part of a ceremony definition verification needs.
///
/// Not the whole format: a verifier checks completion, recorders, and the
/// repetition bound, and has no use for descriptions or role prose. Deserializes
/// from a published definition, ignoring what it does not need.
#[derive(Debug, Clone, Deserialize)]
pub struct Definition {
    pub slug: String,
    pub version: String,
    pub completion: Predicate,
    pub evidence: Evidence,
    #[serde(default)]
    pub steps: BTreeMap<String, DefinitionStep>,
}

/// A step as the definition declares it.
#[derive(Debug, Clone, Deserialize)]
pub struct DefinitionStep {
    #[serde(rename = "type", default)]
    pub type_uri: Option<String>,
    #[serde(rename = "maxRounds", default)]
    pub max_rounds: Option<u32>,
    #[serde(default)]
    pub terminal: bool,
}

/// The definition's evidence declaration.
#[derive(Debug, Clone, Deserialize)]
pub struct Evidence {
    pub level: String,
    #[serde(default)]
    pub recorders: Vec<String>,
}

/// One step as a receipt enumerates it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptStep {
    pub step: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<u32>,
    #[serde(rename = "typeUri")]
    pub type_uri: String,
    pub issuer: String,
    pub id: String,
    #[serde(rename = "digestMultibase")]
    pub digest_multibase: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal: Option<bool>,
}

/// The payload of a `trust-ceremony-receipt/0.1` document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub enactment: String,
    #[serde(
        rename = "parentEnactment",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_enactment: Option<String>,
    pub definition: String,
    #[serde(rename = "definitionDigest")]
    pub definition_digest: String,
    pub complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    pub steps: Vec<ReceiptStep>,
}

/// What a verification concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The enactment satisfies its definition's completion rule, includes a
    /// terminal step, and every step document supplied matched its digest.
    Complete,
    /// Structurally sound but not complete — the completion rule is unsatisfied,
    /// or no terminal step is present. A prefix looks exactly like this, which
    /// is the point.
    Incomplete { reason: String },
    /// Verification could not be performed. **Not** a failure: the definition
    /// could not be resolved, or its digest did not match what the steps pinned,
    /// so nothing has been learned either way.
    Unverifiable { reason: String },
    /// The receipt is invalid — it contradicts itself or a document it names.
    Invalid { reason: String },
}

/// Verification errors distinct from a verification *outcome*.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("canonicalization failed: {0}")]
    Canonicalization(String),
    #[error("digest encoding failed: {0}")]
    DigestEncoding(String),
    #[error("salt is not valid multibase: {0}")]
    Salt(String),
}

/// Verify a receipt against its definition and whatever step documents the
/// caller holds.
///
/// `held` maps a step document's `id` to the full document. It may be empty:
/// a verifier that holds no step documents still checks the recorder's
/// attestation and the shape of the enactment.
///
/// `recorder` is the receipt document's `issuer` — passed in rather than read
/// from a payload, because the party that signed the receipt is an envelope
/// fact and a payload could claim anything.
pub fn verify(
    receipt: &Receipt,
    definition: &Definition,
    definition_digest: &str,
    recorder: &str,
    held: &BTreeMap<String, Value>,
    digester: &dyn Digester,
) -> Result<Outcome, Error> {
    // The definition must be the one the steps were enacted under. A mismatch
    // means this receipt cannot be checked against this definition — it does not
    // mean the receipt is bad.
    if receipt.definition_digest != definition_digest {
        return Ok(Outcome::Unverifiable {
            reason: format!(
                "receipt pins definition digest {} but the definition supplied digests to {}",
                receipt.definition_digest, definition_digest
            ),
        });
    }

    // The recorder must be one the definition names. A receipt from a party the
    // definition never authorised to record is not evidence about this ceremony.
    if !definition.evidence.recorders.is_empty()
        && !definition.evidence.recorders.iter().any(|r| r == recorder)
    {
        return Ok(Outcome::Invalid {
            reason: format!("recorder {recorder} is not named by the definition"),
        });
    }

    let mut instances: BTreeMap<String, usize> = BTreeMap::new();
    for s in &receipt.steps {
        // An enumerated step the definition does not declare cannot be checked
        // and cannot count toward completion.
        let Some(declared) = definition.steps.get(&s.step) else {
            return Ok(Outcome::Invalid {
                reason: format!("step \"{}\" is not declared by the definition", s.step),
            });
        };
        if let Some(expected) = &declared.type_uri {
            if &s.type_uri != expected {
                return Ok(Outcome::Invalid {
                    reason: format!(
                        "step \"{}\" enacted {} but the definition declares {expected}",
                        s.step, s.type_uri
                    ),
                });
            }
        }
        let round = s.round.unwrap_or(1);
        if round > declared.max_rounds.unwrap_or(1) {
            return Ok(Outcome::Invalid {
                reason: format!(
                    "step \"{}\" is round {round} but the definition permits {}",
                    s.step,
                    declared.max_rounds.unwrap_or(1)
                ),
            });
        }
        *instances.entry(s.step.clone()).or_insert(0) += 1;

        // Recompute the digest of any document we actually hold. A mismatch is
        // the receipt naming a document other than the one we have.
        if let Some(doc) = held.get(&s.id) {
            let salt = decode_salt(receipt.salt.as_deref())?;
            let recomputed = digester.digest(doc, &salt)?;
            if recomputed != s.digest_multibase {
                return Ok(Outcome::Invalid {
                    reason: format!("step \"{}\" digest mismatch for document {}", s.step, s.id),
                });
            }
        }
    }

    // Truncation check. A chain catches an omitted step through its successor,
    // and the trailing steps have none — so without this a valid prefix passes
    // as a completed enactment.
    if !receipt.steps.iter().any(|s| s.terminal.unwrap_or(false)) {
        return Ok(Outcome::Incomplete {
            reason: "no enumerated step is terminal; this is a prefix, not a completed enactment"
                .into(),
        });
    }

    // The recorder's own `complete` is deliberately not consulted.
    if !definition.completion.is_satisfied_by(&instances) {
        return Ok(Outcome::Incomplete {
            reason: "the definition's completion rule is not satisfied by the enumerated steps"
                .into(),
        });
    }

    Ok(Outcome::Complete)
}

fn decode_salt(salt: Option<&str>) -> Result<Vec<u8>, Error> {
    match salt {
        None => Ok(Vec::new()),
        Some(s) => {
            #[cfg(feature = "affinidi")]
            {
                multibase::decode(s)
                    .map(|(_, bytes)| bytes)
                    .map_err(|e| Error::Salt(e.to_string()))
            }
            #[cfg(not(feature = "affinidi"))]
            {
                Ok(s.as_bytes().to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests;
