//! Inbound-document orchestration for SPEC.md §7.2 item 2 and items 4–8.
//!
//! [`consume_inbound`] is the framework-level helper a *consumer* uses to
//! avoid hand-wiring the eight-step §7.2 list across every spec they
//! implement. It accepts a typed [`TrustTask<P>`] (items 1 and 3 having
//! already been handled by the caller's [`Dispatcher`](crate::Dispatcher) /
//! `serde` pipeline), runs the remaining framework checks, optionally
//! verifies the `proof` and the payload schema, and either calls the caller's
//! business handler or builds the [`ErrorResponse`] routed per §8.1.
//!
//! ```rust,ignore
//! use trust_tasks_rs::{
//!     consume_inbound, ConsumeChecks, ConsumeOutcome, InMemoryReplayGuard,
//!     PayloadPolicy, ProofPolicy, TrustTask,
//! };
//!
//! // One guard per consumer, not one per document: it *is* the record.
//! static GUARD: LazyLock<InMemoryReplayGuard> = LazyLock::new(Default::default);
//!
//! async fn on_inbound<P>(
//!     transport: &MyHandler,
//!     verifier: &MyVerifier,
//!     doc: TrustTask<P>,
//! ) where
//!     P: trust_tasks_rs::Payload + serde::Serialize + Send + Sync,
//! {
//!     let outcome = consume_inbound(
//!         transport,
//!         ProofPolicy::Verify(verifier),
//!         PayloadPolicy::Validate(schema_validator),
//!         // `acl/grant` is consequential: a replayed envelope must not grant
//!         // twice. Use `not_consequential()` only where the spec says so.
//!         ConsumeChecks::consequential(&*GUARD),
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
//!         // §7.2 item 11: already executed. Not an error — return the prior
//!         // result if there is one, otherwise emit nothing.
//!         ConsumeOutcome::Duplicate { prior_response, .. } => {
//!             if let Some(prior) = prior_response { emit_json(prior) }
//!         }
//!     }
//! }
//! ```
//!
//! The function does not attempt item 1 (framework outer-schema validation)
//! or item 3 (unknown Type URI). Those belong to the caller's deserialize +
//! [`Dispatcher`](crate::Dispatcher) pipeline — by the time you hold a
//! `TrustTask<P>` they have already succeeded.
//!
//! Item 2 (payload schema) is deliberately *not* in that list, though it once
//! was. Deserializing into `P` performs most of it — required members, member
//! types, `additionalProperties`, and the string constraints typify emits as
//! validating newtypes — but not the constraints a Rust type cannot carry
//! (`minProperties`, `minItems` on an optional array, conditional subschemas).
//! Treating the whole of item 2 as "already succeeded" therefore overstated
//! what the parse step had established, so the residue is now a policy the
//! caller chooses: see [`PayloadPolicy`].

use std::future::Future;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::document::{ErrorResponse, TrustTask};
use crate::error::RejectReason;
use crate::freshness::{FreshnessPolicy, StaleReason};
use crate::payload::Payload;
use crate::proof::{ProofVerifier, VerificationError};
use crate::replay::{document_digest, DocumentDigest, ReplayGuard, ReplayPolicy, ReplayVerdict};
use crate::transport::{ResolvedParties, TransportHandler};

/// The two stateful consumer checks SPEC.md §7.2 requires and that no runtime
/// in this repo implemented before 0.11.17: the freshness bound over
/// `issuedAt` / `expiresAt`, and the duplicate-execution record of item 11.
///
/// They travel together because the spec ties them together. §7.2 (*Bounding
/// the record*) makes the acceptance window and the replay record's retention
/// **the same bound**: a consumer "**MUST NOT** accept for execution a
/// document older than the window over which it retains records". Passing them
/// as one argument is what stops a deployment configuring a five-minute record
/// and an unbounded acceptance window, which reads as a working replay defence
/// and is not one.
///
/// Like [`PayloadPolicy`], this is a *required* argument rather than a
/// defaulted one. Whether the task a consumer implements is *consequential*
/// (§2) is a decision only that consumer can make, and the failure mode of
/// getting it wrong silently — an ACL grant applied twice by a mediator retry
/// — is not one to discover after the fact.
pub struct ConsumeChecks<'a> {
    /// Bounds the document in time (SPEC §4.2, §7.2 item 4).
    pub freshness: FreshnessPolicy,
    /// Applies (or knowingly disapplies) the duplicate-execution rule of
    /// SPEC §7.2 item 11.
    pub replay: ReplayPolicy<'a>,
}

