//! SPEC.md §7.2 item 11 and §8.4, end to end through `consume_inbound`.
//!
//! The unit tests in `src/replay.rs` pin the guard's own behaviour. These pin
//! the property that actually matters to a deployment: that the *pipeline*
//! declines to call the handler a second time, because the handler is where
//! the consequential effect lives. A guard that answers `Duplicate` correctly
//! while `consume_inbound` executes anyway would pass every unit test and
//! grant the ACL entry twice.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use chrono::{DateTime, TimeDelta, Utc};
use trust_tasks_rs::handlers::NoopHandler;
use trust_tasks_rs::specs::acl::grant::v0_1 as grant;
use trust_tasks_rs::{
    consume_inbound, ConsumeChecks, ConsumeOutcome, ErrorResponse, FreshnessPolicy,
    InMemoryReplayGuard, NoValidator, PayloadPolicy, Proof, ProofPolicy, ProofVerifier,
    ReplayGuard, ReplayPolicy, StandardCode, TrustTask, VerificationError,
};

/// Pins the `V` type parameter on the `AcceptUnverified` path. These tests are
/// about item 11, not item 7 — the documents carry a `proof` only because
/// `acl/grant` declares one REQUIRED.
struct NoVerifier;

#[async_trait::async_trait]
impl ProofVerifier for NoVerifier {
    async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: serde::Serialize + Send + Sync,
    {
        Ok(())
    }
}

const ME: &str = "did:web:maintainer.example";
const THEM: &str = "did:web:org.example";

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

fn entry(role: &str) -> grant::AclEntry {
    grant::AclEntry {
        allowed_keys: None,
        subject: "did:web:alice.example".into(),
        role: role.into(),
        scopes: vec![],
        label: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        expires_at: None,
        approve: None,
        step_up: None,
        ext: None,
    }
}

fn proof() -> Proof {
    Proof {
        proof_type: "DataIntegrityProof".into(),
        cryptosuite: "eddsa-jcs-2022".into(),
        verification_method: "did:web:org.example#key-1".into(),
        created: t("2026-08-26T12:00:00Z"),
        proof_purpose: "assertionMethod".into(),
        proof_value: "zAAA".into(),
        extra: Default::default(),
    }
}

/// `acl/grant` is `IS_PROOF_REQUIRED`, so every document here carries one.
fn request(id: &str, role: &str, issued_at: DateTime<Utc>) -> TrustTask<grant::Payload> {
    let mut doc = TrustTask::for_payload(
        id,
        grant::Payload {
            entry: entry(role),
            reason: None,
            ext: None,
        },
    );
    doc.issuer = Some(THEM.into());
    doc.recipient = Some(ME.into());
    doc.issued_at = Some(issued_at);
    doc.proof = Some(proof());
    doc
}

/// Counts handler invocations. The count is the assertion: item 11 is a
/// statement about how many times the consequential effect happens.
#[derive(Default)]
struct Executions(AtomicUsize);

