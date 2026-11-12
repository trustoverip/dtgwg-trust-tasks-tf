//! Inbound-document orchestration for SPEC.md §7.2: items 2–8, item 11
//! (the duplicate-execution record) and item 13 (the freshness bound).
//!
//! [`consume_inbound`] is the framework-level helper a *consumer* uses to
//! avoid hand-wiring the §7.2 list across every spec they implement. It
//! accepts a typed [`TrustTask<P>`] (item 1 having already been handled by the
//! caller's `serde` pipeline), runs the remaining framework checks, optionally
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
//!         ConsumeOutcome::Suppressed        => {} // identityMismatch w/o transport sender
//!         // §7.2 item 11: already executed. Not an error — return the prior
//!         // result if there is one, otherwise emit nothing.
//!         ConsumeOutcome::Duplicate { prior_response, .. } => {
//!             if let Some(prior) = prior_response { emit_json(prior) }
//!         }
//!     }
//! }
//! ```
//!
//! The function does not attempt item 1 (framework outer-schema validation);
//! that belongs to the caller's deserialize step. Item 3 is checked here:
//! deserializing into `TrustTask<P>` does not compare `type` with `P`, so
//! [`consume_inbound`] refuses a document whose `type` is neither `P`'s Type
//! URI nor a later version declaring an extension chain to it (SPEC §5.2) —
//! `unsupportedVersion` for the same slug, `unsupportedType` otherwise.
//!
//! Item 2 (payload schema) is deliberately *not* in that list, though it once
//! was. Deserializing into `P` performs most of it — required members, member
//! types, `additionalProperties`, and the string constraints typify emits as
//! validating newtypes — but not the constraints a Rust type cannot carry
//! (`minProperties`, `minItems` on an optional array, conditional subschemas).
//! Treating the whole of item 2 as "already succeeded" therefore overstated
//! what the parse step had established, so the residue is now a policy the
//! caller chooses: see [`PayloadPolicy`].

use std::collections::BTreeSet;
use std::future::Future;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::async_dispatch::AsyncDispatcher;
use crate::dispatcher::{build_error_response, RequestOrigin};
use crate::document::{ErrorResponse, TrustTask};
use crate::error::{ErrorPayload, RejectReason};
use crate::freshness::{FreshnessPolicy, StaleReason};
use crate::payload::{Payload, SpecPolicy};
use crate::proof::{ProofVerifier, VerificationError};
use crate::replay::{
    document_digest, received_document_digest, DocumentDigest, ReplayGuard, ReplayPolicy,
    ReplayVerdict,
};
use crate::transport::{ResolvedParties, TransportHandler};

