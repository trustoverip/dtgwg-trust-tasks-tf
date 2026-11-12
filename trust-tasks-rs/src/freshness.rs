//! Freshness bounds over `issuedAt` / `expiresAt` (SPEC.md §4.2, §7.2).
//!
//! # Why this exists
//!
//! Until this module, the only temporal check the library made was §7.2 item 4
//! — `expiresAt`, and only where the producer chose to set it. `issuedAt` was
//! parsed and never looked at. That leaves two holes:
//!
//! * A document stamped a year in the future is accepted, and (worse) is
//!   accepted *again* for the whole of that year.
//! * A document with `expiresAt` at or before its own `issuedAt` is internally
//!   contradictory — it asserts a validity interval that never contained a
//!   valid instant — and was accepted whenever the clock happened to sit
//!   before the expiry.
//!
//! The third hole is the one that makes [`crate::ReplayGuard`] implementable
//! at all. SPEC §7.2 (*Bounding the record*) ties the duplicate-execution
//! record to the acceptance window and says the two bounds are the same
//! bound: a consumer "**MUST NOT** accept for execution a document older than
//! the window over which it retains records", and one that "can establish
//! neither an `expiresAt` nor an age for a document has no window in which to
//! place it, and **MUST NOT** execute a *consequential Trust Task* on it".
//! Without [`FreshnessPolicy::max_age`] there is no window, so a replay record
//! for a document carrying no `expiresAt` would have to be kept forever.
//!
//! # Choosing a policy
//!
//! [`FreshnessPolicy::default`] enforces only the two internal-consistency
//! rules, which no conforming producer can fail. It is the policy
//! [`consume_inbound`](crate::consume_inbound) applies.
//!
//! [`FreshnessPolicy::consequential`] is the posture §7.2's bounding paragraph
//! describes: `issuedAt` REQUIRED, and a bounded acceptance window. Use it —
//! or something stricter — for any specification whose execution is
//! *consequential* (§2), which is exactly the set for which item 11 applies.

use chrono::{DateTime, TimeDelta, Utc};

use crate::document::TrustTask;
use crate::error::RejectReason;

/// The clock-skew tolerance SPEC §4.2 sanctions for temporal comparisons.
/// The spec's wording is "typically ≤ 60s"; this is that bound.
pub const DEFAULT_SKEW: TimeDelta = TimeDelta::seconds(60);

/// The acceptance window [`FreshnessPolicy::consequential`] applies.
///
/// Five minutes is long enough to survive a mediator queue, a retry with
/// backoff, and a modest clock disagreement, and short enough that the replay
/// record it bounds stays small. It is a default, not a rule: a deployment
/// whose transport buffers for longer must widen it *and* widen the retention
/// of its [`ReplayGuard`](crate::ReplayGuard) to match, because §7.2 makes
/// them one bound.
pub const DEFAULT_MAX_AGE: TimeDelta = TimeDelta::minutes(5);

/// The expiry horizon [`FreshnessPolicy::consequential`] applies: the furthest
/// after `issuedAt` a stated `expiresAt` is honoured.
///
/// A document's `expiresAt` fixes its acceptance window and the replay
/// record's retention (SPEC §7.2, *Bounding the record*), so without a horizon
/// the producer alone decides how long the consumer must remember its `id`.
/// Thirty days covers a store-and-forward delivery to a recipient that is
/// offline for weeks; a later `expiresAt` is clamped to it, not refused.
pub const DEFAULT_EXPIRY_HORIZON: TimeDelta = TimeDelta::days(30);

