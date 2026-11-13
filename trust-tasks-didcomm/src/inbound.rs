//! [`DidcommConsumer`] — the guarded inbound path: unpack, then run the
//! framework's §7.2 pipeline with the duplicate-execution record of item 11
//! and the freshness bound it needs already wired.
//!
//! # Why this type exists
//!
//! Through 0.12 this crate stopped at [`unpack_trust_task`](crate::unpack_trust_task): it handed back a
//! document and a [`DidcommHandler`](crate::DidcommHandler) and left every
//! consumer to assemble [`consume_inbound`](trust_tasks_rs::consume_inbound)
//! for itself. Assembling it correctly means choosing a
//! [`ReplayGuard`] and a
//! [`FreshnessPolicy`], and the failure mode
//! of not choosing one is silent — the deployment works, right up until a
//! mediator redelivers.
//!
//! **The mediator is the point.** `bindings/didcomm/0.2` §6 records that this
//! transport guarantees no freshness and that a mediator "can drop, delay,
//! reorder, and re-deliver". A queued message replayed after an
//! acknowledgement is lost is the single most likely source of a duplicate in
//! a DIDComm deployment — far more likely than a hostile replay — and SPEC
//! §10.1 is explicit that the two are indistinguishable at the document layer,
//! which is why §7.2 item 11 makes absorbing them a normative requirement
//! rather than an anti-abuse measure.
//!
//! The record, the freshness bound and the pipeline are
//! [`GuardedConsumer`]'s, shared with the TSP and DIDComm v1 bindings; a
//! `DidcommConsumer` dereferences to one. What is this binding's is opening
//! the envelope and the §3.1 thread cross-check.
//!
//! # Keyed on the document `id`, never on the transport identifier
//!
//! SPEC §7.2 (*Keying and comparison for item 11*): the key is the document
//! `id` alone, and "transport request identifiers, transport message
//! identifiers, and execution handles **MUST NOT** substitute".
//!
//! For this binding that rules out the DIDComm `@id`, the `thid`, and the
//! `pthid`. It is not a formality: a mediator redelivery is a *fresh* DIDComm
//! message carrying the *same* Trust Task document, so it has a new `@id` —
//! and under [`pack_trust_task`](crate::pack_trust_task) it may carry a new
//! `thid` too, since `thid` falls back to the document `id` only where
//! `threadId` is absent. A record keyed on any of those would see the
//! redelivery as new and execute the task a second time, which is the precise
//! failure this module exists to prevent. The
//! `redelivery_under_a_fresh_transport_id_is_absorbed` test in
//! `tests/replay.rs` pins that by asserting the two envelopes' DIDComm `@id`s
//! differ before feeding both through this path.
//!
//! # An authenticated body that does not parse is answered
//!
//! Binding §4: a `body` that fails to deserialise, and a §3.1 thread
//! disagreement, are `malformedRequest`. The sender is authenticated, so
//! [`receive`](DidcommConsumer::receive) answers them as
//! [`ConsumeOutcome::Rejected`], addressed to the verified sender and packed
//! back with [`pack_trust_task`](crate::pack_trust_task) like any other answer
//! — through 0.38 they came back as `Err` with nothing to answer, and a `?`
//! on `receive` dropped them. Only an envelope-level refusal is still an
//! `Err(DidcommError)`.
//!
//! # One consumer per process, not one per message
//!
//! The guard **is** the record. A `DidcommConsumer` constructed per inbound
//! message has an empty record every time and absorbs nothing — build one and
//! keep it for the lifetime of the process, or hand every replica a shared
//! store with [`DidcommConsumer::with_replay_guard`].

use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

use affinidi_messaging_didcomm::DIDCommAgent;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorResponse, FreshnessPolicy, GuardedConsumer, Payload,
    PayloadPolicy, PayloadValidator, ProofPolicy, ProofVerifier, ReplayGuard, ResolvedParties,
    TrustTask,
};

use crate::error::DidcommError;
use crate::pack::{open, open_from, Opened, SenderAllowlist};

