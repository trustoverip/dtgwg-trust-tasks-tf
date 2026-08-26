/**
 * Freshness bounds over `issuedAt` / `expiresAt` (SPEC §4.2, §7.2).
 *
 * Mirrors `freshness.rs` in trust-tasks-rs, deliberately closely: a TypeScript
 * consumer and a Rust one must reach the same verdict on the same document.
 *
 * Until this module, the only temporal check either runtime made was §7.2
 * item 4 — `expiresAt`, and only where the producer chose to set it.
 * `issuedAt` was carried on the type and looked at by nobody. That accepted a
 * document stamped a year in the future (for the whole of that year), and one
 * whose `expiresAt` sat at or before its own `issuedAt` — a validity interval
 * that never contained a valid instant.
 *
 * It is also what makes {@link ReplayGuard} implementable. SPEC §7.2
 * (*Bounding the record*) ties the duplicate-execution record to the
 * acceptance window and says the two bounds are the same bound: a consumer
 * "**MUST NOT** accept for execution a document older than the window over
 * which it retains records". Without {@link FreshnessPolicy.maxAgeMs} there is
 * no window, so a record for a document carrying no `expiresAt` would have to
 * be kept forever.
 */

import type { RejectReason, TrustTaskDocument } from "./document.js";

/** The clock-skew tolerance SPEC §4.2 sanctions ("typically ≤ 60s"), in ms. */
export const DEFAULT_SKEW_MS = 60_000;

/**
 * The acceptance window {@link consequentialFreshness} applies, in ms.
 *
 * Five minutes survives a mediator queue, a retry with backoff and a modest
 * clock disagreement, and keeps the record it bounds small. A deployment whose
 * transport buffers for longer must widen it *and* widen its guard's retention
 * to match — §7.2 makes them one bound.
 */
export const DEFAULT_MAX_AGE_MS = 5 * 60_000;

/** How a consumer bounds a document in time before acting on it. */
export interface FreshnessPolicy {
  /**
   * Tolerance applied to the document's timestamps against this consumer's
   * clock, per SPEC §4.2. Applied to the future-dating check and to
   * {@link maxAgeMs}; **not** to `expiresAt`, which `validateBasic` compares
   * against the raw `now` it is given.
   */
  readonly skewMs: number;
  /**
   * Oldest `issuedAt` this consumer accepts, measured back from `now`.
   * `undefined` means unbounded — the pre-existing behaviour, and the only
   * setting under which a document carrying neither timestamp is acceptable.
   */
  readonly maxAgeMs?: number;
  /** Reject a document carrying no `issuedAt`, with `malformedRequest`. */
  readonly requireIssuedAt?: boolean;
}

/**
 * The minimum every consumer should apply: reject a future-dated document and
 * one whose stated validity interval is empty. No conforming producer emits
 * either, so this costs a correct deployment nothing.
 *
 * Deliberately sets no `maxAgeMs` — an acceptance window depends on how long
 * the transport may hold a message, which is a deployment fact, and a library
 * that guessed one would start refusing documents that had arrived for years.
 */
export const DEFAULT_FRESHNESS: FreshnessPolicy = { skewMs: DEFAULT_SKEW_MS };

/**
 * The posture SPEC §7.2 (*Bounding the record*) describes for a *consequential
 * Trust Task*: `issuedAt` REQUIRED and a bounded acceptance window, so every
 * accepted document sits inside a window a {@link ReplayGuard} can retain a
 * record for.
 */
export const CONSEQUENTIAL_FRESHNESS: FreshnessPolicy = {
  skewMs: DEFAULT_SKEW_MS,
  maxAgeMs: DEFAULT_MAX_AGE_MS,
  requireIssuedAt: true,
};

/** Wire-safe reason for a `malformedRequest` from a future-dated `issuedAt`. */
export const FUTURE_ISSUED_AT =
  "issuedAt is in the future beyond the consumer's skew tolerance (SPEC §4.2)";

/** Wire-safe reason for a `malformedRequest` from `expiresAt <= issuedAt`. */
export const EXPIRY_NOT_AFTER_ISSUANCE =
  "expiresAt is not after issuedAt: the document states an empty validity interval (SPEC §4.2)";

