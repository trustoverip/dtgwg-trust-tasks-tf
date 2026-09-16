/// Freshness bounds over `issuedAt` / `expiresAt` (SPEC §4.2, §7.2).
///
/// Hand-written. Mirrors `freshness.rs` in trust-tasks-rs,
/// `_runtime/freshness.ts` in @openvtc/trust-tasks and `trusttasks/freshness.go`
/// in trust-tasks-go, deliberately closely: a Dart consumer and a Rust,
/// TypeScript or Go one must reach the same verdict on the same document.
///
/// Without a freshness bound the only temporal check a runtime makes is §7.2
/// item 4 — `expiresAt`, and only where the producer chose to set it. That
/// accepts a document stamped a year in the future for the whole of that year,
/// and one whose `expiresAt` sits at or before its own `issuedAt`: a validity
/// interval that never contained a valid instant.
///
/// It is also what makes [ReplayGuard] implementable. SPEC §7.2 (*Bounding the
/// record*) ties the duplicate-execution record to the acceptance window and
/// says the two bounds are the same bound: a consumer "**MUST NOT** accept for
/// execution a document older than the window over which it retains records".
/// Without [FreshnessPolicy.maxAge] there is no window, so a record for a
/// document carrying no `expiresAt` would have to be kept forever.
library;

import 'codes.dart';
import 'document.dart';

/// The clock-skew tolerance SPEC §4.2 sanctions ("typically ≤ 60s").
const Duration defaultSkew = Duration(seconds: 60);

/// The acceptance window [consequentialChecks] applies.
///
/// Five minutes survives a mediator queue, a retry with backoff and a modest
/// clock disagreement, and keeps the record it bounds small. A deployment whose
/// transport buffers for longer must widen it *and* widen its guard's retention
/// to match — §7.2 makes them one bound.
const Duration defaultMaxAge = Duration(minutes: 5);

/// How a consumer bounds a document in time before acting on it.
class FreshnessPolicy {
  const FreshnessPolicy({
    this.skew = defaultSkew,
    this.maxAge,
    this.requireIssuedAt = false,
  });

  /// The minimum every consumer should apply: reject a future-dated document and
  /// one whose stated validity interval is empty. No conforming producer emits
  /// either, so this costs a correct deployment nothing.
  ///
  /// Deliberately sets no [maxAge] — an acceptance window depends on how long
  /// the transport may hold a message, which is a deployment fact, and a library
  /// that guessed one would start refusing documents that had arrived for years.
  static const FreshnessPolicy defaults = FreshnessPolicy();

  /// The posture SPEC §7.2 (*Bounding the record*) describes for a
  /// *consequential Trust Task* (§2): `issuedAt` REQUIRED and a bounded
  /// acceptance window, so every accepted document sits inside a window a
  /// [ReplayGuard] can retain a record for.
  static const FreshnessPolicy consequential = FreshnessPolicy(
    maxAge: defaultMaxAge,
    requireIssuedAt: true,
  );

  /// Tolerance applied to the document's timestamps against this consumer's
  /// clock, per SPEC §4.2. Applied to the future-dating check and to [maxAge];
  /// **not** to `expiresAt`, which [validateBasic] compares against the raw
  /// `now` it is given.
  final Duration skew;

  /// The oldest `issuedAt` this consumer accepts, measured back from `now`. Null
  /// means unbounded — and is the only setting under which a document carrying
  /// neither timestamp is acceptable.
  final Duration? maxAge;

  /// Reject a document carrying no `issuedAt`, with `malformedRequest`.
  final bool requireIssuedAt;
}

/// Wire-safe reason for a `malformedRequest` from a future-dated `issuedAt`.
const String futureIssuedAt =
    "issuedAt is in the future beyond the consumer's skew tolerance (SPEC §4.2)";

/// Wire-safe reason for a `malformedRequest` from `expiresAt <= issuedAt`.
const String expiryNotAfterIssuance =
    'expiresAt is not after issuedAt: the document states an empty validity '
    'interval (SPEC §4.2)';

