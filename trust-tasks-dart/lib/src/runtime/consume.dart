/// Inbound-document orchestration for SPEC.md §7.2 item 2, items 4–8, and the
/// two stateful checks — the freshness bound of item 4 and the
/// duplicate-execution record of item 11.
///
/// Hand-written. Mirrors `consume.rs` in trust-tasks-rs, `_runtime/consume.ts`
/// in @openvtc/trust-tasks and `trusttasks/consume.go` in trust-tasks-go,
/// deliberately closely: a Dart consumer and a Rust, TypeScript or Go one must
/// reach the same verdict on the same document, or the reference implementations
/// disagree about what conforms.
///
/// ```dart
/// // One guard per consumer, not one per document: it *is* the record.
/// final guard = InMemoryReplayGuard();
///
/// final outcome = await consumeInbound<acl_grant.Payload, acl_grant.Response>(
///   transport: transport,
///   spec: acl_grant.spec,
///   proofPolicy: ProofPolicy.verify(myVerifier),
///   payloadPolicy: PayloadPolicy.validate(mySchemaEngine),
///   // acl/grant is consequential: a replayed envelope must not grant twice.
///   checks: consequentialChecks(guard),
///   doc: doc,
///   myVid: 'did:web:maintainer.example',
///   now: DateTime.now(),
///   newErrorId: () => uuid.v4(),
///   payloadToJson: (p) => p.toJson(),
///   handler: (accepted, parties) async =>
///       respondWith(accepted, uuid.v4(), await applyGrant(accepted.payload)),
/// );
///
/// switch (outcome) {
///   case Handled(:final response):   emit(response);
///   case Rejected(:final error):     emit(error);
///   case Suppressed(:final reason):  log(reason);
///   case Accepted():                 break; // fire-and-forget
///   case DuplicateOutcome(:final priorResponse):
///     if (priorResponse != null) emit(priorResponse);
/// }
/// ```
///
/// Items 1 (framework schema) and 3 (unknown `type`) are *not* attempted here —
/// they belong to the caller's parse and dispatch.
///
/// Item 2 (payload schema) is. A Dart class enforces member *types* on decode
/// but nothing else — not `minLength`, not `pattern`, not the `oneOf` mutual
/// exclusion — so the residue that matters is exactly what a validator catches.
/// That is why [payloadPolicy] is a required argument rather than an optional
/// one that defaults to skipping.
library;

import 'codes.dart';
import 'document.dart';
import 'freshness.dart';
import 'replay.dart';
import 'transport.dart';

/// The two stateful §7.2 checks: the freshness bound over `issuedAt` /
/// `expiresAt`, and the duplicate-execution record of item 11.
///
/// They travel together because the spec ties them together. §7.2 (*Bounding the
/// record*) makes the acceptance window and the record's retention **the same
/// bound**. Passing them as one argument is what stops a deployment configuring
/// a five-minute record and an unbounded acceptance window, which reads as a
/// working replay defence and is not one.
///
/// Like [PayloadPolicy], this is a *required* argument. Whether the task a
/// consumer implements is *consequential* (§2) is a decision only that consumer
/// can make, and the failure mode of getting it wrong silently — an ACL grant
/// applied twice by a mediator retry — is not one to discover after the fact.
class ConsumeChecks {
  const ConsumeChecks({required this.freshness, required this.replay});

  /// Bounds the document in time (SPEC §4.2, §7.2 item 4).
  final FreshnessPolicy freshness;

  /// Applies (or knowingly disapplies) SPEC §7.2 item 11.
  final ReplayPolicy replay;
}

/// The posture for a *consequential Trust Task* (§2): item 11 enforced against
/// [guard], and the bounded acceptance window that makes the record droppable.
///
/// The right choice for any task whose execution grants access, moves value,
/// discloses a secret, or otherwise cannot be undone by ignoring the next
/// document.
ConsumeChecks consequentialChecks(ReplayGuard guard) => ConsumeChecks(
      freshness: FreshnessPolicy.consequential,
      replay: ReplayPolicy.guarded(guard),
    );

/// The posture for a task that is **not** consequential, or whose specification
/// "explicitly declares repeated execution safe and intended" — the narrow
/// disapplication item 11 permits.
///
/// Keeps no record. Still applies the default freshness policy, because a
/// future-dated document and one with an empty validity interval are malformed
/// whatever the task does.
ConsumeChecks notConsequentialChecks() => const ConsumeChecks(
      freshness: FreshnessPolicy.defaults,
      replay: ReplayPolicy.notConsequential(),
    );