/// The two stateful consumer checks SPEC.md §7.2 requires and that no runtime
/// in this repo implemented before 0.11.18: the freshness bound over
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
    /// Bounds the document in time (SPEC §4.2; §7.2 item 4, expiry, and
    /// item 13, the acceptance window over `issuedAt`).
    pub freshness: FreshnessPolicy,
    /// Applies (or knowingly disapplies) the duplicate-execution rule of
    /// SPEC §7.2 item 11.
    pub replay: ReplayPolicy<'a>,
    /// Refuse, with `proofRequired`, a document attributable to nobody: one
    /// that carries no `proof` and arrived over a transport that
    /// authenticated no sender ([`TransportContext::issuer`](crate::TransportContext::issuer)
    /// is `None`). Off by default.
    ///
    /// With no authenticated sender and no proof, SPEC §4.8.1 resolves the
    /// issuer to whatever in-band `issuer` string the sender typed, and every
    /// authorization decision a handler makes from it is the sender's to
    /// choose. A binding that authenticates some senders and not others — the
    /// HTTPS binding, where a bearer token is optional on the wire — turns
    /// this on; a message binding whose transport always authenticates the
    /// sender has nothing for it to refuse. Runs after identity resolution and
    /// before the proof is looked at. See [`Self::require_attribution`].
    pub require_attribution: bool,
    /// The DID methods `proof.verificationMethod` may name, checked **before**
    /// the proof policy runs, so an unacceptable method never reaches a
    /// verifier — and so never reaches the resolver behind it, which for
    /// `did:web` is an outbound request to a host the sender chose. `None`, the
    /// default, screens nothing. A `verificationMethod` that is not a DID URL
    /// fails any screen. Refused `proofInvalid`, with a message that does not
    /// name the accepted set. See [`Self::with_proof_did_methods`].
    pub proof_did_methods: Option<BTreeSet<String>>,
    /// The size, in bytes, of the document as received (its UTF-8 JSON text,
    /// `proof` included, transport envelope excluded) — when the binding knows
    /// it. With it, a document larger than its specification's declared
    /// `maxDocumentBytes` ([`Payload::MAX_DOCUMENT_BYTES`]) — or, for a
    /// specification that declares none, than
    /// [`default_max_document_bytes`](Self::default_max_document_bytes) — is
    /// refused `malformedRequest` as soon as the type is known, before anything
    /// else reads it. See [`Self::with_received_bytes`].
    pub received_bytes: Option<usize>,
    /// The bound [`received_bytes`](Self::received_bytes) is held to for a
    /// specification that declares no `maxDocumentBytes`: the binding's own
    /// transport bound. `None` leaves such a document unbounded here.
    pub default_max_document_bytes: Option<usize>,
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
            require_attribution: false,
            proof_did_methods: None,
            received_bytes: None,
            default_max_document_bytes: None,
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
            require_attribution: false,
            proof_did_methods: None,
            received_bytes: None,
            default_max_document_bytes: None,
        }
    }

    /// Builder: override the freshness policy.
    #[must_use]
    pub fn with_freshness(mut self, freshness: FreshnessPolicy) -> Self {
        self.freshness = freshness;
        self
    }

    /// Builder: set [`require_attribution`](Self#structfield.require_attribution).
    #[must_use]
    pub fn require_attribution(mut self, require: bool) -> Self {
        self.require_attribution = require;
        self
    }

    /// Builder: set [`proof_did_methods`](Self#structfield.proof_did_methods)
    /// — the DID methods (`"key"`, `"web"`, …) `proof.verificationMethod` may
    /// name.
    #[must_use]
    pub fn with_proof_did_methods<I, S>(mut self, methods: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.proof_did_methods = Some(methods.into_iter().map(Into::into).collect());
        self
    }

    /// Builder: set [`received_bytes`](Self#structfield.received_bytes) and
    /// [`default_max_document_bytes`](Self#structfield.default_max_document_bytes)
    /// — the received document's size, and the bound for a specification that
    /// declares none.
    #[must_use]
    pub fn with_received_bytes(mut self, bytes: usize, default_bound: Option<usize>) -> Self {
        self.received_bytes = Some(bytes);
        self.default_max_document_bytes = default_bound;
        self
    }

    /// The size refusal for a document whose specification declares
    /// `declared` — `None` when the document is within its bound or no size
    /// is known.
    fn size_rejection(&self, declared: Option<usize>) -> Option<RejectReason> {
        let received = self.received_bytes?;
        let bound = declared.or(self.default_max_document_bytes)?;
        (received > bound).then(|| RejectReason::MalformedRequest {
            reason: "document exceeds the size bound declared for its type".to_string(),
        })
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
    /// `proofInvalid`. This is the safe default for any consumer that
    /// expects to honour in-band proofs.
    Verify(&'a V),

    /// As [`Verify`](Self::Verify), but over the document **as it was
    /// received** — the `&Value` the caller parsed `doc` from — through
    /// [`ProofVerifier::verify_received`].
    ///
    /// Prefer it whenever the received JSON is still in hand, as it is in
    /// every transport binding. [`Verify`](Self::Verify) re-serialises the
    /// typed document, and a genuine proof fails whenever that differs from
    /// what the producer signed: a payload member the type defaults rather
    /// than omits, a member it does not model, a timestamp written `+00:00`.
    ///
    /// The value must be the document `doc` was parsed from.
    /// [`consume_inbound`] re-parses it and refuses (`proofInvalid`) when the
    /// two disagree, since a proof over one document vouches for nothing
    /// about another.
    ///
    /// The §7.2 item 11 digest is taken over this value too, with or without
    /// a proof, so two documents under one `id` that differ only in a member
    /// `P` does not model are an `idConflict`. Under a policy that carries no
    /// received document the digest is of `doc`'s serialisation, which cannot
    /// see such a member.
    VerifyReceived(&'a V, &'a Value),

    /// Reject documents that carry an in-band proof with
    /// `malformedRequest`. Use this when the consumer has integrity
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

    /// [`RejectIfPresent`](Self::RejectIfPresent), carrying the document as
    /// it was received — the `&Value` `doc` was parsed from — so the §7.2
    /// item 11 digest is taken over it rather than over `doc`'s
    /// serialisation. What [`with_received`](Self::with_received) makes of
    /// `RejectIfPresent`.
    RejectIfPresentReceived(&'a Value),

    /// [`AcceptUnverified`](Self::AcceptUnverified), carrying the document as
    /// it was received for the §7.2 item 11 digest. What
    /// [`with_received`](Self::with_received) makes of `AcceptUnverified`.
    AcceptUnverifiedReceived(&'a Value),
}

impl<'a, V: ProofVerifier + ?Sized> ProofPolicy<'a, V> {
    /// Bind this policy to the document as received: [`Verify`](Self::Verify)
    /// becomes [`VerifyReceived`](Self::VerifyReceived) over `received`,
    /// [`RejectIfPresent`](Self::RejectIfPresent) becomes
    /// [`RejectIfPresentReceived`](Self::RejectIfPresentReceived) and
    /// [`AcceptUnverified`](Self::AcceptUnverified) becomes
    /// [`AcceptUnverifiedReceived`](Self::AcceptUnverifiedReceived). A policy
    /// already bound to a received document is returned unchanged.
    ///
    /// For a pipeline that takes a caller's policy and parses the document
    /// itself — every transport binding's `receive` — so the caller asks for
    /// verification and gets it over the bytes that were signed, and the
    /// item-11 record compares those bytes whichever policy was asked for.
    pub fn with_received<'b>(self, received: &'b Value) -> ProofPolicy<'b, V>
    where
        'a: 'b,
    {
        match self {
            ProofPolicy::Verify(v) => ProofPolicy::VerifyReceived(v, received),
            ProofPolicy::RejectIfPresent => ProofPolicy::RejectIfPresentReceived(received),
            ProofPolicy::AcceptUnverified => ProofPolicy::AcceptUnverifiedReceived(received),
            other => other,
        }
    }
}

/// Evaluates a payload against its `payload.schema.json` (SPEC.md §7.2 item 2).
///
/// The framework does not bundle a JSON Schema implementation, for the same
/// reason it does not bundle a cryptosuite: the choice of engine, its draft
/// support, and its resource limits belong to the consumer. Implement this
/// over whichever validator you already run — or, with the `validate`
/// feature, use `validate::SchemaValidator`, an implementation on the
/// `jsonschema` crate that compiles each schema once.
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
    /// [`PayloadPolicy::accept_unvalidated()`], or spelled out,
    /// `PayloadPolicy::<NoValidator>::AcceptUnvalidated`.
    AcceptUnvalidated,
}

impl PayloadPolicy<'static, NoValidator> {
    /// [`AcceptUnvalidated`](Self::AcceptUnvalidated) with the validator type
    /// pinned to [`NoValidator`], so the call needs no turbofish.
    pub const fn accept_unvalidated() -> Self {
        Self::AcceptUnvalidated
    }
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
///     PayloadPolicy::<NoValidator>::AcceptUnvalidated, // or PayloadPolicy::accept_unvalidated()
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
    /// SPEC.md §8.1 routing rule for `identityMismatch`: the in-band
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

/// Run SPEC.md §7.2 items 2–8, 11 and 13 against `doc`, then either call
/// `handler` or build the routed error response per §8.1. Which freshness
/// bound and which duplicate-execution record apply is whatever `checks`
/// carries.
///
/// `P` is the typed payload. Item 1 happens upstream, in the caller's parse;
/// item 3 is checked here, against `P`'s Type URI. Item 2 is *shared*: deserializing into `P` performed
/// most of it, and `payload_policy` decides whether the residue typify cannot
/// express is checked here too — see [`PayloadPolicy`], which is worth
/// reading before choosing.
///
/// `transport` supplies the transport-derived identity for the §4.8.1
/// cross-check (item 6) and the §8.1 routing exception for
/// `identityMismatch`.
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
    // §7.2 item 3. Deserializing into `TrustTask<P>` does not look at `type`,
    // so a document of another specification whose payload happens to fit
    // `P` would otherwise be consumed under `P`'s policy. Accept `P`'s own
    // Type URI, or a later version whose declared extension chain reaches it
    // (SPEC §5.2, as `extension::read_as` reads one); refuse anything else
    // before the record is written, as the dispatched path does.
    if let Err(reason) = crate::extension::reading_of::<P>(&doc.type_uri) {
        return route_rejection(transport, &doc, reason, error_id_factory).into();
    }
    // SPEC §12.4 — the size bound the specification declares, now that the
    // type is known.
    if let Some(reason) = checks.size_rejection(P::MAX_DOCUMENT_BYTES) {
        return route_rejection(transport, &doc, reason, error_id_factory).into();
    }
    consume_core(
        transport,
        policy,
        payload_policy,
        checks,
        doc,
        my_vid,
        now,
        error_id_factory,
        SpecFacts {
            policy: SpecPolicy::of::<P>(),
            schema: P::PAYLOAD_SCHEMA,
        },
        is_received_form_of::<P>,
        handler,
    )
    .await
    .into()
}