impl<'a> ConsumeChecks<'a> {
    /// The posture for a *consequential Trust Task* (§2): item 11 enforced
    /// against `guard`, and the bounded acceptance window
    /// ([`FreshnessPolicy::consequential`]) that makes the record droppable.
    ///
    /// This is the right constructor for any task whose execution grants
    /// access, moves value, discloses a secret, or otherwise cannot be undone
    /// by ignoring the next document.
    pub fn consequential(guard: &'a dyn ReplayGuard) -> Self {
        Self {
            freshness: FreshnessPolicy::consequential(),
            replay: ReplayPolicy::Guard(guard),
        }
    }

    /// The posture for a task that is **not** consequential, or for one whose
    /// *Trust Task specification* "explicitly declares repeated execution safe
    /// and intended" — the narrow disapplication item 11 permits.
    ///
    /// Keeps no record. Still applies [`FreshnessPolicy::default`], because
    /// a future-dated document and one with an empty validity interval are
    /// malformed whatever the task does.
    pub fn not_consequential() -> Self {
        Self {
            freshness: FreshnessPolicy::default(),
            replay: ReplayPolicy::NotConsequential,
        }
    }

    /// Builder: override the freshness policy.
    #[must_use]
    pub fn with_freshness(mut self, freshness: FreshnessPolicy) -> Self {
        self.freshness = freshness;
        self
    }
}

/// How [`consume_inbound`] handles a document's `proof` member, per
/// SPEC.md §7.2 item 7.
///
/// `consume_inbound` does not assume what kind of integrity guarantees
/// the consumer relies on. Some deployments verify Data Integrity proofs
/// in-band; some have transport-layer integrity (signed DIDComm, mTLS-
/// bound HTTPS) and accept in-band proofs only opportunistically; some
/// have no integrity guarantees at all. The variants below make that
/// decision explicit at the call site.
///
/// `Payload::IS_PROOF_REQUIRED` is consulted independently of the
/// policy: a spec that requires a proof rejects a proofless document
/// regardless of which policy was chosen.
#[non_exhaustive]
pub enum ProofPolicy<'a, V: ProofVerifier + ?Sized> {
    /// Verify the proof when present using `V`. When `doc.proof` is
    /// `Some`, the verifier is consulted and failures map to
    /// `proof_invalid`. This is the safe default for any consumer that
    /// expects to honour in-band proofs.
    Verify(&'a V),

    /// Reject documents that carry an in-band proof with
    /// `malformed_request`. Use this when the consumer has integrity
    /// guarantees from another layer (e.g. transport-bound signing) and
    /// is deliberately not verifying in-band proofs — silently dropping
    /// a producer-supplied proof would mislead the producer about the
    /// guarantees of the exchange, so the framework rejects the document
    /// instead.
    RejectIfPresent,

    /// SECURITY: accept any document, with or without a proof, without
    /// verifying. Use only when the transport already provides
    /// equivalent integrity end-to-end (or the consumer has accepted
    /// the policy decision not to honour in-band proofs from this
    /// counterparty). This is the explicit opt-out — the variant name
    /// is deliberately uncomfortable to type.
    AcceptUnverified,
}

/// Evaluates a payload against its `payload.schema.json` (SPEC.md §7.2 item 2).
///
/// The framework does not bundle a JSON Schema implementation, for the same
/// reason it does not bundle a cryptosuite: the choice of engine, its draft
/// support, and its resource limits belong to the consumer. Implement this
/// over whichever validator you already run — or over
/// [`crate::validate::against_schema`], which the `validate` feature provides
/// on top of the `jsonschema` crate.
pub trait PayloadValidator {
    /// Check `payload` against `schema_json`.
    ///
    /// `schema_json` is [`Payload::PAYLOAD_SCHEMA`] — a string constant
    /// inlined from this repo at codegen time, so it is trusted input. Return
    /// a human-readable reason on failure; it lands in the `malformedRequest`
    /// error's message.
    fn validate(&self, schema_json: &str, payload: &Value) -> Result<(), String>;
}

/// How [`consume_inbound`] performs SPEC.md §7.2 item 2 — payload-schema
/// validation.
///
/// **Read this before reaching for [`AcceptUnvalidated`](Self::AcceptUnvalidated).**
/// In this library, item 2 is *mostly* already done by the time you hold a
/// `TrustTask<P>`: deserializing into the generated types enforces required
/// members, member types, `additionalProperties: false`, and the `pattern` /
/// `minLength` constraints typify expresses as validating newtypes. A
/// document that got this far has cleared all of that.
///
/// What it has *not* cleared is everything typify cannot express in a Rust
/// type — `minProperties`, `minItems` on an optional array, conditional
/// subschemas. That residue is what [`Validate`](Self::Validate) catches, and
/// it is why the choice is a required argument rather than a default: a
/// consumer should decide knowingly whether that residue matters to it, not
/// discover later that it never checked.
///
/// (The TypeScript binding is in a different position entirely — its types are
/// erased at runtime, so *nothing* is enforced without a validator. Both
/// libraries take the policy as a required argument so the two reach the same
/// verdict on the same document.)
#[non_exhaustive]
pub enum PayloadPolicy<'a, V: PayloadValidator + ?Sized> {
    /// Validate `doc.payload` against [`Payload::PAYLOAD_SCHEMA`] using `V`.
    /// Failures map to `malformedRequest` per §7.2 item 2.
    ///
    /// Where the payload type carries no schema (`PAYLOAD_SCHEMA` is `None`,
    /// i.e. the hand-modelled `trust-task-error`) there is nothing to check
    /// and the document passes: the Rust type it deserialized into is itself
    /// the constraint.
    Validate(&'a V),

    /// Accept the payload on the strength of deserialization alone, without
    /// the residual schema check.
    ///
    /// Defensible — deserialization is a real check here, not a formality —
    /// but it is a decision, and this names it.
    ///
    /// This variant carries no validator, so nothing pins `V`. Write it as
    /// `PayloadPolicy::<NoValidator>::AcceptUnvalidated`.
    AcceptUnvalidated,
}