/// Wire-safe reason for a `malformedRequest` from a missing `issuedAt`.
const String issuedAtRequired =
    'issuedAt is required by consumer policy (SPEC §7.2, bounding the '
    'duplicate-execution record)';

/// Wire message for a document outside the consumer's acceptance window.
///
/// A constant, not a rendering of the window or the consumer's clock: §10.4
/// keeps consumer-side state off the wire, and echoing the delta would turn
/// every rejection into a remote `ntpdate` — and a probe for the window's exact
/// boundary — for an unauthenticated sender.
const String staleWireMessage =
    "document is outside the consumer's acceptance window (SPEC §7.2)";

/// Apply [policy] to this document's `issuedAt` / `expiresAt`. Returns null when
/// the document is acceptable.
///
/// This is the freshness half of SPEC §7.2 item 4 that [validateBasic] does not
/// cover. [consumeInbound] calls it for you.
RejectReason? validateFreshness<P>(
  TrustTaskDocument<P> doc,
  DateTime now,
  FreshnessPolicy policy,
) {
  RejectReason malformed(String message) =>
      RejectReason(code: StandardCode.malformedRequest, message: message);
  RejectReason stale() => const RejectReason(
        code: StandardCode.expired,
        message: staleWireMessage,
      );

  final issuedAtRaw = doc.issuedAt;
  if (issuedAtRaw != null) {
    final issuedAt = DateTime.tryParse(issuedAtRaw);
    if (issuedAt == null) {
      return malformed('issuedAt is not a valid RFC 3339 timestamp');
    }
    if (issuedAt.isAfter(now.add(policy.skew))) {
      return malformed(futureIssuedAt);
    }

    final expiresAtRaw = doc.expiresAt;
    if (expiresAtRaw != null) {
      // A malformed `expiresAt` is [validateBasic]'s to report; skip it here
      // rather than raise a second, differently-worded rejection for it.
      final expiresAt = DateTime.tryParse(expiresAtRaw);
      if (expiresAt != null && !expiresAt.isAfter(issuedAt)) {
        return malformed(expiryNotAfterIssuance);
      }
    }

    final maxAge = policy.maxAge;
    if (maxAge != null && issuedAt.add(maxAge + policy.skew).isBefore(now)) {
      return stale();
    }
    return null;
  }

  if (policy.requireIssuedAt) return malformed(issuedAtRequired);

  // No `issuedAt`. A policy with a window cannot place the document in it unless
  // the producer supplied an `expiresAt` instead (SPEC §7.2, *Bounding the
  // record*).
  if (policy.maxAge != null && doc.expiresAt == null) return stale();
  return null;
}

/// The instant past which a replay record for [doc] may be dropped — the end of
/// this consumer's willingness to execute it, which SPEC §7.2 makes the same
/// instant as the end of the record's required retention.
///
/// `expiresAt` fixes it where present; otherwise `issuedAt + maxAge`. Null means
/// this policy places no bound on the document, in which case a consumer **MUST
/// NOT** execute a consequential task on it — there is no window in which to
/// keep the record.
DateTime? recordExpiry<P>(
  TrustTaskDocument<P> doc,
  FreshnessPolicy policy,
  DateTime now,
) {
  final expiresAtRaw = doc.expiresAt;
  if (expiresAtRaw != null) {
    final expiresAt = DateTime.tryParse(expiresAtRaw);
    if (expiresAt != null) return expiresAt;
  }
  final maxAge = policy.maxAge;
  if (maxAge == null) return null;

  // Fall back to `now` when the producer stamped no usable `issuedAt`: the
  // record then lives a full window from first sight, which is the longest the
  // document could still be arriving from a queue.
  final issuedAtRaw = doc.issuedAt;
  final base =
      issuedAtRaw == null ? now : (DateTime.tryParse(issuedAtRaw) ?? now);
  return base.add(maxAge);
}
