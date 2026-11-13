//! [`DidcommV1Consumer`] — the guarded inbound path: extract the document from
//! the `~attach` decorator, then run the framework's §7.2 pipeline with the
//! duplicate-execution record of item 11 and the freshness bound it needs
//! already wired.
//!
//! # Why this type exists
//!
//! [`unpack_trust_task`](crate::unpack_trust_task) stops at the document. The
//! consumer obligations of SPEC §7.2 still have to run over it, and two of
//! them are stateful — item 11's record and the acceptance window that lets
//! that record be dropped. Leaving both to the caller meant that the crate
//! shipped a defence nobody switched on, and the failure mode of forgetting is
//! silent: everything works until a mediator redelivers.
//!
//! **The mediator is the point.** `bindings/didcomm-v1/0.2` §6 records that v1
//! offers **no** anti-replay construct and that a mediator "can drop, delay,
//! reorder, and re-deliver". Aries store-and-forward mediation admits
//! arbitrary delay on top. A queued message replayed after an acknowledgement
//! is lost is the most likely duplicate in this system by a wide margin — far
//! more likely than a hostile replay — and SPEC §12.1 is explicit that at the
//! document layer the two are indistinguishable, which is why item 11 is a
//! normative requirement rather than an anti-abuse measure.
//!
//! # Keyed on the document `id`, never on the transport identifier
//!
//! SPEC §7.2 (*Keying and comparison for item 11*): the key is the document
//! `id` alone, and "transport request identifiers, transport message
//! identifiers, and execution handles **MUST NOT** substitute".
//!
//! For this binding that rules out the v1 message `@id` and the `~thread`
//! decorator's `thid` / `pthid`. `MessageV1`'s own documentation makes the
//! same point — "this is the *transport* identifier … a Trust Task document's
//! `id` is a different value with a different lifetime" — and a redelivery
//! demonstrates it: the mediator emits a *new* v1 message, with a freshly
//! generated `@id`, carrying the *same* document in the same attachment. A
//! record keyed on `@id` would see two messages, admit the second, and execute
//! the task twice.
//!
//! [`consume_inbound`](trust_tasks_rs::consume_inbound) takes the `id` from the document it was handed, so a
//! transport identifier is never in a position to substitute.
//!
//! # Defaults
//!
//! [`DidcommV1Consumer::new`] keeps the record **on**, backed by an in-process
//! [`InMemoryReplayGuard`], and applies [`FreshnessPolicy::consequential`]
//! (`issuedAt` required, five-minute acceptance window).
//! [`DidcommV1Consumer::without_replay_record`] is the explicit opt-out and
//! documents what it re-opens.
//!
//! # One consumer per process, not one per message
//!
//! The guard **is** the record. A consumer constructed per inbound message has
//! an empty record every time and absorbs nothing — build one and keep it, or
//! hand every replica a shared store with
//! [`DidcommV1Consumer::with_replay_guard`].
//!
//! # An authenticated attachment that does not parse is answered
//!
//! Binding §4: an attachment that does not deserialise as a document, and a
//! `~thread` disagreement, are `malformedRequest`. The sender is
//! authenticated, so [`receive`](DidcommV1Consumer::receive) answers them as
//! [`ConsumeOutcome::Rejected`], addressed to the authenticated sender —
//! through 0.38 they came back as `Err` with nothing to answer. A message with
//! no Trust Task attachment at all has no document `id` to answer and is still
//! an `Err(DidcommV1Error)`.
//!
//! The record, the freshness bound and the pipeline are
//! [`GuardedConsumer`]'s, shared with the DIDComm v2.1 and TSP bindings; a
//! `DidcommV1Consumer` dereferences to one.

use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

use affinidi_messaging_didcomm_v1::UnpackResult;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
#[cfg(doc)]
use trust_tasks_rs::InMemoryReplayGuard;
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorResponse, FreshnessPolicy, GuardedConsumer, Payload,
    PayloadPolicy, PayloadValidator, ProofPolicy, ProofVerifier, ReplayGuard, ResolvedParties,
    TrustTask,
};

use crate::error::DidcommV1Error;
use crate::pack::{open, Opened};

/// A DIDComm v1 *consumer*: the inbound half of the binding over a
/// [`GuardedConsumer`] — SPEC §7.2's duplicate-execution record and freshness
/// bound already wired.
///
/// Construct one and keep it: it holds the record. See the
/// [module documentation](self) for why the record is keyed on the document
/// `id` rather than the v1 message `@id`. It dereferences to its
/// [`GuardedConsumer`], whose [`consume`](GuardedConsumer::consume) and
/// [`consume_dispatched`](GuardedConsumer::consume_dispatched) take a document
/// an Aries framework's own delivery loop has already extracted, so the
/// record covers that path too.
///
/// ```rust,ignore
/// use trust_tasks_didcomm_v1::DidcommV1Consumer;
/// use trust_tasks_rs::{ConsumeOutcome, NoValidator, PayloadPolicy, ProofPolicy};
///
/// // One per process — the guard *is* the record.
/// static INBOUND: LazyLock<DidcommV1Consumer> = LazyLock::new(DidcommV1Consumer::new);
///
/// let outcome: ConsumeOutcome<MyResponse> = INBOUND
///     .receive(
///         agent.unpack(&wire)?,
///         &my_did,
///         ProofPolicy::Verify(&verifier),
///         PayloadPolicy::<NoValidator>::AcceptUnvalidated,
///         chrono::Utc::now(),
///         || format!("urn:uuid:{}", uuid::Uuid::new_v4()),
///         |doc, parties| async move { Ok(doc.respond_with("resp-1", execute(parties)?)) },
///     )
///     .await?; // only a message-level refusal: nobody to answer
/// ```
#[derive(Debug, Default)]
pub struct DidcommV1Consumer {
    guarded: GuardedConsumer,
}

