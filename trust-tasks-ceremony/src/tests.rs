//! Enactment-level conformance fixtures.
//!
//! The design note lists these by name as the thing that had to exist before
//! "strictly verifiable" was more than an aspiration: a truncated receipt, a
//! mismatched `definitionDigest`, a replayed round, an unresolvable `prev`.
//! Each is an attack or a mistake that must not verify, and a checker never
//! shown to fail is not evidence of anything — so the happy path is asserted
//! alongside them, in the same shape.
//!
//! Two forgeries were verifiable before and are not now, and they have the most
//! coverage here: a **recorder-minted terminal marker**, which turned a
//! truncated prefix into `Complete`, and a **duplicated instance**, which raised
//! a `threshold.ofStep` count by listing one party's step twice.
//!
//! Fixtures are built rather than pasted: a step's digest is computed from the
//! document under the enactment salt, so a test cannot accidentally assert
//! against a digest that no document produces.

use super::*;
use serde_json::json;

const DIGEST: &str = "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d";
const ENACTMENT: &str = "urn:uuid:8f21b0c4";
const DEFINITION: &str = "https://trusttasks.org/ceremony/vtc/member-onboarding/0.1";

const APPLICANT: &str = "did:web:applicant.example";
const COMMUNITY: &str = "did:web:community.example";
const APPROVER_ONE: &str = "did:web:approver-one.example";
const APPROVER_TWO: &str = "did:web:approver-two.example";
const STRANGER: &str = "did:web:stranger.example";

const T_APPLY: &str = "https://trusttasks.org/spec/vtc/join-requests/submit/0.1";
const T_STATUS: &str = "https://trusttasks.org/spec/vtc/join-requests/status/0.1";
const T_DECIDE: &str = "https://trusttasks.org/spec/vtc/join-requests/decide/0.1";
const T_ENDORSE: &str = "https://trusttasks.org/spec/vtc/endorsements/issue/0.1";

// --- the definition under test -----------------------------------------

fn definition_value() -> Value {
    json!({
        "slug": "vtc/member-onboarding",
        "version": "0.1",
        "evidence": { "level": "receipt", "recorders": ["community"] },
        "roles": {
            "applicant": { "cardinality": "one" },
            "community": { "cardinality": "one" },
            "approver":  { "cardinality": "many" },
            "oracle":    { "cardinality": "one", "evidentiary": true }
        },
        "steps": {
            "apply":      { "kind": "task", "type": T_APPLY,
                            "issuer": "applicant", "recipient": "community", "prev": [] },
            "supplement": { "kind": "task", "type": T_STATUS,
                            "issuer": "applicant", "recipient": "community",
                            "maxRounds": 3, "optional": true, "prev": ["apply"] },
            "decide":     { "kind": "task", "type": T_DECIDE,
                            "issuer": "community", "recipient": "applicant",
                            "prev": ["apply"], "terminal": true },
            "endorse":    { "kind": "task", "type": T_ENDORSE,
                            "issuer": "approver", "recipient": "community",
                            "multiplicity": "perRole", "prev": ["apply"] }
        },
        "completion": { "allOf": ["apply", "decide"] }
    })
}

fn definition() -> Definition {
    serde_json::from_value(definition_value()).unwrap()
}

/// The definition with a threshold over the instances of one `perRole` step —
/// the governance shape that de-duplication has to protect.
fn threshold_definition() -> Definition {
    let mut v = definition_value();
    v["completion"] = json!({
        "allOf": ["apply", "decide", { "threshold": { "n": 2, "ofStep": "endorse" } }]
    });
    serde_json::from_value(v).unwrap()
}

fn bindings() -> Bindings {
    Bindings::unbound()
        .with("applicant", [APPLICANT])
        .with("community", [COMMUNITY])
        .with("approver", [APPROVER_ONE, APPROVER_TWO])
}

// --- building an enactment ---------------------------------------------

fn salt() -> String {
    multibase::encode(multibase::Base::Base58Btc, [7u8; 16])
}