/// Verifies a document's Data Integrity proof (SPEC §4.7).
///
/// The document arrives as the decoded JSON map [consumeInbound] holds it in,
/// because a Data Integrity verifier works over the serialization rather than
/// over the typed payload. Return normally when the proof verifies against the
/// in-band `issuer`; throw, or return false, when it does not.
abstract interface class ProofVerifier {
  Future<bool> verify(Map<String, dynamic> doc);
}

/// How [consumeInbound] treats a document's `proof` member (§7.2 item 7).
///
/// The framework does not assume what integrity guarantees a consumer relies on.
/// Some verify Data Integrity proofs in-band; some have transport-layer
/// integrity (signed DIDComm, mTLS-bound HTTPS) and accept in-band proofs only
/// opportunistically; some have none. Making the choice explicit at the call
/// site is the point.
///
/// `spec.isProofRequired` is consulted independently of the policy: a
/// specification that requires a proof rejects a proofless document whichever
/// policy is chosen.
sealed class ProofPolicy {
  const ProofPolicy();

  /// Verify the proof when present. Failures map to `proofInvalid`. The safe
  /// default for any consumer that honours in-band proofs.
  const factory ProofPolicy.verify(ProofVerifier verifier) = VerifyProof;

  /// Reject a document that carries an in-band proof, with `malformedRequest`.
  /// For consumers with integrity from another layer that are deliberately not
  /// verifying in-band proofs — silently dropping a producer-supplied proof
  /// would mislead the producer about the guarantees of the exchange.
  const factory ProofPolicy.rejectIfPresent() = RejectProofIfPresent;

  /// SECURITY: accept any document, proof or not, without verifying. Only where
  /// the transport already provides equivalent end-to-end integrity. This is the
  /// explicit opt-out, and the name is deliberately uncomfortable to type.
  const factory ProofPolicy.acceptUnverified() = AcceptProofUnverified;
}

final class VerifyProof extends ProofPolicy {
  const VerifyProof(this.verifier);
  final ProofVerifier verifier;
}

final class RejectProofIfPresent extends ProofPolicy {
  const RejectProofIfPresent();
}

final class AcceptProofUnverified extends ProofPolicy {
  const AcceptProofUnverified();
}

/// Evaluates a payload against its schema (SPEC §7.2 item 2).
abstract interface class PayloadValidator {
  /// Check [payload] against [schemaJson] — the value from
  /// [SpecPolicy.payloadSchema], a JSON Schema 2020-12 document with all
  /// cross-file `$ref`s already inlined, so no resolver is needed. It is
  /// generated from this repo at build time, so it is trusted input.
  ///
  /// Return null to accept, or a human-readable reason to reject; the reason
  /// lands in the `malformedRequest` message so a caller can see what failed.
  String? validate(String schemaJson, Object? payload);
}

/// How [consumeInbound] performs SPEC §7.2 item 2 — payload-schema validation.
///
/// This package bundles no JSON Schema implementation, for the same reason it
/// bundles no cryptosuite: the engine, its draft support and its resource limits
/// are the consumer's choice, and a zero-dependency package is worth keeping. So
/// the schema ships with the generated library and the validator comes from you.
sealed class PayloadPolicy {
  const PayloadPolicy();

  /// Validate against `spec.payloadSchema`. Failures map to `malformedRequest`.
  const factory PayloadPolicy.validate(PayloadValidator validator) =
      ValidatePayload;

  /// Skip item 2 entirely. Appropriate only where something upstream — an API
  /// gateway, a schema-validating transport — has already performed it on the
  /// same bytes.
  const factory PayloadPolicy.acceptUnvalidated() = AcceptPayloadUnvalidated;
}

final class ValidatePayload extends PayloadPolicy {
  const ValidatePayload(this.validator);
  final PayloadValidator validator;
}

final class AcceptPayloadUnvalidated extends PayloadPolicy {
  const AcceptPayloadUnvalidated();
}

