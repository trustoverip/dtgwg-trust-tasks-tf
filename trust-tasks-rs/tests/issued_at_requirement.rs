//! SPEC.md §7.3 item 17 — a *Trust Task specification* defining a
//! *consequential Trust Task* requires the `issuedAt` member, raising §4.2's
//! **SHOULD** to a **MUST** for documents conforming to it.
//!
//! The obligation reaches a consumer as [`Payload::IS_ISSUED_AT_REQUIRED`],
//! emitted by the codegen from the spec's `issuedAtRequirement` front matter
//! and consulted by [`TrustTask::enforce_spec_policy`] — the same route
//! `IS_PROOF_REQUIRED` and `IS_RECIPIENT_REQUIRED` already take.
//!
//! No published spec declares `issuedAtRequirement` yet (that is a follow-up,
//! spec by spec), so the generated types cannot exercise the `true` case. The
//! hand-rolled impls below stand in for what a declaring spec will generate,
//! exactly as `audience_binding.rs` does for `IS_BEARER`.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use trust_tasks_rs::{
    specs::trust_task_discovery::v0_1 as discovery, Payload, RejectReason, StandardCode, TrustTask,
};

#[derive(Serialize, Deserialize)]
struct RotateKey {
    did: String,
}

/// What a spec declaring `issuedAtRequirement: {requirement: REQUIRED}`
/// generates. Nothing else is overridden — the point is that this one
/// constant is enough.
impl Payload for RotateKey {
    const TYPE_URI: &'static str = "https://example.com/spec/rotate-key/0.1";
    const IS_ISSUED_AT_REQUIRED: bool = true;
}

#[derive(Serialize, Deserialize)]
struct ReadStatus {
    did: String,
}

/// A hand-written impl that names none of the policy constants — the case the
/// trait default exists for. `trust-task-error` is hand-modelled this way in
/// `error.rs`, and adding a defaultless constant to the trait would have
/// broken it and every downstream impl. That this compiles is the test.
impl Payload for ReadStatus {
    const TYPE_URI: &'static str = "https://example.com/spec/read-status/0.1";
}

fn rotate(issued: bool) -> TrustTask<RotateKey> {
    let mut doc = TrustTask::for_payload(
        "rot-1",
        RotateKey {
            did: "did:web:alice.example".into(),
        },
    );
    doc.issuer = Some("did:web:alice.example".into());
    doc.recipient = Some("did:web:registrar.example".into());
    if issued {
        doc.issued_at = Some(Utc::now());
    }
    doc
}

/// The rejection is `malformedRequest`, not `expired`: §8.3 defines no
/// dedicated code, and `expired` names a document that was once acceptable —
/// which one that could never be placed in a window never was. It is the same
/// code §7.2 item 13 already uses for the other freshness rejections.
#[test]
fn missing_issued_at_is_rejected_as_malformed_when_the_spec_requires_it() {
    let err = rotate(false)
        .enforce_spec_policy()
        .expect_err("a spec declaring issuedAt REQUIRED must reject a document without one");

    match &err {
        RejectReason::MalformedRequest { reason } => {
            assert_eq!(
                reason,
                trust_tasks_rs::freshness::ISSUED_AT_REQUIRED_BY_SPEC
            );
        }
        other => panic!("expected MalformedRequest, got {other:?}"),
    }
    assert_eq!(err.code(), StandardCode::MalformedRequest);
}

#[test]
fn present_issued_at_satisfies_the_requirement() {
    rotate(true)
        .enforce_spec_policy()
        .expect("a document carrying issuedAt passes");
}

/// The default is `false`, so a hand-written impl that says nothing about
/// freshness behaves exactly as it did before the constant existed.
#[test]
fn a_spec_that_declares_nothing_still_accepts_a_document_without_issued_at() {
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(!<ReadStatus as Payload>::IS_ISSUED_AT_REQUIRED);
    }

    let mut doc = TrustTask::for_payload(
        "read-1",
        ReadStatus {
            did: "did:web:alice.example".into(),
        },
    );
    doc.issuer = Some("did:web:alice.example".into());
    doc.enforce_spec_policy()
        .expect("no declaration means the §4.2 SHOULD applies, and a consumer may accept");
}

/// The additive guarantee for this change: no published spec declares
/// `issuedAtRequirement`, so every generated type keeps the trait default and
/// no document that was accepted before is rejected now.
#[test]
fn no_generated_spec_requires_issued_at_yet() {
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(!<discovery::Payload as Payload>::IS_ISSUED_AT_REQUIRED);
        assert!(!<discovery::Response as Payload>::IS_ISSUED_AT_REQUIRED);
    }
}
