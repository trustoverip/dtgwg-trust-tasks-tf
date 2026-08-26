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
//! Four consequences are built into the API rather than left to callers:
//!
//! * [`Outcome::Unverifiable`] is distinct from a failure. A verifier that
//!   cannot resolve the definition has learned nothing, which is not the same as
//!   having learned the receipt is bad.
//! * A receipt whose steps do not include a `terminal` one is a **prefix**, and
//!   is reported as incomplete however `complete` is set — the recorder's own
//!   determination is never trusted (§7.4 of the design note).
//! * **`terminal` is a value the terminal step's issuer signs, not one the
//!   recorder may assert.** [`Outcome::Complete`] therefore requires that the
//!   verifier hold a step document whose *signed* `ceremony.terminal` is true
//!   and whose definition declares it terminal. A verifier holding no terminal
//!   step document gets [`Outcome::Unverifiable`], because nothing but the
//!   recorder's word would be backing the marker the truncation defence rests
//!   on.
//! * Holding none of the step documents is a supported, non-degraded case. You
//!   verify the recorder's attestation and the shape of the flow, and learn
//!   nothing about content — which is correct. It is *also* the case in which
//!   the truncation defence cannot be evaluated, so it yields `Unverifiable`
//!   rather than `Complete`: an honest "I could not check this" instead of a
//!   completion claim resting on the party whose truncation it exists to catch.
//!
//! # What this crate checks
//!
//! Against the eight-step list of `docs/design-notes/trust-ceremonies.md` §7.9:
//!
//! | § | Check | Here |
//! |---|---|---|
//! | 1 | Group by `enactment`, reject disagreement on `definitionDigest` | over held documents |
//! | 2 | Resolve each `step` in the definition | yes |
//! | 3 | `round` within the repetition bound | yes |
//! | 4 | Walk `prev`, check the salted digests | over held documents |
//! | 5 | `issuer` / `recipient` against the definition's roles | yes, given [`Bindings`] |
//! | 6 | Recurse into a nested ceremony step | **no** — reported `Unverifiable` |
//! | 7 | Evaluate the completion predicate | yes |
//! | 8 | Confirm a terminal step | yes, from the *signed* marker |
//!
//! Two things are checkable only for documents the caller holds, and are
//! disclaimed rather than silently skipped: the `prev` chain walk (§4) and the
//! terminal marker (§8). A receipt whose terminal step document is absent is
//! `Unverifiable`, and one whose held documents chain cleanly has had exactly
//! that subset of its ordering checked. `maxDuration` is not evaluated at all —
//! the receipt payload carries no `issuedAt`, so the issuance window is not
//! derivable from a receipt.
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
///
/// Both count instances that [`verify`] has already de-duplicated. An
/// instance count is only a threshold if listing one party's step twice cannot
/// raise it.
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
    /// `instances` maps a step name to how many **distinct** instances of it are
    /// present — more than one where a step is `perRole` (one per bound party)
    /// or repeats under a bounded-repetition rule. [`verify`] rejects a receipt
    /// that enumerates the same instance twice before it builds this map, which
    /// is what stops a recorder inflating a threshold by repetition.
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

/// Whether a step enacts a Trust Task or nests another ceremony.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    /// The step enacts a Trust Task, named by its Type URI.
    #[default]
    Task,
    /// The step is itself a ceremony, whose evidence is that child's own
    /// receipt. This crate does not recurse into one — see [`verify`].
    Ceremony,
}

/// Whether a step runs once, or once per VID bound to its issuing role.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Multiplicity {
    /// One instance per round.
    #[default]
    Single,
    /// One instance per VID bound to the issuing role — N approvers each
    /// returning one decision. Instances are discriminated by the document's
    /// **signed** `issuer`, which is what makes two entries differing only in
    /// `issuer` two steps rather than one step listed twice.
    PerRole,
}