/// How a consumer bounds a document in time before acting on it.
///
/// Every field is explicit; there is no "unset means whatever the library
/// felt like" case. See the [module documentation](self) for the reasoning and
/// for which constructor to reach for.
///
/// `#[non_exhaustive]`: build one from [`Default`] or
/// [`consequential`](Self::consequential) and the `with_*` builders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct FreshnessPolicy {
    /// Tolerance applied when comparing the document's timestamps against the
    /// consumer's own clock, per the allowance in SPEC §4.2. Applied to the
    /// future-dating check and to [`max_age`](Self::max_age).
    ///
    /// Not applied to `expiresAt`, which
    /// [`validate_basic`](TrustTask::validate_basic) evaluates against the raw
    /// `now`: that comparison is documented as taking an already-adjusted
    /// instant from the caller, and moving it here would change the meaning of
    /// every existing call site.
    pub skew: TimeDelta,

    /// The oldest `issuedAt` this consumer will accept, measured back from
    /// `now`, for a document that carries **no** `expiresAt`. `None` means
    /// unbounded — the only setting under which a document carrying neither
    /// timestamp can be accepted.
    ///
    /// A document that states an `expiresAt` is bounded by it instead (SPEC
    /// §7.2, *Bounding the record*: the maximum age applies "where `expiresAt`
    /// is absent"), up to [`expiry_horizon`](Self::expiry_horizon).
    pub max_age: Option<TimeDelta>,

    /// The furthest after `issuedAt` a stated `expiresAt` is honoured. A
    /// document whose `expiresAt` lies beyond `issuedAt + expiry_horizon` is
    /// treated as expiring there: accepted until then (plus
    /// [`skew`](Self::skew)) and refused as `expired` after, never refused for
    /// stating the later instant. `None` honours any `expiresAt`.
    ///
    /// The horizon runs from `issuedAt`, so it does not apply to a document
    /// without one: such a document is bounded by its `expiresAt` alone. A
    /// horizon counted from first sight would drop the record while the
    /// document is still acceptable, and a later copy would run again. Set
    /// [`require_issued_at`](Self::require_issued_at) to refuse such documents.
    pub expiry_horizon: Option<TimeDelta>,

    /// Reject a document that carries no `issuedAt` at all, with
    /// `malformedRequest`.
    ///
    /// A document with no `issuedAt` and no `expiresAt` cannot be placed in
    /// any window, which SPEC §7.2 forbids executing a consequential task on.
    /// Setting this makes that refusal happen at the framework layer with a
    /// code the producer can act on, rather than at the replay guard with one
    /// that says nothing about what to fix.
    pub require_issued_at: bool,
}

impl Default for FreshnessPolicy {
    /// The minimum every consumer should apply: reject a future-dated document
    /// and reject one whose stated validity interval is empty. Both are
    /// producer bugs or forgeries — no conforming producer emits either — so
    /// enforcing them costs a correct deployment nothing.
    ///
    /// It deliberately sets **no** `max_age`. An acceptance window is a
    /// deployment decision (it depends on how long the transport may hold a
    /// message), and a library that guessed one would start silently rejecting
    /// documents that had been arriving for years.
    fn default() -> Self {
        Self {
            skew: DEFAULT_SKEW,
            max_age: None,
            expiry_horizon: None,
            require_issued_at: false,
        }
    }
}

impl FreshnessPolicy {
    /// The posture SPEC §7.2 (*Bounding the record*) describes for a
    /// *consequential Trust Task*: `issuedAt` REQUIRED and a bounded
    /// acceptance window ([`DEFAULT_MAX_AGE`] without an `expiresAt`, the
    /// `expiresAt` up to [`DEFAULT_EXPIRY_HORIZON`] with one), so that every
    /// accepted document sits inside a window the
    /// [`ReplayGuard`](crate::ReplayGuard) can retain a record for.
    pub fn consequential() -> Self {
        Self {
            skew: DEFAULT_SKEW,
            max_age: Some(DEFAULT_MAX_AGE),
            expiry_horizon: Some(DEFAULT_EXPIRY_HORIZON),
            require_issued_at: true,
        }
    }

    /// Builder: set the acceptance window.
    #[must_use]
    pub fn with_max_age(mut self, max_age: TimeDelta) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Builder: set the expiry horizon.
    #[must_use]
    pub fn with_expiry_horizon(mut self, horizon: TimeDelta) -> Self {
        self.expiry_horizon = Some(horizon);
        self
    }

