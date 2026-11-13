/// What a binding server or consumer did with a document, for the operator.
///
/// Hand-written. The HTTPS server (`package:trust_tasks_https`) and the DIDComm
/// and TSP consumers take an optional [ServerEventHook] and report to it what
/// the wire deliberately does not say: the exception a handler threw (answered
/// `internalError`), the identity disagreement behind a suppressed refusal
/// (§8.1), why a proof failed, an authenticator that threw. SPEC §12.4 keeps
/// all of that off the wire; without a hook it was nowhere at all.
///
/// The events are for **logs**. They carry exactly what §12.4 says never to
/// send a peer — an exception's text, a DID a resolver was asked about — so do
/// not echo them into a response.
///
/// ```dart
/// final server = HttpsServer(
///   localVid: 'did:web:maintainer.example',
///   onEvent: (event) => switch (event) {
///     HandlerFailed(:final error, :final stackTrace) =>
///       log.severe('handler threw', error, stackTrace),
///     DocumentSuppressed(:final consistency) =>
///       log.warning('suppressed identityMismatch: $consistency'),
///     _ => log.fine(event),
///   },
/// );
/// ```
///
/// A hook never changes what is answered: it runs after the decision, its
/// return value is ignored, and anything it throws — or a `Future` it returns
/// completing with an error — is caught and dropped ([emitServerEvent]). It is
/// called synchronously and not awaited, so a hook that does I/O should hand
/// the event off rather than block on it.
library;

import 'dart:async';

import 'document.dart';
import 'transport.dart';

/// Receives [ServerEvent]s. May be asynchronous; it is not awaited.
typedef ServerEventHook = FutureOr<void> Function(ServerEvent event);

/// Call [hook] with [event], if there is a hook, swallowing whatever it throws.
///
/// The one place a binding calls a hook, so none of them can let an observer's
/// failure into the pipeline: a logging bug must never turn into a different
/// answer on the wire, or into an exception the HTTP stack sees.
void emitServerEvent(ServerEventHook? hook, ServerEvent event) {
  if (hook == null) return;
  try {
    final result = hook(event);
    if (result is Future<Object?>) {
      // Not awaited: an observer must not hold up the answer. Its error is
      // handled here so it never reaches the zone as an uncaught one.
      unawaited(result.then<void>((_) {}, onError: (Object _) {}));
    }
  } on Object {
    // Deliberately ignored; see above.
  }
}

/// Something a binding server or consumer reports to its [ServerEventHook].
///
/// Sealed, so a `switch` over it is exhaustive. Produced by this library family
/// and never parsed from a peer, so closing it costs no forward compatibility;
/// a new event in a later release is a breaking change, as it is for
/// [ConsumeOutcome].
sealed class ServerEvent {
  const ServerEvent();
}

/// The bytes are not a Trust Task document at all — not JSON, not an object,
/// or no string `id` / `type` (§7.2 item 1). Answered as a malformed body.
final class DocumentMalformed extends ServerEvent {
  const DocumentMalformed({required this.error, required this.stackTrace});

  /// What the parser threw.
  final Object error;
  final StackTrace stackTrace;

  @override
  String toString() => 'DocumentMalformed($error)';
}

/// The HTTPS bearer authenticator threw. The request carried on as
/// unauthenticated, which is what an authenticator that fails has done.
final class AuthenticatorFailed extends ServerEvent {
  const AuthenticatorFailed({required this.error, required this.stackTrace});

  final Object error;
  final StackTrace stackTrace;

  @override
  String toString() => 'AuthenticatorFailed($error)';
}

/// The envelope parsed, but the payload did not decode as the specification's
/// type. Answered `malformedRequest`, without the decoder's words (§12.4).
final class PayloadDecodeFailed extends ServerEvent {
  const PayloadDecodeFailed({
    required this.documentId,
    required this.type,
    required this.error,
    required this.stackTrace,
  });

  final String documentId;
  final String type;

  /// What the decoder threw — it names members and types, which is why the
  /// wire message does not carry it.
  final Object error;
  final StackTrace stackTrace;

  @override
  String toString() => 'PayloadDecodeFailed($type $documentId: $error)';
}

/// The proof verifier did not accept a document's proof. Answered
/// `proofInvalid` with a constant message (§10.4, §12.4).
final class ProofRejected extends ServerEvent {
  const ProofRejected({
    required this.documentId,
    required this.type,
    this.error,
    this.stackTrace,
  });

  final String documentId;
  final String type;

  /// What the verifier threw, or null when it returned false. A verifier that
  /// wants its reason in the log throws it — for example a wrapper over
  /// `trust_tasks_proof`'s `verifyDetailed` that throws the
  /// `ProofVerification` it got back.
  final Object? error;
  final StackTrace? stackTrace;

  @override
  String toString() =>
      'ProofRejected($type $documentId${error == null ? '' : ': $error'})';
}

/// A handler threw something other than a [Refusal]. Answered
/// `internalError`; the exception is here and nowhere else.
final class HandlerFailed extends ServerEvent {
  const HandlerFailed({
    required this.documentId,
    required this.type,
    required this.error,
    required this.stackTrace,
  });

  final String documentId;
  final String type;
  final Object error;
  final StackTrace stackTrace;

  @override
  String toString() => 'HandlerFailed($type $documentId: $error)';
}

/// A refusal that §8.1 forbids answering — an `identityMismatch` with no
/// authenticated sender — so nothing (HTTPS: an answer indistinguishable from
/// a malformed body) was sent. `consumeInbound`'s [Suppressed] says callers
/// SHOULD log it; this is that log line.
final class DocumentSuppressed extends ServerEvent {
  const DocumentSuppressed({
    required this.documentId,
    required this.type,
    required this.reason,
    this.consistency,
  });

  final String documentId;
  final String type;
  final RejectReason reason;

  /// Which party disagreed and the two values, where the binding knows them.
  final ConsistencyError? consistency;

  @override
  String toString() => 'DocumentSuppressed($type $documentId: '
      '${consistency ?? reason.message})';
}

/// A document was refused with a routed error response — any framework check,
/// or a handler's [Refusal]. [error] is exactly what was sent.
final class DocumentRejected extends ServerEvent {
  const DocumentRejected({
    required this.documentId,
    required this.type,
    required this.error,
  });

  final String documentId;
  final String type;
  final ErrorResponse error;

  @override
  String toString() =>
      'DocumentRejected($type $documentId: ${error.payload.code} '
      '${error.payload.message})';
}

/// Something failed outside every step above — an encoder that threw on the
/// response, an adapter's I/O — and was answered `internalError` (or a bare
/// `500`) rather than escaping.
final class InternalFailure extends ServerEvent {
  const InternalFailure({required this.error, required this.stackTrace});

  final Object error;
  final StackTrace stackTrace;

  @override
  String toString() => 'InternalFailure($error)';
}