/// [`consume_inbound`] for a consumer that takes a document of **any** type
/// and routes it on its `type` with an [`AsyncDispatcher`].
///
/// This is the consumer [`SpecPolicy`] describes: one that holds a
/// `TrustTask<serde_json::Value>`, so it has no `P` to read a specification's
/// declarations from. Here they come from the dispatcher instead — each
/// [`AsyncDispatcher::on_async::<P>`](AsyncDispatcher::on_async) registration
/// records `P`'s payload schema and [`SpecPolicy`] — and the rest is
/// [`consume_inbound`]'s pipeline, run by the same code: freshness, the §4.8.1
/// cross-check, proof verification, the per-spec checks and the §7.2 item 11
/// duplicate-execution record, in the same order.
///
/// # Order
///
/// 1. **Item 3 — routing.** A `type` whose slug no handler is registered for
///    is refused `unsupportedType`; a registered slug at a `MAJOR.MINOR` that
///    neither has a handler nor reaches one along its declared extension chain
///    (SPEC §5.2) is refused `unsupportedVersion`. A forward-routed payload is
///    pruned to what the implemented version knows, as
///    [`AsyncDispatcher::dispatch`] does.
/// 2. **The size bound** — with [`ConsumeChecks::received_bytes`], a document
///    larger than the registered `P`'s `MAX_DOCUMENT_BYTES` (or the default
///    bound, for one that declares none) is `malformedRequest`.
/// 3. **The payload deserializes** into the registered `P` — the part of item
///    2 the typed path performs by parsing — or the document is
///    `malformedRequest`.
/// 4. Everything [`consume_inbound`] does, with the registered `P`'s schema
///    and policy.
/// 5. The handler, through [`AsyncDispatcher::dispatch`], with the context
///    `context` builds from the resolved parties.
///
/// Every refusal in 1–4 happens **before** the record is written, so an
/// unrouteable or malformed document never burns its `id`.
///
/// # Handlers
///
/// The dispatcher's return type is the handler's answer:
/// `Result<TrustTask<R>, ErrorResponse>`, typically with `R = Value` so that
/// handlers for different specifications can share one dispatcher. `context`
/// receives the SPEC §4.8.1-resolved parties — what [`consume_inbound`] hands
/// its handler — and returns the dispatcher's `Ctx`: `|parties| parties`
/// where that is all a handler needs, or a tuple with shared state.
///
/// `error_id_factory` is invoked at most once, for whichever error response
/// is built.
#[allow(clippy::too_many_arguments)]
pub async fn consume_dispatched<Ctx, R, T, V, W>(
    transport: &T,
    policy: ProofPolicy<'_, V>,
    payload_policy: PayloadPolicy<'_, W>,
    checks: ConsumeChecks<'_>,
    dispatcher: &AsyncDispatcher<Ctx, Result<TrustTask<R>, ErrorResponse>>,
    doc: TrustTask<Value>,
    my_vid: &str,
    now: DateTime<Utc>,
    error_id_factory: impl FnOnce() -> String,
    context: impl FnOnce(ResolvedParties) -> Ctx,
) -> ConsumeOutcome<R>
where
    R: Serialize,
    T: TransportHandler + Sync + ?Sized,
    V: ProofVerifier + ?Sized,
    W: PayloadValidator + ?Sized,
{
    consume_dispatched_answer(
        transport,
        policy,
        payload_policy,
        checks,
        dispatcher,
        doc,
        my_vid,
        now,
        error_id_factory,
        context,
    )
    .await
    .into()
}

/// What [`consume_dispatched_answer`] returns: [`ConsumeOutcome`], with the
/// handler's answer in [`Handled`](Self::Handled) whatever its type.
///
/// Every variant but `Handled` means what the same variant of
/// [`ConsumeOutcome`] means.
// `Rejected` is as large as `ConsumeOutcome::Handled`'s response document;
// boxing it would only move the allocation onto every caller, as
// `ConsumeOutcome` does not.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum AnswerOutcome<A> {
    /// Every check passed and the handler answered `A`.
    Handled(A),
    /// See [`ConsumeOutcome::Rejected`].
    Rejected(ErrorResponse),
    /// See [`ConsumeOutcome::Suppressed`].
    Suppressed,
    /// See [`ConsumeOutcome::Duplicate`].
    Duplicate {
        /// The response the first execution produced, if the
        /// [`ReplayGuard`](crate::ReplayGuard) retained one.
        prior_response: Option<Value>,
        /// Whether that first execution is still running.
        in_flight: bool,
    },
}

impl<R> From<AnswerOutcome<TrustTask<R>>> for ConsumeOutcome<R> {
    fn from(outcome: AnswerOutcome<TrustTask<R>>) -> Self {
        match outcome {
            AnswerOutcome::Handled(response) => ConsumeOutcome::Handled(response),
            AnswerOutcome::Rejected(error) => ConsumeOutcome::Rejected(error),
            AnswerOutcome::Suppressed => ConsumeOutcome::Suppressed,
            AnswerOutcome::Duplicate {
                prior_response,
                in_flight,
            } => ConsumeOutcome::Duplicate {
                prior_response,
                in_flight,
            },
        }
    }
}