    /// Builder: set the clock-skew tolerance.
    #[must_use]
    pub fn with_skew(mut self, skew: TimeDelta) -> Self {
        self.skew = skew;
        self
    }

    /// Builder: require an in-band `issuedAt`.
    #[must_use]
    pub fn requiring_issued_at(mut self) -> Self {
        self.require_issued_at = true;
        self
    }

    /// The instant at which a record for `doc` may be dropped — the end of
    /// this consumer's willingness to execute it, which SPEC §7.2 makes the
    /// same instant as the end of the replay record's required retention.
    ///
    /// `expiresAt` fixes it where present, clamped to
    /// `issuedAt + expiry_horizon + skew` where it lies beyond the horizon.
    /// Otherwise it is `issuedAt + max_age + skew`. Each is the same bound
    /// [`validate_freshness`](TrustTask::validate_freshness) refuses from, so a
    /// replay arriving inside the skew allowance still finds the record. The
    /// window is half-open — a document is acceptable strictly before this
    /// instant and refused at it, as for `expiresAt` — so a guard that drops a
    /// record once `retain_until <= now` never drops one the pipeline would
    /// still execute.
    /// `None` means this policy places no bound on the
    /// document, in which case a consumer **MUST NOT** execute a consequential
    /// task on it: it has no window in which to retain the record.
    ///
    /// Where the producer stamped neither `issuedAt` nor `expiresAt`, `now`
    /// stands in for the `issuedAt`: the record then lives a full window from
    /// first sight, which is the longest the document could still be arriving
    /// from a queue.
    pub fn record_expiry<P>(
        &self,
        doc: &TrustTask<P>,
        now: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        if let Some(expires_at) = doc.expires_at {
            // No `issuedAt`, no horizon: see `expiry_horizon`.
            return Some(match doc.issued_at.and_then(|i| self.horizon_end(i)) {
                Some(horizon_end) if expires_at > horizon_end => horizon_end + self.skew,
                _ => expires_at,
            });
        }
        let max_age = self.max_age?;
        Some(doc.issued_at.unwrap_or(now) + max_age + self.skew)
    }

    /// `issued_at + expiry_horizon`, or `None` with no horizon (or one past
    /// the representable range, which bounds nothing).
    fn horizon_end(&self, issued_at: DateTime<Utc>) -> Option<DateTime<Utc>> {
        issued_at.checked_add_signed(self.expiry_horizon?)
    }
}