/// The outcome of [consumeInbound].
///
/// A sealed hierarchy, so a `switch` over it is exhaustive. Unlike the wire
/// values in `codes.dart`, these are produced by this library and never parsed
/// from a peer, so closing the set costs nothing in forward compatibility.
sealed class ConsumeOutcome<R> {
  const ConsumeOutcome();
}

/// Every check passed and the caller's handler produced a response.
final class Handled<R> extends ConsumeOutcome<R> {
  const Handled(this.response);
  final TrustTaskDocument<R> response;
}

/// A framework check failed, or the handler refused. Either way the document is
/// already addressed per §8.1 — emit it over the transport.
final class Rejected<R> extends ConsumeOutcome<R> {
  const Rejected(this.error);
  final ErrorResponse error;
}

/// §8.1: the rejection was `identityMismatch` and the transport authenticated no
/// sender, so no response may be emitted — one would be an oracle.
///
/// Callers SHOULD log this: silent suppression is the spec rule, but *invisible*
/// suppression is an operational footgun.
final class Suppressed<R> extends ConsumeOutcome<R> {
  const Suppressed(this.reason);
  final RejectReason reason;
}

/// Every check passed and the handler completed without producing a document — a
/// fire-and-forget task.
///
/// SPEC §4.4.1: a specification that defines no success response is one whose
/// consumers **MUST NOT** emit a `#response`-variant document. Emit nothing.
final class Accepted<R> extends ConsumeOutcome<R> {
  const Accepted();
}

/// SPEC §7.2 item 11: a document with this `id` and this content was already
/// accepted for execution. **The handler was not called**, and the consequential
/// effect did not happen a second time. This is the §8.4 retry being absorbed,
/// which is what makes retrying safe.
///
/// **Not an error.** §7.2 (*Disposition of a duplicate*): "In no case is a
/// duplicate reported as `taskFailed`; the task did not fail, it already
/// happened." Folding this into [Rejected] would report a failure that did not
/// occur.
///
/// What to emit:
///
/// * [priorResponse] non-null — emit it. It is the response document the first
///   execution produced, which §7.2 says the consumer SHOULD return.
/// * [priorResponse] null, [inFlight] false — the specification defines no
///   success response (§4.4.1), or the guard retains none. Emit nothing.
/// * [inFlight] true — the first execution has not finished. §7.2: the consumer
///   SHOULD "return or expose the existing execution state rather than begin
///   another".
final class DuplicateOutcome<R> extends ConsumeOutcome<R> {
  const DuplicateOutcome({this.priorResponse, this.inFlight = false});
  final Object? priorResponse;
  final bool inFlight;
}

/// Wire message for the [RejectProofIfPresent] path.
const String proofNotAcceptedByPolicy =
    'in-band proof not accepted by consumer policy (SPEC §7.2 item 7)';

/// Wire message for `proofInvalid`.
///
/// Constant by design — see the §10.4 note at the rejection site. The
/// counterparts in the other runtimes are kept equal to it.
const String proofInvalidWireMessage = 'proof verification failed';

/// Wire message for `idConflict` (SPEC §7.2 item 11, §8.3).
const String idConflictWireMessage =
    'a different document has already been accepted under this id '
    '(SPEC §7.2 item 11)';

/// Wire message for a replay-record outage. A constant, so a store's hostname or
/// connection string never reaches the wire (SPEC §10.4).
const String replayRecordUnavailable = 'temporarily unavailable';

/// The business handler, called only once every framework check has passed.
///
/// Return a success response, or null for a specification that defines no
/// success response (§4.4.1) — which yields [Accepted]. Throw a [Refusal] to
/// refuse with a built error response; any other exception propagates to the
/// caller of [consumeInbound].
typedef Handler<P, R> = Future<TrustTaskDocument<R>?> Function(
  TrustTaskDocument<P> doc,
  ResolvedParties parties,
);

/// Thrown by a [Handler] to refuse with a specific error response.
///
/// The response is passed through verbatim — the framework does not re-apply
/// §8.1 routing to it. A handler refusing for identity-style reasons MUST
/// address the response itself (see [reject]); [rejectWith] copies `doc.issuer`
/// into `recipient`, which is safe for ordinary refusals but not for one that
/// contests that identity.
class Refusal implements Exception {
  const Refusal(this.response);
  final ErrorResponse response;

  @override
  String toString() => 'Refusal(${response.payload.code})';
}

