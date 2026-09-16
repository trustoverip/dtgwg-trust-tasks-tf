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

// FreshnessPolicy is how a consumer bounds a document in time before acting on
// it.
type FreshnessPolicy struct {
	// Skew is the tolerance applied to the document's timestamps against this
	// consumer's clock, per SPEC §4.2. Applied to the future-dating check and to
	// MaxAge; not to expiresAt, which [ValidateBasic] compares against the raw
	// now it is given.
	Skew time.Duration

	// MaxAge is the oldest issuedAt this consumer accepts, measured back from
	// now. Zero means unbounded — and is the only setting under which a document
	// carrying neither timestamp is acceptable.
	MaxAge time.Duration

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
		RequireIssuedAt: true,
	}
}

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
			}
		}

		if policy.MaxAge > 0 && issuedAt.Add(policy.MaxAge+policy.Skew).Before(now) {
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
// expiresAt fixes it where present; otherwise issuedAt + MaxAge. The second
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
	return base.Add(policy.MaxAge), true
}