impl<P> TrustTask<P> {
    /// Apply `policy` to this document's `issuedAt` / `expiresAt`.
    ///
    /// This is the freshness half of SPEC §7.2 item 4 that
    /// [`validate_basic`](Self::validate_basic) does not cover.
    /// [`consume_inbound`](crate::consume_inbound) calls it for you; call it
    /// directly only if you compose the §7.2 pipeline by hand.
    ///
    /// Checks, in order:
    ///
    /// 1. `issuedAt` beyond `now + skew` → `malformedRequest`. A document
    ///    cannot have been produced after the moment it arrived.
    /// 2. `expiresAt <= issuedAt` → `malformedRequest`. The document states a
    ///    validity interval containing no valid instant.
    /// 3. `issuedAt` absent while [`FreshnessPolicy::require_issued_at`] →
    ///    `malformedRequest`.
    /// 4. With an `expiresAt`: `issuedAt + expiry_horizon + skew` at or
    ///    before `now` → `expired`. The `expiresAt` itself is
    ///    [`validate_basic`](Self::validate_basic)'s to apply, and `max_age`
    ///    is not applied at all (SPEC §7.2, *Bounding the record*).
    /// 5. Without one: `issuedAt + max_age + skew` at or before `now` →
    ///    `expired`. The window is half-open, ending where
    ///    [`FreshnessPolicy::record_expiry`] does.
    /// 6. Neither `issuedAt` nor `expiresAt`, under a policy that sets a
    ///    `max_age` → `expired`. There is no window to place the document in.
    pub fn validate_freshness(
        &self,
        now: DateTime<Utc>,
        policy: &FreshnessPolicy,
    ) -> Result<(), RejectReason> {
        if let Some(issued_at) = self.issued_at {
            if issued_at > now + policy.skew {
                return Err(RejectReason::MalformedRequest {
                    reason: FUTURE_ISSUED_AT.to_string(),
                });
            }
            if let Some(expires_at) = self.expires_at {
                if expires_at <= issued_at {
                    return Err(RejectReason::MalformedRequest {
                        reason: EXPIRY_NOT_AFTER_ISSUANCE.to_string(),
                    });
                }
                // The producer's `expiresAt` is the window. Past the horizon
                // it is clamped there, not refused: the document was
                // acceptable, and is now merely older than this consumer
                // will remember it.
                if let Some(horizon_end) = policy.horizon_end(issued_at) {
                    if expires_at > horizon_end && horizon_end + policy.skew <= now {
                        return Err(RejectReason::Stale {
                            detail: StaleReason::OlderThanWindow,
                        });
                    }
                }
                return Ok(());
            }
            if let Some(max_age) = policy.max_age {
                if issued_at + max_age + policy.skew <= now {
                    return Err(RejectReason::Stale {
                        detail: StaleReason::OlderThanWindow,
                    });
                }
            }
            return Ok(());
        }

        if policy.require_issued_at {
            return Err(RejectReason::MalformedRequest {
                reason: ISSUED_AT_REQUIRED.to_string(),
            });
        }
        // No `issuedAt`. A policy with a window cannot place the document in
        // it unless the producer supplied an `expiresAt` instead (SPEC §7.2,
        // *Bounding the record*).
        if policy.max_age.is_some() && self.expires_at.is_none() {
            return Err(RejectReason::Stale {
                detail: StaleReason::Unboundable,
            });
        }
        Ok(())
    }
}

/// Wire-safe reason for a `malformedRequest` raised by a future-dated
/// `issuedAt`. A constant, not a rendering of the two timestamps: the
/// consumer's clock is consumer-internal state, and echoing the delta turns
/// every rejection into a remote `ntpdate` for an unauthenticated sender.
pub const FUTURE_ISSUED_AT: &str =
    "issuedAt is in the future beyond the consumer's skew tolerance (SPEC §4.2)";

/// Wire-safe reason for a `malformedRequest` raised by `expiresAt <= issuedAt`.
/// Both values are the producer's own, so naming the rule leaks nothing.
pub const EXPIRY_NOT_AFTER_ISSUANCE: &str =
    "expiresAt is not after issuedAt: the document states an empty validity interval (SPEC §4.2)";

/// Wire-safe reason for a `malformedRequest` raised by a missing `issuedAt`
/// under a policy that requires one.
pub const ISSUED_AT_REQUIRED: &str =
    "issuedAt is required by consumer policy (SPEC §7.2, bounding the duplicate-execution record)";

/// Wire-safe reason for a `malformedRequest` raised by a missing `issuedAt`
/// on a specification that declares it REQUIRED (SPEC §7.3 item 17).
///
/// Distinct from [`ISSUED_AT_REQUIRED`], which names the *consumer's* own
/// policy: this one names an obligation published by the specification, so
/// the producer can fix it by reading the registry entry rather than by
/// guessing at the consumer's configuration. Raised by
/// [`TrustTask::enforce_spec_policy`](crate::TrustTask::enforce_spec_policy),
/// not by [`TrustTask::validate_freshness`], because it is keyed on the
/// payload type rather than on a policy value.
pub const ISSUED_AT_REQUIRED_BY_SPEC: &str =
    "issuedAt is required by this Trust Task specification (SPEC §7.3 item 17)";

