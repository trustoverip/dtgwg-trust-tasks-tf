//! [`TspConsumer`] — the guarded inbound path: open the TSP seal, then run the
//! framework's §7.2 pipeline with the duplicate-execution record of item 11
//! and the freshness bound it needs already wired.
//!
//! # Why this type exists
//!
//! [`unpack_trust_task`] stops at the document. The consumer obligations of
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
//! [`consume_inbound`] takes the `id` from the document it was handed, so a
//! transport identifier is never in a position to substitute.
//!
//! # Defaults
//!
//! [`TspConsumer::new`] keeps the record **on**, backed by an in-process
//! [`InMemoryReplayGuard`], and applies [`FreshnessPolicy::consequential`]
//! (`issuedAt` required, five-minute acceptance window).
//! [`TspConsumer::without_replay_record`] is the explicit opt-out and
//! documents what it re-opens.
//!
//! # One consumer per process, not one per message
//!
//! The guard **is** the record. A consumer constructed per inbound message has
//! an empty record every time and absorbs nothing — build one and keep it, or
//! hand every replica a shared store with [`TspConsumer::with_replay_guard`].

use std::future::Future;
use std::sync::Arc;

use affinidi_tsp::{PrivateVid, ResolvedVid};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use trust_tasks_rs::{
    consume_inbound, ConsumeChecks, ConsumeOutcome, ErrorResponse, FreshnessPolicy,
    InMemoryReplayGuard, Payload, PayloadPolicy, PayloadValidator, ProofPolicy, ProofVerifier,
    ReplayGuard, ReplayPolicy, ResolvedParties, TrustTask,
};

use crate::error::TspError;
use crate::handler::TspHandler;
use crate::pack::unpack_trust_task;

/// Where a [`TspConsumer`] keeps the SPEC §7.2 item 11 record.
enum Record {
    /// The batteries-included default: an in-process LRU.
    InProcess(InMemoryReplayGuard),
    /// A caller-supplied guard — the shape a replicated deployment needs.
    Shared(Arc<dyn ReplayGuard>),
    /// No record at all. Conformant only where the task is not
    /// *consequential*; see [`TspConsumer::without_replay_record`].
    Disabled,
}

/// A TSP *consumer*: the inbound half of the binding with SPEC §7.2's two
/// stateful checks — item 11's duplicate-execution record and the freshness
/// bound that lets the record be dropped — already wired onto it.
///
/// Construct one and keep it: it holds the record. See the
/// [module documentation](self) for why the record is keyed on the document
/// `id` rather than on anything the TSP envelope carries.
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
///     .await?;
/// ```
pub struct TspConsumer {
    record: Record,
    freshness: FreshnessPolicy,
}

impl std::fmt::Debug for TspConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let record = match &self.record {
            Record::InProcess(guard) => format!("in-process ({} records)", guard.len()),
            Record::Shared(_) => "caller-supplied".to_string(),
            Record::Disabled => "disabled".to_string(),
        };
        f.debug_struct("TspConsumer")
            .field("replay_record", &record)
            .field("freshness", &self.freshness)
            .finish()
    }
}

impl Default for TspConsumer {
    fn default() -> Self {
        Self::new()
    }
}

impl TspConsumer {
    /// A consumer with the duplicate-execution record **on** (in-process) and
    /// [`FreshnessPolicy::consequential`] applied.
    ///
    /// These are the defaults because they are the ones a deployment cannot
    /// discover it is missing: nothing about a consumer without a record looks
    /// wrong until an intermediary re-forwards a queued message and the task
    /// runs twice.
    pub fn new() -> Self {
        Self {
            record: Record::InProcess(InMemoryReplayGuard::default()),
            freshness: FreshnessPolicy::consequential(),
        }
    }

    /// A consumer whose record is `guard` — the shape a replicated deployment
    /// needs.
    ///
    /// [`InMemoryReplayGuard`] is correct only for a single process: two
    /// replicas serving the same VID each hold their own map, so a document
    /// accepted by one is `Fresh` at the other and the consequential effect
    /// happens twice. A replicated deployment **MUST** pass a guard backed by
    /// a store all its replicas share.
    pub fn with_replay_guard(guard: Arc<dyn ReplayGuard>) -> Self {
        Self {
            record: Record::Shared(guard),
            freshness: FreshnessPolicy::consequential(),
        }
    }