/// A DIDComm v2.1 *consumer*: the inbound half of the binding over a
/// [`GuardedConsumer`] — SPEC §7.2's duplicate-execution record and freshness
/// bound already wired.
///
/// Construct one and keep it: it holds the record. It dereferences to its
/// [`GuardedConsumer`], whose [`consume`](GuardedConsumer::consume) and
/// [`consume_dispatched`](GuardedConsumer::consume_dispatched) take a document
/// the caller unpacked itself — a mediator SDK's own delivery loop, most
/// often — so the record covers that path too.
///
/// ```rust,ignore
/// use trust_tasks_didcomm::{pack_trust_task, DidcommConsumer};
/// use trust_tasks_rs::{ConsumeOutcome, NoValidator, PayloadPolicy, ProofPolicy};
///
/// // One per process — the guard *is* the record.
/// static INBOUND: LazyLock<DidcommConsumer> = LazyLock::new(DidcommConsumer::new);
///
/// let outcome: ConsumeOutcome<MyResponse> = INBOUND
///     .receive(
///         wire,
///         &agent,
///         Some(&peer_did),
///         &my_did,
///         ProofPolicy::Verify(&verifier),
///         PayloadPolicy::<NoValidator>::AcceptUnvalidated,
///         chrono::Utc::now(),
///         || format!("urn:uuid:{}", uuid::Uuid::new_v4()),
///         |doc, parties| async move { Ok(doc.respond_with("resp-1", execute(parties)?)) },
///     )
///     .await?; // only an envelope-level refusal: nobody to answer
///
/// match outcome {
///     ConsumeOutcome::Handled(response) => send(response),
///     // Includes a body that did not parse, addressed to the verified sender.
///     ConsumeOutcome::Rejected(error) => send(error),
///     ConsumeOutcome::Suppressed => {}
///     // §7.2: a duplicate is not an error. Return what the first execution
///     // produced, or say nothing — never `taskFailed`.
///     ConsumeOutcome::Duplicate { prior_response: Some(prior), .. } => send_json(prior),
///     ConsumeOutcome::Duplicate { .. } => {}
/// }
/// ```
#[derive(Debug, Default)]
pub struct DidcommConsumer {
    guarded: GuardedConsumer,
}

impl Deref for DidcommConsumer {
    type Target = GuardedConsumer;

    fn deref(&self) -> &GuardedConsumer {
        &self.guarded
    }
}

impl From<GuardedConsumer> for DidcommConsumer {
    fn from(guarded: GuardedConsumer) -> Self {
        Self { guarded }
    }
}

impl DidcommConsumer {
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
    /// duplicate-execution record.** A mediator that redelivers a queued
    /// envelope, which `bindings/didcomm/0.2` §6 says it may do at any time,
    /// then executes the task twice. Read the linked documentation first.
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