/// Which of the two acceptance-window failures produced a
/// [`RejectReason::Stale`].
///
/// Carried for the operator's log. Both render the same wire message, because
/// the window itself is consumer policy and §10.4 keeps consumer-side state
/// off the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaleReason {
    /// `issuedAt` is further back than the consumer's acceptance window: its
    /// maximum age, or for a document with an `expiresAt`, its expiry horizon.
    OlderThanWindow,
    /// The document carries neither `issuedAt` nor `expiresAt`, so the
    /// consumer has no window in which to place it (SPEC §7.2).
    Unboundable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StandardCode;
    use crate::TypeUri;

    fn doc_at(
        issued_at: Option<DateTime<Utc>>,
        expires_at: Option<DateTime<Utc>>,
    ) -> TrustTask<serde_json::Value> {
        let mut doc = TrustTask::new(
            "req-1",
            TypeUri::canonical("acl/change-role", 0, 1).unwrap(),
            serde_json::json!({}),
        );
        doc.issued_at = issued_at;
        doc.expires_at = expires_at;
        doc
    }

    fn t(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    #[test]
    fn future_issued_at_is_rejected_beyond_the_skew_tolerance() {
        let now = t("2026-08-26T12:00:00Z");
        let policy = FreshnessPolicy::default();

        // Inside the 60s tolerance: accepted, because a producer's clock may
        // legitimately run slightly fast.
        let ok = doc_at(Some(t("2026-08-26T12:00:30Z")), None);
        assert!(ok.validate_freshness(now, &policy).is_ok());

        // Beyond it: refused.
        let bad = doc_at(Some(t("2026-08-26T12:05:00Z")), None);
        let err = bad.validate_freshness(now, &policy).unwrap_err();
        assert_eq!(err.code(), StandardCode::MalformedRequest);
        assert!(err.wire_message().ends_with(FUTURE_ISSUED_AT));
    }

    /// The wire message must not name the consumer's clock or the size of the
    /// disagreement — that is an NTP oracle for an unauthenticated sender.
    #[test]
    fn future_issued_at_message_carries_no_consumer_clock() {
        let now = t("2026-08-26T12:00:00Z");
        let bad = doc_at(Some(t("2031-01-01T00:00:00Z")), None);
        let msg = bad
            .validate_freshness(now, &FreshnessPolicy::default())
            .unwrap_err()
            .wire_message();
        assert!(
            !msg.contains("2026"),
            "wire message leaked the clock: {msg}"
        );
        assert!(
            !msg.contains("2031"),
            "wire message echoed the input: {msg}"
        );
    }

    #[test]
    fn expiry_at_or_before_issuance_is_rejected() {
        let now = t("2026-08-26T12:00:00Z");
        let policy = FreshnessPolicy::default();

        for expires in ["2026-08-26T11:59:00Z", "2026-08-26T11:59:30Z"] {
            let doc = doc_at(Some(t("2026-08-26T11:59:30Z")), Some(t(expires)));
            let err = doc.validate_freshness(now, &policy).unwrap_err();
            assert_eq!(err.code(), StandardCode::MalformedRequest);
            assert!(err.wire_message().ends_with(EXPIRY_NOT_AFTER_ISSUANCE));
        }

        // Strictly after: fine.
        let good = doc_at(
            Some(t("2026-08-26T11:59:30Z")),
            Some(t("2026-08-26T12:30:00Z")),
        );
        assert!(good.validate_freshness(now, &policy).is_ok());
    }

    #[test]
    fn max_age_bounds_the_acceptance_window() {
        let now = t("2026-08-26T12:00:00Z");
        let policy = FreshnessPolicy::default().with_max_age(TimeDelta::minutes(5));

        let fresh = doc_at(Some(t("2026-08-26T11:58:00Z")), None);
        assert!(fresh.validate_freshness(now, &policy).is_ok());

        let stale = doc_at(Some(t("2026-08-26T11:30:00Z")), None);
        let err = stale.validate_freshness(now, &policy).unwrap_err();
        assert_eq!(err.code(), StandardCode::Expired);
    }

    /// The acceptance window and the record's retention end at one instant,
    /// and the window is half-open: a document is refused at exactly the
    /// instant a guard may drop its record (`retain_until <= now`), so there
    /// is no instant at which it is both acceptable and forgotten.
    #[test]
    fn the_acceptance_window_closes_where_the_record_expires() {
        let issued = t("2026-08-26T12:00:00Z");
        let policy = FreshnessPolicy::consequential();
        let tick = TimeDelta::milliseconds(1);

        let doc = doc_at(Some(issued), None);
        let end = policy.record_expiry(&doc, issued).unwrap();
        assert_eq!(end, issued + DEFAULT_MAX_AGE + DEFAULT_SKEW);
        assert!(doc.validate_freshness(end - tick, &policy).is_ok());
        assert_eq!(
            doc.validate_freshness(end, &policy).unwrap_err().code(),
            StandardCode::Expired
        );

        // The same for an expiry clamped to the horizon.
        let far = doc_at(Some(issued), Some(issued + TimeDelta::days(365)));
        let end = policy.record_expiry(&far, issued).unwrap();
        assert_eq!(end, issued + DEFAULT_EXPIRY_HORIZON + DEFAULT_SKEW);
        assert!(far.validate_freshness(end - tick, &policy).is_ok());
        assert_eq!(
            far.validate_freshness(end, &policy).unwrap_err().code(),
            StandardCode::Expired
        );
    }

    #[test]
    fn a_document_with_no_timestamps_is_unboundable_under_a_window() {
        let now = t("2026-08-26T12:00:00Z");

        // No window configured: the pre-existing behaviour is preserved.
        let doc = doc_at(None, None);
        assert!(doc
            .validate_freshness(now, &FreshnessPolicy::default())
            .is_ok());

        // A window configured, and nothing to place the document in it.
        let windowed = FreshnessPolicy::default().with_max_age(TimeDelta::minutes(5));
        let err = doc.validate_freshness(now, &windowed).unwrap_err();
        assert_eq!(err.code(), StandardCode::Expired);

        // …unless the producer supplied an `expiresAt` instead, which is a
        // window of its own.
        let bounded = doc_at(None, Some(t("2026-08-26T12:30:00Z")));
        assert!(bounded.validate_freshness(now, &windowed).is_ok());
    }

    #[test]
    fn require_issued_at_refuses_a_document_without_one() {
        let now = t("2026-08-26T12:00:00Z");
        let doc = doc_at(None, Some(t("2026-08-26T12:30:00Z")));
        let err = doc
            .validate_freshness(now, &FreshnessPolicy::consequential())
            .unwrap_err();
        assert_eq!(err.code(), StandardCode::MalformedRequest);
        assert!(err.wire_message().ends_with(ISSUED_AT_REQUIRED));
    }

    #[test]
    fn record_expiry_prefers_expires_at_then_falls_back_to_the_window() {
        let now = t("2026-08-26T12:00:00Z");
        let policy = FreshnessPolicy::consequential();

        let with_expiry = doc_at(
            Some(t("2026-08-26T11:59:00Z")),
            Some(t("2026-08-26T18:00:00Z")),
        );
        assert_eq!(
            policy.record_expiry(&with_expiry, now),
            Some(t("2026-08-26T18:00:00Z"))
        );

        let without = doc_at(Some(t("2026-08-26T11:59:00Z")), None);
        assert_eq!(
            policy.record_expiry(&without, now),
            Some(t("2026-08-26T12:05:00Z"))
        );

        // No window and no expiry: nothing to bound the record with, which is
        // the case §7.2 forbids executing a consequential task under.
        assert_eq!(
            FreshnessPolicy::default().record_expiry(&without, now),
            None
        );
    }

    /// SPEC §7.2 (*Bounding the record*): the maximum age applies "where
    /// `expiresAt` is absent". A document stating a later expiry is acceptable
    /// until then, however old its `issuedAt`, and the record lasts as long.
    #[test]
    fn a_stated_expiry_replaces_the_maximum_age() {
        let now = t("2026-08-26T12:00:00Z");
        let policy = FreshnessPolicy::consequential();

        // Two hours old, far past the five-minute max age, expiry still ahead.
        let held = doc_at(
            Some(t("2026-08-26T10:00:00Z")),
            Some(t("2026-08-26T16:00:00Z")),
        );
        assert!(held.validate_freshness(now, &policy).is_ok());
        assert_eq!(
            policy.record_expiry(&held, now),
            Some(t("2026-08-26T16:00:00Z"))
        );

        // The same age with no expiry is still refused by the max age.
        let unstated = doc_at(Some(t("2026-08-26T10:00:00Z")), None);
        assert_eq!(
            unstated
                .validate_freshness(now, &policy)
                .unwrap_err()
                .code(),
            StandardCode::Expired
        );
    }

    /// An expiry beyond the horizon is clamped to it, not refused: accepted
    /// until `issuedAt + horizon + skew`, refused as `expired` from then, and
    /// the record kept exactly that long.
    #[test]
    fn an_expiry_beyond_the_horizon_is_clamped_to_it() {
        let policy = FreshnessPolicy::consequential();
        let issued = t("2026-08-01T00:00:00Z");
        let far = doc_at(Some(issued), Some(t("2027-08-01T00:00:00Z")));
        let horizon_end = issued + DEFAULT_EXPIRY_HORIZON;

        // Inside the horizon: accepted, though the expiry is a year out.
        let early = t("2026-08-02T00:00:00Z");
        assert!(far.validate_freshness(early, &policy).is_ok());
        assert_eq!(
            policy.record_expiry(&far, early),
            Some(horizon_end + DEFAULT_SKEW)
        );

        // Past the horizon but inside the skew tolerance: still accepted, and
        // the record still held.
        let edge = horizon_end + TimeDelta::seconds(30);
        assert!(far.validate_freshness(edge, &policy).is_ok());
        assert!(policy.record_expiry(&far, edge).unwrap() >= edge);

        // Beyond both: refused as stale, not malformed.
        let late = horizon_end + DEFAULT_SKEW + TimeDelta::seconds(1);
        assert_eq!(
            far.validate_freshness(late, &policy).unwrap_err().code(),
            StandardCode::Expired
        );

        // An expiry inside the horizon is not moved.
        let near = doc_at(Some(issued), Some(t("2026-08-20T00:00:00Z")));
        assert_eq!(
            policy.record_expiry(&near, early),
            Some(t("2026-08-20T00:00:00Z"))
        );

        // No horizon: any stated expiry is honoured.
        let unbounded = FreshnessPolicy::default().with_max_age(DEFAULT_MAX_AGE);
        assert!(far.validate_freshness(late, &unbounded).is_ok());
        assert_eq!(
            unbounded.record_expiry(&far, late),
            Some(t("2027-08-01T00:00:00Z"))
        );
    }

    /// Without an `issuedAt` the horizon has nothing to run from. Counting it
    /// from first sight would drop the record while the document is still
    /// acceptable, and the next copy would run again, so the expiry stands.
    #[test]
    fn the_horizon_needs_an_issued_at() {
        let now = t("2026-08-26T12:00:00Z");
        let policy = FreshnessPolicy::default()
            .with_max_age(DEFAULT_MAX_AGE)
            .with_expiry_horizon(DEFAULT_EXPIRY_HORIZON);
        let doc = doc_at(None, Some(t("2027-08-26T12:00:00Z")));
        assert!(doc.validate_freshness(now, &policy).is_ok());
        assert_eq!(
            policy.record_expiry(&doc, now),
            Some(t("2027-08-26T12:00:00Z"))
        );
    }

    #[test]
    fn record_outlives_the_skew_allowance_on_acceptance() {
        // 5m30s old: past max_age, inside max_age + skew, so still accepted.
        // A record dropped at issuedAt + max_age would let this replay run twice.
        let now = t("2026-08-26T12:00:00Z");
        let policy = FreshnessPolicy::consequential();
        let late = doc_at(Some(t("2026-08-26T11:54:30Z")), None);
        assert!(late.validate_freshness(now, &policy).is_ok());
        assert_eq!(
            policy.record_expiry(&late, now),
            Some(t("2026-08-26T12:00:30Z"))
        );
    }
}
