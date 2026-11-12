package trusttasks

// Freshness bounds over issuedAt / expiresAt (SPEC §4.2, §7.2).
//
// Mirrors freshness.rs in trust-tasks-rs and freshness.ts in
// @openvtc/trust-tasks, deliberately closely: a Go consumer and a Rust or
// TypeScript one must reach the same verdict on the same document.
//
// Without a freshness bound the only temporal check a runtime makes is §7.2 item
// 4 — expiresAt, and only where the producer chose to set it. That accepts a
// document stamped a year in the future for the whole of that year, and one whose
// expiresAt sits at or before its own issuedAt: a validity interval that never
// contained a valid instant.
//
// It is also what makes [ReplayGuard] implementable. SPEC §7.2 (*Bounding the
// record*) ties the duplicate-execution record to the acceptance window and says
// the two bounds are the same bound: a consumer "MUST NOT accept for execution a
// document older than the window over which it retains records". Without
// [FreshnessPolicy.MaxAge] there is no window, so a record for a document
// carrying no expiresAt would have to be kept forever.

import "time"

// DefaultSkew is the clock-skew tolerance SPEC §4.2 sanctions ("typically ≤ 60s").
const DefaultSkew = 60 * time.Second

// DefaultMaxAge is the acceptance window [ConsequentialChecks] applies.
//
// Five minutes survives a mediator queue, a retry with backoff and a modest clock
// disagreement, and keeps the record it bounds small. A deployment whose
// transport buffers for longer must widen it and widen its guard's retention to
// match — §7.2 makes them one bound.
const DefaultMaxAge = 5 * time.Minute

// DefaultExpiryHorizon is the expiry horizon [ConsequentialFreshness] applies:
// the furthest after issuedAt a stated expiresAt is honoured.
//
// A document's expiresAt fixes its acceptance window and the replay record's
// retention (SPEC §7.2, *Bounding the record*), so without a horizon the
// producer alone decides how long the consumer must remember its id. Thirty days
// covers a store-and-forward delivery to a recipient offline for weeks; a later
// expiresAt is clamped to it, not refused. Mirrors Rust's DEFAULT_EXPIRY_HORIZON.
const DefaultExpiryHorizon = 30 * 24 * time.Hour

// FreshnessPolicy is how a consumer bounds a document in time before acting on
// it.
type FreshnessPolicy struct {
	// Skew is the tolerance applied to the document's timestamps against this
	// consumer's clock, per SPEC §4.2. Applied to the future-dating check and to
	// MaxAge; not to expiresAt, which [ValidateBasic] compares against the raw
	// now it is given.
	Skew time.Duration

	// MaxAge is the oldest issuedAt this consumer accepts, measured back from
	// now, for a document that carries no expiresAt. Zero means unbounded — and
	// is the only setting under which a document carrying neither timestamp is
	// acceptable.
	//
	// A document that states an expiresAt is bounded by it instead (SPEC §7.2,
	// *Bounding the record*: the maximum age applies "where expiresAt is
	// absent"), up to ExpiryHorizon.
	MaxAge time.Duration

	// ExpiryHorizon is the furthest after issuedAt a stated expiresAt is
	// honoured. A document whose expiresAt lies beyond issuedAt + ExpiryHorizon
	// is treated as expiring there: accepted until then (plus Skew) and refused
	// as expired after, never refused for stating the later instant; its replay
	// record is kept until the same instant. Nil honours any expiresAt; a zero
	// horizon clamps at issuedAt — as None / undefined / null and zero do in the
	// Rust, TypeScript and Dart runtimes. Unlike MaxAge, zero is not "unbounded".
	//
	// The horizon runs from issuedAt, so it does not apply to a document without
	// one: that is bounded by its expiresAt alone. Set RequireIssuedAt to refuse
	// such documents.
	ExpiryHorizon *time.Duration

	// RequireIssuedAt rejects a document carrying no issuedAt, with
	// malformedRequest.
	RequireIssuedAt bool
}

// DefaultFreshness is the minimum every consumer should apply: reject a
// future-dated document and one whose stated validity interval is empty. No
// conforming producer emits either, so this costs a correct deployment nothing.
//
// Deliberately sets no MaxAge — an acceptance window depends on how long the
// transport may hold a message, which is a deployment fact, and a library that
// guessed one would start refusing documents that had arrived for years.
func DefaultFreshness() FreshnessPolicy {
	return FreshnessPolicy{Skew: DefaultSkew}
}

// ConsequentialFreshness is the posture SPEC §7.2 (*Bounding the record*)
// describes for a consequential Trust Task (§2): issuedAt REQUIRED and a bounded
// acceptance window, so every accepted document sits inside a window a
// [ReplayGuard] can retain a record for.
func ConsequentialFreshness() FreshnessPolicy {
	return FreshnessPolicy{
		Skew:            DefaultSkew,
		MaxAge:          DefaultMaxAge,
		ExpiryHorizon:   ptrTo(DefaultExpiryHorizon),
		RequireIssuedAt: true,
	}
}

// horizonEnd is issuedAt + ExpiryHorizon; false with no horizon.
func (p FreshnessPolicy) horizonEnd(issuedAt time.Time) (time.Time, bool) {
	if p.ExpiryHorizon == nil {
		return time.Time{}, false
	}
	return issuedAt.Add(*p.ExpiryHorizon), true
}

func ptrTo[T any](v T) *T { return &v }