impl Executions {
    fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

async fn consume(
    guard: &dyn ReplayGuard,
    executions: &Executions,
    doc: TrustTask<grant::Payload>,
    now: DateTime<Utc>,
) -> ConsumeOutcome<grant::Response> {
    consume_inbound(
        &NoopHandler::new(),
        ProofPolicy::<NoVerifier>::AcceptUnverified,
        PayloadPolicy::<NoValidator>::AcceptUnvalidated,
        ConsumeChecks::consequential(guard),
        doc,
        ME,
        now,
        || "err-1".to_string(),
        |req, _parties| async move {
            executions.0.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ErrorResponse>(req.respond_with(
                "resp-1",
                grant::Response {
                    entry: entry("admin"),
                    ext: None,
                },
            ))
        },
    )
    .await
}

/// §8.4: "retrying" is re-sending the document bit-for-bit. §7.2 item 11
/// obliges the consumer to absorb it. The handler runs exactly once.
#[tokio::test]
async fn an_identical_resend_is_absorbed_and_never_executes_twice() {
    let guard = InMemoryReplayGuard::new(64);
    let executions = Executions::default();
    let now = t("2026-08-26T12:00:00Z");
    let doc = request("req-1", "admin", now);

    let first = consume(&guard, &executions, doc.clone(), now).await;
    assert!(matches!(first, ConsumeOutcome::Handled(_)), "{first:?}");
    assert_eq!(executions.count(), 1);

    // The mediator redelivers. Same bytes, same `id`.
    let second = consume(&guard, &executions, doc.clone(), now).await;
    match second {
        ConsumeOutcome::Duplicate {
            prior_response,
            in_flight,
        } => {
            assert!(!in_flight, "the first execution had completed");
            let prior = prior_response.expect("the first response should be retained");
            assert_eq!(prior["id"], "resp-1");
            assert_eq!(prior["recipient"], THEM);
        }
        other => panic!("expected Duplicate, got {other:?}"),
    }
    assert_eq!(
        executions.count(),
        1,
        "the consequential effect happened twice"
    );

    // …and again, arbitrarily often.
    let third = consume(&guard, &executions, doc, now).await;
    assert!(
        matches!(third, ConsumeOutcome::Duplicate { .. }),
        "{third:?}"
    );
    assert_eq!(executions.count(), 1);
}

/// §7.2 item 11: a document whose `id` matches one already accepted but whose
/// content differs is `idConflict`, and MUST NOT be treated as a retry.
#[tokio::test]
async fn differing_content_under_a_reused_id_is_an_id_conflict() {
    let guard = InMemoryReplayGuard::new(64);
    let executions = Executions::default();
    let now = t("2026-08-26T12:00:00Z");

    let first = consume(&guard, &executions, request("req-1", "reader", now), now).await;
    assert!(matches!(first, ConsumeOutcome::Handled(_)), "{first:?}");

    // Same `id`, escalated role. This is the attack the rule is for.
    let second = consume(&guard, &executions, request("req-1", "admin", now), now).await;
    match second {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::IdConflict.into());
            assert!(!err.payload.retryable);
        }
        other => panic!("expected Rejected(idConflict), got {other:?}"),
    }
    assert_eq!(executions.count(), 1, "the escalated document executed");
}

/// §8.4 again, from the other side: a *re-signed* proof over identical content
/// is a different document under a reused `id`, not a retry. Absorbing it
/// silently would be the one disposition both §7.2 item 11 and §4.9.3 rule
/// out — and it is why the item-11 digest covers `proof`, unlike the §4.9.3
/// task digest.
#[tokio::test]
async fn a_re_signed_proof_over_identical_content_is_an_id_conflict() {
    let guard = InMemoryReplayGuard::new(64);
    let executions = Executions::default();
    let now = t("2026-08-26T12:00:00Z");

    let doc = request("req-1", "admin", now);
    consume(&guard, &executions, doc.clone(), now).await;

    let mut resigned = doc;
    resigned.proof.as_mut().unwrap().proof_value = "zBBB".into();
    match consume(&guard, &executions, resigned, now).await {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::IdConflict.into())
        }
        other => panic!("expected Rejected(idConflict), got {other:?}"),
    }
    assert_eq!(executions.count(), 1);
}

/// §7.2 (*Bounding the record*): retention and willingness-to-execute are the
/// same bound. Past it the record is released — and the document is refused
/// under item 4 anyway, so releasing the key cannot enable a replay.
#[tokio::test]
async fn the_record_is_released_when_the_acceptance_window_closes() {
    let guard = InMemoryReplayGuard::new(64);
    let executions = Executions::default();
    let issued = t("2026-08-26T12:00:00Z");
    let doc = request("req-1", "admin", issued);

    consume(&guard, &executions, doc.clone(), issued).await;
    assert_eq!(guard.len(), 1);

    // `consequential()` uses a five-minute window; the record's retention
    // deadline is `issuedAt + 5m`.
    guard.purge_expired(t("2026-08-26T12:06:00Z"));
    assert_eq!(guard.len(), 0, "the record outlived its bound");

    // The same document arriving after the window is refused on freshness,
    // never reaching the (now empty) record. The handler still runs once in
    // total.
    let late = consume(&guard, &executions, doc, t("2026-08-26T12:10:00Z")).await;
    match late {
        ConsumeOutcome::Rejected(err) => assert_eq!(err.payload.code, StandardCode::Expired.into()),
        other => panic!("expected Rejected(expired), got {other:?}"),
    }
    assert_eq!(executions.count(), 1);
}