impl Deref for DidcommV1Consumer {
    type Target = GuardedConsumer;

    fn deref(&self) -> &GuardedConsumer {
        &self.guarded
    }
}

impl From<GuardedConsumer> for DidcommV1Consumer {
    fn from(guarded: GuardedConsumer) -> Self {
        Self { guarded }
    }
}

impl DidcommV1Consumer {
    /// [`GuardedConsumer::new`]: the record **on** (in-process) and
    /// [`FreshnessPolicy::consequential`] applied.
    pub fn new() -> Self {
        GuardedConsumer::new().into()
    }

    /// [`GuardedConsumer::with_replay_guard`]: the shape a replicated
    /// deployment needs.
    pub fn with_replay_guard(guard: Arc<dyn ReplayGuard>) -> Self {
        GuardedConsumer::with_replay_guard(guard).into()
    }

    /// [`GuardedConsumer::without_replay_record`] — **SECURITY: keep no
    /// duplicate-execution record.** `bindings/didcomm-v1/0.2` §6 records that
    /// v1 offers no anti-replay construct and that a mediator may re-deliver,
    /// so the same document then executes twice. Read the linked
    /// documentation first.
    #[must_use]
    pub fn without_replay_record() -> Self {
        GuardedConsumer::without_replay_record().into()
    }

    /// Builder: replace the freshness policy — see
    /// [`GuardedConsumer::with_freshness`]. A deployment whose mediator queues
    /// for longer than five minutes widens this.
    #[must_use]
    pub fn with_freshness(self, freshness: FreshnessPolicy) -> Self {
        self.guarded.with_freshness(freshness).into()
    }

    /// Extract the document from an unpacked v1 message and run the guarded
    /// §7.2 pipeline over it.
    ///
    /// Every message-level rule of [`unpack_trust_task`](crate::unpack_trust_task)
    /// applies first — the authenticated-sender gate (including the
    /// verkey-bound-to-no-known-DID case v1 has and v2.1 does not), the
    /// carriage gate of §2/§2.3 and the attachment lookup. A refusal there
    /// comes back as `Err(DidcommV1Error)`: there is no attributable party, or
    /// no document, to route a framework error response to.
    ///
    /// An attachment that does not deserialise, or whose thread members
    /// disagree with `~thread`, is answered `malformedRequest` as
    /// [`ConsumeOutcome::Rejected`] to the authenticated sender (binding §4) —
    /// see [`GuardedConsumer::consume_body`]. Beyond that the outcomes are
    /// [`consume_inbound`](trust_tasks_rs::consume_inbound)'s, unchanged —
    /// including [`ConsumeOutcome::Duplicate`], which is **not** an error and
    /// must never be answered with `taskFailed`.
    #[allow(clippy::too_many_arguments)]
    pub async fn receive<P, R, V, W, F, Fut>(
        &self,
        unpacked: UnpackResult,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        handler: F,
    ) -> Result<ConsumeOutcome<R>, DidcommV1Error>
    where
        P: Payload + Serialize + Send + Sync,
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        let Opened {
            body,
            transport,
            threads,
        } = open(unpacked)?;
        Ok(self
            .guarded
            .consume_body(
                &transport,
                body,
                |doc: &TrustTask<Value>| threads.check(doc).map_err(|e| e.into_reject_reason()),
                my_vid,
                proof_policy,
                payload_policy,
                now,
                error_id_factory,
                handler,
            )
            .await?)
    }

    /// [`receive`](Self::receive) for a consumer that takes a document of
    /// **any** type and routes it on its `type` — answering a type it has no
    /// handler for with `unsupportedType`, and a known one at an unhandled
    /// version with `unsupportedVersion`.
    ///
    /// The document is extracted untyped and handed to
    /// [`consume_dispatched`](trust_tasks_rs::consume_dispatched), which reads
    /// each specification's declarations off `dispatcher`'s registrations and
    /// runs the same pipeline as [`receive`](Self::receive) — this consumer's
    /// freshness policy and duplicate-execution record included, and every
    /// refusal before the record is written. The proof is verified over the
    /// document as received. An attachment that does not parse, or fails the
    /// `~thread` cross-check, is answered as in [`receive`](Self::receive).
    ///
    /// `context` builds the dispatcher's per-document `Ctx` from the resolved
    /// parties.
    #[allow(clippy::too_many_arguments)]
    pub async fn receive_dispatched<Ctx, R, V, W>(
        &self,
        unpacked: UnpackResult,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        dispatcher: &AsyncDispatcher<Ctx, Result<TrustTask<R>, ErrorResponse>>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        context: impl FnOnce(ResolvedParties) -> Ctx,
    ) -> Result<ConsumeOutcome<R>, DidcommV1Error>
    where
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
    {
        let Opened {
            body,
            transport,
            threads,
        } = open(unpacked)?;
        Ok(self
            .guarded
            .consume_body_dispatched(
                &transport,
                body,
                |doc: &TrustTask<Value>| threads.check(doc).map_err(|e| e.into_reject_reason()),
                my_vid,
                proof_policy,
                payload_policy,
                dispatcher,
                now,
                error_id_factory,
                context,
            )
            .await?)
    }
}