/** Wire-safe reason for a `malformedRequest` from a missing `issuedAt`. */
export const ISSUED_AT_REQUIRED =
  "issuedAt is required by consumer policy (SPEC §7.2, bounding the duplicate-execution record)";

/**
 * Wire message for a document outside the consumer's acceptance window.
 *
 * A constant, not a rendering of the window or the consumer's clock: §10.4
 * keeps consumer-side state off the wire, and echoing the delta would turn
 * every rejection into a remote `ntpdate` — and a probe for the window's exact
 * boundary — for an unauthenticated sender.
 */
export const STALE_WIRE_MESSAGE = "document is outside the consumer's acceptance window (SPEC §7.2)";

/**
 * Apply `policy` to this document's `issuedAt` / `expiresAt`. Returns `null`
 * when the document is acceptable.
 *
 * This is the freshness half of SPEC §7.2 item 4 that `validateBasic` does not
 * cover. {@link consumeInbound} calls it for you.
 *
 * @param now Milliseconds since the epoch, as from `Date.now()`.
 */
export function validateFreshness<P>(
  doc: TrustTaskDocument<P>,
  now: number,
  policy: FreshnessPolicy,
): RejectReason | null {
  const malformed = (message: string): RejectReason => ({
    code: "malformedRequest",
    message,
    retryable: false,
  });
  const stale = (): RejectReason => ({
    code: "expired",
    message: STALE_WIRE_MESSAGE,
    retryable: false,
  });

  if (doc.issuedAt !== undefined) {
    const issuedAt = Date.parse(doc.issuedAt);
    if (Number.isNaN(issuedAt)) {
      return malformed("issuedAt is not a valid RFC 3339 timestamp");
    }
    if (issuedAt > now + policy.skewMs) return malformed(FUTURE_ISSUED_AT);

    if (doc.expiresAt !== undefined) {
      const expiresAt = Date.parse(doc.expiresAt);
      // A malformed `expiresAt` is `validateBasic`'s to report; skip it here
      // rather than raise a second, differently-worded rejection for it.
      if (!Number.isNaN(expiresAt) && expiresAt <= issuedAt) {
        return malformed(EXPIRY_NOT_AFTER_ISSUANCE);
      }
    }

    if (policy.maxAgeMs !== undefined && issuedAt + policy.maxAgeMs + policy.skewMs < now) {
      return stale();
    }
    return null;
  }

  if (policy.requireIssuedAt === true) return malformed(ISSUED_AT_REQUIRED);

  // No `issuedAt`. A policy with a window cannot place the document in it
  // unless the producer supplied an `expiresAt` instead (SPEC §7.2, *Bounding
  // the record*).
  if (policy.maxAgeMs !== undefined && doc.expiresAt === undefined) return stale();
  return null;
}

/**
 * The instant past which a replay record for `doc` may be dropped — the end of
 * this consumer's willingness to execute it, which SPEC §7.2 makes the same
 * instant as the end of the record's required retention.
 *
 * `expiresAt` fixes it where present; otherwise `issuedAt + maxAgeMs`.
 * `undefined` means this policy places no bound on the document, in which case
 * a consumer **MUST NOT** execute a consequential task on it — there is no
 * window in which to keep the record.
 */
export function recordExpiry<P>(
  doc: TrustTaskDocument<P>,
  policy: FreshnessPolicy,
  now: number,
): number | undefined {
  if (doc.expiresAt !== undefined) {
    const expiresAt = Date.parse(doc.expiresAt);
    if (!Number.isNaN(expiresAt)) return expiresAt;
  }
  if (policy.maxAgeMs === undefined) return undefined;
  const issuedAt = doc.issuedAt === undefined ? NaN : Date.parse(doc.issuedAt);
  // Fall back to `now` when the producer stamped no usable `issuedAt`: the
  // record then lives a full window from first sight, which is the longest the
  // document could still be arriving from a queue.
  return (Number.isNaN(issuedAt) ? now : issuedAt) + policy.maxAgeMs;
}
