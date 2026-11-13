//! [`TspConsumer`] — the guarded inbound path: open the TSP seal, then run the
//! framework's §7.2 pipeline with the duplicate-execution record of item 11
//! and the freshness bound it needs already wired.
//!
//! # Why this type exists
//!
//! [`unpack_trust_task`](crate::unpack_trust_task) stops at the document. The consumer obligations of
//! SPEC §7.2 still have to run over it, and two of them are stateful — item
//! 11's record and the acceptance window that lets that record be dropped.
//! Leaving both to the caller meant the crate shipped a defence nobody
//! switched on, and the failure mode of forgetting is silent: everything works
//! until something re-delivers.
//!
//! **Re-delivery is the point.** `bindings/tsp/0.1` §7 records that
//! relationship-forming carries nonces but "data messages do not inherently
//! prevent replay", and §5.2's routed and nested carriage puts one or more
//! intermediaries on the path — each of which may drop, delay, and re-deliver
//! the sealed inner message it forwards. An intermediary that re-forwards a
//! queued message after an acknowledgement is lost is the most likely
//! duplicate in this system, far more likely than a hostile replay, and SPEC
//! §10.1 is explicit that at the document layer the two are
//! indistinguishable. That is why item 11 is a normative requirement rather
//! than an anti-abuse measure.
//!
//! The record, the freshness bound and the pipeline are
//! [`GuardedConsumer`]'s, shared with the DIDComm bindings; a `TspConsumer`
//! dereferences to one. What is this binding's is opening the seal.
//!
//! # Keyed on the document `id`, never on the envelope
//!
//! SPEC §7.2 (*Keying and comparison for item 11*): the key is the document
//! `id` alone, and "transport request identifiers, transport message
//! identifiers, and execution handles **MUST NOT** substitute".
//!
//! For this binding that rules out anything the TSP envelope carries — the
//! CESR envelope itself, its nonce, and the relationship state that produced
//! it. A re-send seals the same payload afresh: new ephemeral material, new
//! bytes, an envelope that is a different object in every respect while the
//! document inside is identical. Nothing about the envelope repeats, which is
//! exactly why nothing about it can serve as the key.
//!
//! # An authenticated `document` that does not parse is answered
//!
//! Binding §4: a `document` that fails to deserialise is `malformedRequest`.
//! The sender is authenticated, so [`receive`](TspConsumer::receive) answers it
//! as [`ConsumeOutcome::Rejected`], addressed to the verified sender and sealed
//! back with [`pack_trust_task`](crate::pack_trust_task) like any other answer
//! — through 0.38 it came back as `Err` with nothing to answer. Only a
//! refusal of the seal or the envelope is still an `Err(TspError)`.
//!
//! # One consumer per process, not one per message
//!
//! The guard **is** the record. A consumer constructed per inbound message has
//! an empty record every time and absorbs nothing — build one and keep it, or
//! hand every replica a shared store with [`TspConsumer::with_replay_guard`].

use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

use affinidi_tsp::{PrivateVid, ResolvedVid};
use chrono::{DateTime, Utc};
use serde::Serialize;
use trust_tasks_rs::{
    AsyncDispatcher, ConsumeOutcome, ErrorResponse, FreshnessPolicy, GuardedConsumer, Payload,
    PayloadPolicy, PayloadValidator, ProofPolicy, ProofVerifier, ReplayGuard, ResolvedParties,
    TrustTask,
};

use crate::error::TspError;
use crate::pack::open;

/// A TSP *consumer*: the inbound half of the binding over a
/// [`GuardedConsumer`] — SPEC §7.2's duplicate-execution record and freshness
/// bound already wired.
///
/// Construct one and keep it: it holds the record. It dereferences to its
/// [`GuardedConsumer`], whose [`consume`](GuardedConsumer::consume) and
/// [`consume_dispatched`](GuardedConsumer::consume_dispatched) take a document
/// a TSP stack this crate does not drive has already unsealed, so the record
/// covers that path too.
///
/// ```rust,ignore
/// use trust_tasks_tsp::TspConsumer;
/// use trust_tasks_rs::{ConsumeOutcome, NoValidator, PayloadPolicy, ProofPolicy};
///
/// // One per process — the guard *is* the record.
/// static INBOUND: LazyLock<TspConsumer> = LazyLock::new(TspConsumer::new);
///
/// let outcome: ConsumeOutcome<MyResponse> = INBOUND
///     .receive(
///         &wire,
///         &my_vid,
///         &sender_resolved,
///         &my_vid.id,
///         ProofPolicy::Verify(&verifier),
///         PayloadPolicy::<NoValidator>::AcceptUnvalidated,
///         chrono::Utc::now(),
///         || format!("urn:uuid:{}", uuid::Uuid::new_v4()),
///         |doc, parties| async move { Ok(doc.respond_with("resp-1", execute(parties)?)) },
///     )
///     .await?; // only a refusal of the seal: nobody to answer
/// ```
#[derive(Debug, Default)]
pub struct TspConsumer {
    guarded: GuardedConsumer,
}

