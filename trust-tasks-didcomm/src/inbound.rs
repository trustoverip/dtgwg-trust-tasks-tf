//! [`DidcommConsumer`] — the guarded inbound path: unpack, then run the
//! framework's §7.2 pipeline with the duplicate-execution record of item 11
//! and the freshness bound it needs already wired.
//!
//! # Why this type exists
//!
//! Through 0.12 this crate stopped at [`unpack_trust_task`]: it handed back a
//! document and a [`DidcommHandler`] and left every consumer to assemble
//! [`consume_inbound`] for itself. Assembling it correctly means choosing a
//! [`ReplayGuard`] and a [`FreshnessPolicy`], and the failure mode of not
//! choosing one is silent — the deployment works, right up until a mediator
//! redelivers.
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
//! failure this module exists to prevent.
//!
//! [`consume_inbound`] takes the `id` from the document it was handed, so a
//! transport identifier is never in a position to substitute. The
//! `redelivery_under_a_fresh_transport_id_is_absorbed` test in
//! `tests/replay.rs` pins that by asserting the two envelopes' DIDComm `@id`s
//! differ before feeding both through this path.
//!
//! # Defaults
//!
//! [`DidcommConsumer::new`] keeps the record **on**, backed by an in-process
//! [`InMemoryReplayGuard`], and applies [`FreshnessPolicy::consequential`]
//! (`issuedAt` required, five-minute acceptance window). Both are fail-closed
//! in the sense this crate's earlier releases established: a consumer that has
//! not thought about item 11 gets it anyway, and one that wants it off says so
//! in a line of code that names what it is giving up
//! ([`DidcommConsumer::without_replay_record`]).
//!
//! # One consumer per process, not one per message
//!
//! The guard **is** the record. A `DidcommConsumer` constructed per inbound
//! message has an empty record every time and absorbs nothing — build one and
//! keep it for the lifetime of the process, or hand every replica a shared
//! store with [`DidcommConsumer::with_replay_guard`] (see
//! [`InMemoryReplayGuard`]'s own note on replication).

use std::future::Future;
use std::sync::Arc;

use affinidi_messaging_didcomm::DIDCommAgent;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use trust_tasks_rs::{
    consume_inbound, ConsumeChecks, ConsumeOutcome, ErrorResponse, FreshnessPolicy,
    InMemoryReplayGuard, Payload, PayloadPolicy, PayloadValidator, ProofPolicy, ProofVerifier,
    ReplayGuard, ReplayPolicy, ResolvedParties, TrustTask,
};

use crate::error::DidcommError;
use crate::handler::DidcommHandler;
use crate::pack::{unpack_trust_task, unpack_trust_task_from, SenderAllowlist};

/// Where a [`DidcommConsumer`] keeps the SPEC §7.2 item 11 record.
enum Record {
    /// The batteries-included default: an in-process LRU.
    InProcess(InMemoryReplayGuard),
    /// A caller-supplied guard — the shape a replicated deployment needs.
    Shared(Arc<dyn ReplayGuard>),
    /// No record at all. Conformant only where the task is not
    /// *consequential*; see [`DidcommConsumer::without_replay_record`].
    Disabled,
}

/// A DIDComm v2.1 *consumer*: the inbound half of the binding with SPEC §7.2's
/// two stateful checks — item 11's duplicate-execution record and the
/// freshness bound that lets the record be dropped — already wired onto it.
///
/// Construct one and keep it: it holds the record. See the
/// [module documentation](self) for why the record is keyed on the document
/// `id` rather than the DIDComm `@id`, and for the mediator-redelivery case
/// that motivates it.
///
/// ```rust,ignore
/// use trust_tasks_didcomm::DidcommConsumer;
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
///     .await?;
///
/// match outcome {
///     ConsumeOutcome::Handled(response) => send(response),
///     ConsumeOutcome::Rejected(error) => send(error),
///     ConsumeOutcome::Suppressed => {}
///     // §7.2: a duplicate is not an error. Return what the first execution
///     // produced, or say nothing — never `taskFailed`.
///     ConsumeOutcome::Duplicate { prior_response: Some(prior), .. } => send_json(prior),
///     ConsumeOutcome::Duplicate { .. } => {}
/// }
/// ```
pub struct DidcommConsumer {
    record: Record,
    freshness: FreshnessPolicy,
}

impl std::fmt::Debug for DidcommConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let record = match &self.record {
            Record::InProcess(guard) => format!("in-process ({} records)", guard.len()),
            Record::Shared(_) => "caller-supplied".to_string(),
            Record::Disabled => "disabled".to_string(),
        };
        f.debug_struct("DidcommConsumer")
            .field("replay_record", &record)
            .field("freshness", &self.freshness)
            .finish()
    }
}