/// Pins the validator type on the [`PayloadPolicy::AcceptUnvalidated`] path.
///
/// `AcceptUnvalidated` holds no validator, so type inference has nothing to
/// work from and the call will not compile without naming a type. This is that
/// type:
///
/// ```rust,ignore
/// consume_inbound(
///     transport,
///     ProofPolicy::Verify(verifier),
///     PayloadPolicy::<NoValidator>::AcceptUnvalidated,
///     ConsumeChecks::consequential(&guard),
///     doc,
///     // …
/// )
/// ```
///
/// Its [`PayloadValidator`] impl accepts everything, so it is also usable as a
/// stub in tests — but prefer the explicit `AcceptUnvalidated` variant in
/// production code, because the variant is what a reader greps for.
pub struct NoValidator;

impl PayloadValidator for NoValidator {
    fn validate(&self, _schema_json: &str, _payload: &Value) -> Result<(), String> {
        Ok(())
    }
}

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

    /// SPEC.md §7.2 item 11: a document with this `id` and this content was
    /// already accepted for execution. **The handler was not called**, and the
    /// consequential effect did not happen a second time. This is the §8.4
    /// retry being absorbed, which is the outcome that makes retrying safe.
    ///
    /// **This is not an error.** §7.2 (*Disposition of a duplicate*): "In no
    /// case is a duplicate reported as `taskFailed`; the task did not fail, it
    /// already happened." A caller that folded this into
    /// [`Rejected`](Self::Rejected) would report a failure that did not occur,
    /// and would tell the producer to investigate a working system.
    ///
    /// What to emit:
    ///
    /// * `prior_response` present — emit it. It is the response document the
    ///   first execution produced (a success response, or a non-retryable
    ///   error response; tell them apart by its `type`), which §7.2 says the
    ///   consumer **SHOULD** return.
    /// * `prior_response` absent, `in_flight == false` — the specification
    ///   defines no success response (§4.4.1's fire-and-forget case) or the
    ///   guard retains none. Emit nothing: "there is nothing to return: the
    ///   *consumer* declines to execute again and that silence is the correct
    ///   disposition, not an error."
    /// * `in_flight == true` — the original execution has not finished. §7.2:
    ///   "the *consumer* **SHOULD** return or expose the existing execution
    ///   state rather than begin another."
    Duplicate {
        /// The response the first execution produced, if the
        /// [`ReplayGuard`](crate::ReplayGuard) retained one.
        prior_response: Option<Value>,
        /// Whether that first execution is still running.
        in_flight: bool,
    },
}

