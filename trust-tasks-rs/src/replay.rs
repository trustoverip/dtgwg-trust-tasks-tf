//! Duplicate-execution protection — SPEC.md §7.2 item 11, §8.4, §10.1.
//!
//! # The rule
//!
//! §7.2 item 11 is normative and unconditional for a *consequential Trust
//! Task*: once a consumer has accepted a document with a given `id` for
//! execution, receiving that same document again **MUST NOT** cause the
//! consequential effect a second time, and receiving a *different* document
//! under the same `id` **MUST** be rejected with `idConflict`.
//!
//! §8.4 is the same mechanism seen from the producer's end. A retry is a
//! bit-for-bit identical resend; retrying is safe *precisely because* item 11
//! obliges the consumer to absorb it. Every transport binding in this repo
//! delegates replay defence to the consumer — `bindings/https/0.2` §5 says
//! "Freshness / replay: None", and the DIDComm and TSP bindings say the same —
//! so if the consumer does not do it, nobody does, and an ordinary mediator
//! redelivery grants an ACL entry or releases a vault secret twice by
//! accident. §10.1 makes the point explicitly: "The rule deliberately does not
//! distinguish a hostile replay from a legitimate transport retry, because at
//! the document layer the two are indistinguishable."
//!
//! # The key: `id`, plus a digest of what was accepted under it
//!
//! §7.2 (*Keying and comparison for item 11*) fixes both halves. The key is
//! the document `id` **alone** — "Transport request identifiers, transport
//! message identifiers, and execution handles **MUST NOT** substitute" — and
//! the comparison is over the document's canonical serialization, because "an
//! `id` alone cannot distinguish the retry it must absorb from the conflict it
//! must reject".
//!
//! [`document_digest`] therefore hashes [`crate::canonical_json`] of the
//! whole document rather than the octets as received. Received bytes
//! are the wrong key: a re-indented body, a member order chosen by an
//! intermediary, or a transport that re-serializes in transit would each make
//! a legitimate §8.4 retry look like a *different* document, and the consumer
//! would answer it with `idConflict` — or execute it.
//!
//! ## Including `proof`, and why that is not the §4.9.3 task digest
//!
//! The digest covers the **entire** document, `proof` included. That is a
//! deliberate difference from the *task digest* of §4.9.3, which is computed
//! over `JCS(document ∖ proof)`, and the spec spells the distinction out:
//!
//! > **This is not the document identity of §7.2 item 11.** The two answer
//! > different questions and are deliberately computed differently. Item 11
//! > and §8.4 ask *which serialization arrived*, so a re-signed `proof` over
//! > identical content makes a different document — that is the `idConflict`
//! > case, and the distinction is the whole point of the rule. A citation asks
//! > *what the document says*, so the same statement signed, unsigned, or
//! > re-signed is one document with one task digest.
//!
//! §8.4 says the same from the other side: "A *producer* that 'retries' by
//! re-signing, re-stamping `issuedAt`, or otherwise altering the bytes has not
//! retried — it has issued a different document under a reused `id`, which
//! item 11 requires the *consumer* to reject with `idConflict`." Stripping
//! `proof` before hashing would silently absorb exactly that case as a retry,
//! which is the one outcome both sections rule out.
//!
//! # Bounding the record
//!
//! §7.2 (*Bounding the record*): the consumer "**MUST** retain the record for
//! at least as long as it remains willing to execute that document", and "the
//! two bounds are the same bound". [`FreshnessPolicy::record_expiry`] computes
//! that instant — `expiresAt` where present (no later than the policy's expiry
//! horizon), otherwise `issuedAt + max_age` —
//! and it is what [`ReplayGuard::claim`] is handed as `retain_until`. This is
//! why the freshness policy and the replay guard are one feature and not two.
//!
//! # Disposition of a duplicate
//!
//! §7.2 (*Disposition of a duplicate*) is explicit that a duplicate is not an
//! error: "In no case is a duplicate reported as `taskFailed`; the task did
//! not fail, it already happened." Where the specification defines a success
//! response, the consumer **SHOULD** return the previously determined result;
//! where it defines none, silence is the correct disposition. That is why
//! [`ReplayVerdict::Duplicate`] carries an optional cached response and why
//! [`ConsumeOutcome::Duplicate`](crate::ConsumeOutcome::Duplicate) is a
//! separate outcome from both `Handled` and `Rejected` — a caller that folded
//! it into `Rejected` would be reporting a failure that did not happen.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::canonical::{canonical_json, sha256_hex};
use crate::document::TrustTask;
use crate::error::RejectReason;
use crate::freshness::FreshnessPolicy;

/// The content identity of a *Trust Task document*, per SPEC §7.2's keying
/// paragraph: SHA-256 over the canonical serialization of the whole document.
///
/// Opaque by construction. Its spelling is an implementation detail of this
/// crate's record-keeping and is never placed on the wire — do not confuse it
/// with the §4.9.3 *task digest*, which is a published, multibase-encoded
/// multihash over `document ∖ proof`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentDigest(String);

