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
//!         |accepted| async move {
//!             // ... business logic, returning Ok(TrustTask<Response>) on
//!             // success or Err(RejectReason::*) on a spec-handler refusal
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
use crate::error::{ErrorPayload, RejectReason};
use crate::payload::Payload;
use crate::proof::{ProofVerifier, VerificationError};
use crate::transport::TransportHandler;

/// Possible outcomes of [`consume_inbound`].
#[derive(Debug)]
pub enum ConsumeOutcome<R> {
    /// The document passed every framework check and the caller's
    /// handler produced a success response. The caller emits the
    /// response over the same transport that delivered the request.
    Handled(TrustTask<R>),
    /// A framework check failed, or the caller's handler rejected the
    /// document, and the rejection has a routable recipient. The
    /// returned [`ErrorResponse`] is already addressed per SPEC.md §8.1
    /// — the caller emits it over the transport.
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
/// `verifier` is consulted only when `doc.proof` is present. When the
/// spec declares `proof: REQUIRED` (i.e. `P::IS_BEARER == false` and
/// callers wish to enforce — see the inline note below) and no proof is
/// present, the function emits `proof_required`. Verification failures
/// (cryptosuite unknown, signature invalid, …) map to `proof_invalid`.
///
/// `error_id_factory` is invoked at most once, only when a rejection
/// path needs an `id` for the error response.
///
/// # Note on `proof: REQUIRED`
///
/// `Payload::IS_BEARER` declares the §4.8.3 bearer status of the spec.
/// The framework does NOT carry a `IS_PROOF_REQUIRED` constant today, so
/// this helper applies the conservative default: if `verifier` is `Some`
/// and `doc.proof` is `None` on a non-bearer spec, the document is
/// rejected with `proof_required`. Callers that opt out of strict proof
/// enforcement pass `verifier: None`.
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
    F: FnOnce(TrustTask<P>) -> Fut,
    Fut: Future<Output = Result<TrustTask<R>, RejectReason>>,
{
    // §7.2 items 4 + 5 — expiry and recipient enforcement.
    if let Err(reason) = doc.validate_basic(now, my_vid) {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // §7.2 item 6 — in-band vs transport-derived identity cross-check.
    if let Err(mismatch) = transport.resolve_parties(&doc) {
        return route_rejection(
            transport,
            &doc,
            RejectReason::IdentityMismatch(mismatch),
            error_id_factory,
        );
    }

    // §7.2 item 7 — proof verification (when present) and proof-required
    // enforcement (when a verifier was supplied for a non-bearer spec).
    match (doc.proof.as_ref(), verifier) {
        (Some(_), Some(v)) => match v.verify(&doc).await {
            Ok(()) => {}
            Err(err) => {
                return route_rejection(
                    transport,
                    &doc,
                    proof_error_to_reject(err),
                    error_id_factory,
                );
            }
        },
        (None, Some(_)) if !P::IS_BEARER => {
            return route_rejection(
                transport,
                &doc,
                RejectReason::ProofRequired,
                error_id_factory,
            );
        }
        _ => {}
    }

    // §7.2 item 8 — audience binding (proof + no recipient on a non-
    // bearer spec).
    if let Err(reason) = doc.enforce_audience_binding() {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // All §7.2 checks passed. Hand to the caller's business handler.
    match handler(doc).await {
        Ok(response) => ConsumeOutcome::Handled(response),
        Err(reason) => {
            // We no longer hold `doc` (the handler consumed it). The
            // handler's RejectReason is a spec-handler-level refusal,
            // never `identity_mismatch` (that was already handled
            // upstream). The caller can build the error response with
            // the request metadata they preserved on the way in — but
            // since we can't reach back into the consumed `doc`, we
            // surface the reason and let the caller route. This is the
            // one path where `consume_inbound`'s ergonomic win is
            // partial; in practice handlers are expected to use
            // `TrustTask::respond_with` for success and call
            // `TrustTask::reject_with` themselves (returning an
            // ErrorResponse, not a RejectReason) for refusal. The
            // RejectReason path here is for handlers that defer routing
            // to the framework; they get an unrouted error.
            ConsumeOutcome::Rejected(hand_built_unrouted_error(error_id_factory(), reason))
        }
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

/// Build an unrouted error response — used only for the spec-handler-
/// refused path, where `consume_inbound` has consumed the inbound `doc`
/// and cannot reach back into it for routing metadata.
fn hand_built_unrouted_error(error_id: String, reason: RejectReason) -> ErrorResponse {
    use crate::document::trust_task_error_type_uri;
    ErrorResponse {
        id: error_id,
        thread_id: None,
        type_uri: trust_task_error_type_uri(),
        issuer: None,
        recipient: None,
        issued_at: Some(Utc::now()),
        expires_at: None,
        payload: ErrorPayload::from(reason),
        context: None,
        proof: None,
        extra: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::NoopHandler;
    use crate::specs::acl::grant::v0_1 as grant;

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
        let transport = NoopHandler::new();
        let verifier: Option<&StubVerifier> = None;
        let mut doc = TrustTask::for_payload("req-1", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            verifier,
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-1".to_string(),
            |req| async move {
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
            |_req| async move {
                panic!("handler must not run when validate_basic rejects");
                #[allow(unreachable_code)]
                Err::<TrustTask<grant::Response>, _>(RejectReason::TaskFailed {
                    reason: "unreachable".into(),
                    details: None,
                })
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.recipient.as_deref(), Some("did:web:org.example"));
                assert_eq!(err.payload.code, crate::StandardCode::WrongRecipient.into());
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn proof_required_when_verifier_set_but_proof_absent_on_non_bearer_spec() {
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
            |_req| async move {
                panic!("handler must not run when proof_required fires");
                #[allow(unreachable_code)]
                Err::<TrustTask<grant::Response>, _>(RejectReason::TaskFailed {
                    reason: "unreachable".into(),
                    details: None,
                })
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, crate::StandardCode::ProofRequired.into());
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }
}
