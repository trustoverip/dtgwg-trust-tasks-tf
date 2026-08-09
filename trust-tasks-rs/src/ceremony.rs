//! The `ceremony` envelope member (SPEC.md §4.11).
//!
//! A *Trust Ceremony* is a flow composed of several Trust Tasks. A document
//! carrying [`Ceremony`] records that it is one *step* of one *enactment* — one
//! run of such a flow.
//!
//! Three properties of this type are worth stating, because they are the whole
//! reason it lives on the envelope rather than in a payload:
//!
//! * **No specification changes.** The member is framework-level, so any
//!   existing Trust Task can be composed into a flow its author never
//!   anticipated, with no schema edit and no new version.
//! * **It is signed.** A [`Proof`](crate::Proof) covers the document with
//!   `proof` excluded, so `enactment` and `definition_digest` are bound to the
//!   step by its issuer. A step cannot be lifted into a different enactment, nor
//!   reinterpreted under a definition that gives its step name another meaning.
//! * **It confers no authority.** Membership is an assertion by the issuer, not
//!   a verified fact (§4.11.4, §7.2 item 9). Every authorization decision rests
//!   on `issuer`, `proof`, and local policy exactly as it would for a document
//!   with no ceremony member — which is precisely what makes it safe for a
//!   consumer to ignore this member entirely.

use serde::{Deserialize, Serialize};

/// Records that a document is a step of a Trust Ceremony (SPEC.md §4.11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ceremony {
    /// Identifies one run of a ceremony.
    ///
    /// Globally unique and never reused, on the same terms as the document
    /// [`id`](crate::TrustTask::id) — and unlike `threadId`, which §4.9 does not
    /// require to be either. Evidence about a flow needs a stable anchor, and
    /// this is the value an outside reference names when citing the flow rather
    /// than one exchange (§4.9.1).
    pub enactment: String,

    /// Names this step within the ceremony.
    ///
    /// The step name — not the Type URI — is the step's identity: one Type URI
    /// may serve several steps whose meaning differs by context, and may recur
    /// within a flow.
    pub step: String,

    /// The ceremony definition this step is enacted under (§6.7).
    ///
    /// Optional: a flow whose evidence is only its collected or chained
    /// documents needs no published definition, and requiring one would make
    /// ad-hoc use impossible. Where set, [`definition_digest`](Self::definition_digest)
    /// MUST also be set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,

    /// Multibase-encoded multihash over the RFC 8785 (JCS) canonicalization of
    /// the definition named by [`definition`](Self::definition).
    ///
    /// Pins the definition by *content* rather than by name. Without it the
    /// completion rule, role list and evidence level of a flow are whatever the
    /// URI serves at verification time — which may be years later, under
    /// different control, and retroactively for every enactment already
    /// performed.
    #[serde(
        rename = "definitionDigest",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub definition_digest: Option<String>,

    /// The enactment containing this one, where a ceremony is conducted as a
    /// step of another.
    ///
    /// Takes the posture of `parentThreadId` (§4.9.2): one level, navigation
    /// only, no normative validation semantics, and never equal to this
    /// document's own [`enactment`](Self::enactment). One level is not a depth
    /// limit — the pointers form a chain, so ceremonies nest arbitrarily deep.
    #[serde(
        rename = "parentEnactment",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_enactment: Option<String>,

    /// Distinguishes repetitions of the same step by the same party, where a
    /// definition permits bounded repetition. `None` means round 1.
    ///
    /// Being signed, it is what stops one round's document being replayed as
    /// another's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<u32>,

    /// Marks a step that ends the enactment.
    ///
    /// A set of steps containing none so marked is a *prefix*, not a completed
    /// flow — which is what makes truncation of a ceremony's record detectable,
    /// since the marker cannot be minted without this step issuer's key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal: Option<bool>,

    /// The steps this one follows.
    ///
    /// A *set* rather than a single predecessor, so a flow with concurrent
    /// branches is expressible and a linear chain is the degenerate case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev: Option<Vec<CeremonyPrev>>,
}

/// A reference to a predecessor step (SPEC.md §4.11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeremonyPrev {
    /// The `id` of the predecessor document — globally unique and never reused,
    /// so a verifier can locate what the digest is over.
    pub id: String,

    /// Multibase-encoded multihash over the predecessor document, salted per
    /// enactment.
    ///
    /// Salted because many steps carry near-zero-entropy payloads, and an
    /// unsalted digest over one is a confirmation oracle for any party handed
    /// it — which a chain does by design, since it passes predecessor digests to
    /// parties not entitled to predecessor content.
    #[serde(rename = "digestMultibase")]
    pub digest_multibase: String,
}

impl Ceremony {
    /// Construct a ceremony member with only the required members populated.
    pub fn new(enactment: impl Into<String>, step: impl Into<String>) -> Self {
        Self {
            enactment: enactment.into(),
            step: step.into(),
            definition: None,
            definition_digest: None,
            parent_enactment: None,
            round: None,
            terminal: None,
            prev: None,
        }
    }

    /// Whether this member is internally well-formed per §4.11.
    ///
    /// Checks only what a single document can establish: that a `definition` is
    /// accompanied by its digest, and that the member does not claim to be its
    /// own parent. It deliberately says nothing about whether the enactment
    /// exists as described — no document can establish that, which is why
    /// §4.11.4 forbids granting authority on membership.
    pub fn is_well_formed(&self) -> bool {
        if self.definition.is_some() != self.definition_digest.is_some() {
            return false;
        }
        if self.parent_enactment.as_deref() == Some(self.enactment.as_str()) {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_requires_a_digest() {
        let mut c = Ceremony::new("urn:uuid:e", "apply");
        assert!(c.is_well_formed());
        c.definition = Some("https://trusttasks.org/ceremony/x/0.1".into());
        assert!(
            !c.is_well_formed(),
            "definition without digest is malformed"
        );
        c.definition_digest = Some("zQmExample".into());
        assert!(c.is_well_formed());
    }

    #[test]
    fn an_enactment_cannot_contain_itself() {
        let mut c = Ceremony::new("urn:uuid:e", "apply");
        c.parent_enactment = Some("urn:uuid:e".into());
        assert!(!c.is_well_formed());
        c.parent_enactment = Some("urn:uuid:outer".into());
        assert!(c.is_well_formed());
    }

    #[test]
    fn optional_members_are_omitted_not_nulled() {
        let json = serde_json::to_string(&Ceremony::new("urn:uuid:e", "apply")).unwrap();
        assert_eq!(json, r#"{"enactment":"urn:uuid:e","step":"apply"}"#);
    }
}