impl DocumentDigest {
    /// The digest as a stable, storable string — for a [`ReplayGuard`] backed
    /// by a database, cache, or other external store.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DocumentDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Compute the SPEC §7.2 item 11 document identity for `doc`.
///
/// Fails only if the payload cannot be serialized to JSON, which for a
/// document that arrived over any transport cannot happen — but a
/// caller-constructed `TrustTask<P>` over a hand-written `Serialize` impl can
/// fail, and a guard that silently keyed on a truncated document would be
/// worse than one that refuses.
pub fn document_digest<P: Serialize>(
    doc: &TrustTask<P>,
) -> Result<DocumentDigest, serde_json::Error> {
    let value = serde_json::to_value(doc)?;
    Ok(DocumentDigest(sha256_hex(
        canonical_json(&value).as_bytes(),
    )))
}

/// [`document_digest`] over a document as it arrived, rather than over a
/// typed reading of it: members the reading's type does not model, and the
/// exact spelling of every timestamp, are part of the digest.
///
/// For a pipeline that holds the received JSON, as every transport binding
/// does. The two digests agree whenever the typed reading re-serialises to the
/// received document.
pub fn received_document_digest(received: &Value) -> DocumentDigest {
    DocumentDigest(sha256_hex(canonical_json(received).as_bytes()))
}

/// What a [`ReplayGuard`] says about a document offered for execution.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ReplayVerdict {
    /// This `id` has not been accepted before. The caller may execute, and
    /// **MUST** call [`ReplayGuard::record_response`] afterwards if it wants a
    /// later duplicate answered with the result.
    Fresh,

    /// This `id` was already accepted, under a document with the *same*
    /// digest — a §8.4 retry, or a replay. The caller **MUST NOT** execute
    /// again.
    ///
    /// `prior_response` is the response the first execution produced, where
    /// the guard retained one. `None` means either that the specification
    /// defines no success response (§4.4.1's fire-and-forget case, where
    /// silence is the correct disposition) or that the first execution has not
    /// finished — the two are distinguished by
    /// [`in_flight`](Self::Duplicate::in_flight).
    Duplicate {
        /// The response produced the first time, if one was recorded.
        prior_response: Option<Value>,
        /// `true` while the original execution is still running. SPEC §7.2:
        /// "Where the original execution is still in progress, the *consumer*
        /// **SHOULD** return or expose the existing execution state rather
        /// than begin another."
        in_flight: bool,
    },

    /// This `id` was already accepted, under a document with a *different*
    /// digest. SPEC §7.2 item 11 requires `idConflict`, and requires that the
    /// document **MUST NOT** be treated as a retry of the original.
    Conflict,
}

/// The record could not be consulted or written.
///
/// A guard backed by an external store fails this way when the store is
/// unreachable. Callers **MUST** fail closed: a consumer that cannot establish
/// whether a document is a duplicate has not satisfied item 11, and executing
/// anyway is precisely the double-execution the rule forbids.
/// [`consume_inbound`](crate::consume_inbound) maps this to `unavailable` with
/// `retryable = true`, which is the honest answer — the producer's bit-for-bit
/// resend will be absorbed correctly once the store is back.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("replay record unavailable: {0}")]
pub struct ReplayGuardError(pub String);

impl From<ReplayGuardError> for RejectReason {
    fn from(_: ReplayGuardError) -> Self {
        // The detail stays in the caller's logs: naming the store, its host,
        // or its failure mode on the wire is the §10.4 leak in another
        // costume.
        RejectReason::Unavailable { retry_after: None }
    }
}