fn salt_bytes(salt: Option<&str>) -> Vec<u8> {
    decode_salt(salt).unwrap()
}

fn digest_under(doc: &Value, salt: &[u8]) -> String {
    JcsSha256Digester.digest(doc, salt).unwrap()
}

fn doc(id: &str, step: &str, issuer: &str, recipient: &str, type_uri: &str) -> Value {
    json!({
        "id": id,
        "type": type_uri,
        "issuer": issuer,
        "recipient": recipient,
        "ceremony": { "enactment": ENACTMENT, "definitionDigest": DIGEST, "step": step },
        "payload": {}
    })
}

/// The receipt entry a conforming recorder would write for `doc`.
fn entry(doc: &Value, step: &str, salt: &[u8]) -> ReceiptStep {
    ReceiptStep {
        step: step.into(),
        round: doc["ceremony"]
            .get("round")
            .and_then(Value::as_u64)
            .map(|r| r as u32),
        type_uri: doc["type"].as_str().unwrap().into(),
        issuer: doc["issuer"].as_str().unwrap().into(),
        id: doc["id"].as_str().unwrap().into(),
        digest_multibase: digest_under(doc, salt),
        terminal: doc["ceremony"]
            .get("terminal")
            .and_then(Value::as_bool)
            .filter(|t| *t)
            .map(|_| true),
    }
}

/// A receipt and the documents a verifier holds, built together so every digest
/// is the one the document actually produces.
struct Case {
    receipt: Receipt,
    held: BTreeMap<String, Value>,
}

impl Case {
    fn hold(&mut self, doc: Value) {
        self.held
            .insert(doc["id"].as_str().unwrap().to_string(), doc);
    }

    /// Stop holding a document, keeping it enumerated. This is what a verifier
    /// that received only the receipt looks like for that step.
    fn drop_held(&mut self, id: &str) {
        self.held.remove(id);
    }

    fn step_mut(&mut self, name: &str) -> &mut ReceiptStep {
        self.receipt
            .steps
            .iter_mut()
            .find(|s| s.step == name)
            .expect("no such step")
    }
}

/// `apply` → `decide`, chained and terminal: the enactment that should verify.
fn enactment(salt_value: Option<String>) -> Case {
    let bytes = salt_bytes(salt_value.as_deref());

    let apply = doc("urn:uuid:1", "apply", APPLICANT, COMMUNITY, T_APPLY);
    let mut decide = doc("urn:uuid:2", "decide", COMMUNITY, APPLICANT, T_DECIDE);
    decide["ceremony"]["terminal"] = json!(true);
    decide["ceremony"]["prev"] = json!([
        { "id": "urn:uuid:1", "digestMultibase": digest_under(&apply, &bytes) }
    ]);

    let steps = vec![
        entry(&apply, "apply", &bytes),
        entry(&decide, "decide", &bytes),
    ];
    let mut case = Case {
        receipt: Receipt {
            enactment: ENACTMENT.into(),
            parent_enactment: None,
            definition: DEFINITION.into(),
            definition_digest: DIGEST.into(),
            complete: true,
            salt: salt_value,
            steps,
        },
        held: BTreeMap::new(),
    };
    case.hold(apply);
    case.hold(decide);
    case
}

fn base() -> Case {
    enactment(Some(salt()))
}

fn check(c: &Case) -> Outcome {
    check_full(c, &definition(), COMMUNITY, &bindings())
}

fn check_full(c: &Case, def: &Definition, recorder: &str, b: &Bindings) -> Outcome {
    verify(
        &c.receipt,
        def,
        DIGEST,
        recorder,
        b,
        &c.held,
        &JcsSha256Digester,
    )
    .unwrap()
}

// --- control -----------------------------------------------------------

