/// The integration point between the transport-agnostic document model and a
/// concrete transport binding (HTTPS + mTLS, DIDComm, TSP, an in-memory test
/// loopback, …).
///
/// Hand-written. Mirrors `transport.rs` in trust-tasks-rs, `_runtime/transport.ts`
/// in @openvtc/trust-tasks and `trusttasks/transport.go` in trust-tasks-go. The
/// contract encodes SPEC.md §4.8.1 and §9.2:
///
/// * In-band `issuer` / `recipient` are authoritative when present.
/// * Transport-derived identity fills in absent members.
/// * When both are present they MUST agree; a mismatch is a validation failure
///   (`identityMismatch`, §8.3).
library;

import 'codes.dart';
import 'document.dart';

/// Party identity after §4.8.1 precedence — the values a consumer applies for
/// every subsequent framework rule referencing the issuer or recipient.
class ResolvedParties {
  const ResolvedParties({this.issuer, this.recipient});

  final String? issuer;
  final String? recipient;
}

/// What the transport authenticated about an inbound message.
///
/// A field is null when the transport does not authenticate that party — a plain
/// HTTPS handler with no client certificate leaves [issuer] unset. An entirely
/// empty context is valid; the framework then relies on the in-band members and
/// any `proof` they carry.
class TransportContext {
  const TransportContext({this.issuer, this.recipient});

  final String? issuer;
  final String? recipient;
}

/// Raised when in-band and transport-derived identity disagree (§7.2 item 6).
class ConsistencyError {
  const ConsistencyError({
    required this.party,
    required this.inBand,
    required this.transport,
  });

  /// `'issuer'` or `'recipient'`.
  final String party;
  final String inBand;
  final String transport;

  /// Local diagnostics only. The wire message is [identityMismatchReason]'s,
  /// which deliberately names neither value — see the note there.
  @override
  String toString() =>
      'ConsistencyError: in-band $party "$inBand" does not match '
      'transport-derived "$transport"';
}

/// A transport binding's plug-in for the framework.
abstract interface class TransportHandler {
  /// A stable identifier for this binding, for logs and audit (§9.1, §9.2).
  String get bindingUri;

  /// The identities the transport authenticated for the message under
  /// consideration.
  TransportContext deriveParties();
}

/// The result of [resolveParties]: either the resolved parties or a mismatch.
class PartyResolution {
  const PartyResolution.resolved(ResolvedParties this.parties) : error = null;
  const PartyResolution.mismatch(ConsistencyError this.error) : parties = null;

  final ResolvedParties? parties;
  final ConsistencyError? error;
}

/// Apply §4.8.1 precedence to produce the final [ResolvedParties].
///
/// Reports a [ConsistencyError] when an in-band member is present and disagrees
/// with the transport-derived value for the same party. Callers translate that
/// into an `identityMismatch` error response (§8.3) — or let [consumeInbound] do
/// it.
PartyResolution resolveParties<P>(
  TransportHandler handler,
  TrustTaskDocument<P> doc,
) {
  final ctx = handler.deriveParties();

  final pairs = <(String, String?, String?)>[
    ('issuer', doc.issuer, ctx.issuer),
    ('recipient', doc.recipient, ctx.recipient),
  ];
  for (final (party, inBand, transport) in pairs) {
    if (inBand != null && transport != null && inBand != transport) {
      return PartyResolution.mismatch(
        ConsistencyError(party: party, inBand: inBand, transport: transport),
      );
    }
  }

  return PartyResolution.resolved(
    ResolvedParties(
      issuer: doc.issuer ?? ctx.issuer,
      recipient: doc.recipient ?? ctx.recipient,
    ),
  );
}

/// Build the error response for [doc], applying the §8.1 routing rules.
///
/// Returns null when the rejection is `identityMismatch` and the transport
/// authenticated no sender. §8.1 is explicit that the consumer SHOULD NOT emit a
/// response in that case: the in-band `issuer` is by definition the contested
/// identity, so addressing it would be an oracle, and on any transport that
/// signs error responses it would compel the consumer to emit a signed document
/// about a party that did not take part in the exchange.
ErrorResponse? reject<P>(
  TransportHandler handler,
  TrustTaskDocument<P> doc,
  String id,
  RejectReason reason, {
  Clock? clock,
}) {
  String? recipient;
  if (reason.code == StandardCode.identityMismatch) {
    recipient = handler.deriveParties().issuer;
    if (recipient == null) return null;
  } else {
    recipient = doc.issuer;
  }
  return rejectWithRecipient(
    doc,
    id,
    toErrorPayload(reason),
    recipient,
    clock: clock,
  );
}

/// The `identityMismatch` reason for a [ConsistencyError].
///
/// §8.1 additionally requires the wire `message` to be sanitized: naming the
/// consumer's expected transport-authenticated identity, or echoing the
/// contested in-band value, leaks identity information to a possibly hostile
/// sender. The standard wire form is the code alone with a non-identifying
/// message — so the mismatched values are deliberately *not* included here. Log
/// them locally instead; [ConsistencyError.toString] carries them.
RejectReason identityMismatchReason(ConsistencyError error) =>
    const RejectReason(
      code: StandardCode.identityMismatch,
      message: 'identityMismatch: in-band identity does not match '
          'transport-derived identity',
    );

/// A handler for transports that authenticate nothing — an unauthenticated HTTP
/// POST, a public queue, paper. Party identity comes entirely from the in-band
/// members and whatever `proof` they carry.
class UnauthenticatedTransport implements TransportHandler {
  const UnauthenticatedTransport({
    this.bindingUri = 'urn:trust-tasks:transport:unauthenticated',
  });

  @override
  final String bindingUri;

  @override
  TransportContext deriveParties() => const TransportContext();
}

/// A fixed-identity handler, for tests and for transports resolved out-of-band.
class StaticTransport implements TransportHandler {
  const StaticTransport(
    this.context, {
    this.bindingUri = 'urn:trust-tasks:transport:static',
  });

  final TransportContext context;

  @override
  final String bindingUri;

  @override
  TransportContext deriveParties() => context;
}