/// Build a handler-side refusal addressed to the original producer.
///
/// Convenience over [rejectWith] plus [toErrorPayload] for the common case where
/// a handler refuses for a business reason. Not safe for refusals that contest
/// the in-band identity — see [Refusal].
Refusal refuse<P>(
  TrustTaskDocument<P> request,
  String id,
  RejectReason reason, {
  Clock? clock,
}) =>
    Refusal(rejectWith(request, id, toErrorPayload(reason), clock: clock));

/// Run SPEC §7.2 items 4–8 against [doc], then either call [handler] or build
/// the routed error response per §8.1.
Future<ConsumeOutcome<R>> consumeInbound<P, R>({
  required TransportHandler transport,
  required SpecPolicy spec,
  required ProofPolicy proofPolicy,
  required PayloadPolicy payloadPolicy,
  required ConsumeChecks checks,
  required TrustTaskDocument<P> doc,
  required String myVid,
  required DateTime now,
  required String Function() newErrorId,
  required Object? Function(P payload) payloadToJson,
  required Handler<P, R> handler,
  Clock? clock,
}) async {
  ConsumeOutcome<R> route(RejectReason reason) {
    final error = reject(transport, doc, newErrorId(), reason, clock: clock);
    return error == null ? Suppressed<R>(reason) : Rejected<R>(error);
  }

  // §7.2 item 2 — payload schema. Runs first, in the spec's own order, and
  // before anything that reasons about what the payload means: a payload that is
  // not the shape the specification declares should be refused as malformed
  // rather than interpreted.
  final schema = spec.payloadSchema;
  if (payloadPolicy is ValidatePayload && schema != null) {
    String? failure;
    try {
      failure =
          payloadPolicy.validator.validate(schema, payloadToJson(doc.payload));
    } on Object catch (e) {
      // A validator that throws has not accepted the document. Treating an
      // exception as a pass would make a broken validator indistinguishable from
      // a passing one — the failure mode this policy exists to remove.
      failure = e.toString();
    }
    if (failure != null) {
      final detail = failure.trim();
      return route(
        RejectReason(
          code: StandardCode.malformedRequest,
          message: detail.isEmpty
              ? 'payload does not conform to its schema (SPEC §7.2 item 2)'
              : 'payload does not conform to its schema (SPEC §7.2 item 2): $detail',
        ),
      );
    }
  }

  // §7.2 items 4 + 5a — expiry and wrong-recipient.
  final basic = validateBasic(doc, now, myVid);
  if (basic != null) return route(basic);

  // §7.2 item 4, the other half — the freshness bound over `issuedAt`.
  // `validateBasic` honours `expiresAt`, which is optional and which a producer
  // sets for its own reasons; on its own it leaves a document stamped years ago,
  // or years hence, indefinitely acceptable. It is also what bounds the replay
  // record below: §7.2 makes the acceptance window and the record's retention
  // one bound.
  final fresh = validateFreshness(doc, now, checks.freshness);
  if (fresh != null) return route(fresh);

  // §7.2 item 6 — in-band vs transport-derived identity cross-check.
  final resolution = resolveParties(transport, doc);
  final mismatch = resolution.error;
  if (mismatch != null) return route(identityMismatchReason(mismatch));
  final parties = resolution.parties!;

  // §7.2 item 7 clause B — the consumer's chosen proof policy.
  if (doc.proof != null) {
    switch (proofPolicy) {
      case VerifyProof(:final verifier):
        var ok = false;
        try {
          ok = await verifier.verify(doc.toJson(payloadToJson));
        } on Object {
          ok = false;
        }
        if (!ok) {
          // A constant, never the verifier's own error text. SPEC §10.4 extends
          // the §8.1 identity rule to every code, and a verifier's vocabulary
          // names DIDs it tried to resolve, whether a resolver answered, and
          // what a fetched DID document contained — a resolver-reachability
          // oracle for a sender who is, by construction, unauthenticated. Log
          // the detail; do not send it.
          return route(
            const RejectReason(
              code: StandardCode.proofInvalid,
              message: proofInvalidWireMessage,
            ),
          );
        }
      case RejectProofIfPresent():
        return route(
          const RejectReason(
            code: StandardCode.malformedRequest,
            message: proofNotAcceptedByPolicy,
          ),
        );
      case AcceptProofUnverified():
        break;
    }
  }

  // §7.2 items 5b + 7 clause A + 8 — the policy-driven checks, in one place so
  // this pipeline and any binding-specific one cannot diverge on the check set.
  final policy = enforceSpecPolicy(doc, spec);
  if (policy != null) return route(policy);

  // §7.2 item 11 — the duplicate-execution record. Deliberately **last**:
  // claiming the `id` marks the document as accepted for execution, and a
  // document some earlier check refuses was never accepted. Claiming first would
  // burn the `id` on every malformed or unauthorised arrival, so a corrected
  // resend under the same `id` would come back `idConflict` forever — and an
  // attacker could pre-burn an `id` it had merely observed.
  ReplayGuard? claimedGuard;
  String? claimedDigest;
  final replay = checks.replay;
  if (replay is GuardedReplay) {
    final guard = replay.guard;
    final digest = documentDigest(doc, payloadToJson);

    // §7.2 (*Bounding the record*): "A consumer that can establish neither an
    // `expiresAt` nor an age for a document has no window in which to place it,
    // and MUST NOT execute a consequential Trust Task on it." A guard asked to
    // retain a record forever is not a guard, so refuse rather than pretend.
    final retainUntil = recordExpiry(doc, checks.freshness, now);
    if (retainUntil == null) {
      return route(
        const RejectReason(
          code: StandardCode.expired,
          message: staleWireMessage,
        ),
      );
    }

    ReplayVerdict verdict;
    try {
      verdict = await guard.claim(doc.id, digest, retainUntil, now);
    } on Object {
      // Fail closed. A consumer that cannot consult its record has not satisfied
      // item 11, and executing anyway is exactly the double execution the rule
      // forbids. `unavailable` is retryable, which is the honest answer: the
      // producer's bit-for-bit resend will be absorbed correctly once the store
      // is back. The thrown detail — a hostname, a connection string — stays out
      // of the message, per §10.4.
      return route(
        const RejectReason(
          code: StandardCode.unavailable,
          message: replayRecordUnavailable,
          retryable: true,
        ),
      );
    }

    switch (verdict) {
      case Duplicate(:final priorResponse, :final inFlight):
        return DuplicateOutcome<R>(
          priorResponse: priorResponse,
          inFlight: inFlight,
        );
      case Conflict():
        return route(
          const RejectReason(
            code: StandardCode.idConflict,
            message: idConflictWireMessage,
          ),
        );
      case Fresh():
        claimedGuard = guard;
        claimedDigest = digest;
    }
  }

  /// Record the response for a completed execution, best-effort.
  ///
  /// The effect has already happened, so a guard that cannot cache the response
  /// cannot un-happen it. The record of the *claim* is what item 11 needs and it
  /// is already written; all that is lost is the ability to hand the same
  /// response back, and the duplicate is still absorbed.
  Future<void> settle(Object? response) async {
    if (claimedGuard == null) return;
    try {
      await claimedGuard.recordResponse(doc.id, response);
    } on Object {
      /* see above */
    }
  }

  TrustTaskDocument<R>? response;
  try {
    response = await handler(doc, parties);
  } on Refusal catch (refusal) {
    // §8.4: a retryable refusal has just invited the producer to re-send this
    // document bit-for-bit. Holding the claim would answer that invited retry
    // with the cached failure forever. A non-retryable refusal is final, so the
    // record stands and a replay is answered with the same determination.
    if (refusal.response.payload.retryable) {
      if (claimedGuard != null) {
        try {
          await claimedGuard.release(doc.id, claimedDigest!);
        } on Object {
          /* best-effort */
        }
      }
    } else {
      await settle(refusal.response.toJson((p) => p.toJson()));
    }
    return Rejected<R>(refusal.response);
  }

  if (response == null) {
    // Fire-and-forget: nothing to cache, but the claim stands — the effect
    // happened, and item 11 is about the effect, not about the response.
    await settle(null);
    return Accepted<R>();
  }

  await settle(response);
  return Handled<R>(response);
}

/// Whether a document is an error response rather than a success response.
///
/// Keys off the `trust-task-error` Type URI rather than payload shape: §8 makes
/// the type the discriminator, and a success payload could coincidentally carry
/// a `code` member.
bool isErrorResponse<P>(TrustTaskDocument<P> doc) =>
    bareTypeUri(doc.type).contains('/trust-task-error/');