    /// Unpack an inbound authcrypt envelope and run the guarded §7.2 pipeline
    /// over the document it carries.
    ///
    /// Every transport-level rule of [`unpack_trust_task`](crate::unpack_trust_task) applies first — the
    /// envelope must be authcrypt, the `sender_kid` must be qualified, and
    /// where `expected_sender_did` is supplied it is enforced against the DID
    /// that actually opened the envelope. A refusal there comes back as
    /// `Err(DidcommError)`: there is no authenticated party to route a
    /// framework error response to, and inventing one would answer a party
    /// that has not been authenticated.
    ///
    /// A `body` that does not deserialise, or whose thread members disagree
    /// with the envelope's (§3.1), is answered `malformedRequest` as
    /// [`ConsumeOutcome::Rejected`] to the verified sender (binding §4) — see
    /// [`GuardedConsumer::consume_body`]. Beyond that the outcomes are
    /// [`consume_inbound`](trust_tasks_rs::consume_inbound)'s, unchanged —
    /// including [`ConsumeOutcome::Duplicate`], which is **not** an error and
    /// must never be answered with `taskFailed`.
    #[allow(clippy::too_many_arguments)]
    pub async fn receive<P, R, V, W, F, Fut>(
        &self,
        wire: &str,
        agent: &DIDCommAgent,
        expected_sender_did: Option<&str>,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        handler: F,
    ) -> Result<ConsumeOutcome<R>, DidcommError>
    where
        P: Payload + Serialize + Send + Sync,
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        let opened = open(wire, agent, expected_sender_did)?;
        self.consume_opened(
            opened,
            my_vid,
            proof_policy,
            payload_policy,
            now,
            error_id_factory,
            handler,
        )
        .await
    }

    /// [`receive`](Self::receive) for a consumer that accepts from many peers:
    /// the envelope's `skid` is checked against `allowlist` before anything is
    /// decrypted. See [`unpack_trust_task_from`](crate::unpack_trust_task_from).
    #[allow(clippy::too_many_arguments)]
    pub async fn receive_from<P, R, V, W, F, Fut>(
        &self,
        wire: &str,
        agent: &DIDCommAgent,
        allowlist: &SenderAllowlist,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        handler: F,
    ) -> Result<ConsumeOutcome<R>, DidcommError>
    where
        P: Payload + Serialize + Send + Sync,
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        let opened = open_from(wire, agent, allowlist)?;
        self.consume_opened(
            opened,
            my_vid,
            proof_policy,
            payload_policy,
            now,
            error_id_factory,
            handler,
        )
        .await
    }

    /// [`receive`](Self::receive) for a consumer that takes a document of
    /// **any** type and routes it on its `type` — answering a type it has no
    /// handler for with `unsupportedType`, and a known one at an unhandled
    /// version with `unsupportedVersion`.
    ///
    /// The document is handed to
    /// [`consume_dispatched`](trust_tasks_rs::consume_dispatched), which reads
    /// each specification's declarations off `dispatcher`'s registrations and
    /// runs the same pipeline as [`receive`](Self::receive) — this consumer's
    /// freshness policy and duplicate-execution record included, and every
    /// refusal before the record is written. The proof is verified over the
    /// document as received. A body that does not parse, or fails §3.1, is
    /// answered as in [`receive`](Self::receive).
    ///
    /// `context` builds the dispatcher's per-document `Ctx` from the resolved
    /// parties.
    #[allow(clippy::too_many_arguments)]
    pub async fn receive_dispatched<Ctx, R, V, W>(
        &self,
        wire: &str,
        agent: &DIDCommAgent,
        expected_sender_did: Option<&str>,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        dispatcher: &AsyncDispatcher<Ctx, Result<TrustTask<R>, ErrorResponse>>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        context: impl FnOnce(ResolvedParties) -> Ctx,
    ) -> Result<ConsumeOutcome<R>, DidcommError>
    where
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
    {
        let opened = open(wire, agent, expected_sender_did)?;
        self.consume_opened_dispatched(
            opened,
            my_vid,
            proof_policy,
            payload_policy,
            dispatcher,
            now,
            error_id_factory,
            context,
        )
        .await
    }

    /// [`receive_dispatched`](Self::receive_dispatched) for a consumer that
    /// accepts from many peers: the envelope's `skid` is checked against
    /// `allowlist` before anything is decrypted, as in
    /// [`receive_from`](Self::receive_from).
    #[allow(clippy::too_many_arguments)]
    pub async fn receive_from_dispatched<Ctx, R, V, W>(
        &self,
        wire: &str,
        agent: &DIDCommAgent,
        allowlist: &SenderAllowlist,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        dispatcher: &AsyncDispatcher<Ctx, Result<TrustTask<R>, ErrorResponse>>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        context: impl FnOnce(ResolvedParties) -> Ctx,
    ) -> Result<ConsumeOutcome<R>, DidcommError>
    where
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
    {
        let opened = open_from(wire, agent, allowlist)?;
        self.consume_opened_dispatched(
            opened,
            my_vid,
            proof_policy,
            payload_policy,
            dispatcher,
            now,
            error_id_factory,
            context,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn consume_opened<P, R, V, W, F, Fut>(
        &self,
        opened: Opened,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        handler: F,
    ) -> Result<ConsumeOutcome<R>, DidcommError>
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
        } = opened;
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

    #[allow(clippy::too_many_arguments)]
    async fn consume_opened_dispatched<Ctx, R, V, W>(
        &self,
        opened: Opened,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        dispatcher: &AsyncDispatcher<Ctx, Result<TrustTask<R>, ErrorResponse>>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        context: impl FnOnce(ResolvedParties) -> Ctx,
    ) -> Result<ConsumeOutcome<R>, DidcommError>
    where
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
    {
        let Opened {
            body,
            transport,
            threads,
        } = opened;
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