// Wire-safe reasons for the freshness rejections. Constants so the same text
// reaches the wire from every call site, and so a test can assert on them.
const (
	// FutureIssuedAt reports an issuedAt beyond the consumer's skew tolerance.
	FutureIssuedAt = "issuedAt is in the future beyond the consumer's skew tolerance (SPEC §4.2)"

	// ExpiryNotAfterIssuance reports expiresAt <= issuedAt.
	ExpiryNotAfterIssuance = "expiresAt is not after issuedAt: the document states an empty " +
		"validity interval (SPEC §4.2)"

	// IssuedAtRequired reports a missing issuedAt under a policy that requires one.
	IssuedAtRequired = "issuedAt is required by consumer policy (SPEC §7.2, bounding the " +
		"duplicate-execution record)"

	// StaleWireMessage reports a document outside the consumer's acceptance window.
	//
	// A constant, not a rendering of the window or the consumer's clock: §10.4
	// keeps consumer-side state off the wire, and echoing the delta would turn
	// every rejection into a remote ntpdate — and a probe for the window's exact
	// boundary — for an unauthenticated sender.
	StaleWireMessage = "document is outside the consumer's acceptance window (SPEC §7.2)"
)

// ValidateFreshness applies policy to this document's issuedAt / expiresAt.
// Returns nil when the document is acceptable.
//
// This is the freshness half of SPEC §7.2 item 4 that [ValidateBasic] does not
// cover. [ConsumeInbound] calls it for you.
func ValidateFreshness[P any](
	doc *Document[P],
	now time.Time,
	policy FreshnessPolicy,
) *RejectReason {
	malformed := func(message string) *RejectReason {
		return &RejectReason{Code: CodeMalformedRequest, Message: message}
	}
	stale := func() *RejectReason {
		return &RejectReason{Code: CodeExpired, Message: StaleWireMessage}
	}

	if doc.IssuedAt != nil {
		issuedAt, err := parseTimestamp(*doc.IssuedAt)
		if err != nil {
			return malformed("issuedAt is not a valid RFC 3339 timestamp")
		}
		if issuedAt.After(now.Add(policy.Skew)) {
			return malformed(FutureIssuedAt)
		}

		if doc.ExpiresAt != nil {
			// A malformed expiresAt is [ValidateBasic]'s to report; skip it here
			// rather than raise a second, differently-worded rejection for it.
			if expiresAt, err := parseTimestamp(*doc.ExpiresAt); err == nil {
				if !expiresAt.After(issuedAt) {
					return malformed(ExpiryNotAfterIssuance)
				}
				// The producer's expiresAt is the window, and MaxAge does not
				// apply. Past the horizon it is clamped there, not refused: the
				// document was acceptable, and is now merely older than this
				// consumer will remember it.
				if end, ok := policy.horizonEnd(issuedAt); ok &&
					expiresAt.After(end) && !end.Add(policy.Skew).After(now) {
					return stale()
				}
				return nil
			}
		}

		// The window is half-open: refused from the instant [RecordExpiry] returns,
		// the instant a guard may drop the record, so there is no moment at which
		// the document is both acceptable and forgotten.
		if policy.MaxAge > 0 && !issuedAt.Add(policy.MaxAge+policy.Skew).After(now) {
			return stale()
		}
		return nil
	}

	if policy.RequireIssuedAt {
		return malformed(IssuedAtRequired)
	}

	// No issuedAt. A policy with a window cannot place the document in it unless
	// the producer supplied an expiresAt instead (SPEC §7.2, *Bounding the
	// record*).
	if policy.MaxAge > 0 && doc.ExpiresAt == nil {
		return stale()
	}
	return nil
}

// RecordExpiry is the instant past which a replay record for doc may be dropped —
// the end of this consumer's willingness to execute it, which SPEC §7.2 makes the
// same instant as the end of the record's required retention.
//
// expiresAt fixes it where present, clamped to issuedAt + ExpiryHorizon + Skew
// where it lies beyond the horizon; otherwise issuedAt + MaxAge + Skew. Each is
// the same bound [ValidateFreshness] refuses from, so a replay arriving inside
// the skew allowance still finds the record — and a producer cannot pin a record
// past the horizon by stating a distant expiresAt. The window is half-open — a
// document is acceptable strictly before this instant and refused at it, as for
// expiresAt — so a guard that drops a record once retainUntil is not after now
// never drops one the pipeline would still execute. The second
// return is false when this policy places no bound on the document, in which case
// a consumer MUST NOT execute a consequential task on it — there is no window in
// which to keep the record.
func RecordExpiry[P any](
	doc *Document[P],
	policy FreshnessPolicy,
	now time.Time,
) (time.Time, bool) {
	if doc.ExpiresAt != nil {
		if expiresAt, err := parseTimestamp(*doc.ExpiresAt); err == nil {
			// No issuedAt, no horizon: see ExpiryHorizon.
			if doc.IssuedAt != nil {
				if issuedAt, err := parseTimestamp(*doc.IssuedAt); err == nil {
					if end, ok := policy.horizonEnd(issuedAt); ok && expiresAt.After(end) {
						return end.Add(policy.Skew), true
					}
				}
			}
			return expiresAt, true
		}
	}
	if policy.MaxAge == 0 {
		return time.Time{}, false
	}
	// Fall back to now when the producer stamped no usable issuedAt: the record
	// then lives a full window from first sight, which is the longest the
	// document could still be arriving from a queue.
	base := now
	if doc.IssuedAt != nil {
		if issuedAt, err := parseTimestamp(*doc.IssuedAt); err == nil {
			base = issuedAt
		}
	}
	return base.Add(policy.MaxAge + policy.Skew), true
}