impl Deref for TspConsumer {
    type Target = GuardedConsumer;

    fn deref(&self) -> &GuardedConsumer {
        &self.guarded
    }
}

impl From<GuardedConsumer> for TspConsumer {
    fn from(guarded: GuardedConsumer) -> Self {
        Self { guarded }
    }
}

impl TspConsumer {
    /// [`GuardedConsumer::new`]: the record **on** (in-process) and
    /// [`FreshnessPolicy::consequential`] applied.
    pub fn new() -> Self {
        GuardedConsumer::new().into()
    }

    /// [`GuardedConsumer::with_replay_guard`]: the shape a replicated
    /// deployment — several replicas serving the same VID — needs.
    pub fn with_replay_guard(guard: Arc<dyn ReplayGuard>) -> Self {
        GuardedConsumer::with_replay_guard(guard).into()
    }

    /// [`GuardedConsumer::without_replay_record`] — **SECURITY: keep no
    /// duplicate-execution record.** `bindings/tsp/0.1` §7 states that TSP
    /// data messages "do not inherently prevent replay", and under §5.2's
    /// routed or nested carriage an intermediary may re-send the sealed inner
    /// message with no attacker involved. Read the linked documentation first.
    #[must_use]
    pub fn without_replay_record() -> Self {
        GuardedConsumer::without_replay_record().into()
    }

    /// Builder: replace the freshness policy — see
    /// [`GuardedConsumer::with_freshness`]. A deployment whose routed path may
    /// hold a message for longer than five minutes widens this.
    #[must_use]
    pub fn with_freshness(self, freshness: FreshnessPolicy) -> Self {
        self.guarded.with_freshness(freshness).into()
    }

    /// Open a TSP-sealed message and run the guarded §7.2 pipeline over the
    /// document it carries.
    ///
    /// Every rule of [`unpack_trust_task`](crate::unpack_trust_task) applies first — HPKE authenticated
    /// decryption against `recipient`'s key, signature verification against
    /// `sender`'s, the `Direct`-carriage check, the cleartext-sender
    /// cross-check and the envelope `type` check. A refusal there comes back
    /// as `Err(TspError)`: there is no authenticated party to route a
    /// framework error response to.
    ///
    /// A `document` that does not deserialise is answered `malformedRequest`
    /// as [`ConsumeOutcome::Rejected`] to the verified sender (binding §4) —
    /// see [`GuardedConsumer::consume_body`]. Beyond that the outcomes are
    /// [`consume_inbound`](trust_tasks_rs::consume_inbound)'s, unchanged —
    /// including [`ConsumeOutcome::Duplicate`], which is **not** an error and
    /// must never be answered with `taskFailed`.
    #[allow(clippy::too_many_arguments)]
    pub async fn receive<P, R, V, W, F, Fut>(
        &self,
        wire: &[u8],
        recipient: &PrivateVid,
        sender: &ResolvedVid,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        handler: F,
    ) -> Result<ConsumeOutcome<R>, TspError>
    where
        P: Payload + Serialize + Send + Sync,
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        let (document, transport) = open(wire, recipient, sender)?;
        Ok(self
            .guarded
            .consume_body(
                &transport,
                document,
                |_| Ok(()),
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
    /// The document is handed to
    /// [`consume_dispatched`](trust_tasks_rs::consume_dispatched), which reads
    /// each specification's declarations off `dispatcher`'s registrations and
    /// runs the same pipeline as [`receive`](Self::receive) — this consumer's
    /// freshness policy and duplicate-execution record included, and every
    /// refusal before the record is written. The proof is verified over the
    /// document as received.
    ///
    /// `context` builds the dispatcher's per-document `Ctx` from the resolved
    /// parties. Every outcome, an error response included, is a
    /// `TrustTask` this crate's [`pack_trust_task`](crate::pack_trust_task)
    /// seals back to the sender.
    #[allow(clippy::too_many_arguments)]
    pub async fn receive_dispatched<Ctx, R, V, W>(
        &self,
        wire: &[u8],
        recipient: &PrivateVid,
        sender: &ResolvedVid,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        dispatcher: &AsyncDispatcher<Ctx, Result<TrustTask<R>, ErrorResponse>>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        context: impl FnOnce(ResolvedParties) -> Ctx,
    ) -> Result<ConsumeOutcome<R>, TspError>
    where
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
    {
        let (document, transport) = open(wire, recipient, sender)?;
        Ok(self
            .guarded
            .consume_body_dispatched(
                &transport,
                document,
                |_| Ok(()),
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
