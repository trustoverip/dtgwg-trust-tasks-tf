//! Inbound-document orchestration for SPEC.md §7.2 items 4–8.
//!
//! [`consume_inbound`] is the framework-level helper a *consumer* uses to
//! avoid hand-wiring the eight-step §7.2 list across every spec they
//! implement. It accepts a typed [`TrustTask<P>`] (items 1–3 having already
//! been handled by the caller's [`Dispatcher`](crate::Dispatcher) /
//! `serde` pipeline), runs the remaining framework checks, optionally
//! verifies the `proof`, and either calls the caller's business handler
//! or builds the [`ErrorResponse`] routed per §8.1.
//!
//! ```rust,ignore
//! use trust_tasks_rs::{consume_inbound, ConsumeOutcome, TrustTask};
//!
//! async fn on_inbound<P>(
//!     transport: &MyHandler,
//!     verifier: Option<&MyVerifier>,
//!     doc: TrustTask<P>,
//! ) where
//!     P: trust_tasks_rs::Payload + serde::Serialize + Send + Sync,
//! {
//!     let outcome = consume_inbound(
//!         transport,
//!         verifier,
//!         doc,
//!         "did:web:maintainer.example",
//!         chrono::Utc::now(),
//!         || format!("urn:uuid:{}", uuid::Uuid::new_v4()),
//!         |accepted, parties| async move {
//!             // `parties` carries the SPEC §4.8.1-resolved issuer/recipient;
//!             // handlers can use it without re-running resolve_parties.
//!             //
//!             // On refusal, build an ErrorResponse with whatever code /
//!             // details the spec calls for — `TrustTask::reject_with` or
//!             // `reject_with_recipient` are the routing-safe builders.
//!             Ok(accepted.respond_with("resp-1", build_response(&parties)))
//!         },
//!     )
//!     .await;
//!
//!     match outcome {
//!         ConsumeOutcome::Handled(response) => emit(response),
//!         ConsumeOutcome::Rejected(error)   => emit(error),
//!         ConsumeOutcome::Suppressed        => {} // identity_mismatch w/o transport sender
//!     }
//! }
//! ```
//!
//! The function does not attempt items 1 (framework outer-schema
//! validation), 2 (payload schema validation), or 3 (unknown Type URI).
//! Those belong to the caller's deserialize + [`Dispatcher`](crate::Dispatcher)
//! pipeline — by the time you hold a `TrustTask<P>` they have already
//! succeeded.

use std::future::Future;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::document::{ErrorResponse, TrustTask};
use crate::error::RejectReason;
use crate::payload::Payload;
use crate::proof::{ProofVerifier, VerificationError};
use crate::transport::{ResolvedParties, TransportHandler};

/// Possible outcomes of [`consume_inbound`].
#[derive(Debug)]
pub enum ConsumeOutcome<R> {
    /// The document passed every framework check and the caller's
    /// handler produced a success response. The caller emits the
    /// response over the same transport that delivered the request.
    Handled(TrustTask<R>),
    /// A framework check failed and the rejection has a routable
    /// recipient, OR the caller's handler returned an
    /// [`ErrorResponse`] of its own. Either way the document is
    /// already addressed per SPEC.md §8.1 — the caller emits it over
    /// the transport.
    Rejected(ErrorResponse),
    /// SPEC.md §8.1 routing rule for `identity_mismatch`: the in-band
    /// `issuer` is by definition the contested identity, and the
    /// transport authenticated no sender, so the consumer **SHOULD NOT**
    /// emit any response (doing so would constitute an oracle).
    ///
    /// Callers **SHOULD** log this case for audit — silent suppression
    /// is the spec rule but invisible suppression is an ops footgun.
    Suppressed,
}