#[test]
fn a_well_formed_enactment_verifies() {
    assert_eq!(
        check(&base()),
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
    let mut c = base();
    c.receipt.steps.retain(|s| s.step != "decide");
    c.drop_held("urn:uuid:2");
    c.receipt.complete = true; // the recorder still claims completion
    assert!(
        matches!(check(&c), Outcome::Incomplete { .. }),
        "a valid prefix must not pass as a completed enactment"
    );
}

#[test]
fn a_mismatched_definition_digest_is_unverifiable_not_invalid() {
    // The distinction matters: the verifier has learned nothing, which is not
    // the same as having learned the receipt is bad.
    let mut c = base();
    c.receipt.definition_digest = "zQmSomeOtherDefinitionDigestEntirely00000000000".into();
    assert!(matches!(check(&c), Outcome::Unverifiable { .. }));
}

#[test]
fn a_replayed_round_beyond_the_bound_is_invalid() {
    // Bounded repetition is what makes the `deferred` supplement cycle
    // expressible; the round index is what stops one round's document being
    // replayed as another's, and the bound is what stops it being replayed
    // indefinitely.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut s = doc("urn:uuid:9", "supplement", APPLICANT, COMMUNITY, T_STATUS);
    s["ceremony"]["round"] = json!(4); // definition permits 3
    c.receipt.steps.push(entry(&s, "supplement", &bytes));
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn a_digest_that_does_not_match_the_held_document_is_invalid() {
    // The verifier holds the document the receipt names, and it is not the one
    // the recorder committed to.
    let mut c = base();
    c.held.get_mut("urn:uuid:1").unwrap()["payload"] = json!({ "tampered": true });
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn holding_no_step_documents_is_unverifiable_not_a_failure() {
    // A verifier that holds nothing still checks the recorder's attestation and
    // the shape of the flow, and learns nothing about step content — which is
    // the correct outcome rather than a degraded one.
    //
    // It is ALSO the case in which the terminal marker is backed by nothing but
    // the recorder's word, so the honest answer is that completion could not be
    // verified. `Complete` here was the old behaviour and was the truncation
    // hole: the recorder both asserted the marker and was the party it exists to
    // catch.
    let mut c = base();
    c.held.clear();
    match check(&c) {
        Outcome::Unverifiable { reason } => assert!(
            reason.contains("terminal"),
            "the reason must name the marker that could not be checked, got: {reason}"
        ),
        other => panic!("expected Unverifiable, got {other:?}"),
    }
}

// --- forgery 1: a recorder-minted terminal marker -----------------------

#[test]
fn terminal_on_a_step_the_definition_does_not_declare_terminal_is_invalid() {
    // The forgery in its bluntest form. A recorder truncates before `decide`
    // and marks the step it stopped at as terminal, so the receipt is a prefix
    // wearing a completion. `terminal` is only meaningful on a step the
    // definition permits to end the enactment.
    let mut c = base();
    c.receipt.steps.retain(|s| s.step != "decide");
    c.drop_held("urn:uuid:2");
    c.step_mut("apply").terminal = Some(true);
    assert!(
        matches!(check(&c), Outcome::Invalid { .. }),
        "a recorder must not be able to nominate a terminal step"
    );
}

#[test]
fn a_terminal_marker_no_held_document_backs_is_unverifiable_not_complete() {
    // The forgery in its subtle form, and the one that used to verify. The
    // receipt names a genuinely terminal step and echoes `terminal: true`, but
    // hands over no document — so the marker is a boolean the recorder typed.
    // The step spec is explicit that a recorder cannot mint it "without the
    // terminal step issuer's key"; a verifier that never sees the key's work
    // has not checked that.
    let mut c = base();
    c.drop_held("urn:uuid:2");
    match check(&c) {
        Outcome::Unverifiable { reason } => assert!(
            reason.contains("terminal") && reason.contains("urn:uuid:2"),
            "the reason must name the document that would have carried the marker, got: {reason}"
        ),
        other => panic!("expected Unverifiable, got {other:?}"),
    }
}

#[test]
fn a_terminal_echo_the_signed_document_contradicts_is_invalid() {
    // The recorder claims the step ended the enactment; the step's own issuer
    // did not say so. The signed content wins.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["ceremony"]
        .as_object_mut()
        .unwrap()
        .remove("terminal");
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.step_mut("decide").terminal = Some(true);
    c.hold(decide);
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn every_terminal_candidate_held_and_unmarked_is_a_prefix() {
    // No contradiction to catch here — the recorder echoed nothing — but the
    // verifier holds every step the definition could have ended on and none of
    // them carries the signed marker. That is a prefix, not an unanswerable
    // question, so it is Incomplete rather than Unverifiable.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["ceremony"]
        .as_object_mut()
        .unwrap()
        .remove("terminal");
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.step_mut("decide").terminal = None;
    c.hold(decide);
    assert!(matches!(check(&c), Outcome::Incomplete { .. }));
}

#[test]
fn a_signed_terminal_marker_on_a_non_terminal_step_is_invalid() {
    // The mirror case: a step issuer marks a step terminal that the definition
    // never permitted to end the flow. The definition is pinned by digest, so
    // this is a contradiction rather than a late amendment.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut apply = c.held.remove("urn:uuid:1").unwrap();
    apply["ceremony"]["terminal"] = json!(true);
    let digest = digest_under(&apply, &bytes);
    c.step_mut("apply").digest_multibase = digest.clone();
    // The receipt echoes nothing, so the only thing asserting a terminal step
    // here is the signed document — which the definition contradicts.
    // `decide` commits to `apply`, so its chain entry moves with the digest.
    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["ceremony"]["prev"] = json!([{ "id": "urn:uuid:1", "digestMultibase": digest }]);
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.hold(apply);
    c.hold(decide);
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

// --- forgery 2: an inflated threshold ----------------------------------

/// `apply`, `decide`, and `n` endorsements, each by the issuer given.
fn endorsed(issuers: &[&str]) -> Case {
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    for (i, issuer) in issuers.iter().enumerate() {
        let e = doc(
            &format!("urn:uuid:e{i}"),
            "endorse",
            issuer,
            COMMUNITY,
            T_ENDORSE,
        );
        c.receipt.steps.push(entry(&e, "endorse", &bytes));
        c.hold(e);
    }
    c
}

#[test]
fn a_threshold_over_instances_is_satisfied_by_distinct_bound_parties() {
    // The control. Two approvers, each bound to the `approver` role, each
    // performing the step once. If this does not verify, the rejection below
    // proves nothing.
    let c = endorsed(&[APPROVER_ONE, APPROVER_TWO]);
    assert_eq!(
        check_full(&c, &threshold_definition(), COMMUNITY, &bindings()),
        Outcome::Complete
    );
}

#[test]
fn one_approvers_step_listed_twice_does_not_satisfy_a_two_of_n_threshold() {
    // The forgery. Two entries, two document `id`s, one approver — enough to
    // push `threshold.ofStep n = 2` over the line while only one party ever
    // decided anything. `perRole` says one instance per bound party, and the
    // signed `issuer` is what discriminates them.
    let c = endorsed(&[APPROVER_ONE, APPROVER_ONE]);
    match check_full(&c, &threshold_definition(), COMMUNITY, &bindings()) {
        Outcome::Invalid { reason } => assert!(
            reason.contains("perRole") || reason.contains("more than once"),
            "the reason must name the repetition, got: {reason}"
        ),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn an_unbound_issuer_cannot_stand_in_for_a_second_approver() {
    // De-duplication alone does not close the threshold: a recorder that cannot
    // list one approver twice can invent a second. Only the role bindings say
    // whether an issuer was a party the definition authorised.
    let c = endorsed(&[APPROVER_ONE, STRANGER]);
    assert!(matches!(
        check_full(&c, &threshold_definition(), COMMUNITY, &bindings()),
        Outcome::Invalid { .. }
    ));
}

#[test]
fn the_same_document_enumerated_twice_is_invalid() {
    // An `id` is globally unique and never reused (SPEC §4.3), so two entries
    // sharing one is one step counted twice however the rest is dressed.
    let mut c = base();
    let dup = c.receipt.steps[0].clone();
    c.receipt.steps.push(dup);
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn a_single_multiplicity_step_enumerated_twice_at_one_round_is_invalid() {
    // `apply` is neither `perRole` nor repeatable, so a second entry for it at
    // round 1 is the same instance again — a different `id` and a different
    // issuer do not make it two applications.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let again = doc("urn:uuid:1b", "apply", APPLICANT, COMMUNITY, T_APPLY);
    c.receipt.steps.push(entry(&again, "apply", &bytes));
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn a_repeatable_step_at_distinct_rounds_is_not_a_duplicate() {
    // De-duplication must not break the shape it shares a key with: bounded
    // repetition is two instances of one step by one party, discriminated by
    // the signed round.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    for round in 1..=2 {
        let mut s = doc(
            &format!("urn:uuid:s{round}"),
            "supplement",
            APPLICANT,
            COMMUNITY,
            T_STATUS,
        );
        s["ceremony"]["round"] = json!(round);
        c.receipt.steps.push(entry(&s, "supplement", &bytes));
        c.hold(s);
    }
    assert_eq!(check(&c), Outcome::Complete);
}

// --- role bindings ------------------------------------------------------

#[test]
fn a_step_issuer_the_enactment_did_not_bind_is_invalid() {
    // §7.9 step 5. Without this the receipt says which VID signed a step and
    // nothing says whether that VID was allowed to.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut apply = c.held.remove("urn:uuid:1").unwrap();
    apply["issuer"] = json!(STRANGER);
    let digest = digest_under(&apply, &bytes);
    c.step_mut("apply").issuer = STRANGER.into();
    c.step_mut("apply").digest_multibase = digest.clone();
    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["ceremony"]["prev"] = json!([{ "id": "urn:uuid:1", "digestMultibase": digest }]);
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.hold(apply);
    c.hold(decide);
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn a_document_addressed_outside_the_recipient_role_is_invalid() {
    // The other half of §7.9 step 5, checkable only from a held document: the
    // receipt does not enumerate a recipient.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["recipient"] = json!(STRANGER);
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.hold(decide);
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn unsupplied_role_bindings_are_reported_not_assumed() {
    // A verifier without the enactment's bindings cannot answer either of the
    // two role questions the specification makes a MUST. Saying so is the
    // honest outcome; passing them silently is how the recorder check became a
    // comparison between a VID and a role name.
    match check_full(&base(), &definition(), COMMUNITY, &Bindings::unbound()) {
        Outcome::Unverifiable { reason } => assert!(
            reason.contains("recorder") && reason.contains("binding"),
            "the reason must name what could not be resolved, got: {reason}"
        ),
        other => panic!("expected Unverifiable, got {other:?}"),
    }
}

// --- recorder, declaration, and definition agreement -------------------

#[test]
fn a_recorder_not_bound_to_a_recorder_role_is_rejected() {
    assert!(matches!(
        check_full(&base(), &definition(), STRANGER, &bindings()),
        Outcome::Invalid { .. }
    ));
}

#[test]
fn the_recorders_own_complete_flag_is_never_trusted() {
    // `complete: false` on an enactment that satisfies the rule, and
    // `complete: true` on one that does not, must both be decided by the rule.
    let mut honest_but_wrong = base();
    honest_but_wrong.receipt.complete = false;
    assert_eq!(
        check(&honest_but_wrong),
        Outcome::Complete,
        "a recorder that under-claims does not make a complete enactment incomplete"
    );

    let mut lying = base();
    lying.receipt.steps.retain(|s| s.step != "apply");
    lying.held.clear();
    lying.receipt.complete = true;
    assert!(
        matches!(check(&lying), Outcome::Incomplete { .. }),
        "a recorder that over-claims does not make an incomplete enactment complete"
    );
}

#[test]
fn a_step_enacting_the_wrong_type_uri_is_invalid() {
    let mut c = base();
    c.held.clear();
    c.receipt.steps[0].type_uri = "https://trusttasks.org/spec/acl/grant/0.1".into();
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn a_step_the_definition_does_not_declare_is_invalid() {
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let smuggled = doc(
        "urn:uuid:99",
        "smuggled",
        APPLICANT,
        COMMUNITY,
        "https://example.org/spec/x/0.1",
    );
    c.receipt.steps.push(entry(&smuggled, "smuggled", &bytes));
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

// --- the chain (§7.9 step 4) -------------------------------------------

#[test]
fn a_predecessor_the_receipt_does_not_enumerate_is_invalid() {
    // The omission the chain exists to catch, and the claim the receipt
    // specification makes at "its successor committed to its digest". A recorder
    // drops an intermediate step; the successor's signed `prev` still names it.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut supplement = doc("urn:uuid:s1", "supplement", APPLICANT, COMMUNITY, T_STATUS);
    supplement["ceremony"]["prev"] =
        json!([{ "id": "urn:uuid:1", "digestMultibase": c.receipt.steps[0].digest_multibase }]);

    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["ceremony"]["prev"] = json!([
        { "id": "urn:uuid:s1", "digestMultibase": digest_under(&supplement, &bytes) }
    ]);
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.hold(decide);
    // `supplement` happened and is committed to, but the recorder left it out.
    match check(&c) {
        Outcome::Invalid { reason } => assert!(
            reason.contains("urn:uuid:s1"),
            "the reason must name the omitted step, got: {reason}"
        ),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn a_prev_digest_the_receipt_contradicts_is_invalid() {
    // The recorder enumerates the predecessor, but with a digest other than the
    // one its successor signed a commitment to.
    let mut c = base();
    c.step_mut("apply").digest_multibase = "zQmNotTheDigestTheSuccessorCommittedTo0000000".into();
    c.drop_held("urn:uuid:1"); // so the mismatch is the chain's, not the document's
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn a_prev_naming_a_step_that_is_not_an_ancestor_is_invalid() {
    // The digests can all agree and the ordering still be a fiction. The
    // definition's `prev` graph says `decide` follows `apply`; a document
    // claiming the reverse is not an enactment of this definition.
    // Built from scratch rather than mutated, because two documents that each
    // commit to the other cannot both hold a valid digest — and the point here
    // is an ordering that is false while every digest agrees.
    let salt_value = salt();
    let bytes = salt_bytes(Some(&salt_value));

    let mut decide = doc("urn:uuid:2", "decide", COMMUNITY, APPLICANT, T_DECIDE);
    decide["ceremony"]["terminal"] = json!(true);
    let mut apply = doc("urn:uuid:1", "apply", APPLICANT, COMMUNITY, T_APPLY);
    apply["ceremony"]["prev"] = json!([
        { "id": "urn:uuid:2", "digestMultibase": digest_under(&decide, &bytes) }
    ]);

    let mut c = Case {
        receipt: Receipt {
            enactment: ENACTMENT.into(),
            parent_enactment: None,
            definition: DEFINITION.into(),
            definition_digest: DIGEST.into(),
            complete: true,
            salt: Some(salt_value),
            steps: vec![
                entry(&apply, "apply", &bytes),
                entry(&decide, "decide", &bytes),
            ],
        },
        held: BTreeMap::new(),
    };
    c.hold(apply);
    c.hold(decide);
    match check(&c) {
        Outcome::Invalid { reason } => assert!(
            reason.contains("ancestor"),
            "the reason must name the ordering, got: {reason}"
        ),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn a_chain_that_skips_an_optional_step_still_verifies() {
    // Ancestry is transitive on purpose. `supplement` is optional, so an
    // enactment that skips it chains `decide` straight to `apply` — which is
    // the base case, and would be rejected by a direct-predecessor rule the
    // moment a definition inserted an optional step between them.
    assert_eq!(check(&base()), Outcome::Complete);
}

#[test]
fn a_receipt_carrying_prev_without_the_salt_is_invalid() {
    // Recorder conformance rule 5. Without the salt the chain cannot be
    // recomputed, and the ordering claim rests on nothing.
    let mut c = enactment(None);
    // The digests above were computed under the empty salt, so the failure
    // isolated here is the missing salt rather than a digest mismatch.
    assert_eq!(
        check(&c),
        Outcome::Invalid {
            reason: "a step carries ceremony.prev but the receipt omits the enactment salt, so the chain cannot be recomputed".into()
        }
    );
    c.receipt.salt = Some(multibase::encode(multibase::Base::Base58Btc, [0u8; 16]));
    assert!(
        matches!(check(&c), Outcome::Invalid { .. }),
        "and a salt that is not the one the digests were computed under still fails"
    );
}

// --- envelope agreement (§7.9 step 1) ----------------------------------

#[test]
fn a_held_document_from_another_enactment_is_invalid() {
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["ceremony"]["enactment"] = json!("urn:uuid:some-other-run");
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.hold(decide);
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn a_held_document_pinning_another_definition_is_invalid() {
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["ceremony"]["definitionDigest"] = json!("zQmAnotherDefinitionEntirely000000000000000");
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.hold(decide);
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

#[test]
fn a_document_signed_by_someone_other_than_the_enumerated_issuer_is_invalid() {
    // The receipt attributes a step to a party the document itself does not
    // name. Only checkable with the document in hand, and worth checking: the
    // issuer is what discriminates `perRole` instances.
    let mut c = base();
    let bytes = salt_bytes(c.receipt.salt.as_deref());
    let mut decide = c.held.remove("urn:uuid:2").unwrap();
    decide["issuer"] = json!(APPLICANT);
    c.step_mut("decide").digest_multibase = digest_under(&decide, &bytes);
    c.hold(decide);
    assert!(matches!(check(&c), Outcome::Invalid { .. }));
}

// --- nested ceremonies (§7.9 step 6, not implemented) ------------------

#[test]
fn a_nested_ceremony_step_is_reported_unverifiable() {
    // The child's evidence is its own receipt, verified on its own terms
    // against its own pinned definition. This crate does not recurse, and a
    // receipt containing such a step is therefore not fully checked — which is
    // reported rather than passed over as Complete.
    let mut v = definition_value();
    v["steps"]["endorse"] = json!({
        "kind": "ceremony",
        "ceremony": "https://trusttasks.org/ceremony/vtc/endorsement/0.1",
        "ceremonyDigest": DIGEST,
        "roleMap": { "approver": "endorser" },
        "issuer": "approver",
        "recipient": "community",
        "prev": ["apply"]
    });
    let def: Definition = serde_json::from_value(v).unwrap();

    let c = endorsed(&[APPROVER_ONE]);
    match check_full(&c, &def, COMMUNITY, &bindings()) {
        Outcome::Unverifiable { reason } => assert!(
            reason.contains("nests a ceremony"),
            "the reason must name the nesting, got: {reason}"
        ),
        other => panic!("expected Unverifiable, got {other:?}"),
    }
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

// --- the definition's own shape ----------------------------------------

#[test]
fn the_published_definition_deserializes_with_every_member_verification_needs() {
    // The registry's own definition, not a hand-rolled one: `roles`, `issuer`,
    // `recipient`, `multiplicity` and `prev` went unparsed before, so the
    // verifier could not have checked them however it was written.
    let src = include_str!("../../ceremonies/vtc/member-onboarding/0.1/ceremony.json");
    let def: Definition = serde_json::from_str(src).unwrap();
    assert_eq!(def.slug, "vtc/member-onboarding");
    assert!(def.roles.contains_key("applicant"));
    let decide = &def.steps["decide"];
    assert_eq!(decide.issuer.as_deref(), Some("administrator"));
    assert_eq!(decide.recipient.as_deref(), Some("community"));
    assert_eq!(decide.kind, StepKind::Task);
    assert!(decide.terminal);
    assert_eq!(decide.prev, vec!["apply".to_string()]);
    assert_eq!(def.steps["supplement"].multiplicity, Multiplicity::Single);
    assert!(is_ancestor(&def, "apply", "reciprocate"));
    assert!(!is_ancestor(&def, "reciprocate", "apply"));
}