/// Run SPEC.md §7.2 item 2 and items 4–8 against `doc`, then either call
/// `handler` or build the routed error response per §8.1.
///
/// `P` is the typed payload. Items 1 and 3 happen upstream, in the caller's
/// parse and dispatch. Item 2 is *shared*: deserializing into `P` performed
/// most of it, and `payload_policy` decides whether the residue typify cannot
/// express is checked here too — see [`PayloadPolicy`], which is worth
/// reading before choosing.
///
/// `transport` supplies the transport-derived identity for the §4.8.1
/// cross-check (item 6) and the §8.1 routing exception for
/// `identity_mismatch`.
///
/// `policy` selects how the consumer handles the document's `proof`
/// member (item 7); see [`ProofPolicy`] for the three variants and
/// their security tradeoffs. `Payload::IS_PROOF_REQUIRED` is enforced
/// regardless of the policy.
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
///
/// ⚠ **Handler-built errors and §8.1 routing.** When the handler
/// returns `Err(ErrorResponse)`, `consume_inbound` passes it through
/// verbatim — the framework does *not* re-apply §8.1 routing on the
/// handler-built response. Handlers that need to reject for
/// identity-style reasons (e.g. an authz check against the in-band
/// issuer the framework already accepted) **MUST** use either
/// [`TrustTask::reject_with_recipient`] with an explicit transport-
/// authenticated recipient, or call [`TransportHandler::reject`]
/// directly, which applies the §8.1 policy. Calling
/// [`TrustTask::reject_with`] (which copies `request.issuer` into
/// `recipient`) is safe for most refusals but is **not** safe under
/// rejections that contest the in-band identity.
#[allow(clippy::too_many_arguments)]
pub async fn consume_inbound<P, R, T, V, W, F, Fut>(
    transport: &T,
    policy: ProofPolicy<'_, V>,
    payload_policy: PayloadPolicy<'_, W>,
    checks: ConsumeChecks<'_>,
    doc: TrustTask<P>,
    my_vid: &str,
    now: DateTime<Utc>,
    error_id_factory: impl FnOnce() -> String,
    handler: F,
) -> ConsumeOutcome<R>
where
    P: Payload + Serialize + Send + Sync,
    R: Serialize,
    T: TransportHandler + Sync + ?Sized,
    V: ProofVerifier + ?Sized,
    W: PayloadValidator + ?Sized,
    F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
    Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
{
    let ConsumeChecks { freshness, replay } = checks;
    // §7.2 item 2 — payload schema. Runs first, in the spec's own order, and
    // before any check that consults the payload's meaning: a document whose
    // payload is not the shape the specification declares should be refused as
    // malformed rather than reasoned about. Deserialization into `P` already
    // carried most of this; see `PayloadPolicy` for what is left.
    if let PayloadPolicy::Validate(validator) = payload_policy {
        if let Some(schema) = P::PAYLOAD_SCHEMA {
            match serde_json::to_value(&doc.payload) {
                Ok(value) => {
                    if let Err(reason) = validator.validate(schema, &value) {
                        return route_rejection(
                            transport,
                            &doc,
                            RejectReason::MalformedRequest { reason },
                            error_id_factory,
                        );
                    }
                }
                // A payload that cannot be re-serialized cannot be checked.
                // Refusing is the only safe reading: the alternative is to
                // accept precisely the documents the validator could not see.
                Err(e) => {
                    return route_rejection(
                        transport,
                        &doc,
                        RejectReason::MalformedRequest {
                            reason: format!("payload could not be serialized for validation: {e}"),
                        },
                        error_id_factory,
                    );
                }
            }
        }
    }

    // §7.2 items 4 + 5a — expiry and wrong-recipient enforcement.
    if let Err(reason) = doc.validate_basic(now, my_vid) {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // §7.2 item 4, the other half — the freshness bound over `issuedAt`.
    // `validate_basic` honours `expiresAt`, which is optional and which a
    // producer sets for its own reasons; on its own it leaves a document
    // stamped years ago, or years hence, indefinitely acceptable. It is also
    // what bounds the replay record below: §7.2 (*Bounding the record*) makes
    // the acceptance window and the record's retention one bound.
    if let Err(reason) = doc.validate_freshness(now, &freshness) {
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

    // §7.2 item 7, clause B — apply the consumer's chosen proof policy.
    match (&policy, doc.proof.as_ref()) {
        (ProofPolicy::Verify(v), Some(_)) => {
            if let Err(err) = v.verify(&doc).await {
                return route_rejection(
                    transport,
                    &doc,
                    proof_error_to_reject(err),
                    error_id_factory,
                );
            }
        }
        (ProofPolicy::RejectIfPresent, Some(_)) => {
            return route_rejection(
                transport,
                &doc,
                RejectReason::MalformedRequest {
                    reason: PROOF_NOT_ACCEPTED_BY_POLICY.to_string(),
                },
                error_id_factory,
            );
        }
        // Verify-with-no-proof, AcceptUnverified-with-or-without-proof,
        // RejectIfPresent-with-no-proof — all accept and fall through.
        (ProofPolicy::Verify(_), None)
        | (ProofPolicy::RejectIfPresent, None)
        | (ProofPolicy::AcceptUnverified, _) => {}
    }

    // §7.2 items 5b + 7A + 8 — the flag-driven per-spec checks, in one place
    // (`TrustTask::enforce_spec_policy`) shared with binding pipelines such as
    // the HTTPS server, so the typed check set cannot diverge between paths.
    // The non-typed checks above (expiry, cross-check, proof verification) are
    // applied per-pipeline; this runs after them.
    if let Err(reason) = doc.enforce_spec_policy() {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // §7.2 item 11 — the duplicate-execution record. Deliberately **last**:
    // claiming the `id` marks the document as accepted for execution, and a
    // document that some earlier check refuses was never accepted. Claiming
    // first would burn the `id` on every malformed or unauthorised arrival,
    // so a corrected resend under the same `id` — which §8.4 does not permit
    // anyway, but which producers do — would come back `idConflict` forever,
    // and an attacker could pre-burn an `id` it had merely observed.
    let mut claim: Option<(&dyn ReplayGuard, DocumentDigest)> = None;
    if let ReplayPolicy::Guard(guard) = replay {
        let digest = match document_digest(&doc) {
            Ok(digest) => digest,
            Err(e) => {
                return route_rejection(
                    transport,
                    &doc,
                    RejectReason::malformed_from_serde(&e),
                    error_id_factory,
                );
            }
        };

        // §7.2 (*Bounding the record*): "A *consumer* that can establish
        // neither an `expiresAt` nor an age for a document has no window in
        // which to place it, and **MUST NOT** execute a *consequential Trust
        // Task* on it." A guard asked to retain a record forever is not a
        // guard, so refuse rather than pretend.
        let Some(retain_until) = freshness.record_expiry(&doc, now) else {
            return route_rejection(
                transport,
                &doc,
                RejectReason::Stale {
                    detail: StaleReason::Unboundable,
                },
                error_id_factory,
            );
        };

        match guard.claim(&doc.id, &digest, Some(retain_until), now).await {
            Ok(ReplayVerdict::Fresh) => claim = Some((guard, digest)),
            Ok(ReplayVerdict::Duplicate {
                prior_response,
                in_flight,
            }) => {
                return ConsumeOutcome::Duplicate {
                    prior_response,
                    in_flight,
                };
            }
            Ok(ReplayVerdict::Conflict) => {
                return route_rejection(
                    transport,
                    &doc,
                    RejectReason::IdConflict,
                    error_id_factory,
                );
            }
            // Fail closed. A consumer that cannot consult its record has not
            // satisfied item 11, and executing anyway is exactly the double
            // execution the rule forbids. `unavailable` is retryable, which is
            // the honest answer: the producer's bit-for-bit resend will be
            // absorbed correctly once the store is back.
            Err(e) => {
                return route_rejection(transport, &doc, e.into(), error_id_factory);
            }
        }
    }

    // All §7.2 checks passed. Hand to the caller's business handler.
    let doc_id = doc.id.clone();
    match handler(doc, parties).await {
        Ok(response) => {
            if let Some((guard, _)) = claim {
                // Best-effort: the effect has already happened, so a failure
                // to cache the response cannot un-happen it. The record of the
                // *claim* is what item 11 needs and it is already written; all
                // that is lost is the ability to hand the same response back,
                // and the duplicate is still absorbed. Guards should log their
                // own failures — there is nowhere to report this from here.
                let value = serde_json::to_value(&response).ok();
                let _ = guard.record_response(&doc_id, value.as_ref()).await;
            }
            ConsumeOutcome::Handled(response)
        }
        Err(error_response) => {
            if let Some((guard, digest)) = claim {
                if error_response.payload.retryable {
                    // §8.4: the consumer has just told the producer it MAY
                    // re-send this document bit-for-bit. Holding the claim
                    // would make that retry come back as an absorbed duplicate
                    // carrying the same failure forever — the consumer would
                    // have invited a retry it had already decided to refuse.
                    let _ = guard.release(&doc_id, &digest).await;
                } else {
                    // A non-retryable refusal is final for this document, so
                    // the record stands and a replay is answered with the same
                    // determination rather than re-evaluated.
                    let value = serde_json::to_value(&error_response).ok();
                    let _ = guard.record_response(&doc_id, value.as_ref()).await;
                }
            }
            ConsumeOutcome::Rejected(error_response)
        }
    }
}

/// Wire-safe message for the `RejectIfPresent` rejection path. The
/// in-house diagnostic ("no verifier configured") would let an
/// unauthenticated probe enumerate which endpoints in a fleet lack
/// verifier coverage; this constant intentionally says nothing about
/// the consumer's configuration. Verbose diagnostics belong in logs.
///
/// Shared with transport bindings (e.g. `trust-tasks-https`) that
/// apply the same rule against their own wire so a single string is
/// used framework-wide.
pub const PROOF_NOT_ACCEPTED_BY_POLICY: &str =
    "in-band proof not accepted by consumer policy (SPEC §7.2 item 7)";

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
            allowed_keys: None,
            subject: "did:web:alice.example".into(),
            role: "admin".into(),
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

    /// Refuses everything, so a test can prove the validator was consulted at
    /// all — a validator that is never called passes vacuously, which is the
    /// failure mode this whole change is about.
    struct RejectingValidator;

    impl PayloadValidator for RejectingValidator {
        fn validate(&self, _schema_json: &str, _payload: &Value) -> Result<(), String> {
            Err("stub validator refuses every payload".to_string())
        }
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
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::not_consequential(),
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

    /// §7.2 item 2 runs, and runs *before* the handler. A validator that
    /// refuses everything must stop the document; if the handler is reached,
    /// the policy was never consulted — which was the defect.
    #[tokio::test]
    async fn payload_policy_rejects_before_the_handler_runs() {
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload("req-1", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::Validate(&RejectingValidator),
            ConsumeChecks::not_consequential(),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-1".to_string(),
            |_req, _parties| async move {
                panic!("handler must not run when the payload policy rejects");
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, StandardCode::MalformedRequest.into());
                assert!(
                    err.payload
                        .message
                        .as_deref()
                        .is_some_and(|m| m.contains("refuses every payload")),
                    "the validator's own reason should reach the error message, got: {:?}",
                    err.payload.message
                );
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    /// The opt-out is a real opt-out: the same document the validator would
    /// have refused is accepted when the policy declines to check it.
    #[tokio::test]
    async fn accept_unvalidated_skips_the_check() {
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload("req-1", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::not_consequential(),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-1".to_string(),
            |req, _parties| async move {
                let resp_payload = grant::Response {
                    entry: req.payload.entry.clone(),
                    ext: None,
                };
                Ok(req.respond_with("resp-1", resp_payload))
            },
        )
        .await;

        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
    }

    /// Every generated payload type carries its schema, and it is reachable
    /// without the `validate` feature — the property that makes a caller-
    /// supplied validator possible at all.
    #[test]
    fn generated_payloads_carry_their_schema() {
        assert!(
            <grant::Payload as Payload>::PAYLOAD_SCHEMA.is_some(),
            "request payloads must carry PAYLOAD_SCHEMA"
        );
        assert!(
            <grant::Response as Payload>::PAYLOAD_SCHEMA.is_some(),
            "response payloads must carry PAYLOAD_SCHEMA — the reported defect \
             was found on a response variant"
        );
        // `trust-task-error` is hand-modelled and outside the codegen, so it
        // has no generated schema; `PAYLOAD_SCHEMA` defaults to `None` for any
        // such type, which is what makes the default safe rather than silent.
    }

    #[tokio::test]
    async fn wrong_recipient_routes_error_to_original_issuer() {
        let transport = NoopHandler::new();
        let mut doc = TrustTask::for_payload("req-2", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:someone-else.example".into());

        let outcome: ConsumeOutcome<grant::Response> =
            consume_inbound::<_, _, _, StubVerifier, NoValidator, _, _>(
                &transport,
                ProofPolicy::RejectIfPresent,
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                ConsumeChecks::not_consequential(),
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

    /// SPEC §7.2 item 7 — IS_PROOF_REQUIRED is authoritative regardless
    /// of policy. acl::grant is REQUIRED in front matter, so codegen
    /// set IS_PROOF_REQUIRED=true.
    #[tokio::test]
    async fn recipient_required_fires_when_in_band_recipient_absent() {
        // acl/grant declares its recipient party REQUIRED, so a document with
        // no in-band recipient is malformed (§7.2 item 5b). This check runs
        // before the proof check, so it wins even though grant also requires a
        // proof — and the handler must not run.
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload("req-rr", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        // No recipient, no proof.

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::not_consequential(),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-rr".to_string(),
            |_req, _parties| async move {
                panic!("handler must not run when recipient_required fires");
                #[allow(unreachable_code)]
                Ok::<TrustTask<grant::Response>, ErrorResponse>(unreachable!())
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, StandardCode::MalformedRequest.into());
            }
            other => panic!("expected Rejected(MalformedRequest), got {other:?}"),
        }
    }

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
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::not_consequential(),
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
    /// dropped under `ProofPolicy::RejectIfPresent`. The wire message
    /// MUST NOT mention the consumer's configuration (no "verifier",
    /// no "configured" — those would let an unauthenticated probe
    /// fingerprint the deployment).
    #[tokio::test]
    async fn proof_present_under_reject_if_present_rejected_as_malformed_request() {
        let transport = NoopHandler::new();
        let mut doc = TrustTask::for_payload("req-4", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<grant::Response> =
            consume_inbound::<_, _, _, StubVerifier, NoValidator, _, _>(
                &transport,
                ProofPolicy::RejectIfPresent,
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                ConsumeChecks::not_consequential(),
                doc,
                "did:web:maintainer.example",
                Utc::now(),
                || "err-4".to_string(),
                |_req, _parties| async move {
                    panic!("handler must not run under RejectIfPresent + proof");
                    #[allow(unreachable_code)]
                    Ok::<TrustTask<grant::Response>, ErrorResponse>(unreachable!())
                },
            )
            .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, StandardCode::MalformedRequest.into());
                let msg = err.payload.message.as_deref().unwrap_or("");
                assert!(
                    msg.contains("policy") && msg.contains("§7.2"),
                    "wire message should cite policy + spec, not name internals: {msg}"
                );
                assert!(
                    !msg.contains("verifier"),
                    "wire leak (configuration): {msg}"
                );
                assert!(
                    !msg.contains("configured"),
                    "wire leak (configuration): {msg}"
                );
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
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::not_consequential(),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-5".to_string(),
            |req, _parties| async move {
                // Handler-minted extended code with custom routing. The
                // helper sources the slug from grant::Payload::TYPE_URI so
                // it cannot drift from the type's identity.
                Err(req.reject_with(
                    "err-handler",
                    crate::ErrorPayload::new(grant::Payload::extended_code("role_not_recognized"))
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
                // The handler used `reject_with`, which routes to the
                // original issuer. Verify the §8.1 routing held.
                assert_eq!(err.recipient.as_deref(), Some("did:web:org.example"));
                assert_eq!(err.issuer.as_deref(), Some("did:web:maintainer.example"));
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    /// SPEC §7.2 item 7 — a RECOMMENDED spec (`IS_PROOF_REQUIRED ==
    /// false`) is accepted without a proof under `ProofPolicy::Verify`.
    /// Locks in the per-spec discrimination: regression would force
    /// every spec to behave like REQUIRED.
    #[tokio::test]
    async fn recommended_spec_accepts_proofless_under_verify_policy() {
        use crate::specs::acl::list::v0_1 as list;
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload(
            "req-rec-1",
            list::Payload {
                role: None,
                scope: None,
                direction: None,
                subject_prefix: None,
                page_size: None,
                cursor: None,
                ext: None,
            },
        );
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        // No proof, no IS_PROOF_REQUIRED — handler should run.

        let outcome: ConsumeOutcome<list::Response> = consume_inbound(
            &transport,
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::not_consequential(),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-rec-1".to_string(),
            |req, _parties| async move {
                Ok(req.respond_with(
                    "resp-rec-1",
                    list::Response {
                        entries: vec![],
                        cursor: None,
                        redacted_fields: vec![],
                        truncated: false,
                        ext: None,
                    },
                ))
            },
        )
        .await;

        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
    }

    /// `ProofPolicy::AcceptUnverified` accepts a proof-bearing document
    /// without invoking any verifier. Locks in the explicit opt-out.
    #[tokio::test]
    async fn accept_unverified_passes_proof_bearing_doc_through() {
        use crate::specs::acl::list::v0_1 as list;
        let transport = NoopHandler::new();
        let mut doc = TrustTask::for_payload(
            "req-au-1",
            list::Payload {
                role: None,
                scope: None,
                direction: None,
                subject_prefix: None,
                page_size: None,
                cursor: None,
                ext: None,
            },
        );
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<list::Response> =
            consume_inbound::<_, _, _, StubVerifier, NoValidator, _, _>(
                &transport,
                ProofPolicy::AcceptUnverified,
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                ConsumeChecks::not_consequential(),
                doc,
                "did:web:maintainer.example",
                Utc::now(),
                || "err-au-1".to_string(),
                |req, _parties| async move {
                    Ok(req.respond_with(
                        "resp-au-1",
                        list::Response {
                            entries: vec![],
                            cursor: None,
                            redacted_fields: vec![],
                            truncated: false,
                            ext: None,
                        },
                    ))
                },
            )
            .await;

        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
    }

    /// Verifier returns `Err` → `consume_inbound` maps to `proofInvalid` and
    /// the wire message is the constant, **not** the verifier's error text.
    ///
    /// This test asserted the opposite until 0.11.17. The verifier's text is
    /// the only place a consumer's DID-resolution behaviour is described in
    /// English, and the party receiving it is by construction unauthenticated
    /// — the proof did not verify. SPEC §10.4 extends the §8.1
    /// `identityMismatch` rule to every code for exactly this reason. The
    /// detail is still on the `Display` impl, which is what the operator logs.
    #[tokio::test]
    async fn proof_invalid_rejected_with_verifier_error_message() {
        let transport = NoopHandler::new();
        let verifier = StubVerifier {
            outcome: Err(VerificationError::SignatureInvalid),
        };
        let mut doc = TrustTask::for_payload("req-pi-1", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::not_consequential(),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-pi-1".to_string(),
            |_req, _parties| async move {
                panic!("handler must not run on proof_invalid");
                #[allow(unreachable_code)]
                Ok::<TrustTask<grant::Response>, ErrorResponse>(unreachable!())
            },
        )
        .await;

        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, StandardCode::ProofInvalid.into());
                let msg = err.payload.message.as_deref().unwrap_or("");
                assert_eq!(msg, crate::PROOF_INVALID_WIRE_MESSAGE);
                // The verifier's own vocabulary must not reach the wire.
                assert!(!msg.contains("signature"), "verifier detail leaked: {msg}");
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    /// Handler receives `ResolvedParties` populated by the transport
    /// when the in-band members are absent (SPEC §4.8.1 fill-in path).
    #[tokio::test]
    async fn resolved_parties_filled_in_from_transport_when_in_band_absent() {
        use crate::handlers::InMemoryHandler;
        use crate::specs::acl::list::v0_1 as list;
        let transport = InMemoryHandler::new()
            .with_local("did:web:maintainer.example")
            .with_peer("did:web:org.example");
        let mut doc = TrustTask::for_payload(
            "req-rp-1",
            list::Payload {
                role: None,
                scope: None,
                direction: None,
                subject_prefix: None,
                page_size: None,
                cursor: None,
                ext: None,
            },
        );
        // acl/list declares its recipient party REQUIRED, so `recipient` must be
        // carried in-band (§7.2 item 5b) — the consumer cross-checks it against
        // the transport-authenticated local VID. `issuer` is omitted and is the
        // member filled from the transport-authenticated peer (§4.8.1), which is
        // the behaviour under test.
        doc.recipient = Some("did:web:maintainer.example".into());

        let outcome: ConsumeOutcome<list::Response> =
            consume_inbound::<_, _, _, StubVerifier, NoValidator, _, _>(
                &transport,
                ProofPolicy::RejectIfPresent,
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                ConsumeChecks::not_consequential(),
                doc,
                "did:web:maintainer.example",
                Utc::now(),
                || "err-rp-1".to_string(),
                |req, parties| async move {
                    assert_eq!(parties.issuer.as_deref(), Some("did:web:org.example"));
                    assert_eq!(
                        parties.recipient.as_deref(),
                        Some("did:web:maintainer.example")
                    );
                    Ok(req.respond_with(
                        "resp-rp-1",
                        list::Response {
                            entries: vec![],
                            cursor: None,
                            redacted_fields: vec![],
                            truncated: false,
                            ext: None,
                        },
                    ))
                },
            )
            .await;

        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
    }

    /// SPEC §8.1 — `identity_mismatch` with no transport-authenticated
    /// sender produces `ConsumeOutcome::Suppressed`, not a routed error
    /// (an addressed error would itself be an oracle).
    #[tokio::test]
    async fn identity_mismatch_with_no_transport_sender_is_suppressed() {
        let mut doc = TrustTask::for_payload(
            "req-im-1",
            crate::specs::acl::list::v0_1::Payload {
                role: None,
                scope: None,
                direction: None,
                subject_prefix: None,
                page_size: None,
                cursor: None,
                ext: None,
            },
        );
        doc.issuer = Some("did:web:attacker.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());

        // Reaching Suppressed needs two things at once: resolve_parties
        // must flag a mismatch, and derive_parties must report no
        // transport-authenticated sender to address the error to. No
        // stock fixture does both — NoopHandler reports no sender but
        // never flags a mismatch, and InMemoryHandler flags a mismatch
        // only once a peer is set, which is itself the sender. So the
        // combination is built inline.

        struct MismatchingNoSenderTransport;
        impl crate::TransportHandler for MismatchingNoSenderTransport {
            fn binding_uri(&self) -> &str {
                "urn:test:mismatching-no-sender"
            }
            fn derive_parties(&self) -> crate::TransportContext {
                // No transport-authenticated sender — this is the
                // condition §8.1 calls out: the consumer SHOULD NOT
                // emit a response addressed to the contested in-band
                // identity.
                crate::TransportContext {
                    issuer: None,
                    recipient: None,
                }
            }
            fn resolve_parties<P>(
                &self,
                doc: &TrustTask<P>,
            ) -> Result<crate::ResolvedParties, crate::ConsistencyError> {
                // Unconditionally flag a mismatch so the test exercises
                // the Suppressed path without depending on the noop
                // resolve_parties shortcut.
                Err(crate::ConsistencyError::IssuerMismatch {
                    in_band: doc
                        .issuer
                        .clone()
                        .unwrap_or_else(|| "did:web:in-band.example".into()),
                    transport: "did:web:transport.example".into(),
                })
            }
        }

        let outcome: ConsumeOutcome<crate::specs::acl::list::v0_1::Response> =
            consume_inbound::<_, _, _, StubVerifier, NoValidator, _, _>(
                &MismatchingNoSenderTransport,
                ProofPolicy::RejectIfPresent,
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                ConsumeChecks::not_consequential(),
                doc,
                "did:web:maintainer.example",
                Utc::now(),
                || "err-im-1".to_string(),
                |_req, _parties| async move {
                    panic!("handler must not run on identity_mismatch");
                    #[allow(unreachable_code)]
                    Ok::<TrustTask<_>, ErrorResponse>(unreachable!())
                },
            )
            .await;

        assert!(
            matches!(outcome, ConsumeOutcome::Suppressed),
            "expected Suppressed under identity_mismatch with no transport-authenticated sender"
        );
    }
}