/// Run SPEC.md §7.2 items 4–8 against `doc`, then either call `handler`
/// or build the routed error response per §8.1.
///
/// `P` is the typed payload (already deserialized by the caller — items
/// 1–3 happen upstream). `R` is the success-response payload type the
/// handler returns on acceptance.
///
/// `transport` supplies the transport-derived identity for the §4.8.1
/// cross-check (item 6) and the §8.1 routing exception for
/// `identity_mismatch`.
///
/// `verifier` governs `proof` handling (item 7):
///
/// * `Some(v)` and `doc.proof` present → `v.verify(&doc)` is called;
///   verification failure rejects with `proof_invalid`.
/// * `Some(v)` and `doc.proof` absent → if `P::IS_PROOF_REQUIRED` is
///   `true`, reject with `proof_required`; otherwise accept.
/// * `None` and `doc.proof` present → reject with `malformed_request`
///   (the producer signaled a security contract the consumer cannot
///   honour; silently dropping the proof would mislead the producer
///   about the integrity guarantees of the exchange).
/// * `None` and `doc.proof` absent → if `P::IS_PROOF_REQUIRED` is
///   `true`, reject with `proof_required`; otherwise accept.
///
/// `error_id_factory` is invoked at most once, only when a rejection
/// path needs an `id` for the error response.
///
/// `handler` receives the accepted document and the SPEC §4.8.1-resolved
/// parties (so it can rely on `parties.issuer` / `parties.recipient`
/// without re-running [`TransportHandler::resolve_parties`]). On refusal
/// it builds and returns an [`ErrorResponse`] — typically via
/// [`TrustTask::reject_with`] or [`TrustTask::reject_with_recipient`] —
/// so handlers can mint extended codes (SPEC §8.5), attach
/// task-specific `details`, or apply spec-specific routing without
/// being constrained to the framework's [`RejectReason`] vocabulary.
#[allow(clippy::too_many_arguments)]
pub async fn consume_inbound<P, R, T, V, F, Fut>(
    transport: &T,
    verifier: Option<&V>,
    doc: TrustTask<P>,
    my_vid: &str,
    now: DateTime<Utc>,
    error_id_factory: impl FnOnce() -> String,
    handler: F,
) -> ConsumeOutcome<R>
where
    P: Payload + Serialize + Send + Sync,
    T: TransportHandler + Sync + ?Sized,
    V: ProofVerifier + ?Sized,
    F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
    Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
{
    // §7.2 items 4 + 5 — expiry and recipient enforcement.
    if let Err(reason) = doc.validate_basic(now, my_vid) {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // §7.2 item 6 — in-band vs transport-derived identity cross-check.
    let parties = match transport.resolve_parties(&doc) {
        Ok(p) => p,
        Err(mismatch) => {
            return route_rejection(
                transport,
                &doc,
                RejectReason::IdentityMismatch(mismatch),
                error_id_factory,
            );
        }
    };

    // §7.2 item 7 — proof handling. The combinations are enumerated in
    // the function docstring; the table here mirrors that.
    match (doc.proof.as_ref(), verifier) {
        (Some(_), Some(v)) => {
            if let Err(err) = v.verify(&doc).await {
                return route_rejection(
                    transport,
                    &doc,
                    proof_error_to_reject(err),
                    error_id_factory,
                );
            }
        }
        (Some(_), None) => {
            // SECURITY: a producer-supplied proof is an integrity
            // assertion the consumer would otherwise silently drop.
            // SPEC §4.7.1 + the proof-verification contract require
            // the consumer to honour or reject — never ignore.
            return route_rejection(
                transport,
                &doc,
                RejectReason::MalformedRequest {
                    reason: "document carries a proof but no verifier is configured; \
                             consumer cannot honour the producer's integrity assertion"
                        .to_string(),
                },
                error_id_factory,
            );
        }
        (None, _) if P::IS_PROOF_REQUIRED => {
            return route_rejection(
                transport,
                &doc,
                RejectReason::ProofRequired,
                error_id_factory,
            );
        }
        (None, _) => {}
    }

    // §7.2 item 8 — audience binding (proof + no recipient on a non-
    // bearer spec).
    if let Err(reason) = doc.enforce_audience_binding() {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // All §7.2 checks passed. Hand to the caller's business handler.
    match handler(doc, parties).await {
        Ok(response) => ConsumeOutcome::Handled(response),
        Err(error_response) => ConsumeOutcome::Rejected(error_response),
    }
}

fn route_rejection<P, R, T>(
    transport: &T,
    doc: &TrustTask<P>,
    reason: RejectReason,
    error_id_factory: impl FnOnce() -> String,
) -> ConsumeOutcome<R>
where
    P: Serialize,
    T: TransportHandler + Sync + ?Sized,
{
    match transport.reject(doc, error_id_factory(), reason) {
        Some(error_response) => ConsumeOutcome::Rejected(error_response),
        None => ConsumeOutcome::Suppressed,
    }
}

fn proof_error_to_reject(err: VerificationError) -> RejectReason {
    // Every variant of VerificationError maps to ProofInvalid on the wire
    // per SPEC §8.3 + the ProofVerifier module's documented mapping.
    RejectReason::ProofInvalid {
        reason: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::NoopHandler;
    use crate::proof::Proof;
    use crate::specs::acl::grant::v0_1 as grant;
    use crate::StandardCode;

    fn entry() -> grant::AclEntry {
        grant::AclEntry {
            subject: "did:web:alice.example".into(),
            role: "admin".into(),
            scopes: vec![],
            label: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            expires_at: None,
            ext: None,
        }
    }

    fn grant_payload() -> grant::Payload {
        grant::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        }
    }

    fn dummy_proof() -> Proof {
        Proof {
            proof_type: "DataIntegrityProof".into(),
            cryptosuite: "eddsa-rdfc-2022".into(),
            verification_method: "did:web:org.example#key-1".into(),
            created: Utc::now(),
            proof_purpose: "assertionMethod".into(),
            proof_value: "z3kg".into(),
            extra: Default::default(),
        }
    }

    /// Sentinel verifier that records whether it was invoked and what it
    /// returned. Used to check the §7.2 item 7 paths.
    struct StubVerifier {
        outcome: Result<(), VerificationError>,
    }

    #[async_trait::async_trait]
    impl ProofVerifier for StubVerifier {
        async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
        where
            P: Serialize + Send + Sync,
        {
            match &self.outcome {
                Ok(()) => Ok(()),
                Err(e) => Err(match e {
                    VerificationError::SignatureInvalid => VerificationError::SignatureInvalid,
                    other => VerificationError::Other(other.to_string()),
                }),
            }
        }
    }

    #[tokio::test]
    async fn handler_runs_when_all_checks_pass() {
        // acl/grant is IS_PROOF_REQUIRED=true, so the document MUST
        // carry a proof and we MUST supply a verifier.
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload("req-1", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            Some(&verifier),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-1".to_string(),
            |req, parties| async move {
                // Handler sees the resolved parties without re-deriving.
                assert_eq!(parties.issuer.as_deref(), Some("did:web:org.example"));
                assert_eq!(
                    parties.recipient.as_deref(),
                    Some("did:web:maintainer.example")
                );
                let resp_payload = grant::Response {
                    entry: req.payload.entry.clone(),
                    ext: None,
                };
                Ok(req.respond_with("resp-1", resp_payload))
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Handled(resp) => {
                assert_eq!(resp.id, "resp-1");
                assert_eq!(resp.payload.entry.subject, "did:web:alice.example");
            }
            other => panic!("expected Handled, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn wrong_recipient_routes_error_to_original_issuer() {
        let transport = NoopHandler::new();
        let verifier: Option<&StubVerifier> = None;
        let mut doc = TrustTask::for_payload("req-2", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:someone-else.example".into());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            verifier,
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-2".to_string(),
            |_req, _parties| async move {
                panic!("handler must not run when validate_basic rejects");
                #[allow(unreachable_code)]
                Ok::<TrustTask<grant::Response>, ErrorResponse>(unreachable!())
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.recipient.as_deref(), Some("did:web:org.example"));
                assert_eq!(err.payload.code, StandardCode::WrongRecipient.into());
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    /// SPEC §7.2 item 7 — IS_PROOF_REQUIRED is authoritative; the
    /// `verifier=Some` / `non-bearer` heuristic is gone. acl::grant is
    /// REQUIRED in front matter, so codegen set IS_PROOF_REQUIRED=true.
    #[tokio::test]
    async fn proof_required_fires_for_spec_with_required_proof() {
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload("req-3", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        // No proof.

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            Some(&verifier),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-3".to_string(),
            |_req, _parties| async move {
                panic!("handler must not run when proof_required fires");
                #[allow(unreachable_code)]
                Ok::<TrustTask<grant::Response>, ErrorResponse>(unreachable!())
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, StandardCode::ProofRequired.into());
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    /// SECURITY: a producer-supplied proof MUST NOT be silently
    /// dropped when no verifier is configured.
    #[tokio::test]
    async fn proof_present_with_no_verifier_rejected_as_malformed_request() {
        let transport = NoopHandler::new();
        let verifier: Option<&StubVerifier> = None;
        let mut doc = TrustTask::for_payload("req-4", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            verifier,
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-4".to_string(),
            |_req, _parties| async move {
                panic!("handler must not run when proof-without-verifier rejection fires");
                #[allow(unreachable_code)]
                Ok::<TrustTask<grant::Response>, ErrorResponse>(unreachable!())
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, StandardCode::MalformedRequest.into());
                assert!(err
                    .payload
                    .message
                    .as_deref()
                    .unwrap_or("")
                    .contains("no verifier"));
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    /// Handler-returned ErrorResponse is passed through verbatim — the
    /// handler owns routing for spec-specific refusals.
    #[tokio::test]
    async fn handler_error_response_is_passed_through() {
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload("req-5", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            Some(&verifier),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-5".to_string(),
            |req, _parties| async move {
                // Handler-minted extended code with custom routing.
                Err(req.reject_with(
                    "err-handler",
                    crate::ErrorPayload::new(crate::TrustTaskCode::Extended {
                        slug: "acl/grant".into(),
                        local: "role_not_recognized".into(),
                    })
                    .with_message("role string not in maintainer vocabulary"),
                ))
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.id, "err-handler");
                assert!(matches!(
                    err.payload.code,
                    crate::TrustTaskCode::Extended { ref slug, ref local }
                    if slug == "acl/grant" && local == "role_not_recognized"
                ));
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }
}