/// The part of a ceremony definition verification needs.
///
/// Not the whole format: a verifier checks completion, roles, recorders, the
/// step graph and the repetition bound, and has no use for descriptions or role
/// prose. Deserializes from a published definition, ignoring what it does not
/// need.
#[derive(Debug, Clone, Deserialize)]
pub struct Definition {
    pub slug: String,
    pub version: String,
    pub completion: Predicate,
    pub evidence: Evidence,
    /// Named participants. Bound to actual VIDs at enactment rather than here,
    /// which is why [`verify`] takes [`Bindings`] — a definition alone cannot
    /// say who a step's issuer should have been.
    #[serde(default)]
    pub roles: BTreeMap<String, DefinitionRole>,
    #[serde(default)]
    pub steps: BTreeMap<String, DefinitionStep>,
}

/// A role as the definition declares it.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DefinitionRole {
    /// `one` or `many`. `many` binds the role to a *set* of VIDs at enactment.
    #[serde(default)]
    pub cardinality: Option<String>,
    /// A party essential to the ceremony's meaning that issues and receives no
    /// step. A ceremony cannot attest one, and no step may name it.
    #[serde(default)]
    pub evidentiary: bool,
}

/// A step as the definition declares it.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DefinitionStep {
    #[serde(default)]
    pub kind: StepKind,
    #[serde(rename = "type", default)]
    pub type_uri: Option<String>,
    /// Role name that issues this step.
    #[serde(default)]
    pub issuer: Option<String>,
    /// Role name that receives it.
    #[serde(default)]
    pub recipient: Option<String>,
    #[serde(default)]
    pub multiplicity: Multiplicity,
    #[serde(rename = "maxRounds", default)]
    pub max_rounds: Option<u32>,
    /// Step names this step depends on. The definition's ordering claim; a
    /// document's signed `ceremony.prev` is checked against it.
    #[serde(default)]
    pub prev: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    /// Whether this step **may** end the enactment. Necessary but not
    /// sufficient: the marker that closes truncation is the `ceremony.terminal`
    /// a step's own issuer signed, and this only says which steps are permitted
    /// to carry it.
    #[serde(default)]
    pub terminal: bool,
}

/// The definition's evidence declaration.
#[derive(Debug, Clone, Deserialize)]
pub struct Evidence {
    pub level: String,
    /// **Role names** — not VIDs — permitted to issue the receipt. Resolving
    /// them to VIDs needs the enactment's [`Bindings`].
    #[serde(default)]
    pub recorders: Vec<String>,
}

/// The enactment's role → VID bindings.
///
/// A definition names roles; an enactment binds each to one VID, or to a set of
/// them where the role's cardinality is `many`. Without this a verifier cannot
/// answer either of the two questions the receipt specification makes it a
/// **MUST** to answer — whether the recorder is one the definition names, and
/// whether a step's issuer was the party the definition says should have issued
/// it — because both compare a VID on the wire against a role name in the
/// definition.
///
/// Distribute it alongside the enactment identifier and the salt. It is not
/// derivable from the receipt: a receipt payload could claim any binding it
/// liked, which is the same reason [`verify`] takes the recorder as an envelope
/// fact rather than reading one out of the payload.
///
/// [`Bindings::unbound`] is the honest way to say you do not have them.
/// [`verify`] then reports what it could not check rather than passing it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Bindings(BTreeMap<String, BTreeSet<String>>);

impl Bindings {
    /// No bindings at all. Every role-dependent check is reported as unchecked.
    pub fn unbound() -> Self {
        Self::default()
    }

    /// Bind a role to the VIDs the enactment gave it.
    #[must_use]
    pub fn with<R, V, I>(mut self, role: R, vids: I) -> Self
    where
        R: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = V>,
    {
        self.0
            .entry(role.into())
            .or_default()
            .extend(vids.into_iter().map(Into::into));
        self
    }

    /// The VIDs bound to `role`, or `None` where the enactment bound none —
    /// which is not the same as binding it to the empty set.
    pub fn vids(&self, role: &str) -> Option<&BTreeSet<String>> {
        self.0.get(role)
    }

