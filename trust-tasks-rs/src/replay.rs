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
//! that instant — `expiresAt` where present, otherwise `issuedAt + max_age` —
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

/// A bounded, in-process [`ReplayGuard`]: an LRU map from `id` to the digest
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
/// Eviction is by capacity as well as by `retain_until`: a burst of distinct
/// documents can push an older record out before its retention deadline, and a
/// replay arriving after that would be accepted. Size the capacity above the
/// number of distinct documents the widest acceptance window can hold.
pub struct InMemoryReplayGuard {
    capacity: usize,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    entries: HashMap<String, Entry>,
    /// Recency index: tick → id. Lets eviction find the least-recently-used
    /// key without scanning, and keeps `InMemoryReplayGuard` dependency-free.
    recency: BTreeMap<u64, String>,
    tick: u64,
}

struct Entry {
    digest: DocumentDigest,
    retain_until: Option<DateTime<Utc>>,
    response: Option<Value>,
    completed: bool,
    tick: u64,
}

impl InMemoryReplayGuard {
    /// A guard retaining at most `capacity` records.
    ///
    /// # Panics
    ///
    /// If `capacity` is zero. A guard that retains nothing answers `Fresh` to
    /// every arrival, which is indistinguishable from having no guard at all —
    /// and would be a silent, total defeat of item 11 rather than a visible
    /// misconfiguration.
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "InMemoryReplayGuard capacity must be non-zero"
        );
        Self {
            capacity,
            inner: Mutex::new(Inner::default()),
        }
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
    /// record as absent, so calling this is an optimisation (it reclaims
    /// memory) rather than a correctness requirement.
    pub fn purge_expired(&self, now: DateTime<Utc>) {
        let mut inner = self.inner.lock().expect("replay guard mutex");
        let expired: Vec<String> = inner
            .entries
            .iter()
            .filter(|(_, e)| e.is_expired_at(now))
            .map(|(id, _)| id.clone())
            .collect();
        for id in expired {
            inner.remove(&id);
        }
    }
}

impl Default for InMemoryReplayGuard {
    /// 10 000 records — a few megabytes at typical document sizes, and enough
    /// to cover a five-minute acceptance window at ~33 documents per second.
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl Entry {
    fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        matches!(self.retain_until, Some(t) if t <= now)
    }
}

impl Inner {
    fn remove(&mut self, id: &str) {
        if let Some(entry) = self.entries.remove(id) {
            self.recency.remove(&entry.tick);
        }
    }

    fn touch(&mut self, id: &str) {
        self.tick += 1;
        let tick = self.tick;
        if let Some(entry) = self.entries.get_mut(id) {
            self.recency.remove(&entry.tick);
            entry.tick = tick;
            self.recency.insert(tick, id.to_string());
        }
    }

    fn evict_to_capacity(&mut self, capacity: usize) {
        while self.entries.len() > capacity {
            let Some((_, victim)) = self.recency.iter().next().map(|(k, v)| (*k, v.clone())) else {
                break;
            };
            self.remove(&victim);
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
            // A conflicting document is not a use of the record, so it does
            // not refresh the record's recency — otherwise a flood of
            // conflicts could pin an entry and evict live ones.
            if matches!(verdict, ReplayVerdict::Duplicate { .. }) {
                inner.touch(id);
            }
            return Ok(verdict);
        }

        inner.tick += 1;
        let tick = inner.tick;
        inner.entries.insert(
            id.to_string(),
            Entry {
                digest: digest.clone(),
                retain_until,
                response: None,
                completed: false,
                tick,
            },
        );
        inner.recency.insert(tick, id.to_string());
        let capacity = self.capacity;
        inner.evict_to_capacity(capacity);
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
        // Only release the claim this digest made. A concurrent arrival that
        // legitimately holds the key must not have it taken away by a
        // different document's cleanup.
        if inner
            .entries
            .get(id)
            .is_some_and(|entry| &entry.digest == digest && !entry.completed)
        {
            inner.remove(id);
        }
        Ok(())
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

    #[tokio::test]
    async fn capacity_evicts_the_least_recently_used_record() {
        let guard = InMemoryReplayGuard::new(2);
        let now = t("2026-08-26T12:00:00Z");
        let d = |id: &str| document_digest(&doc(id, json!({}))).unwrap();

        guard.claim("a", &d("a"), None, now).await.unwrap();
        guard.claim("b", &d("b"), None, now).await.unwrap();
        // Touch `a` so `b` becomes the least recently used.
        guard.claim("a", &d("a"), None, now).await.unwrap();
        guard.claim("c", &d("c"), None, now).await.unwrap();

        assert_eq!(guard.len(), 2);
        assert!(matches!(
            guard.claim("a", &d("a"), None, now).await.unwrap(),
            ReplayVerdict::Duplicate { .. }
        ));
        assert_eq!(
            guard.claim("b", &d("b"), None, now).await.unwrap(),
            ReplayVerdict::Fresh
        );
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