/// [`consume_dispatched`] for a dispatcher whose handlers answer with any
/// serialisable `A` rather than a response document — most usefully an
/// `Option`, so that one dispatcher can hold handlers for specifications that
/// define a success response and for those that define none (SPEC §4.4.1,
/// fire-and-forget), which acknowledge with `None`.
///
/// The pipeline, its order and its refusals are [`consume_dispatched`]'s; the
/// only difference is what the §7.2 item 11 record retains for a later
/// duplicate. An answer is recorded as its serialisation, except that one
/// which serialises to JSON `null` — `None` — is recorded as *no response*, so
/// a duplicate of an acknowledged fire-and-forget document is
/// `Duplicate { prior_response: None, .. }`: absorbed in silence, as §7.2
/// prescribes, rather than answered with a `null`. A response document never
/// serialises to `null`, so nothing else is affected.
#[allow(clippy::too_many_arguments)]
pub async fn consume_dispatched_answer<Ctx, A, T, V, W>(
    transport: &T,
    policy: ProofPolicy<'_, V>,
    payload_policy: PayloadPolicy<'_, W>,
    checks: ConsumeChecks<'_>,
    dispatcher: &AsyncDispatcher<Ctx, Result<A, ErrorResponse>>,
    mut doc: TrustTask<Value>,
    my_vid: &str,
    now: DateTime<Utc>,
    error_id_factory: impl FnOnce() -> String,
    context: impl FnOnce(ResolvedParties) -> Ctx,
) -> AnswerOutcome<A>
where
    A: Serialize,
    T: TransportHandler + Sync + ?Sized,
    V: ProofVerifier + ?Sized,
    W: PayloadValidator + ?Sized,
{
    // §7.2 item 3. Answered before anything else: there is no specification
    // to apply the remaining checks under.
    let (target, forwarded) = match dispatcher.route(&doc.type_uri) {
        Ok(route) => route,
        Err(reason) => return route_rejection(transport, &doc, reason, error_id_factory),
    };
    let facts = dispatcher.facts(&target);

    // SPEC §12.4 — the size bound the implemented specification declares, the
    // moment the type is known and before anything else reads the document.
    if let Some(reason) = checks.size_rejection(facts.max_document_bytes) {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    let prune = |payload: &mut Value| -> Result<(), RejectReason> {
        match facts.schema {
            Some(schema) if forwarded => crate::extension::prune_forward(payload, schema),
            _ => Ok(()),
        }
    };
    if let Err(reason) = prune(&mut doc.payload) {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // The typed path's parse, done here so a payload the handler's `P` cannot
    // hold is refused before the record rather than after it.
    if let Err(reason) = (facts.check)(&doc.payload) {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // One factory, two possible callers: the pipeline (a refusal before the
    // handler) and the dispatch below (a refusal after it). Exactly one of
    // them can need it, since a pipeline refusal returns without dispatching.
    let ids = std::sync::Mutex::new(Some(error_id_factory));
    let take_id = || {
        ids.lock()
            .ok()
            .and_then(|mut factory| factory.take())
            .map(|factory| factory())
            .unwrap_or_default()
    };

    consume_core(
        transport,
        policy,
        payload_policy,
        checks,
        doc,
        my_vid,
        now,
        take_id,
        SpecFacts {
            policy: facts.policy,
            schema: facts.schema,
        },
        // The proof was made over the document as received; the document being
        // consumed is that one with its payload pruned along a forward route.
        |received: &Value, doc: &TrustTask<Value>| {
            let Ok(mut parsed) = serde_json::from_value::<TrustTask<Value>>(received.clone())
            else {
                return false;
            };
            if prune(&mut parsed.payload).is_err() {
                return false;
            }
            match (serde_json::to_value(&parsed), serde_json::to_value(doc)) {
                (Ok(a), Ok(b)) => a == b,
                _ => false,
            }
        },
        |doc: TrustTask<Value>, parties: ResolvedParties| {
            let origin = RequestOrigin::of(&doc);
            let ctx = context(parties);
            async move {
                match dispatcher.dispatch(doc, ctx).await {
                    Ok(answer) => answer,
                    // Unreachable in practice: routing, the payload parse and
                    // the policy were all checked above, against the same
                    // registration. Answered rather than assumed away.
                    Err(reason) => Err(build_error_response(
                        take_id(),
                        origin,
                        ErrorPayload::from(reason),
                    )),
                }
            }
        },
    )
    .await
}

/// What the per-specification checks of the pipeline read off the type being
/// consumed: from `P` on the typed path, from the dispatcher's registration on
/// the dispatched one. One struct, so the two paths run one body.
struct SpecFacts {
    policy: SpecPolicy,
    schema: Option<&'static str>,
}

/// The body of [`consume_inbound`] and [`consume_dispatched`]. Everything that
/// differs between a typed and a routed document is in `spec` and
/// `is_received_form`; every check, its order, and the item-11 record are
/// here once.
#[allow(clippy::too_many_arguments)]
async fn consume_core<P, S, T, V, W, F, Fut>(
    transport: &T,
    policy: ProofPolicy<'_, V>,
    payload_policy: PayloadPolicy<'_, W>,
    checks: ConsumeChecks<'_>,
    doc: TrustTask<P>,
    my_vid: &str,
    now: DateTime<Utc>,
    error_id_factory: impl FnOnce() -> String,
    spec: SpecFacts,
    is_received_form: impl FnOnce(&Value, &TrustTask<P>) -> bool,
    handler: F,
) -> AnswerOutcome<S>
where
    P: Serialize + Send + Sync,
    S: Serialize,
    T: TransportHandler + Sync + ?Sized,
    V: ProofVerifier + ?Sized,
    W: PayloadValidator + ?Sized,
    F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
    Fut: Future<Output = Result<S, ErrorResponse>>,
{
    let ConsumeChecks {
        freshness,
        replay,
        require_attribution,
        proof_did_methods,
        // Applied by the callers, which know the specification's bound.
        received_bytes: _,
        default_max_document_bytes: _,
    } = checks;
    // §7.2 item 2 — payload schema. Runs first, in the spec's own order, and
    // before any check that consults the payload's meaning: a document whose
    // payload is not the shape the specification declares should be refused as
    // malformed rather than reasoned about. Deserialization into `P` already
    // carried most of this; see `PayloadPolicy` for what is left.
    if let PayloadPolicy::Validate(validator) = payload_policy {
        if let Some(schema) = spec.schema {
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

    // §7.2 item 6 first, when the transport authenticated no sender. The only
    // identity that can then disagree is the recipient, and §8.1 suppresses
    // that refusal: there is nobody to address it to, and a `wrongRecipient`
    // answer (item 5a, below) would tell an unauthenticated prober which VIDs
    // this consumer is not. Run first, a misaddressed document stays
    // indistinguishable from any other suppressed one. With an authenticated
    // sender the checks run in §7.2 order.
    let authenticated = transport.derive_parties().issuer.is_some();
    let mut parties = None;
    if !authenticated {
        match transport.resolve_parties(&doc) {
            Ok(p) => parties = Some(p),
            Err(mismatch) => {
                return route_rejection(
                    transport,
                    &doc,
                    RejectReason::IdentityMismatch(mismatch),
                    error_id_factory,
                );
            }
        }
    }

    // §7.2 items 4 + 5a — expiry and wrong-recipient enforcement.
    if let Err(reason) = doc.validate_basic(now, my_vid) {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // §7.2 item 13 — the freshness bound over `issuedAt`.
    // `validate_basic` honours `expiresAt`, which is optional and which a
    // producer sets for its own reasons; on its own it leaves a document
    // stamped years ago, or years hence, indefinitely acceptable. It is also
    // what bounds the replay record below: §7.2 (*Bounding the record*) makes
    // the acceptance window and the record's retention one bound.
    if let Err(reason) = doc.validate_freshness(now, &freshness) {
        return route_rejection(transport, &doc, reason, error_id_factory);
    }

    // §7.2 item 6 — in-band vs transport-derived identity cross-check, unless
    // it ran first above.
    let parties = match parties {
        Some(p) => p,
        None => match transport.resolve_parties(&doc) {
            Ok(p) => p,
            Err(mismatch) => {
                return route_rejection(
                    transport,
                    &doc,
                    RejectReason::IdentityMismatch(mismatch),
                    error_id_factory,
                );
            }
        },
    };

    // Attribution (`ConsumeChecks::require_attribution`): with neither an
    // authenticated sender nor a proof, the issuer §4.8.1 resolved is a string
    // the sender typed.
    if require_attribution && !authenticated && doc.proof.is_none() {
        return route_rejection(
            transport,
            &doc,
            RejectReason::ProofRequired,
            error_id_factory,
        );
    }

    // The DID-method screen (`ConsumeChecks::proof_did_methods`), ahead of the
    // proof policy so an unacceptable method never reaches a verifier or the
    // resolver behind it.
    if let (Some(allowed), Some(proof)) = (&proof_did_methods, &doc.proof) {
        if !did_method_allowed(&proof.verification_method, allowed) {
            return route_rejection(
                transport,
                &doc,
                RejectReason::ProofInvalid {
                    // Does not name the accepted methods: that set is
                    // deployment configuration, and echoing it would let a
                    // probe fingerprint the fleet.
                    reason: "verification method is not acceptable under this consumer's \
                             proof policy"
                        .to_string(),
                },
                error_id_factory,
            );
        }
    }

    // The document as it arrived, where the policy carries it, and whether it
    // is the one being consumed. The proof is verified over it, and the item-11
    // digest below is taken over it, because both concern the whole received
    // document — members `doc`'s type does not model included.
    let received = match &policy {
        ProofPolicy::VerifyReceived(_, received)
        | ProofPolicy::RejectIfPresentReceived(received)
        | ProofPolicy::AcceptUnverifiedReceived(received) => {
            Some((*received, is_received_form(received, &doc)))
        }
        ProofPolicy::Verify(_) | ProofPolicy::RejectIfPresent | ProofPolicy::AcceptUnverified => {
            None
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
        (ProofPolicy::VerifyReceived(v, _), Some(_)) => {
            let verified = if let Some((received, true)) = received {
                v.verify_received(received).await
            } else {
                Err(VerificationError::Other(
                    "the received document is not the one being consumed".to_string(),
                ))
            };
            if let Err(err) = verified {
                return route_rejection(
                    transport,
                    &doc,
                    proof_error_to_reject(err),
                    error_id_factory,
                );
            }
        }
        (ProofPolicy::RejectIfPresent | ProofPolicy::RejectIfPresentReceived(_), Some(_)) => {
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
        | (ProofPolicy::VerifyReceived(..), None)
        | (ProofPolicy::RejectIfPresent | ProofPolicy::RejectIfPresentReceived(_), None)
        | (ProofPolicy::AcceptUnverified | ProofPolicy::AcceptUnverifiedReceived(_), _) => {}
    }

    // §7.2 items 5b + 7A + 8 — the flag-driven per-spec checks, in one place
    // (`SpecPolicy::enforce`, which `TrustTask::enforce_spec_policy` also runs)
    // shared with binding pipelines such as the HTTPS server, so the check set
    // cannot diverge between paths. The non-typed checks above (expiry,
    // cross-check, proof verification) are applied per-pipeline; this runs
    // after them.
    if let Err(reason) = spec.policy.enforce(&doc) {
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
        // Over the document as received when it is in hand: SPEC §7.2 compares
        // the whole received document, and `doc`'s re-serialisation drops
        // members its type does not model and re-renders the timestamps.
        let digest = match received {
            Some((received, true)) => Ok(received_document_digest(received)),
            _ => document_digest(&doc),
        };
        let digest = match digest {
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

        match guard
            .claim_for(
                parties.issuer.as_deref(),
                &doc.id,
                &digest,
                Some(retain_until),
                now,
            )
            .await
        {
            Ok(ReplayVerdict::Fresh) => claim = Some((guard, digest)),
            Ok(ReplayVerdict::Duplicate {
                prior_response,
                in_flight,
            }) => {
                return AnswerOutcome::Duplicate {
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
    //
    // `claim` is held across this await: if the handler panics, or this
    // future is dropped before the handler returns, its destructor releases
    // the claim (see `HeldClaim`). Once the handler has returned, the claim is
    // settled below exactly once.
    let doc_id = doc.id.clone();
    let claim = claim.map(|(guard, digest)| HeldClaim {
        guard,
        id: doc_id.clone(),
        digest: Some(digest),
    });
    let outcome = handler(doc, parties).await;
    match outcome {
        Ok(response) => {
            if let Some(claim) = claim {
                // The effect has happened, so from here the claim is never
                // released: a resend must be absorbed, not executed again.
                // Best-effort recording: a failure to cache the response cannot
                // un-happen the effect. The record of the *claim* is what item
                // 11 needs and it is already written; all that is lost is the
                // ability to hand the same response back, and the duplicate is
                // still absorbed. Guards should log their own failures — there
                // is nowhere to report this from here.
                let (guard, _) = claim.settle();
                record_settled(guard, &doc_id, &response).await;
            }
            AnswerOutcome::Handled(response)
        }
        Err(error_response) => {
            if let Some(claim) = claim {
                if error_response.payload.retryable {
                    // §8.4: the consumer has just told the producer it MAY
                    // re-send this document bit-for-bit. Holding the claim
                    // would make that retry come back as an absorbed duplicate
                    // carrying the same failure forever — the consumer would
                    // have invited a retry it had already decided to refuse.
                    //
                    // `claim` stays armed across the await, so a pipeline
                    // dropped mid-release still releases through its
                    // destructor, and is disarmed on the very next statement.
                    let _ = claim.guard.release(&doc_id, claim.digest()).await;
                    claim.settle();
                } else {
                    // A non-retryable refusal is final for this document, so
                    // the record stands and a replay is answered with the same
                    // determination rather than re-evaluated.
                    let (guard, _) = claim.settle();
                    record_settled(guard, &doc_id, &error_response).await;
                }
            }
            AnswerOutcome::Rejected(error_response)
        }
    }
}

/// Record `response` against a claim whose handler returned, best-effort.
///
/// A `Serialize` impl that panics still settles the claim — with no response
/// to hand back, so a resend is absorbed rather than left in flight until the
/// record expires — before the panic continues unchanged.
async fn record_settled<T: Serialize>(guard: &dyn ReplayGuard, id: &str, response: &T) {
    let encoded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        serde_json::to_value(response)
    }));
    match encoded {
        Ok(value) => {
            // `null` is the acknowledgement of a specification that defines no
            // success response (`consume_dispatched_answer`): there is no
            // response to hand a duplicate back, only the completion to record.
            let value = value.ok().filter(|v| !v.is_null());
            let _ = guard.record_response(id, value.as_ref()).await;
        }
        Err(panic) => {
            let _ = guard.record_response(id, None).await;
            std::panic::resume_unwind(panic);
        }
    }
}

/// The item-11 claim [`consume_core`] holds while the handler runs.
///
/// Released from its destructor unless [`settle`](Self::settle) took it first.
/// That covers the two ways the handler can fail to return: a panic, which
/// unwinds through the await and drops the claim, and cancellation, where the
/// future running the pipeline is dropped mid-await (a request timeout, a
/// client disconnect). Either way nothing says the effect happened, and
/// holding the claim would answer every resend with an in-flight duplicate
/// until the record expired. The panic itself continues unchanged.
struct HeldClaim<'g> {
    guard: &'g dyn ReplayGuard,
    id: String,
    /// `None` once settled.
    digest: Option<DocumentDigest>,
}

impl<'g> HeldClaim<'g> {
    /// The digest the claim was made with.
    fn digest(&self) -> &DocumentDigest {
        self.digest.as_ref().expect("an armed claim has a digest")
    }

    /// The handler returned and the claim is being recorded or has been
    /// released: disarm the destructor.
    fn settle(mut self) -> (&'g dyn ReplayGuard, DocumentDigest) {
        let digest = self.digest.take().expect("a claim is settled once");
        (self.guard, digest)
    }
}

impl Drop for HeldClaim<'_> {
    fn drop(&mut self) {
        if let Some(digest) = self.digest.take() {
            // A destructor that panics while a handler's panic unwinds aborts
            // the process; a guard that panics here loses only this release.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.guard.release_abandoned(&self.id, &digest);
            }));
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

fn route_rejection<P, A, T>(
    transport: &T,
    doc: &TrustTask<P>,
    reason: RejectReason,
    error_id_factory: impl FnOnce() -> String,
) -> AnswerOutcome<A>
where
    P: Serialize,
    T: TransportHandler + Sync + ?Sized,
{
    match transport.reject(doc, error_id_factory(), reason) {
        Some(error_response) => AnswerOutcome::Rejected(error_response),
        None => AnswerOutcome::Suppressed,
    }
}

/// `true` when `verification_method` is a DID URL whose method is in
/// `allowed`. Anything that is not a DID URL (`did:<method>:<rest>`) fails —
/// an `https://…` verification method would hand the resolver an arbitrary
/// URL directly.
fn did_method_allowed(verification_method: &str, allowed: &BTreeSet<String>) -> bool {
    let Some(rest) = verification_method.strip_prefix("did:") else {
        return false;
    };
    let Some((method, remainder)) = rest.split_once(':') else {
        return false;
    };
    if method.is_empty() || remainder.is_empty() {
        return false;
    }
    allowed.contains(method)
}

/// Whether `received` parses to the same document as `doc`, compared through
/// the typed form both pass through — so a proof checked over `received`
/// covers the document the handler is given.
fn is_received_form_of<P: Payload>(received: &Value, doc: &TrustTask<P>) -> bool {
    let parsed = serde_json::from_value::<TrustTask<P>>(received.clone())
        .ok()
        .or_else(|| {
            // SPEC §5.2 "Declared extension": a document of a later version
            // that extends `P`'s is consumed as `P` once its payload is pruned
            // (`extension::read_as`, as the dispatchers do). Re-derive that
            // reading here so such a document is recognised as the one being
            // consumed — while the verifier is still handed `received`, the
            // document as signed, extension members and all.
            let untyped = serde_json::from_value::<TrustTask<Value>>(received.clone()).ok()?;
            if !crate::extension::extension_reaches(&untyped.type_uri, &P::type_uri()) {
                return None;
            }
            crate::extension::read_as::<P>(untyped).ok()
        });
    match parsed {
        Some(parsed) => match (serde_json::to_value(&parsed), serde_json::to_value(doc)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        },
        None => false,
    }
}

fn proof_error_to_reject(err: VerificationError) -> RejectReason {
    // Every variant of VerificationError maps to ProofInvalid on the wire
    // per SPEC §8.3 + the ProofVerifier module's documented mapping.
    RejectReason::ProofInvalid {
        reason: err.to_string(),
    }
}

#[cfg(all(test, feature = "acl"))]
// its specimens are acl payloads
// The specimens are prebuilt modules, deprecated in favour of generating the
// tasks an application uses (trust-tasks-build).
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::handlers::NoopHandler;
    use crate::proof::Proof;
    use crate::specs::acl::grant::v0_2 as grant;
    use crate::StandardCode;

    fn entry() -> grant::AclEntry {
        serde_json::from_value(serde_json::json!({
            "subject": "did:web:alice.example",
            "role": "admin",
            "act": { "scope": "all" },
            "keys": { "scope": "all" },
            "capabilities": { "scope": "ceiling" }
        }))
        .expect("a valid acl/grant/0.2 entry")
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
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
        doc.proof = Some(dummy_proof());

        let outcome: ConsumeOutcome<grant::Response> = consume_inbound(
            &transport,
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::accept_unvalidated(),
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
                assert_eq!(resp.payload.entry.subject.as_str(), "did:web:alice.example");
            }
            other => panic!("expected Handled, got {other:?}"),
        }
    }

    /// A document of `type_uri` carrying an `acl/grant/0.2` payload, consumed
    /// as `acl/grant/0.2` against `guard`.
    async fn consume_typed_as_grant(
        type_uri: &str,
        guard: &dyn ReplayGuard,
    ) -> ConsumeOutcome<grant::Response> {
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload("req-type", grant_payload());
        doc.type_uri = type_uri.parse().expect("a valid Type URI");
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.issued_at = Some(Utc::now());
        doc.proof = Some(dummy_proof());
        consume_inbound(
            &transport,
            ProofPolicy::Verify(&verifier),
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            ConsumeChecks::consequential(guard),
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err-type".to_string(),
            |req, _parties| async move {
                let entry = req.payload.entry.clone();
                Ok(req.respond_with("resp-type", grant::Response { entry, ext: None }))
            },
        )
        .await
    }

    /// §7.2 item 3 on the typed path: a payload that fits `P` does not make a
    /// document of another specification a document of `P`'s.
    #[tokio::test]
    async fn typed_consume_refuses_a_document_of_another_type() {
        let guard = crate::replay::InMemoryReplayGuard::default();
        let cases = [
            (
                "https://trusttasks.org/spec/chat/message/1.0",
                StandardCode::UnsupportedType,
            ),
            // The same slug under another authority is another specification.
            (
                "https://other.example/spec/acl/grant/0.2",
                StandardCode::UnsupportedType,
            ),
            // The same specification at a version that does not reach 0.2.
            (
                "https://trusttasks.org/spec/acl/grant/9.9",
                StandardCode::UnsupportedVersion,
            ),
            // The response variant is not the request.
            (
                "https://trusttasks.org/spec/acl/grant/0.2#response",
                StandardCode::UnsupportedType,
            ),
        ];
        for (type_uri, code) in cases {
            match consume_typed_as_grant(type_uri, &guard).await {
                ConsumeOutcome::Rejected(err) => {
                    assert_eq!(err.payload.code, code.into(), "for {type_uri}");
                }
                other => panic!("{type_uri}: expected Rejected, got {other:?}"),
            }
        }
        // Refused before the record: the document's own type is still served
        // under the same `id`, and the `#request` spelling is the same type.
        for type_uri in [
            "https://trusttasks.org/spec/acl/grant/0.2#request",
            "https://trusttasks.org/spec/acl/grant/0.2",
        ] {
            let guard = crate::replay::InMemoryReplayGuard::default();
            assert!(
                matches!(
                    consume_typed_as_grant(type_uri, &guard).await,
                    ConsumeOutcome::Handled(_)
                ),
                "{type_uri} is the implemented type"
            );
        }
        assert!(matches!(
            consume_typed_as_grant("https://trusttasks.org/spec/acl/grant/0.2", &guard).await,
            ConsumeOutcome::Handled(_)
        ));
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
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
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
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
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
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
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
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
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
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
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
        use crate::specs::acl::list::v0_3 as list;
        let transport = NoopHandler::new();
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = TrustTask::for_payload("req-rec-1", list::Payload::default());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
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
                        next_cursor: None,
                        redacted_fields: vec![],
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
        use crate::specs::acl::list::v0_3 as list;
        let transport = NoopHandler::new();
        let mut doc = TrustTask::for_payload("req-au-1", list::Payload::default());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
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
                            next_cursor: None,
                            redacted_fields: vec![],
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
    /// This test asserted the opposite until 0.11.18. The verifier's text is
    /// the only place a consumer's DID-resolution behaviour is described in
    /// English, and the party receiving it is by construction unauthenticated
    /// — the proof did not verify. SPEC §12.4 extends the §8.1
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
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());
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
        use crate::specs::acl::list::v0_3 as list;
        let transport = InMemoryHandler::new()
            .with_local("did:web:maintainer.example")
            .with_peer("did:web:org.example");
        let mut doc = TrustTask::for_payload("req-rp-1", list::Payload::default());
        // acl/list declares its recipient party REQUIRED, so `recipient` must be
        // carried in-band (§7.2 item 5b) — the consumer cross-checks it against
        // the transport-authenticated local VID. `issuer` is omitted and is the
        // member filled from the transport-authenticated peer (§4.8.1), which is
        // the behaviour under test.
        doc.recipient = Some("did:web:maintainer.example".into());
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());

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
                            next_cursor: None,
                            redacted_fields: vec![],
                            ext: None,
                        },
                    ))
                },
            )
            .await;

        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
    }

    /// SPEC §8.1 — `identityMismatch` with no transport-authenticated
    /// sender produces `ConsumeOutcome::Suppressed`, not a routed error
    /// (an addressed error would itself be an oracle).
    #[tokio::test]
    async fn identity_mismatch_with_no_transport_sender_is_suppressed() {
        let mut doc = TrustTask::for_payload(
            "req-im-1",
            crate::specs::acl::list::v0_3::Payload::default(),
        );
        doc.issuer = Some("did:web:attacker.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        // acl/grant declares issuedAtRequirement: REQUIRED (SPEC §7.3
        // item 17), so a document meant to be judged on any other ground has
        // to carry a stamped instant to be placed in an acceptance window.
        doc.issued_at = Some(Utc::now());

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

        let outcome: ConsumeOutcome<crate::specs::acl::list::v0_3::Response> =
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

    // ── The options binding pipelines add (ConsumeChecks) ──────────────────

    fn list_doc(id: &str) -> TrustTask<crate::specs::acl::list::v0_3::Payload> {
        let mut doc = TrustTask::for_payload(id, crate::specs::acl::list::v0_3::Payload::default());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.issued_at = Some(Utc::now());
        doc
    }

    fn list_response() -> crate::specs::acl::list::v0_3::Response {
        crate::specs::acl::list::v0_3::Response {
            entries: vec![],
            next_cursor: None,
            redacted_fields: vec![],
            ext: None,
        }
    }

    async fn consume_list(
        transport: &crate::handlers::InMemoryHandler,
        policy: ProofPolicy<'_, StubVerifier>,
        checks: ConsumeChecks<'_>,
        doc: TrustTask<crate::specs::acl::list::v0_3::Payload>,
    ) -> ConsumeOutcome<crate::specs::acl::list::v0_3::Response> {
        consume_inbound(
            transport,
            policy,
            PayloadPolicy::accept_unvalidated(),
            checks,
            doc,
            "did:web:maintainer.example",
            Utc::now(),
            || "err".to_string(),
            |req, _parties| async move { Ok(req.respond_with("resp", list_response())) },
        )
        .await
    }

    fn code_of<R: std::fmt::Debug>(outcome: ConsumeOutcome<R>) -> crate::TrustTaskCode {
        match outcome {
            ConsumeOutcome::Rejected(err) => err.payload.code,
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    /// `require_attribution`: no authenticated sender and no proof is
    /// `proofRequired`; either one on its own passes.
    #[tokio::test]
    async fn require_attribution_refuses_a_document_attributable_to_nobody() {
        use crate::handlers::InMemoryHandler;
        let verifier = StubVerifier { outcome: Ok(()) };
        let anonymous = InMemoryHandler::new().with_local("did:web:maintainer.example");
        let checks = || ConsumeChecks::not_consequential().require_attribution(true);

        let outcome = consume_list(
            &anonymous,
            ProofPolicy::Verify(&verifier),
            checks(),
            list_doc("a-1"),
        )
        .await;
        assert_eq!(code_of(outcome), StandardCode::ProofRequired.into());

        // Off by default.
        let outcome = consume_list(
            &anonymous,
            ProofPolicy::Verify(&verifier),
            ConsumeChecks::not_consequential(),
            list_doc("a-2"),
        )
        .await;
        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));

        // A proof attributes it.
        let mut signed = list_doc("a-3");
        signed.proof = Some(dummy_proof());
        let outcome =
            consume_list(&anonymous, ProofPolicy::Verify(&verifier), checks(), signed).await;
        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));

        // So does an authenticated sender.
        let authenticated = InMemoryHandler::new()
            .with_local("did:web:maintainer.example")
            .with_peer("did:web:org.example");
        let outcome = consume_list(
            &authenticated,
            ProofPolicy::Verify(&verifier),
            checks(),
            list_doc("a-4"),
        )
        .await;
        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
    }

    /// `proof_did_methods` refuses an unlisted method before the verifier is
    /// consulted, without naming the accepted set.
    #[tokio::test]
    async fn proof_did_methods_screens_before_the_verifier() {
        use crate::handlers::InMemoryHandler;
        let transport = InMemoryHandler::new()
            .with_local("did:web:maintainer.example")
            .with_peer("did:web:org.example");
        // A verifier that would pass: the screen is what refuses.
        let verifier = StubVerifier { outcome: Ok(()) };
        let mut doc = list_doc("d-1");
        doc.proof = Some(dummy_proof()); // did:web:org.example#key-1
        let outcome = consume_list(
            &transport,
            ProofPolicy::Verify(&verifier),
            ConsumeChecks::not_consequential().with_proof_did_methods(["key"]),
            doc,
        )
        .await;
        match outcome {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, StandardCode::ProofInvalid.into());
                let message = err.payload.message.unwrap_or_default();
                assert!(
                    !message.contains("key"),
                    "names the accepted set: {message}"
                );
            }
            other => panic!("expected Rejected, got {other:?}"),
        }

        let mut doc = list_doc("d-2");
        doc.proof = Some(dummy_proof());
        let outcome = consume_list(
            &transport,
            ProofPolicy::Verify(&verifier),
            ConsumeChecks::not_consequential().with_proof_did_methods(["web"]),
            doc,
        )
        .await;
        assert!(matches!(outcome, ConsumeOutcome::Handled(_)));

        assert!(!did_method_allowed(
            "https://evil.example/key",
            &BTreeSet::from(["web".to_string()])
        ));
        assert!(!did_method_allowed(
            "did:web",
            &BTreeSet::from(["web".to_string()])
        ));
    }

    /// `received_bytes` is held to the specification's declared bound, or to
    /// the binding's default for one that declares none, before anything else.
    #[tokio::test]
    async fn received_bytes_are_held_to_the_declared_or_default_bound() {
        use crate::handlers::InMemoryHandler;
        let transport = InMemoryHandler::new()
            .with_local("did:web:maintainer.example")
            .with_peer("did:web:org.example");
        let verifier = StubVerifier { outcome: Ok(()) };
        // acl/list declares no bound, so the default applies.
        assert_eq!(
            <crate::specs::acl::list::v0_3::Payload as Payload>::MAX_DOCUMENT_BYTES,
            None
        );
        let outcome = consume_list(
            &transport,
            ProofPolicy::Verify(&verifier),
            ConsumeChecks::not_consequential().with_received_bytes(2048, Some(1024)),
            list_doc("s-1"),
        )
        .await;
        assert_eq!(code_of(outcome), StandardCode::MalformedRequest.into());

        for checks in [
            ConsumeChecks::not_consequential().with_received_bytes(512, Some(1024)),
            ConsumeChecks::not_consequential().with_received_bytes(1 << 30, None),
            ConsumeChecks::not_consequential(),
        ] {
            let outcome = consume_list(
                &transport,
                ProofPolicy::Verify(&verifier),
                checks,
                list_doc("s-2"),
            )
            .await;
            assert!(matches!(outcome, ConsumeOutcome::Handled(_)));
        }
    }

    /// `consume_dispatched_answer` with an `Option` answer: `None` — an
    /// acknowledgement — is recorded as no response, so its duplicate is
    /// absorbed in silence rather than answered with `null`.
    #[tokio::test]
    async fn an_acknowledgement_is_recorded_as_no_response() {
        use crate::handlers::InMemoryHandler;
        use crate::specs::acl::list::v0_3 as list;
        let transport = InMemoryHandler::new()
            .with_local("did:web:maintainer.example")
            .with_peer("did:web:org.example");
        let guard = crate::replay::InMemoryReplayGuard::default();
        let dispatcher = AsyncDispatcher::<(), Result<Option<Value>, ErrorResponse>>::new()
            .on_async::<list::Payload, _, _>(|_req, ()| async { Ok(None) });
        let doc: TrustTask<Value> =
            serde_json::from_value(serde_json::to_value(list_doc("ack-1")).unwrap()).unwrap();

        let mut outcomes = Vec::new();
        for _ in 0..2 {
            outcomes.push(
                consume_dispatched_answer(
                    &transport,
                    ProofPolicy::<StubVerifier>::RejectIfPresent,
                    PayloadPolicy::accept_unvalidated(),
                    ConsumeChecks::consequential(&guard),
                    &dispatcher,
                    doc.clone(),
                    "did:web:maintainer.example",
                    Utc::now(),
                    || "err".to_string(),
                    |_| (),
                )
                .await,
            );
        }
        assert!(matches!(outcomes[0], AnswerOutcome::Handled(None)));
        assert!(
            matches!(
                outcomes[1],
                AnswerOutcome::Duplicate {
                    prior_response: None,
                    in_flight: false
                }
            ),
            "{:?}",
            outcomes[1]
        );
    }

    /// A verifier that binds the proof to the exact received JSON: its
    /// `proofValue` must be the SHA-256 of the JCS form of the document
    /// without `proof`. Any member dropped or changed between signing and
    /// verifying — pruning included — fails it.
    struct DigestVerifier;

    fn digest_without_proof(doc: &Value) -> String {
        let mut unsigned = doc.clone();
        unsigned.as_object_mut().unwrap().remove("proof");
        crate::sha256_hex(crate::canonical_json(&unsigned).as_bytes())
    }

    #[async_trait::async_trait]
    impl ProofVerifier for DigestVerifier {
        async fn verify<P>(&self, doc: &TrustTask<P>) -> Result<(), VerificationError>
        where
            P: Serialize + Send + Sync,
        {
            self.verify_received(&serde_json::to_value(doc).unwrap())
                .await
        }

        async fn verify_received(&self, received: &Value) -> Result<(), VerificationError> {
            let claimed = received["proof"]["proofValue"].as_str().unwrap_or_default();
            if claimed == digest_without_proof(received) {
                Ok(())
            } else {
                Err(VerificationError::SignatureInvalid)
            }
        }
    }

    /// SPEC §5.2 "Declared extension" through the full pipeline: a signed
    /// document of a (fictional) `acl/grant/0.9` declared to extend `0.2` is
    /// read as `0.2` with its added members pruned, and its proof is verified
    /// over the document **as received** — added members and all — not over
    /// the pruned one, over which it would fail.
    #[tokio::test]
    async fn a_signed_extended_document_verifies_over_what_was_received() {
        crate::extension::test_index::declare(
            "https://trusttasks.org/spec/acl/grant/0.9",
            Some("0.2"),
        );
        let mut doc = TrustTask::for_payload("req-ext", grant_payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        doc.issued_at = Some(Utc::now());
        let mut received = serde_json::to_value(&doc).unwrap();
        received["type"] = Value::from("https://trusttasks.org/spec/acl/grant/0.9");
        received["payload"]["priority"] = Value::from(3);
        received["payload"]["entry"]["note"] = Value::from("added in 0.9");
        let mut proof = serde_json::to_value(dummy_proof()).unwrap();
        proof["proofValue"] = Value::from(digest_without_proof(&received));
        received["proof"] = proof;

        let consume = |received: Value| async move {
            let typed = crate::extension::read_as::<grant::Payload>(
                serde_json::from_value(received.clone()).unwrap(),
            )
            .expect("0.9 reads as 0.2");
            // The typed document is pruned, so the proof does not hold over
            // it — which is why the verifier must never see it.
            let typed_value = serde_json::to_value(&typed).unwrap();
            assert_ne!(
                typed_value["proof"]["proofValue"].as_str().unwrap(),
                digest_without_proof(&typed_value),
            );
            consume_inbound::<_, _, _, _, NoValidator, _, _>(
                &NoopHandler::new(),
                ProofPolicy::VerifyReceived(&DigestVerifier, &received),
                PayloadPolicy::<NoValidator>::AcceptUnvalidated,
                ConsumeChecks::not_consequential(),
                typed,
                "did:web:maintainer.example",
                Utc::now(),
                || "err-ext".to_string(),
                |req, _parties| async move {
                    let entry = req.payload.entry.clone();
                    Ok(req.respond_with("resp-ext", grant::Response { entry, ext: None }))
                },
            )
            .await
        };

        match consume(received.clone()).await {
            ConsumeOutcome::Handled(resp) => assert_eq!(
                resp.type_uri.to_string(),
                "https://trusttasks.org/spec/acl/grant/0.9#response"
            ),
            other => panic!("expected Handled, got {other:?}"),
        }

        // The verifier saw the added members: change one after signing and
        // the proof fails, although pruning would have removed it.
        let mut tampered = received;
        tampered["payload"]["entry"]["note"] = Value::from("changed");
        match consume(tampered).await {
            ConsumeOutcome::Rejected(err) => {
                assert_eq!(err.payload.code, StandardCode::ProofInvalid.into())
            }
            other => panic!("expected Rejected(proofInvalid), got {other:?}"),
        }
    }
}