    /// Whether no role is bound at all.
    pub fn is_unbound(&self) -> bool {
        self.0.is_empty()
    }
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
    /// The recorder's **echo** of the step document's `ceremony.terminal`.
    ///
    /// Recorder-supplied, and therefore never sufficient on its own: the whole
    /// point of the terminal marker is that a recorder cannot mint it. [`verify`]
    /// treats this as a claim to be checked against the signed value in the step
    /// document, never as the marker itself.
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
    /// The enactment satisfies its definition's completion rule; a step document
    /// the verifier holds carries a **signed** `ceremony.terminal` on a step the
    /// definition declares terminal; every held document matched its digest and
    /// chained; and every role-dependent check was answerable.
    Complete,
    /// Structurally sound but not complete — the completion rule is unsatisfied,
    /// or no enumerated step is one the definition could end on. A prefix looks
    /// exactly like this, which is the point.
    Incomplete { reason: String },
    /// Verification could not be performed, in whole or in part. **Not** a
    /// failure: the definition could not be resolved, the terminal step document
    /// is not held, the enactment's role bindings were not supplied, or a nested
    /// ceremony would have to be verified on its own terms — so on that point
    /// nothing has been learned either way.
    Unverifiable { reason: String },
    /// The receipt is invalid — it contradicts itself, the definition, or a
    /// document it names.
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

/// Verify a receipt against its definition, the enactment's role bindings, and
/// whatever step documents the caller holds.
///
/// `held` maps a step document's `id` to the full document. It may be empty: a
/// verifier that holds no step documents still checks the recorder's
/// attestation and the shape of the enactment. What it cannot then check is the
/// signed terminal marker, so the best available outcome in that case is
/// [`Outcome::Unverifiable`] — see the crate documentation.
///
/// `recorder` is the receipt document's `issuer` — passed in rather than read
/// from a payload, because the party that signed the receipt is an envelope
/// fact and a payload could claim anything.
///
/// `bindings` is the enactment's role → VID map. Pass [`Bindings::unbound`] if
/// you do not have it; the role-dependent rules are then reported as unchecked
/// rather than assumed to pass.
///
/// # What makes a receipt `Invalid`
///
/// Beyond the digest and declaration checks, three forgeries a recorder could
/// otherwise construct are rejected here:
///
/// * **A minted terminal marker.** `terminal` on a receipt entry whose step the
///   definition does not declare terminal, or which contradicts the signed
///   `ceremony.terminal` of a document the verifier holds.
/// * **A repeated instance.** Two entries sharing a document `id`; two entries
///   for one step at one round; two entries for one `perRole` step at one round
///   from one issuer. Each would otherwise raise a `threshold.ofStep` count by
///   listing one party's step more than once.
/// * **An omitted intermediate step.** A held document's signed `ceremony.prev`
///   naming a predecessor the receipt does not enumerate, commits to a digest
///   the receipt disagrees with, or names a step that is not an ancestor of this
///   one in the definition's graph.
pub fn verify(
    receipt: &Receipt,
    definition: &Definition,
    definition_digest: &str,
    recorder: &str,
    bindings: &Bindings,
    held: &BTreeMap<String, Value>,
    digester: &dyn Digester,
) -> Result<Outcome, Error> {
    // Things this run could not settle. They are collected rather than returned
    // at once, so a receipt that is outright Invalid is reported as Invalid
    // rather than as unverifiable; and they are never dropped, so a receipt that
    // would otherwise be Complete is downgraded to Unverifiable instead of
    // being credited with a check that did not happen.
    let mut unchecked: Vec<String> = Vec::new();

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

    // The recorder must be a VID bound to a recorder role. `evidence.recorders`
    // holds ROLE NAMES, so this is unanswerable without the enactment's
    // bindings — and an unanswerable MUST is reported, not skipped.
    if !definition.evidence.recorders.is_empty() {
        let mut matched = false;
        let mut unbound: Vec<&str> = Vec::new();
        for role in &definition.evidence.recorders {
            match bindings.vids(role) {
                Some(vids) => {
                    if vids.contains(recorder) {
                        matched = true;
                    }
                }
                None => unbound.push(role.as_str()),
            }
        }
        if !matched {
            if unbound.is_empty() {
                return Ok(Outcome::Invalid {
                    reason: format!(
                        "recorder {recorder} is not bound to any recorder role the definition names ({})",
                        definition.evidence.recorders.join(", ")
                    ),
                });
            }
            unchecked.push(format!(
                "the recorder {recorder} could not be confirmed: the definition names recorder role(s) {} for which no VID binding was supplied",
                unbound.join(", ")
            ));
        }
    }

    let salt = decode_salt(receipt.salt.as_deref())?;

    let mut instances: BTreeMap<String, usize> = BTreeMap::new();
    // De-duplication keys. `id` is globally unique per SPEC §4.3, so a repeat is
    // one document listed twice; the instance key is what a definition says
    // legitimately distinguishes two instances of one step.
    let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
    let mut seen_instances: BTreeSet<(&str, u32, Option<&str>)> = BTreeSet::new();
    let mut unbound_roles: BTreeSet<&str> = BTreeSet::new();
    // Steps the definition permits to end the enactment.
    let mut terminal_candidates: Vec<&ReceiptStep> = Vec::new();
    let mut terminal_candidates_held = 0usize;
    let mut terminal_confirmed: Option<&ReceiptStep> = None;
    // (step that carries them, its prev entries) for held documents only.
    let mut chain: Vec<(&ReceiptStep, Vec<PrevRef>)> = Vec::new();

    for s in &receipt.steps {
        // An enumerated step the definition does not declare cannot be checked
        // and cannot count toward completion.
        let Some(declared) = definition.steps.get(&s.step) else {
            return Ok(Outcome::Invalid {
                reason: format!("step \"{}\" is not declared by the definition", s.step),
            });
        };

        // A nested ceremony's evidence is its own receipt, verified on its own
        // terms against its own pinned definition. This crate does not recurse,
        // and a receipt containing one is therefore not fully checkable here —
        // which is reported rather than passed over.
        if declared.kind == StepKind::Ceremony {
            unchecked.push(format!(
                "step \"{}\" nests a ceremony; its child receipt is verified on its own terms and this verifier does not recurse into one",
                s.step
            ));
        } else if let Some(expected) = &declared.type_uri {
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

        // --- de-duplication -------------------------------------------------
        //
        // Without this, `*instances.entry(..) += 1` counts whatever the recorder
        // typed, and `threshold.ofStep n = 2` is satisfied by listing one
        // approver's single decision twice.
        if !seen_ids.insert(s.id.as_str()) {
            return Ok(Outcome::Invalid {
                reason: format!(
                    "document {} is enumerated more than once; an `id` is globally unique, so this is one step counted twice",
                    s.id
                ),
            });
        }
        // Only `perRole` makes the issuer a discriminator: the definition says
        // one instance per bound party. Anywhere else, two entries for one step
        // at one round are the same instance however they are signed.
        let discriminator =
            (declared.multiplicity == Multiplicity::PerRole).then_some(s.issuer.as_str());
        if !seen_instances.insert((s.step.as_str(), round, discriminator)) {
            return Ok(Outcome::Invalid {
                reason: match discriminator {
                    Some(issuer) => format!(
                        "step \"{}\" round {round} is enumerated more than once for issuer {issuer}; a perRole step admits one instance per bound party",
                        s.step
                    ),
                    None => format!(
                        "step \"{}\" is enumerated more than once at round {round}; the definition declares it single-multiplicity, so this is one instance counted twice",
                        s.step
                    ),
                },
            });
        }

        // --- role binding ---------------------------------------------------
        //
        // A distinct issuer per entry is only a distinct *party* if each is one
        // the definition authorised. Otherwise a recorder inflates a threshold
        // with invented VIDs instead of repeated ones.
        match &declared.issuer {
            Some(role) => {
                // A definition that declares no roles at all is one deserialized
                // for a narrower purpose; where it does declare them, the role a
                // step names must be among them and must not be evidentiary.
                if !definition.roles.is_empty() {
                    match definition.roles.get(role) {
                        Some(r) if r.evidentiary => {
                            return Ok(Outcome::Invalid {
                                reason: format!(
                                    "step \"{}\" names evidentiary role \"{role}\" as its issuer; an evidentiary role exchanges no Trust Task document",
                                    s.step
                                ),
                            });
                        }
                        Some(_) => {}
                        None => {
                            return Ok(Outcome::Unverifiable {
                                reason: format!(
                                    "the definition declares step \"{}\" issued by role \"{role}\", which it does not define",
                                    s.step
                                ),
                            });
                        }
                    }
                }
                match bindings.vids(role) {
                    Some(vids) if !vids.contains(&s.issuer) => {
                        return Ok(Outcome::Invalid {
                            reason: format!(
                                "step \"{}\" was issued by {} but the enactment binds role \"{role}\" to {}",
                                s.step,
                                s.issuer,
                                vids.iter().cloned().collect::<Vec<_>>().join(", ")
                            ),
                        });
                    }
                    Some(_) => {}
                    None => {
                        unbound_roles.insert(role.as_str());
                    }
                }
            }
            None => unchecked.push(format!(
                "step \"{}\" has no issuer role in the definition, so its issuer {} could not be bound to one",
                s.step, s.issuer
            )),
        }

        // --- terminal ---------------------------------------------------
        //
        // `ReceiptStep::terminal` is the recorder's echo. The definition says
        // which steps MAY end the enactment; the step issuer's signature says
        // which one DID.
        let echoed_terminal = s.terminal.unwrap_or(false);
        if echoed_terminal && !declared.terminal {
            return Ok(Outcome::Invalid {
                reason: format!(
                    "the receipt marks step \"{}\" terminal, but the definition does not declare it a terminal step; a recorder cannot create one by asserting it",
                    s.step
                ),
            });
        }
        if declared.terminal {
            terminal_candidates.push(s);
        }

        // --- held documents -------------------------------------------------
        if let Some(doc) = held.get(&s.id) {
            let recomputed = digester.digest(doc, &salt)?;
            if recomputed != s.digest_multibase {
                return Ok(Outcome::Invalid {
                    reason: format!("step \"{}\" digest mismatch for document {}", s.step, s.id),
                });
            }

            // §7.9 step 1: one enactment, one pinned definition. A document that
            // disagrees is a step of a different ceremony.
            let ceremony = doc.get("ceremony");
            if let Some(enactment) = ceremony
                .and_then(|c| c.get("enactment"))
                .and_then(Value::as_str)
            {
                if enactment != receipt.enactment {
                    return Ok(Outcome::Invalid {
                        reason: format!(
                            "document {} carries enactment {enactment} but the receipt reports on {}",
                            s.id, receipt.enactment
                        ),
                    });
                }
            }
            if let Some(pinned) = ceremony
                .and_then(|c| c.get("definitionDigest"))
                .and_then(Value::as_str)
            {
                if pinned != receipt.definition_digest {
                    return Ok(Outcome::Invalid {
                        reason: format!(
                            "document {} pins definition digest {pinned} but the receipt pins {}",
                            s.id, receipt.definition_digest
                        ),
                    });
                }
            }
            if let Some(signed_step) = ceremony.and_then(|c| c.get("step")).and_then(Value::as_str)
            {
                if signed_step != s.step {
                    return Ok(Outcome::Invalid {
                        reason: format!(
                            "document {} signs step \"{signed_step}\" but the receipt enumerates it as \"{}\"",
                            s.id, s.step
                        ),
                    });
                }
            }
            let signed_round = ceremony
                .and_then(|c| c.get("round"))
                .and_then(Value::as_u64)
                .unwrap_or(1);
            if signed_round != u64::from(round) {
                return Ok(Outcome::Invalid {
                    reason: format!(
                        "document {} signs round {signed_round} but the receipt enumerates round {round}",
                        s.id
                    ),
                });
            }
            if let Some(signed_issuer) = doc.get("issuer").and_then(Value::as_str) {
                if signed_issuer != s.issuer {
                    return Ok(Outcome::Invalid {
                        reason: format!(
                            "document {} is signed by {signed_issuer} but the receipt attributes it to {}",
                            s.id, s.issuer
                        ),
                    });
                }
            }
            // The recipient exists only on the document; the receipt does not
            // enumerate it, so this half of §7.9 step 5 is checkable only here.
            if let (Some(role), Some(rcpt)) = (
                declared.recipient.as_deref(),
                doc.get("recipient").and_then(Value::as_str),
            ) {
                if let Some(vids) = bindings.vids(role) {
                    if !vids.contains(rcpt) {
                        return Ok(Outcome::Invalid {
                            reason: format!(
                                "document {} was addressed to {rcpt} but the enactment binds recipient role \"{role}\" to {}",
                                s.id,
                                vids.iter().cloned().collect::<Vec<_>>().join(", ")
                            ),
                        });
                    }
                } else {
                    unbound_roles.insert(role);
                }
            }

            // The marker itself, from the signed content.
            let signed_terminal = ceremony
                .and_then(|c| c.get("terminal"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if echoed_terminal && !signed_terminal {
                return Ok(Outcome::Invalid {
                    reason: format!(
                        "the receipt marks step \"{}\" terminal but document {} does not carry ceremony.terminal in its signed content",
                        s.step, s.id
                    ),
                });
            }
            if signed_terminal && !declared.terminal {
                return Ok(Outcome::Invalid {
                    reason: format!(
                        "document {} carries ceremony.terminal but the definition does not declare step \"{}\" terminal",
                        s.id, s.step
                    ),
                });
            }
            if declared.terminal {
                terminal_candidates_held += 1;
                if signed_terminal && terminal_confirmed.is_none() {
                    terminal_confirmed = Some(s);
                }
            }

            chain.push((s, prev_refs(ceremony)));
        }

        *instances.entry(s.step.clone()).or_insert(0) += 1;
    }

    if !unbound_roles.is_empty() {
        unchecked.push(format!(
            "no VID binding was supplied for role(s) {}, so the issuer or recipient of the steps they govern could not be checked against the definition",
            unbound_roles.iter().copied().collect::<Vec<_>>().join(", ")
        ));
    }

    // --- the chain ----------------------------------------------------------
    //
    // §7.9 step 4. This is what carries the receipt specification's claim that
    // "its successor committed to its digest": an omitted intermediate step is
    // caught here, and nowhere else. Only held documents carry `ceremony.prev`,
    // so a verifier holding none has this checked over the empty set — which is
    // exactly what the spec's "the recorder attests ordering" is worth without
    // documents in hand.
    let by_id: BTreeMap<&str, &ReceiptStep> =
        receipt.steps.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut any_prev = false;
    for (s, prevs) in &chain {
        for p in prevs {
            any_prev = true;
            let Some(predecessor) = by_id.get(p.id.as_str()) else {
                return Ok(Outcome::Invalid {
                    reason: format!(
                        "document {} commits to predecessor {} in its signed ceremony.prev, but the receipt does not enumerate it; a recorder MUST enumerate every step it observed",
                        s.id, p.id
                    ),
                });
            };
            if predecessor.digest_multibase != p.digest_multibase {
                return Ok(Outcome::Invalid {
                    reason: format!(
                        "document {} commits to predecessor {} with digest {} but the receipt gives that document digest {}",
                        s.id, p.id, p.digest_multibase, predecessor.digest_multibase
                    ),
                });
            }
            // The definition's ordering claim. A transitive ancestor is accepted
            // because an enactment that skips an optional step legitimately
            // chains past it; the same step is accepted because a bounded
            // repetition chains to its own previous round.
            if predecessor.step != s.step && !is_ancestor(definition, &predecessor.step, &s.step) {
                return Ok(Outcome::Invalid {
                    reason: format!(
                        "document {} follows step \"{}\", which is not an ancestor of \"{}\" in the definition's prev graph",
                        s.id, predecessor.step, s.step
                    ),
                });
            }
        }
    }
    // Recorder conformance rule 5: the salt is REQUIRED wherever any step
    // carried `prev`, because without it the chain cannot be recomputed.
    if any_prev && receipt.salt.is_none() {
        return Ok(Outcome::Invalid {
            reason: "a step carries ceremony.prev but the receipt omits the enactment salt, so the chain cannot be recomputed".into(),
        });
    }

    // The recorder's own `complete` is deliberately not consulted.
    if !definition.completion.is_satisfied_by(&instances) {
        return Ok(Outcome::Incomplete {
            reason: "the definition's completion rule is not satisfied by the enumerated steps"
                .into(),
        });
    }

    // --- truncation ---------------------------------------------------------
    //
    // A chain catches an omitted step through its successor, and the trailing
    // steps have none — so without this a valid prefix passes as a completed
    // enactment. The marker that closes it is signed by the terminal step's
    // issuer; a receipt entry asserting it is the recorder's word for the one
    // thing the recorder is specifically not trusted on.
    if terminal_candidates.is_empty() {
        return Ok(Outcome::Incomplete {
            reason:
                "no enumerated step is one the definition declares terminal; this is a prefix, not a completed enactment"
                    .into(),
        });
    }
    match terminal_confirmed {
        Some(_) => {}
        None if terminal_candidates_held == terminal_candidates.len() => {
            return Ok(Outcome::Incomplete {
                reason: format!(
                    "every step the definition could end on is held ({}) and none carries ceremony.terminal in its signed content; this is a prefix",
                    terminal_candidates
                        .iter()
                        .map(|s| s.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }
        None => unchecked.push(format!(
            "no terminal step document is held: the definition declares step(s) {} terminal and the receipt enumerates {}, but ceremony.terminal is signed by the step's issuer and the receipt's echo of it is the recorder's own word",
            terminal_candidates
                .iter()
                .map(|s| s.step.as_str())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", "),
            terminal_candidates
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }

    if !unchecked.is_empty() {
        return Ok(Outcome::Unverifiable {
            reason: unchecked.join("; "),
        });
    }

    Ok(Outcome::Complete)
}

/// One entry of a document's signed `ceremony.prev`.
struct PrevRef {
    id: String,
    digest_multibase: String,
}

fn prev_refs(ceremony: Option<&Value>) -> Vec<PrevRef> {
    ceremony
        .and_then(|c| c.get("prev"))
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|e| {
                    Some(PrevRef {
                        id: e.get("id")?.as_str()?.to_string(),
                        digest_multibase: e.get("digestMultibase")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `ancestor` precedes `of` anywhere in the definition's `prev` graph.
///
/// Transitive rather than direct, because a definition's optional steps may be
/// skipped and the enactment then chains past them. The visited set is what
/// keeps this terminating on a definition whose graph has a cycle — a
/// publication error the ceremony checks reject, but not one a verifier should
/// hang on.
fn is_ancestor(definition: &Definition, ancestor: &str, of: &str) -> bool {
    let mut stack = vec![of];
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    while let Some(name) = stack.pop() {
        let Some(step) = definition.steps.get(name) else {
            continue;
        };
        for p in &step.prev {
            if p == ancestor {
                return true;
            }
            if seen.insert(p.as_str()) {
                stack.push(p.as_str());
            }
        }
    }
    false
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