    /// **SECURITY: keep no duplicate-execution record.**
    ///
    /// This re-opens exactly what SPEC §7.2 item 11 closes. With no record the
    /// same document delivered twice executes twice — and this transport
    /// provides nothing to stop that happening by accident: `bindings/tsp/0.1`
    /// §7 states that TSP data messages "do not inherently prevent replay",
    /// and under §5.2's routed or nested carriage an intermediary may re-send
    /// the sealed inner message with no attacker involved.
    ///
    /// Item 11 permits this only where the task is not *consequential* (SPEC
    /// §2), or where the *Trust Task specification* "explicitly declares
    /// repeated execution safe and intended". Both are properties of the
    /// operation, not of the consumer's convenience.
    ///
    /// The freshness bound drops to [`FreshnessPolicy::default`] with it:
    /// [`FreshnessPolicy::consequential`]'s acceptance window exists to bound
    /// a record that is no longer being kept. The two internal-consistency
    /// checks — a future-dated document, an empty validity interval — still
    /// apply; no conforming producer emits either.
    #[must_use]
    pub fn without_replay_record() -> Self {
        Self {
            record: Record::Disabled,
            freshness: FreshnessPolicy::default(),
        }
    }

    /// Builder: replace the freshness policy.
    ///
    /// SPEC §7.2 (*Bounding the record*) makes the acceptance window and the
    /// record's retention **one bound** — [`FreshnessPolicy::record_expiry`]
    /// is what this consumer hands the guard as `retain_until`. Widening the
    /// window widens retention with it; there is no second TTL, and a
    /// deployment whose routed path may hold a message for longer than five
    /// minutes should widen this rather than reach for one.
    #[must_use]
    pub fn with_freshness(mut self, freshness: FreshnessPolicy) -> Self {
        self.freshness = freshness;
        self
    }

    /// The freshness policy in force.
    pub fn freshness(&self) -> &FreshnessPolicy {
        &self.freshness
    }

    /// The guard backing the record, or `None` where the record is disabled.
    pub fn replay_guard(&self) -> Option<&dyn ReplayGuard> {
        match &self.record {
            Record::InProcess(guard) => Some(guard),
            Record::Shared(guard) => Some(guard.as_ref()),
            Record::Disabled => None,
        }
    }

    /// The [`ConsumeChecks`] this consumer applies — the freshness policy and
    /// the replay record as one argument, because SPEC §7.2 ties them
    /// together.
    pub fn checks(&self) -> ConsumeChecks<'_> {
        ConsumeChecks {
            freshness: self.freshness,
            replay: match self.replay_guard() {
                Some(guard) => ReplayPolicy::Guard(guard),
                None => ReplayPolicy::NotConsequential,
            },
        }
    }

    /// Open a TSP-sealed message and run the guarded §7.2 pipeline over the
    /// document it carries.
    ///
    /// Every rule of [`unpack_trust_task`] applies first — HPKE authenticated
    /// decryption against `recipient`'s key, signature verification against
    /// `sender`'s, the `Direct`-carriage check, the cleartext-sender
    /// cross-check and the envelope `type` check. A refusal there comes back
    /// as `Err(TspError)`: there is no authenticated party to route a
    /// framework error response to.
    ///
    /// Beyond that the arguments are [`consume_inbound`]'s and its outcomes
    /// are returned unchanged — including
    /// [`ConsumeOutcome::Duplicate`](trust_tasks_rs::ConsumeOutcome::Duplicate),
    /// which is **not** an error and must never be answered with `taskFailed`.
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
        P: Payload + Serialize + DeserializeOwned + Send + Sync,
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        let (doc, transport) = unpack_trust_task::<P>(wire, recipient, sender)?;
        Ok(self
            .consume(
                &transport,
                doc,
                my_vid,
                proof_policy,
                payload_policy,
                now,
                error_id_factory,
                handler,
            )
            .await)
    }

    /// The guarded pipeline over a document the caller opened itself.
    ///
    /// Use it where a TSP stack this crate does not drive has already
    /// unsealed the message and produced a [`TspHandler`], so the record
    /// covers that path too.
    ///
    /// The verdicts of SPEC §7.2's *Disposition of a duplicate* are applied by
    /// [`consume_inbound`]: `Fresh` dispatches and then records the response;
    /// a duplicate returns
    /// [`ConsumeOutcome::Duplicate`](trust_tasks_rs::ConsumeOutcome::Duplicate)
    /// carrying the prior response — or `in_flight` where the first execution
    /// has not finished — **without** dispatching; a differing document under
    /// a reused `id` is rejected `idConflict`; and a guard that cannot answer
    /// fails closed as `unavailable` with `retryable = true`, never by
    /// executing.
    #[allow(clippy::too_many_arguments)]
    pub async fn consume<P, R, V, W, F, Fut>(
        &self,
        transport: &TspHandler,
        doc: TrustTask<P>,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        handler: F,
    ) -> ConsumeOutcome<R>
    where
        P: Payload + Serialize + Send + Sync,
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        consume_inbound(
            transport,
            proof_policy,
            payload_policy,
            self.checks(),
            doc,
            my_vid,
            now,
            error_id_factory,
            handler,
        )
        .await
    }
}
