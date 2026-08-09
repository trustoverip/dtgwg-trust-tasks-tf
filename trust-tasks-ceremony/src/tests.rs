//! Enactment-level conformance fixtures.
//!
//! The design note lists these by name as the thing that had to exist before
//! "strictly verifiable" was more than an aspiration: a truncated receipt, a
//! mismatched `definitionDigest`, a replayed round, an unresolvable `prev`.
//! Each is an attack or a mistake that must not verify, and a checker never
//! shown to fail is not evidence of anything — so the happy path is asserted
//! alongside them, in the same shape.

use super::*;
use serde_json::json;

const DIGEST: &str = "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d";

fn definition() -> Definition {
    serde_json::from_value(json!({
        "slug": "vtc/member-onboarding",
        "version": "0.1",
        "evidence": { "level": "receipt", "recorders": ["community"] },
        "steps": {
            "apply":       { "type": "https://trusttasks.org/spec/vtc/join-requests/submit/0.1" },
            "supplement":  { "type": "https://trusttasks.org/spec/vtc/join-requests/status/0.1",
                             "maxRounds": 3 },
            "decide":      { "type": "https://trusttasks.org/spec/vtc/join-requests/decide/0.1",
                             "terminal": true },
            "endorse":     { "type": "https://trusttasks.org/spec/vtc/endorsements/issue/0.1" }
        },
        "completion": { "allOf": ["apply", "decide"] }
    }))
    .unwrap()
}

fn step(name: &str, id: &str, type_uri: &str, terminal: bool) -> ReceiptStep {
    ReceiptStep {
        step: name.into(),
        round: None,
        type_uri: type_uri.into(),
        issuer: "did:web:applicant.example".into(),
        id: id.into(),
        digest_multibase: DIGEST.into(),
        terminal: Some(terminal),
    }
}

fn receipt() -> Receipt {
    Receipt {
        enactment: "urn:uuid:8f21b0c4".into(),
        parent_enactment: None,
        definition: "https://trusttasks.org/ceremony/vtc/member-onboarding/0.1".into(),
        definition_digest: DIGEST.into(),
        complete: true,
        salt: None,
        steps: vec![
            step(
                "apply",
                "urn:uuid:1",
                "https://trusttasks.org/spec/vtc/join-requests/submit/0.1",
                false,
            ),
            step(
                "decide",
                "urn:uuid:2",
                "https://trusttasks.org/spec/vtc/join-requests/decide/0.1",
                true,
            ),
        ],
    }
}

fn check(r: &Receipt, recorder: &str, held: &BTreeMap<String, Value>) -> Outcome {
    verify(r, &definition(), DIGEST, recorder, held, &JcsSha256Digester).unwrap()
}

// --- control -----------------------------------------------------------

#[test]
fn a_well_formed_enactment_verifies() {
    assert_eq!(
        check(&receipt(), "community", &BTreeMap::new()),
        Outcome::Complete,
        "if this fails, every rejection below proves nothing"
    );
}

// --- the four the design note named ------------------------------------

#[test]
fn a_truncated_receipt_is_a_prefix_not_a_completion() {
    // The attack the terminal marker exists for: a recorder stops the record
    // just before the step that would have changed the outcome. Every remaining
    // link still chains, because detection depends on a successor and the
    // dropped step has none.
    let mut r = receipt();
    r.steps.retain(|s| s.step != "decide");
    r.complete = true; // the recorder still claims completion
    assert!(
        matches!(
            check(&r, "community", &BTreeMap::new()),
            Outcome::Incomplete { .. }
        ),
        "a valid prefix must not pass as a completed enactment"
    );
}

#[test]
fn a_mismatched_definition_digest_is_unverifiable_not_invalid() {
    // The distinction matters: the verifier has learned nothing, which is not
    // the same as having learned the receipt is bad.
    let mut r = receipt();
    r.definition_digest = "zQmSomeOtherDefinitionDigestEntirely00000000000".into();
    assert!(matches!(
        check(&r, "community", &BTreeMap::new()),
        Outcome::Unverifiable { .. }
    ));
}

#[test]
fn a_replayed_round_beyond_the_bound_is_invalid() {
    // Bounded repetition is what makes the `deferred` supplement cycle
    // expressible; the round index is what stops one round's document being
    // replayed as another's, and the bound is what stops it being replayed
    // indefinitely.
    let mut r = receipt();
    let mut s = step(
        "supplement",
        "urn:uuid:9",
        "https://trusttasks.org/spec/vtc/join-requests/status/0.1",
        false,
    );
    s.round = Some(4); // definition permits 3
    r.steps.push(s);
    assert!(matches!(
        check(&r, "community", &BTreeMap::new()),
        Outcome::Invalid { .. }
    ));
}

#[test]
fn a_digest_that_does_not_match_the_held_document_is_invalid() {
    // The "unresolvable prev" case in its checkable form: the verifier holds
    // the document the receipt names, and it is not the one the recorder
    // committed to.
    let mut held = BTreeMap::new();
    held.insert(
        "urn:uuid:1".to_string(),
        json!({"id": "urn:uuid:1", "payload": {"tampered": true}}),
    );
    assert!(matches!(
        check(&receipt(), "community", &held),
        Outcome::Invalid { .. }
    ));
}