impl Default for DidcommConsumer {
    fn default() -> Self {
        Self::new()
    }
}

impl DidcommConsumer {
    /// A consumer with the duplicate-execution record **on** (in-process) and
    /// [`FreshnessPolicy::consequential`] applied.
    ///
    /// These are the defaults because they are the ones a deployment cannot
    /// discover it is missing: nothing about a consumer without a record looks
    /// wrong until a mediator redelivers an `acl/grant` and it is applied
    /// twice.
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
    /// replicas behind a load balancer each hold their own map, so a document
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
    /// same document delivered twice executes twice: a mediator that
    /// redelivers a queued envelope — which `bindings/didcomm/0.2` §6 says it
    /// may do at any time, and which needs no attacker — grants the ACL entry,
    /// releases the secret, or moves the value a second time, and a replayed
    /// envelope is indistinguishable from that.
    ///
    /// Item 11 permits this only where the task is not *consequential* (SPEC
    /// §2), or where the *Trust Task specification* "explicitly declares
    /// repeated execution safe and intended". Both are properties of the
    /// operation, not of the consumer's convenience.
    ///
    /// The freshness bound drops to [`FreshnessPolicy::default`] with it:
    /// [`FreshnessPolicy::consequential`]'s acceptance window exists to bound
    /// a record that is no longer being kept, and leaving it on would refuse
    /// documents for the sake of a defence that has been turned off. The two
    /// internal-consistency checks — a future-dated document, an empty
    /// validity interval — still apply; no conforming producer emits either.
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
    /// window therefore widens retention with it; there is no second TTL to
    /// configure, and a deployment whose mediator queues for longer than five
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
    ///
    /// Exposed for a caller that drives [`consume_inbound`] itself (a
    /// dispatcher over several payload types, say) and wants the same two
    /// checks without re-deriving them.
    pub fn checks(&self) -> ConsumeChecks<'_> {
        ConsumeChecks {
            freshness: self.freshness,
            replay: match self.replay_guard() {
                Some(guard) => ReplayPolicy::Guard(guard),
                None => ReplayPolicy::NotConsequential,
            },
        }
    }

    /// Unpack an inbound authcrypt envelope and run the guarded §7.2 pipeline
    /// over the document it carries.
    ///
    /// Every conformance rule of [`unpack_trust_task`] applies first — the
    /// envelope must be authcrypt, the `sender_kid` must be qualified, and
    /// where `expected_sender_did` is supplied it is enforced against the DID
    /// that actually opened the envelope. A transport-layer refusal comes back
    /// as `Err(DidcommError)`: there is no document to route a framework error
    /// response to, and inventing one would answer a party that has not been
    /// authenticated.
    ///
    /// Beyond that the arguments are [`consume_inbound`]'s, and its outcomes
    /// are returned unchanged — including
    /// [`ConsumeOutcome::Duplicate`],
    /// which is **not** an error and must never be answered with `taskFailed`.
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
        P: Payload + Serialize + DeserializeOwned + Send + Sync,
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        let (doc, transport) = unpack_trust_task::<P>(wire, agent, expected_sender_did)?;
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

    /// [`receive`](Self::receive) for a consumer that accepts from many peers:
    /// the envelope's `skid` is checked against `allowlist` before anything is
    /// decrypted. See [`unpack_trust_task_from`].
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
        P: Payload + Serialize + DeserializeOwned + Send + Sync,
        R: Serialize,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        let (doc, transport) = unpack_trust_task_from::<P>(wire, agent, allowlist)?;
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

    /// The guarded pipeline over a document the caller unpacked itself.
    ///
    /// Use it where the envelope was opened by an agent this crate does not
    /// drive — a mediator SDK's own delivery loop, most often — so the record
    /// covers that path too. `transport` is the [`DidcommHandler`] the unpack
    /// produced; it supplies the §4.8.1 transport-authenticated identities.
    ///
    /// The verdicts of SPEC §7.2's *Disposition of a duplicate* are applied by
    /// [`consume_inbound`]: `Fresh` dispatches and then records the response;
    /// a duplicate returns
    /// [`ConsumeOutcome::Duplicate`]
    /// carrying the prior response — or `in_flight` where the first execution
    /// has not finished — **without** dispatching; a differing document under
    /// a reused `id` is rejected `idConflict`; and a guard that cannot answer
    /// fails closed as `unavailable` with `retryable = true`, never by
    /// executing.
    #[allow(clippy::too_many_arguments)]
    pub async fn consume<P, R, V, W, F, Fut>(
        &self,
        transport: &DidcommHandler,
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