/// A retryable refusal releases the claim: §8.4 has just told the producer it
/// MAY re-send this document bit-for-bit, and holding the claim would answer
/// that invited retry with the cached failure forever.
#[tokio::test]
async fn a_retryable_refusal_releases_the_claim_so_the_invited_retry_can_run() {
    let guard = InMemoryReplayGuard::new(64);
    let now = t("2026-08-26T12:00:00Z");
    let doc = request("req-1", "admin", now);
    let attempts = AtomicUsize::new(0);
    let attempts = &attempts;

    let run = |doc: TrustTask<grant::Payload>| async {
        consume_inbound(
            &NoopHandler::new(),
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::consequential(&guard),
            doc,
            ME,
            now,
            || "err-1".to_string(),
            |req: TrustTask<grant::Payload>, _parties| async move {
                // Fail the first attempt with a retryable code, succeed after.
                if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                    Err(req.reject_with(
                        "err-1",
                        trust_tasks_rs::RejectReason::Unavailable { retry_after: None },
                    ))
                } else {
                    Ok(req.respond_with(
                        "resp-1",
                        grant::Response {
                            entry: entry("admin"),
                            ext: None,
                        },
                    ))
                }
            },
        )
        .await
    };

    let first: ConsumeOutcome<grant::Response> = run(doc.clone()).await;
    match first {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::Unavailable.into());
            assert!(err.payload.retryable, "unavailable is retryable per §8.3");
        }
        other => panic!("expected Rejected(unavailable), got {other:?}"),
    }

    // The invited bit-for-bit retry reaches the handler rather than the cache.
    let second: ConsumeOutcome<grant::Response> = run(doc).await;
    assert!(matches!(second, ConsumeOutcome::Handled(_)), "{second:?}");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

/// A *non*-retryable refusal is final for that document, so the record stands
/// and a replay is answered with the same determination rather than
/// re-evaluated.
#[tokio::test]
async fn a_non_retryable_refusal_keeps_the_claim() {
    let guard = InMemoryReplayGuard::new(64);
    let now = t("2026-08-26T12:00:00Z");
    let doc = request("req-1", "admin", now);
    let attempts = AtomicUsize::new(0);
    let attempts = &attempts;

    let run = |doc: TrustTask<grant::Payload>| async {
        consume_inbound(
            &NoopHandler::new(),
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::consequential(&guard),
            doc,
            ME,
            now,
            || "err-1".to_string(),
            |req: TrustTask<grant::Payload>, _parties| async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<TrustTask<grant::Response>, _>(req.reject_with(
                    "err-1",
                    trust_tasks_rs::RejectReason::PermissionDenied {
                        reason: "not a maintainer of this repository".into(),
                    },
                ))
            },
        )
        .await
    };

    let first: ConsumeOutcome<grant::Response> = run(doc.clone()).await;
    assert!(matches!(first, ConsumeOutcome::Rejected(_)), "{first:?}");

    let second: ConsumeOutcome<grant::Response> = run(doc).await;
    match second {
        ConsumeOutcome::Duplicate { prior_response, .. } => {
            let prior = prior_response.expect("the refusal should be retained");
            assert_eq!(prior["payload"]["code"], "permissionDenied");
        }
        other => panic!("expected Duplicate, got {other:?}"),
    }
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "a settled refusal was re-evaluated"
    );
}