#[test]
fn holding_no_step_documents_is_not_a_failure() {
    // A verifier that holds nothing still checks the recorder's attestation and
    // the shape of the flow. It learns nothing about step content, which is the
    // correct outcome rather than a degraded one.
    assert_eq!(
        check(&receipt(), "community", &BTreeMap::new()),
        Outcome::Complete
    );
}

// --- recorder, declaration, and definition agreement -------------------

#[test]
fn a_recorder_the_definition_does_not_name_is_rejected() {
    assert!(matches!(
        check(&receipt(), "did:web:stranger.example", &BTreeMap::new()),
        Outcome::Invalid { .. }
    ));
}

#[test]
fn the_recorders_own_complete_flag_is_never_trusted() {
    // `complete: false` on an enactment that satisfies the rule, and
    // `complete: true` on one that does not, must both be decided by the rule.
    let mut honest_but_wrong = receipt();
    honest_but_wrong.complete = false;
    assert_eq!(
        check(&honest_but_wrong, "community", &BTreeMap::new()),
        Outcome::Complete,
        "a recorder that under-claims does not make a complete enactment incomplete"
    );

    let mut lying = receipt();
    lying.steps.retain(|s| s.step != "apply");
    lying.complete = true;
    assert!(
        matches!(
            check(&lying, "community", &BTreeMap::new()),
            Outcome::Incomplete { .. }
        ),
        "a recorder that over-claims does not make an incomplete enactment complete"
    );
}

#[test]
fn a_step_enacting_the_wrong_type_uri_is_invalid() {
    let mut r = receipt();
    r.steps[0].type_uri = "https://trusttasks.org/spec/acl/grant/0.1".into();
    assert!(matches!(
        check(&r, "community", &BTreeMap::new()),
        Outcome::Invalid { .. }
    ));
}

#[test]
fn a_step_the_definition_does_not_declare_is_invalid() {
    let mut r = receipt();
    r.steps.push(step(
        "smuggled",
        "urn:uuid:99",
        "https://example.org/spec/x/0.1",
        false,
    ));
    assert!(matches!(
        check(&r, "community", &BTreeMap::new()),
        Outcome::Invalid { .. }
    ));
}

// --- completion predicates ---------------------------------------------

fn counts(pairs: &[(&str, usize)]) -> BTreeMap<String, usize> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn threshold_over_instances_counts_one_step_many_times() {
    // task-consent's minApprovals: N approvers each perform ONE decision, and
    // which approvers they are is not known when the definition is written.
    let p: Predicate =
        serde_json::from_value(json!({"threshold": {"n": 3, "ofStep": "endorse"}})).unwrap();
    assert!(!p.is_satisfied_by(&counts(&[("endorse", 2)])));
    assert!(p.is_satisfied_by(&counts(&[("endorse", 3)])));
    assert!(p.is_satisfied_by(&counts(&[("endorse", 5)])));
}

#[test]
fn threshold_over_distinct_steps_counts_each_step_once() {
    // "any 3 of these 5 endorsements" — five different things, not one thing
    // five times. Ten instances of one step must not satisfy it.
    let p: Predicate =
        serde_json::from_value(json!({"threshold": {"n": 3, "of": ["a", "b", "c", "d", "e"]}}))
            .unwrap();
    assert!(!p.is_satisfied_by(&counts(&[("a", 10)])));
    assert!(!p.is_satisfied_by(&counts(&[("a", 1), ("b", 1)])));
    assert!(p.is_satisfied_by(&counts(&[("a", 1), ("b", 1), ("c", 1)])));
}

#[test]
fn nested_predicates_compose() {
    let p: Predicate = serde_json::from_value(json!({
        "allOf": ["apply", {"anyOf": ["reciprocate", "decline"]}]
    }))
    .unwrap();
    assert!(!p.is_satisfied_by(&counts(&[("apply", 1)])));
    assert!(p.is_satisfied_by(&counts(&[("apply", 1), ("decline", 1)])));
}

#[test]
fn a_threshold_declaring_neither_shape_is_not_satisfied() {
    // The definition schema forbids this; a hand-built Predicate can reach it,
    // and an unanswerable predicate must not default to true.
    let p = Predicate::Threshold {
        threshold: Threshold {
            n: 1,
            of: None,
            of_step: None,
        },
    };
    assert!(!p.is_satisfied_by(&counts(&[("anything", 9)])));
}

#[test]
fn referenced_steps_finds_names_inside_thresholds() {
    let p: Predicate = serde_json::from_value(json!({
        "allOf": ["apply", {"threshold": {"n": 2, "ofStep": "endorse"}}]
    }))
    .unwrap();
    let mut found = BTreeSet::new();
    p.referenced_steps(&mut found);
    assert_eq!(
        found,
        ["apply", "endorse"]
            .iter()
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>()
    );
}