/// The consumer-side record that makes SPEC §7.2 item 11 true.
///
/// Object-safe and `async` (via `async-trait`), so a deployment can hand
/// [`consume_inbound`](crate::consume_inbound) a `&dyn ReplayGuard` backed by
/// Redis, Postgres, DynamoDB, or anything else that survives a process
/// restart. [`InMemoryReplayGuard`] is the batteries-included default and is
/// correct for a single-process consumer; it is **not** correct behind a load
/// balancer, where two replicas would each accept the same document once. See
/// its documentation.
#[async_trait::async_trait]
pub trait ReplayGuard: Send + Sync {
    /// Claim `id` for execution on behalf of a document with content identity
    /// `digest`.
    ///
    /// `retain_until` is the instant past which the record may be dropped —
    /// [`FreshnessPolicy::record_expiry`], which SPEC §7.2 makes the same
    /// instant as the end of the consumer's willingness to execute the
    /// document. An implementation **SHOULD** treat a record whose
    /// `retain_until` has passed as absent, so that the key is released rather
    /// than conflicting forever with a document nobody would execute.
    ///
    /// Implementations **MUST** make claim-and-record atomic with respect to
    /// concurrent calls: two simultaneous deliveries of the same document must
    /// not both receive [`ReplayVerdict::Fresh`]. That is the whole guarantee.
    async fn claim(
        &self,
        id: &str,
        digest: &DocumentDigest,
        retain_until: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<ReplayVerdict, ReplayGuardError>;

    /// [`claim`](Self::claim), told which party the document is attributed
    /// to: the issuer SPEC §4.8.1 resolved for it, `None` when there is none.
    ///
    /// The pipeline calls this rather than `claim`, so a guard can share its
    /// capacity out between parties — [`InMemoryReplayGuard`] does, so that
    /// one party cannot hold every slot. The default ignores `issuer` and calls
    /// `claim`, which is what every guard did before this method existed.
    async fn claim_for(
        &self,
        issuer: Option<&str>,
        id: &str,
        digest: &DocumentDigest,
        retain_until: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<ReplayVerdict, ReplayGuardError> {
        let _ = issuer;
        self.claim(id, digest, retain_until, now).await
    }

    /// Attach the response a completed execution produced, so a later
    /// duplicate can be answered with it per SPEC §7.2 (*Disposition of a
    /// duplicate*) rather than merely absorbed in silence.
    ///
    /// Optional: the default implementation records nothing, which yields
    /// `Duplicate { prior_response: None, in_flight: false }` on a re-arrival.
    /// That still satisfies item 11 — the effect does not happen twice — and
    /// is the right shape for a fire-and-forget specification, which has no
    /// response to return.
    async fn record_response(
        &self,
        id: &str,
        response: Option<&Value>,
    ) -> Result<(), ReplayGuardError> {
        let _ = (id, response);
        Ok(())
    }

    /// Release a claim whose execution never began — for example because a
    /// check *after* the claim refused the document.
    ///
    /// Without this, a document rejected downstream of the claim would burn
    /// its `id` and a corrected resend under the same `id` would come back
    /// `idConflict` forever. The default implementation does nothing, which is
    /// safe but leaves that record in place until `retain_until`.
    async fn release(&self, id: &str, digest: &DocumentDigest) -> Result<(), ReplayGuardError> {
        let _ = (id, digest);
        Ok(())
    }

    /// Release a claim whose handler never returned — it panicked, or the
    /// future running [`consume_inbound`](crate::consume_inbound) was dropped
    /// (a request timeout, a client disconnect) while the handler, or the
    /// release of a refused document's claim, was awaited.
    ///
    /// Nothing says the effect happened, and holding the claim would answer
    /// every resend with an in-flight duplicate until `retain_until`. This is
    /// called from a destructor, possibly while a panic unwinds, at most once
    /// per claim, so it cannot `.await`. It **MUST NOT** block, **MUST NOT**
    /// require a runtime context (it may run on a thread with no async runtime
    /// current, or while one shuts down), and **MUST NOT** panic.
    ///
    /// The default polls [`release`](Self::release) once with a no-op waker
    /// and drops it. That completes for a guard whose `release` does not
    /// suspend ([`InMemoryReplayGuard`] among them). For one whose `release`
    /// suspends — a network store — the release is abandoned and the record
    /// stays until `retain_until`: no worse than without this hook, and still
    /// safe for item 11. Such a guard should override this to hand the release
    /// to something that outlives the call, such as a channel to a task the
    /// guard owns, without assuming a runtime is current.
    fn release_abandoned(&self, id: &str, digest: &DocumentDigest) {
        let mut release = self.release(id, digest);
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        let _ = release.as_mut().poll(&mut cx);
    }
}

#[async_trait::async_trait]
impl<T: ReplayGuard + ?Sized> ReplayGuard for &T {
    async fn claim(
        &self,
        id: &str,
        digest: &DocumentDigest,
        retain_until: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<ReplayVerdict, ReplayGuardError> {
        (**self).claim(id, digest, retain_until, now).await
    }

    async fn claim_for(
        &self,
        issuer: Option<&str>,
        id: &str,
        digest: &DocumentDigest,
        retain_until: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<ReplayVerdict, ReplayGuardError> {
        (**self)
            .claim_for(issuer, id, digest, retain_until, now)
            .await
    }

    async fn record_response(
        &self,
        id: &str,
        response: Option<&Value>,
    ) -> Result<(), ReplayGuardError> {
        (**self).record_response(id, response).await
    }

    async fn release(&self, id: &str, digest: &DocumentDigest) -> Result<(), ReplayGuardError> {
        (**self).release(id, digest).await
    }

    fn release_abandoned(&self, id: &str, digest: &DocumentDigest) {
        (**self).release_abandoned(id, digest)
    }
}

/// How a consumer applies SPEC §7.2 item 11 in
/// [`consume_inbound`](crate::consume_inbound).
///
/// The choice is required rather than defaulted, for the reason
/// [`PayloadPolicy`](crate::PayloadPolicy) is: a consumer should decide
/// knowingly whether the task it implements is *consequential*, not discover
/// after an incident that it never kept a record.
#[non_exhaustive]
pub enum ReplayPolicy<'a> {
    /// Apply item 11 using this guard. The correct setting for any
    /// consequential specification.
    Guard(&'a dyn ReplayGuard),

    /// Do not keep a duplicate-execution record.
    ///
    /// Conformant **only** where the task is not consequential (§2), or where
    /// the *Trust Task specification* "explicitly declares repeated execution
    /// safe and intended" — the narrow disapplication item 11 allows, which is
    /// a property of the operation and not of the consumer's convenience.
    NotConsequential,
}

/// A bounded, in-process [`ReplayGuard`]: a map from `id` to the digest
/// accepted under it, its retention deadline, and the response it produced.
///
/// # Suitable when
///
/// One process is the sole consumer for the `recipient` VID it serves, and
/// losing the record on restart is acceptable — that is, the window in which a
/// replay could arrive is shorter than the process's uptime, or the transport
/// will not redeliver across a restart.
///
/// # Not suitable when
///
/// The consumer is replicated. Two replicas behind a load balancer each hold
/// their own map, so the same document accepted by replica A is `Fresh` at
/// replica B and the consequential effect happens twice — the exact failure
/// item 11 exists to prevent. Replicated deployments **MUST** back
/// [`ReplayGuard`] with a store shared by every replica, which is why the seam
/// is a trait.
///
/// # When it is full
///
/// A record is dropped only once its `retain_until` has passed — never to make
/// room. When a new `id` arrives and every one of the `capacity` slots holds a
/// record still inside its retention window, the claim fails with a
/// [`ReplayGuardError`], which the pipeline answers `unavailable` (retryable):
/// the guard refuses to execute a document it could not then remember, rather
/// than forgetting one it promised to. Records whose window has closed are
/// reclaimed first, so the guard is only ever full of live records.
///
/// The capacity is therefore a ceiling on distinct documents *per retention
/// window*, not a cache size. A record lives until the document's `expiresAt`
/// where it has one (capped by the freshness policy's expiry horizon, 30 days
/// by default), and until `issuedAt` + the maximum age otherwise. Size it above
/// the number of distinct documents your consumers will accept inside that
/// window; [`InMemoryReplayGuard::default`] is sized for a short window, and a
/// deployment admitting long-lived documents should either raise it, lower
/// the expiry horizon ([`FreshnessPolicy`]), or back
/// [`ReplayGuard`] with a store that can hold the window.
///
/// A document carrying an `expiresAt` but no `issuedAt` is not bounded by the
/// horizon (it runs from `issuedAt`): under a policy that does not require
/// `issuedAt`, its record lives until its `expiresAt`. The consequential
/// policy requires `issuedAt`.
///
/// # One party's share
///
/// So that one party cannot fill the guard and hold every slot for the length
/// of the expiry horizon — refusing every other party in the meantime — no
/// issuer may hold more than a share of the capacity: a tenth by default
/// ([`with_per_issuer_limit`](Self::with_per_issuer_limit) changes it). A party
/// at its share is refused `unavailable` for its *own* new documents; other
/// parties are unaffected. The issuer is the one the pipeline resolved
/// (SPEC §4.8.1) and passes to [`claim_for`](ReplayGuard::claim_for); documents
/// with no resolved issuer share one allowance, and [`claim`](ReplayGuard::claim)
/// called directly counts against that same allowance. The share is per
/// *identifier*: on a transport where anyone can mint identifiers, a party
/// using many of them is limited only by the overall capacity.
///
/// # Memory
///
/// The guard holds at most `capacity` records, each its `id`, a digest and the
/// response cached for duplicates, so its worst case is roughly `capacity` ×
/// (the largest `id` + the largest response) the transport admits.
pub struct InMemoryReplayGuard {
    capacity: usize,
    per_issuer: usize,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    entries: HashMap<String, Entry>,
    /// Expiry index: (retain_until, sequence) → id, for every record with a
    /// retention deadline. Lets a full guard reclaim the records whose window
    /// has closed without scanning, and keeps `InMemoryReplayGuard`
    /// dependency-free.
    expiry: BTreeMap<(DateTime<Utc>, u64), String>,
    /// Live records per issuer (`None`: no resolved issuer).
    per_issuer: HashMap<Option<String>, usize>,
    seq: u64,
}

struct Entry {
    issuer: Option<String>,
    digest: DocumentDigest,
    retain_until: Option<DateTime<Utc>>,
    response: Option<Value>,
    completed: bool,
    seq: u64,
}

impl InMemoryReplayGuard {
    /// A guard retaining at most `capacity` live records; see
    /// [*When it is full*](InMemoryReplayGuard#when-it-is-full).
    ///
    /// A `capacity` of zero takes [`DEFAULT_REPLAY_CAPACITY`]. It is not
    /// treated as "retain nothing": a guard that retains nothing answers
    /// `Fresh` to every arrival, which is indistinguishable from having no
    /// guard at all — a silent, total defeat of item 11. All four libraries
    /// read a capacity below one this way (`docs/runtime-parity.md`).
    pub fn new(capacity: usize) -> Self {
        let capacity = if capacity == 0 {
            DEFAULT_REPLAY_CAPACITY
        } else {
            capacity
        };
        Self {
            capacity,
            per_issuer: capacity.div_ceil(10),
            inner: Mutex::new(Inner::default()),
        }
    }

    /// Builder: let one issuer hold at most `limit` live records (default: a
    /// tenth of the capacity, rounded up). Clamped to `1..=capacity`; see
    /// [*One party's share*](InMemoryReplayGuard#one-partys-share).
    #[must_use]
    pub fn with_per_issuer_limit(mut self, limit: usize) -> Self {
        self.per_issuer = limit.clamp(1, self.capacity);
        self
    }

    /// Number of records currently retained. Exposed for tests and metrics.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("replay guard mutex").entries.len()
    }

    /// Whether the guard is currently retaining nothing.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Drop every record whose `retain_until` has passed.
    ///
    /// [`claim`](ReplayGuard::claim) already treats an individual expired
    /// record as absent, and reclaims expired records itself when the guard is
    /// full, so calling this is an optimisation (it reclaims memory sooner)
    /// rather than a correctness requirement.
    pub fn purge_expired(&self, now: DateTime<Utc>) {
        self.inner
            .lock()
            .expect("replay guard mutex")
            .purge_expired(now);
    }
}

impl Default for InMemoryReplayGuard {
    /// [`DEFAULT_REPLAY_CAPACITY`] records.
    fn default() -> Self {
        Self::new(DEFAULT_REPLAY_CAPACITY)
    }
}

/// The capacity of [`InMemoryReplayGuard::default`]: 100 000 live records.
///
/// Enough for the consequential policy's five-minute acceptance window at
/// ~330 distinct documents per second. Each record holds the `id`, a digest
/// and the cached response, so the memory it bounds is roughly 100 000 × the
/// size of a typical response. Documents carrying an `expiresAt` hold their
/// record until then (up to the expiry horizon), so a consumer accepting them
/// should size for that window instead — when the guard is full of live
/// records it refuses new documents `unavailable` rather than forgetting one.
pub const DEFAULT_REPLAY_CAPACITY: usize = 100_000;

impl Entry {
    fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        matches!(self.retain_until, Some(t) if t <= now)
    }
}

impl Inner {
    /// Drop `id`'s record if it is an unfinished claim made by `digest`. Only
    /// the claim this digest made: a concurrent arrival that legitimately
    /// holds the key must not have it taken away by a different document's
    /// cleanup.
    fn release(&mut self, id: &str, digest: &DocumentDigest) {
        if self
            .entries
            .get(id)
            .is_some_and(|entry| &entry.digest == digest && !entry.completed)
        {
            self.remove(id);
        }
    }

    fn remove(&mut self, id: &str) {
        if let Some(entry) = self.entries.remove(id) {
            if let Some(until) = entry.retain_until {
                self.expiry.remove(&(until, entry.seq));
            }
            self.forget_issuer(&entry.issuer);
        }
    }

    fn forget_issuer(&mut self, issuer: &Option<String>) {
        if let Some(count) = self.per_issuer.get_mut(issuer) {
            *count -= 1;
            if *count == 0 {
                self.per_issuer.remove(issuer);
            }
        }
    }

    fn held_by(&self, issuer: &Option<String>) -> usize {
        self.per_issuer.get(issuer).copied().unwrap_or(0)
    }

    /// Drop every record whose retention deadline is at or before `now`.
    fn purge_expired(&mut self, now: DateTime<Utc>) {
        while let Some((&(until, _), _)) = self.expiry.first_key_value() {
            if until > now {
                break;
            }
            let Some((_, id)) = self.expiry.pop_first() else {
                break;
            };
            if let Some(entry) = self.entries.remove(&id) {
                self.forget_issuer(&entry.issuer);
            }
        }
    }
}

#[async_trait::async_trait]
impl ReplayGuard for InMemoryReplayGuard {
    async fn claim(
        &self,
        id: &str,
        digest: &DocumentDigest,
        retain_until: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<ReplayVerdict, ReplayGuardError> {
        self.claim_for(None, id, digest, retain_until, now).await
    }

    async fn claim_for(
        &self,
        issuer: Option<&str>,
        id: &str,
        digest: &DocumentDigest,
        retain_until: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<ReplayVerdict, ReplayGuardError> {
        let issuer = issuer.map(str::to_string);
        let mut inner = self.inner.lock().expect("replay guard mutex");

        // An expired record is treated as absent, per the trait contract: the
        // consumer would refuse the document under §7.2 item 4 or the
        // acceptance window anyway, so holding the key would only manufacture
        // a permanent `idConflict` for an `id` nobody can use.
        if inner
            .entries
            .get(id)
            .is_some_and(|entry| entry.is_expired_at(now))
        {
            inner.remove(id);
        }

        if let Some(entry) = inner.entries.get(id) {
            let verdict = if &entry.digest == digest {
                ReplayVerdict::Duplicate {
                    prior_response: entry.response.clone(),
                    in_flight: !entry.completed,
                }
            } else {
                ReplayVerdict::Conflict
            };
            return Ok(verdict);
        }

        // Full: reclaim the records whose window has closed, and if every
        // slot still holds a live record, refuse rather than drop one. A
        // record dropped inside its window is a replay accepted later.
        // Likewise for one issuer's share, so that one party cannot hold
        // every slot and refuse everyone else.
        if inner.entries.len() >= self.capacity || inner.held_by(&issuer) >= self.per_issuer {
            inner.purge_expired(now);
            if inner.entries.len() >= self.capacity {
                return Err(ReplayGuardError(format!(
                    "in-memory replay guard is full: {} records are inside their retention window",
                    inner.entries.len()
                )));
            }
            if inner.held_by(&issuer) >= self.per_issuer {
                return Err(ReplayGuardError(format!(
                    "in-memory replay guard: this issuer holds its share of {} records",
                    self.per_issuer
                )));
            }
        }

        inner.seq += 1;
        let seq = inner.seq;
        inner.entries.insert(
            id.to_string(),
            Entry {
                issuer: issuer.clone(),
                digest: digest.clone(),
                retain_until,
                response: None,
                completed: false,
                seq,
            },
        );
        if let Some(until) = retain_until {
            inner.expiry.insert((until, seq), id.to_string());
        }
        *inner.per_issuer.entry(issuer).or_insert(0) += 1;
        Ok(ReplayVerdict::Fresh)
    }

    async fn record_response(
        &self,
        id: &str,
        response: Option<&Value>,
    ) -> Result<(), ReplayGuardError> {
        let mut inner = self.inner.lock().expect("replay guard mutex");
        if let Some(entry) = inner.entries.get_mut(id) {
            entry.response = response.cloned();
            entry.completed = true;
        }
        Ok(())
    }

    async fn release(&self, id: &str, digest: &DocumentDigest) -> Result<(), ReplayGuardError> {
        let mut inner = self.inner.lock().expect("replay guard mutex");
        inner.release(id, digest);
        Ok(())
    }

    /// Synchronous already, so it releases directly. Called from a destructor,
    /// possibly while a handler's panic unwinds, so a poisoned lock is
    /// recovered rather than turned into a second panic.
    fn release_abandoned(&self, id: &str, digest: &DocumentDigest) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.release(id, digest);
    }
}

/// Convenience: the `retain_until` argument for [`ReplayGuard::claim`],
/// computed from a document and the consumer's freshness policy.
pub fn retain_until<P>(
    doc: &TrustTask<P>,
    policy: &FreshnessPolicy,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    policy.record_expiry(doc, now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TypeUri;
    use serde_json::json;

    fn t(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    fn doc(id: &str, payload: Value) -> TrustTask<Value> {
        TrustTask::new(id, TypeUri::canonical("acl/grant", 0, 1).unwrap(), payload)
    }

    /// A capacity of zero takes the default rather than retaining nothing,
    /// as in the TypeScript, Go and Dart guards (`docs/runtime-parity.md`).
    #[test]
    fn a_zero_capacity_takes_the_default() {
        let guard = InMemoryReplayGuard::new(0);
        assert_eq!(guard.capacity, DEFAULT_REPLAY_CAPACITY);
        assert_eq!(guard.per_issuer, DEFAULT_REPLAY_CAPACITY.div_ceil(10));
    }

    #[test]
    fn digest_ignores_member_order_but_not_content() {
        let a = doc("req-1", json!({"role": "admin", "subject": "alice"}));
        let b = doc("req-1", json!({"subject": "alice", "role": "admin"}));
        let c = doc("req-1", json!({"subject": "mallory", "role": "admin"}));

        assert_eq!(document_digest(&a).unwrap(), document_digest(&b).unwrap());
        assert_ne!(document_digest(&a).unwrap(), document_digest(&c).unwrap());
    }

    /// SPEC §4.9.3: item 11 asks *which serialization arrived*, so a re-signed
    /// `proof` over identical content is a **different** document. This is the
    /// case that must not be quietly absorbed as a retry.
    #[test]
    fn digest_covers_the_proof_member() {
        let mut signed = doc("req-1", json!({"role": "admin"}));
        signed.proof = Some(crate::Proof {
            proof_type: "DataIntegrityProof".into(),
            cryptosuite: "eddsa-jcs-2022".into(),
            verification_method: "did:web:org.example#key-1".into(),
            created: t("2026-08-26T12:00:00Z"),
            proof_purpose: "assertionMethod".into(),
            proof_value: "zAAA".into(),
            extra: Default::default(),
        });

        let mut resigned = signed.clone();
        resigned.proof.as_mut().unwrap().proof_value = "zBBB".into();

        let unsigned = doc("req-1", json!({"role": "admin"}));

        assert_ne!(
            document_digest(&signed).unwrap(),
            document_digest(&resigned).unwrap(),
            "a re-signed proof must make a different document (SPEC §8.4)"
        );
        assert_ne!(
            document_digest(&signed).unwrap(),
            document_digest(&unsigned).unwrap(),
            "stripping the proof must not reproduce the item 11 identity"
        );
    }

    #[tokio::test]
    async fn first_arrival_is_fresh_and_the_identical_resend_is_a_duplicate() {
        let guard = InMemoryReplayGuard::new(8);
        let now = t("2026-08-26T12:00:00Z");
        let d = doc("req-1", json!({"role": "admin"}));
        let digest = document_digest(&d).unwrap();

        assert_eq!(
            guard.claim("req-1", &digest, None, now).await.unwrap(),
            ReplayVerdict::Fresh
        );
        assert_eq!(
            guard.claim("req-1", &digest, None, now).await.unwrap(),
            ReplayVerdict::Duplicate {
                prior_response: None,
                in_flight: true,
            }
        );
    }

    #[tokio::test]
    async fn a_recorded_response_is_returned_to_the_duplicate() {
        let guard = InMemoryReplayGuard::new(8);
        let now = t("2026-08-26T12:00:00Z");
        let digest = document_digest(&doc("req-1", json!({}))).unwrap();

        guard.claim("req-1", &digest, None, now).await.unwrap();
        guard
            .record_response("req-1", Some(&json!({"granted": true})))
            .await
            .unwrap();

        assert_eq!(
            guard.claim("req-1", &digest, None, now).await.unwrap(),
            ReplayVerdict::Duplicate {
                prior_response: Some(json!({"granted": true})),
                in_flight: false,
            }
        );
    }

    #[tokio::test]
    async fn differing_content_under_a_reused_id_conflicts() {
        let guard = InMemoryReplayGuard::new(8);
        let now = t("2026-08-26T12:00:00Z");
        let first = document_digest(&doc("req-1", json!({"role": "reader"}))).unwrap();
        let second = document_digest(&doc("req-1", json!({"role": "admin"}))).unwrap();

        guard.claim("req-1", &first, None, now).await.unwrap();
        assert_eq!(
            guard.claim("req-1", &second, None, now).await.unwrap(),
            ReplayVerdict::Conflict
        );
        // The conflict must not have displaced the record it conflicted with:
        // an attacker who could evict the original by sending a variant would
        // then replay the original successfully.
        assert_eq!(
            guard.claim("req-1", &first, None, now).await.unwrap(),
            ReplayVerdict::Duplicate {
                prior_response: None,
                in_flight: true,
            }
        );
    }

    #[tokio::test]
    async fn the_record_is_released_once_its_retention_deadline_passes() {
        let guard = InMemoryReplayGuard::new(8);
        let issued = t("2026-08-26T12:00:00Z");
        let expiry = t("2026-08-26T12:05:00Z");
        let digest = document_digest(&doc("req-1", json!({}))).unwrap();

        assert_eq!(
            guard
                .claim("req-1", &digest, Some(expiry), issued)
                .await
                .unwrap(),
            ReplayVerdict::Fresh
        );
        // Still inside the window.
        assert!(matches!(
            guard
                .claim("req-1", &digest, Some(expiry), t("2026-08-26T12:04:59Z"))
                .await
                .unwrap(),
            ReplayVerdict::Duplicate { .. }
        ));
        // At the deadline the record is dropped — §7.2 makes retention and
        // willingness-to-execute the same bound, and past it the document is
        // refused under item 4 anyway.
        assert_eq!(
            guard
                .claim("req-1", &digest, Some(expiry), expiry)
                .await
                .unwrap(),
            ReplayVerdict::Fresh
        );
    }

    #[tokio::test]
    async fn purge_expired_reclaims_records() {
        let guard = InMemoryReplayGuard::new(8);
        let now = t("2026-08-26T12:00:00Z");
        let digest = document_digest(&doc("req-1", json!({}))).unwrap();
        guard
            .claim("req-1", &digest, Some(t("2026-08-26T12:05:00Z")), now)
            .await
            .unwrap();
        assert_eq!(guard.len(), 1);

        guard.purge_expired(now);
        assert_eq!(guard.len(), 1);
        guard.purge_expired(t("2026-08-26T13:00:00Z"));
        assert_eq!(guard.len(), 0);
    }

    /// A full guard never drops a record inside its retention window to make
    /// room: it refuses the new document instead, which the pipeline answers
    /// `unavailable`. Dropping one would let that document execute again.
    #[tokio::test]
    async fn a_full_guard_refuses_rather_than_dropping_a_live_record() {
        let guard = InMemoryReplayGuard::new(2).with_per_issuer_limit(2);
        let now = t("2026-08-26T12:00:00Z");
        let later = t("2026-09-20T12:00:00Z");
        let d = |id: &str| document_digest(&doc(id, json!({}))).unwrap();

        guard.claim("a", &d("a"), Some(later), now).await.unwrap();
        guard.claim("b", &d("b"), None, now).await.unwrap();
        let refused = guard.claim("c", &d("c"), Some(later), now).await;
        assert!(refused.is_err(), "expected a refusal, got {refused:?}");
        assert_eq!(
            RejectReason::from(refused.unwrap_err()).code(),
            crate::StandardCode::Unavailable
        );

        // Both records survive, and duplicates are still recognised while full.
        assert_eq!(guard.len(), 2);
        for id in ["a", "b"] {
            assert!(matches!(
                guard.claim(id, &d(id), Some(later), now).await.unwrap(),
                ReplayVerdict::Duplicate { .. }
            ));
        }
    }

    /// Records whose window has closed are reclaimed when the guard is full,
    /// so a full guard holds only live records.
    #[tokio::test]
    async fn a_full_guard_reclaims_expired_records_first() {
        let guard = InMemoryReplayGuard::new(2).with_per_issuer_limit(2);
        let now = t("2026-08-26T12:00:00Z");
        let soon = t("2026-08-26T12:05:00Z");
        let later = t("2026-08-26T13:00:00Z");
        let d = |id: &str| document_digest(&doc(id, json!({}))).unwrap();

        guard.claim("a", &d("a"), Some(soon), now).await.unwrap();
        guard.claim("b", &d("b"), Some(later), now).await.unwrap();
        let after_a = t("2026-08-26T12:10:00Z");
        assert_eq!(
            guard
                .claim("c", &d("c"), Some(later), after_a)
                .await
                .unwrap(),
            ReplayVerdict::Fresh
        );
        assert_eq!(guard.len(), 2);
        assert!(matches!(
            guard
                .claim("b", &d("b"), Some(later), after_a)
                .await
                .unwrap(),
            ReplayVerdict::Duplicate { .. }
        ));
    }

    /// One issuer at its share is refused for its own new documents only;
    /// every other party keeps its access, and the share frees as records
    /// expire.
    #[tokio::test]
    async fn one_issuer_cannot_hold_every_slot() {
        let guard = InMemoryReplayGuard::new(10);
        let now = t("2026-08-26T12:00:00Z");
        let month = t("2026-09-25T12:00:00Z");
        let d = |id: &str| document_digest(&doc(id, json!({}))).unwrap();

        // A tenth of 10 is 1.
        assert_eq!(
            guard
                .claim_for(Some("did:example:a"), "a1", &d("a1"), Some(month), now)
                .await
                .unwrap(),
            ReplayVerdict::Fresh
        );
        let refused = guard
            .claim_for(Some("did:example:a"), "a2", &d("a2"), Some(month), now)
            .await;
        assert_eq!(
            RejectReason::from(refused.unwrap_err()).code(),
            crate::StandardCode::Unavailable
        );
        // Its own record is still recognised.
        assert!(matches!(
            guard
                .claim_for(Some("did:example:a"), "a1", &d("a1"), Some(month), now)
                .await
                .unwrap(),
            ReplayVerdict::Duplicate { .. }
        ));
        // Another party, and the no-issuer allowance, are unaffected.
        for (issuer, id) in [(Some("did:example:b"), "b1"), (None, "n1")] {
            assert_eq!(
                guard
                    .claim_for(issuer, id, &d(id), Some(month), now)
                    .await
                    .unwrap(),
                ReplayVerdict::Fresh
            );
        }
        // Once its record expires the share frees.
        let later = t("2026-09-26T12:00:00Z");
        assert_eq!(
            guard
                .claim_for(Some("did:example:a"), "a2", &d("a2"), None, later)
                .await
                .unwrap(),
            ReplayVerdict::Fresh
        );
        // And the limit is configurable.
        let wide = InMemoryReplayGuard::new(10).with_per_issuer_limit(3);
        for id in ["x1", "x2", "x3"] {
            wide.claim_for(Some("did:example:a"), id, &d(id), None, now)
                .await
                .unwrap();
        }
        assert!(wide
            .claim_for(Some("did:example:a"), "x4", &d("x4"), None, now)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn release_frees_an_unfinished_claim_but_not_a_completed_one() {
        let guard = InMemoryReplayGuard::new(8);
        let now = t("2026-08-26T12:00:00Z");
        let digest = document_digest(&doc("req-1", json!({}))).unwrap();

        guard.claim("req-1", &digest, None, now).await.unwrap();
        guard.release("req-1", &digest).await.unwrap();
        assert_eq!(
            guard.claim("req-1", &digest, None, now).await.unwrap(),
            ReplayVerdict::Fresh
        );

        guard.record_response("req-1", None).await.unwrap();
        guard.release("req-1", &digest).await.unwrap();
        assert!(
            matches!(
                guard.claim("req-1", &digest, None, now).await.unwrap(),
                ReplayVerdict::Duplicate { .. }
            ),
            "a completed execution's record must survive a stray release"
        );
    }

    /// `release_abandoned` releases synchronously — directly for the in-memory
    /// guard, and through the default for a guard that only implements the
    /// async `release` — and, like `release`, never frees a completed record.
    #[tokio::test]
    async fn release_abandoned_frees_an_unfinished_claim_without_awaiting() {
        /// Implements only the async methods, so it takes the default.
        struct Delegating(InMemoryReplayGuard);

        #[async_trait::async_trait]
        impl ReplayGuard for Delegating {
            async fn claim(
                &self,
                id: &str,
                digest: &DocumentDigest,
                retain_until: Option<DateTime<Utc>>,
                now: DateTime<Utc>,
            ) -> Result<ReplayVerdict, ReplayGuardError> {
                self.0.claim(id, digest, retain_until, now).await
            }
            async fn record_response(
                &self,
                id: &str,
                response: Option<&Value>,
            ) -> Result<(), ReplayGuardError> {
                self.0.record_response(id, response).await
            }
            async fn release(
                &self,
                id: &str,
                digest: &DocumentDigest,
            ) -> Result<(), ReplayGuardError> {
                self.0.release(id, digest).await
            }
        }

        let now = t("2026-08-26T12:00:00Z");
        let digest = document_digest(&doc("req-1", json!({}))).unwrap();
        let guards: [&dyn ReplayGuard; 2] = [
            &InMemoryReplayGuard::new(8),
            &Delegating(InMemoryReplayGuard::new(8)),
        ];
        for guard in guards {
            guard.claim("req-1", &digest, None, now).await.unwrap();
            guard.release_abandoned("req-1", &digest);
            assert_eq!(
                guard.claim("req-1", &digest, None, now).await.unwrap(),
                ReplayVerdict::Fresh
            );

            guard.record_response("req-1", None).await.unwrap();
            guard.release_abandoned("req-1", &digest);
            assert!(matches!(
                guard.claim("req-1", &digest, None, now).await.unwrap(),
                ReplayVerdict::Duplicate { .. }
            ));
        }
    }

    /// The documented degradation of the default `release_abandoned`: a guard
    /// whose `release` suspends (as a network round trip does) is polled once,
    /// returns `Pending`, and is dropped. The call returns without blocking and
    /// without a runtime, and the claim stays until its retention deadline.
    #[test]
    fn the_default_release_abandoned_gives_up_on_a_release_that_suspends() {
        use std::future::Future;
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct SuspendingGuard {
            inner: InMemoryReplayGuard,
            polls: AtomicUsize,
        }

        #[async_trait::async_trait]
        impl ReplayGuard for SuspendingGuard {
            async fn claim(
                &self,
                id: &str,
                digest: &DocumentDigest,
                retain_until: Option<DateTime<Utc>>,
                now: DateTime<Utc>,
            ) -> Result<ReplayVerdict, ReplayGuardError> {
                self.inner.claim(id, digest, retain_until, now).await
            }
            async fn release(
                &self,
                id: &str,
                digest: &DocumentDigest,
            ) -> Result<(), ReplayGuardError> {
                self.polls.fetch_add(1, Ordering::SeqCst);
                // A store round trip that never completes within one poll.
                std::future::pending::<()>().await;
                self.inner.release(id, digest).await
            }
        }

        let guard = SuspendingGuard {
            inner: InMemoryReplayGuard::new(8),
            polls: AtomicUsize::new(0),
        };
        let now = t("2026-08-26T12:00:00Z");
        let digest = document_digest(&doc("req-1", json!({}))).unwrap();
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        let claimed = std::pin::pin!(guard.claim("req-1", &digest, None, now)).poll(&mut cx);
        assert!(matches!(
            claimed,
            std::task::Poll::Ready(Ok(ReplayVerdict::Fresh))
        ));

        // No runtime is current here; the call must still return at once.
        guard.release_abandoned("req-1", &digest);
        assert_eq!(guard.polls.load(Ordering::SeqCst), 1, "polled exactly once");
        assert_eq!(guard.inner.len(), 1, "the record stays until retain_until");
    }

    /// The seam has to be usable as a trait object, or a deployment cannot
    /// swap in a Redis- or Postgres-backed store at runtime.
    #[test]
    fn the_guard_is_object_safe() {
        let guard = InMemoryReplayGuard::default();
        let _erased: &dyn ReplayGuard = &guard;
        let _boxed: Box<dyn ReplayGuard> = Box::new(InMemoryReplayGuard::new(4));
    }

    #[test]
    fn a_store_outage_maps_to_unavailable_and_leaks_no_detail() {
        let reason: RejectReason =
            ReplayGuardError("redis://replay-1.internal: connection refused".into()).into();
        assert_eq!(reason.code(), crate::StandardCode::Unavailable);
        let wire = reason.wire_message();
        assert!(
            !wire.contains("redis"),
            "wire message leaked the store: {wire}"
        );
        assert!(
            !wire.contains("internal"),
            "wire message leaked the host: {wire}"
        );
    }
}