/// A document refused by an *earlier* §7.2 check must not burn its `id`.
/// Claiming before the checks would let an observer pre-burn any `id` it had
/// merely seen, and would turn every transient refusal into a permanent
/// `idConflict` for that document.
#[tokio::test]
async fn a_document_refused_before_the_claim_does_not_burn_its_id() {
    let guard = InMemoryReplayGuard::new(64);
    let executions = Executions::default();
    let now = t("2026-08-26T12:00:00Z");

    // Addressed to somebody else — refused under item 5a, before the claim.
    let mut misaddressed = request("req-1", "admin", now);
    misaddressed.recipient = Some("did:web:someone-else.example".into());
    let refused = consume(&guard, &executions, misaddressed, now).await;
    assert!(
        matches!(refused, ConsumeOutcome::Rejected(_)),
        "{refused:?}"
    );
    assert_eq!(guard.len(), 0, "a refused document claimed the id");

    // The correctly addressed document under the same `id` still runs.
    let ok = consume(&guard, &executions, request("req-1", "admin", now), now).await;
    assert!(matches!(ok, ConsumeOutcome::Handled(_)), "{ok:?}");
    assert_eq!(executions.count(), 1);
}

/// §7.2 (*Bounding the record*): a document the consumer cannot place in any
/// window MUST NOT have a consequential task executed on it. There is nowhere
/// to keep the record, so there is no way to satisfy item 11 for it.
#[tokio::test]
async fn an_unboundable_document_is_refused_rather_than_executed() {
    let guard = InMemoryReplayGuard::new(64);
    let executions = Executions::default();
    let now = t("2026-08-26T12:00:00Z");

    let mut doc = request("req-1", "admin", now);
    doc.issued_at = None;
    let executions = &executions;

    // A policy that keeps a record but refuses to require `issuedAt` still
    // cannot bound a document carrying neither timestamp.
    let checks = ConsumeChecks {
        freshness: FreshnessPolicy::default().with_max_age(TimeDelta::minutes(5)),
        replay: ReplayPolicy::Guard(&guard),
    };
    let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
        &NoopHandler::new(),
        ProofPolicy::<NoVerifier>::AcceptUnverified,
        PayloadPolicy::<NoValidator>::AcceptUnvalidated,
        checks,
        doc,
        ME,
        now,
        || "err-1".to_string(),
        |req, _parties| async move {
            executions.0.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ErrorResponse>(req.respond_with(
                "resp-1",
                grant::Response {
                    entry: entry("admin"),
                    ext: None,
                },
            ))
        },
    )
    .await;

    match outcome {
        ConsumeOutcome::Rejected(err) => assert_eq!(err.payload.code, StandardCode::Expired.into()),
        other => panic!("expected Rejected, got {other:?}"),
    }
    assert_eq!(executions.count(), 0);
}

/// A guard whose store is down must fail closed. Executing while unable to
/// consult the record is precisely the double execution item 11 forbids.
#[tokio::test]
async fn a_guard_that_cannot_reach_its_store_fails_closed() {
    struct BrokenGuard;

    #[async_trait::async_trait]
    impl ReplayGuard for BrokenGuard {
        async fn claim(
            &self,
            _id: &str,
            _digest: &trust_tasks_rs::DocumentDigest,
            _retain_until: Option<DateTime<Utc>>,
            _now: DateTime<Utc>,
        ) -> Result<trust_tasks_rs::ReplayVerdict, trust_tasks_rs::ReplayGuardError> {
            Err(trust_tasks_rs::ReplayGuardError(
                "redis://replay-1.internal:6379: connection refused".into(),
            ))
        }
    }

    let executions = Executions::default();
    let now = t("2026-08-26T12:00:00Z");
    let guard: Arc<dyn ReplayGuard> = Arc::new(BrokenGuard);

    let outcome = consume(
        guard.as_ref(),
        &executions,
        request("req-1", "admin", now),
        now,
    )
    .await;
    match outcome {
        ConsumeOutcome::Rejected(err) => {
            assert_eq!(err.payload.code, StandardCode::Unavailable.into());
            assert!(err.payload.retryable, "the resend must be invited");
            let msg = err.payload.message.as_deref().unwrap_or("");
            assert!(!msg.contains("redis"), "store detail on the wire: {msg}");
        }
        other => panic!("expected Rejected(unavailable), got {other:?}"),
    }
    assert_eq!(executions.count(), 0, "executed without a usable record");
}
