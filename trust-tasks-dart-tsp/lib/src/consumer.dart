import 'dart:typed_data';

import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:trust_tasks/trust_tasks.dart';

import 'envelope.dart';

/// What [TspConsumer.receive] made of one TSP message.
final class Received<R> {
  const Received({
    required this.outcome,
    required this.transport,
    required this.reply,
  });

  /// The §7.2 outcome, exactly as `consumeInbound` defines it.
  final ConsumeOutcome<R> outcome;

  /// Who sent it (authenticated) and who received it.
  final TspTransport transport;

  /// The document to send back, as JSON, or null when nothing is to be sent:
  /// a fire-and-forget success, a suppressed `identityMismatch` (§8.1), or a
  /// duplicate with no retained response (§7.1). Seal it with
  /// [TspConsumer.packReply].
  final Map<String, dynamic>? reply;
}

/// The guarded inbound path for the TSP binding: resolve the sender, open the
/// sealed message, then run SPEC §7.2 with the duplicate-execution record of
/// item 11 and the freshness bound it needs — both **on by default**.
///
/// §7 records that TSP data messages "do not inherently prevent replay", and
/// under §5's routed and nested carriage an intermediary may re-send the sealed
/// inner message. So an ordinary re-forward — no attacker involved — executes a
/// consequential task twice unless the consumer keeps the record. The record is
/// keyed on the document `id`, never the TSP envelope, which is resealed with
/// fresh material on every send (§7.1).
///
/// **One consumer per process, not one per message.** The guard *is* the
/// record; a consumer built per message remembers nothing. Replicas share a
/// store through [replayGuard].
final class TspConsumer {
  TspConsumer({
    required this.recipient,
    VidResolver? resolver,
    this.allowedSenders,
    ReplayGuard? replayGuard,
    this.replayProtection = true,
    this.freshness = FreshnessPolicy.consequential,
    this.proofVerifier,
    this.payloadValidator,
    DateTime Function() clock = DateTime.now,
    this.onEvent,
    this.extendsIndex,
  }) : resolver = resolver ?? SsiVidResolver(),
       replayGuard = replayGuard ?? InMemoryReplayGuard(),
       _clock = clock;

  /// The identity messages are addressed to, and replies are sealed from.
  final PrivateVid recipient;

  /// Resolves a sender VID to the public VID whose signature is checked. The
  /// default resolves `did:key`, `did:peer`, `did:web` and `did:webvh`.
  final VidResolver resolver;

  /// When set, only these sender VIDs are opened; checked against the message's
  /// named sender before it is resolved or opened.
  final Set<String>? allowedSenders;

  /// The §7.2 item 11 record. The in-process default is wrong for a replicated
  /// consumer: each replica would execute the same document once.
  final ReplayGuard replayGuard;

  /// Keep the record. Turn off only where every task handled declares repeated
  /// execution safe.
  final bool replayProtection;

  /// The acceptance window, and so also the record's retention.
  final FreshnessPolicy freshness;

  /// Verifies in-band proofs. Null refuses a proof-bearing document with
  /// `malformedRequest` rather than ignoring the proof. TSP authenticates the
  /// sender end-to-end, so a proof may be omitted over this binding in direct
  /// and nested modes (§5.3) — unless the specification requires one.
  final ProofVerifier? proofVerifier;

  /// Validates payloads against their schema (§7.2 item 2).
  final PayloadValidator? payloadValidator;

  /// Told what happened to each document, for logs and metrics; see
  /// [ServerEvent]. Null reports nothing. Never changes an outcome: it is
  /// called after the decision, not awaited, and whatever it throws is
  /// dropped.
  final ServerEventHook? onEvent;

  /// The `extends` index a later version is read forward along; null uses the
  /// generated one, which covers the public registry. A consumer of
  /// third-party (§6.5) specifications passes its own (see [extensionChain]).
  final Map<String, String>? extendsIndex;

  void _emit(ServerEvent event) => emitServerEvent(onEvent, event);

  final DateTime Function() _clock;

  /// Resolve the sender, open [wire], and run it through §7.2 for the
  /// specification [spec].
  ///
  /// [decode] reads the request payload and [encode] writes it back (for the
  /// replay digest). [encodeResponse] writes [handler]'s response payload. The
  /// handler returns the response document — build it with `respondWith` — or
  /// null for a specification that defines no success response, or throws a
  /// `Refusal`.
  ///
  /// Throws [TspEnvelopeException] for a message that never reached the pipeline
  /// (§4): nothing can be sent back for those.
  Future<Received<R>> receive<P, R>(
    Uint8List wire, {
    required SpecPolicy spec,
    required P Function(Map<String, dynamic> json) decode,
    required Object? Function(P payload) encode,
    required Object? Function(R payload) encodeResponse,
    required Handler<P, R> handler,
  }) async {
    final named = advertisedSender(wire);
    if (named == null) {
      throw const TspEnvelopeException(
        TspFailure.notForThisReceiver,
        'not a TSP message this consumer can read',
      );
    }
    if (allowedSenders != null && !allowedSenders!.contains(named)) {
      throw TspEnvelopeException(
        TspFailure.senderNotAllowed,
        'sender $named is not on this consumer\'s allowlist',
      );
    }
    final PublicVid sender;
    try {
      sender = await resolver.resolve(named);
    } on Object catch (e) {
      throw TspEnvelopeException(
        TspFailure.notForThisReceiver,
        'cannot resolve sender $named: $e',
      );
    }
    final unpacked = await unpackTrustTask(
      wire,
      recipient: recipient,
      sender: sender,
    );
    return consume<P, R>(
      unpacked,
      spec: spec,
      decode: decode,
      encode: encode,
      encodeResponse: encodeResponse,
      handler: handler,
    );
  }

  /// [receive] for a document already unpacked — for a caller that routes on
  /// `unpacked.document['type']` before choosing the specification.
  Future<Received<R>> consume<P, R>(
    UnpackedTrustTask unpacked, {
    required SpecPolicy spec,
    required P Function(Map<String, dynamic> json) decode,
    required Object? Function(P payload) encode,
    required Object? Function(R payload) encodeResponse,
    required Handler<P, R> handler,
  }) async {
    final transport = unpacked.transport;
    final json = unpacked.document;
    final now = _clock();

    Received<R> refused(TrustTaskDocument<Object?> doc, RejectReason reason) {
      final error = reject(transport, doc, newUrnUuid(), reason);
      _report(doc, reason, error, transport);
      return Received<R>(
        outcome: error == null ? Suppressed<R>(reason) : Rejected<R>(error),
        transport: transport,
        reply: error?.toJson((p) => p.toJson()),
      );
    }

    // unpackTrustTask guarantees string `id` and `type`, so a body whose other
    // members fail to parse can still be answered — the sender is authenticated.
    TrustTaskDocument<Object?> envelopeDoc;
    try {
      envelopeDoc = TrustTaskDocument<Object?>.fromJson(json, (p) => p);
    } on Object catch (error, stackTrace) {
      _emit(DocumentMalformed(error: error, stackTrace: stackTrace));
      String? str(String key) =>
          json[key] is String ? json[key] as String : null;
      envelopeDoc = TrustTaskDocument<Object?>(
        id: json['id'] as String,
        type: json['type'] as String,
        issuer: str('issuer'),
        recipient: str('recipient'),
        threadId: str('threadId'),
        parentThreadId: str('parentThreadId'),
        payload: null,
      );
      return refused(
        envelopeDoc,
        const RejectReason(
          code: StandardCode.malformedRequest,
          message: 'body is not a well-formed Trust Task document',
        ),
      );
    }

    // Read as [spec]'s version (SPEC §5.2): unchanged for that version, pruned
    // to what its schema knows for a later one whose declared `extends` chain
    // reaches it, and `unsupportedVersion` / `unsupportedType` otherwise.
    // `json` stays as received; the proof and the item-11 digest are over it.
    final VersionRead<P> read;
    try {
      P payloadFromJson(Object? p) => decode(p! as Map<String, dynamic>);
      final index = extendsIndex;
      read = index == null
          ? readForVersion<P>(json, spec, payloadFromJson)
          : readForVersion<P>(json, spec, payloadFromJson, index: index);
    } on Object catch (error, stackTrace) {
      _emit(
        PayloadDecodeFailed(
          documentId: envelopeDoc.id,
          type: envelopeDoc.type,
          error: error,
          stackTrace: stackTrace,
        ),
      );
      return refused(
        envelopeDoc,
        const RejectReason(
          code: StandardCode.malformedRequest,
          message: 'payload does not match the specification',
        ),
      );
    }
    final TrustTaskDocument<P> doc;
    switch (read) {
      case VersionRefused(:final document, :final reason):
        return refused(document, reason);
      case VersionAccepted(doc: final accepted):
        doc = accepted;
    }

    final verifier = proofVerifier;
    final validator = payloadValidator;
    final outcome = await consumeInbound<P, R>(
      transport: transport,
      spec: spec,
      proofPolicy: verifier == null
          ? const ProofPolicy.rejectIfPresent()
          : ProofPolicy.verify(
              _AsReceived(
                verifier,
                json,
                onRejected: (error, stackTrace) => _emit(
                  ProofRejected(
                    documentId: doc.id,
                    type: doc.type,
                    error: error,
                    stackTrace: stackTrace,
                  ),
                ),
              ),
            ),
      payloadPolicy: validator == null
          ? const PayloadPolicy.acceptUnvalidated()
          : PayloadPolicy.validate(validator),
      checks: ConsumeChecks(
        freshness: freshness,
        replay: replayProtection
            ? ReplayPolicy.guarded(replayGuard)
            : const ReplayPolicy.notConsequential(),
      ),
      doc: doc,
      myVid: transport.local ?? '',
      now: now,
      newErrorId: newUrnUuid,
      payloadToJson: encode,
      responseToJson: encodeResponse,
      // The document as it arrived: the item-11 digest is over it, so a member
      // the payload type does not model still tells two documents apart.
      received: json,
      extendsIndex: extendsIndex,
      handler: (accepted, parties) async {
        try {
          return await handler(accepted, parties);
        } on Refusal {
          rethrow;
        } on Object catch (error, stackTrace) {
          // Still propagates to the caller, as before; the hook is told too,
          // so a consumer that logs through it sees every failure in one place.
          _emit(
            HandlerFailed(
              documentId: accepted.id,
              type: accepted.type,
              error: error,
              stackTrace: stackTrace,
            ),
          );
          rethrow;
        }
      },
    );

    switch (outcome) {
      case Rejected(:final error):
        _emit(
          DocumentRejected(documentId: doc.id, type: doc.type, error: error),
        );
      case Suppressed(:final reason):
        _emit(
          DocumentSuppressed(
            documentId: doc.id,
            type: doc.type,
            reason: reason,
            consistency: resolveParties(transport, doc).error,
          ),
        );
      case Handled() || Accepted() || DuplicateOutcome():
        break;
    }

    final reply = switch (outcome) {
      Handled(:final response) => response.toJson(encodeResponse),
      Rejected(:final error) => error.toJson((p) => p.toJson()),
      Accepted() || Suppressed() => null,
      // Always the recorded JSON — a success or an error response document.
      DuplicateOutcome(:final priorResponse) => priorResponse,
    };
    return Received<R>(outcome: outcome, transport: transport, reply: reply);
  }

  /// Tell [onEvent] about a refusal built before `consumeInbound` ran: the
  /// routed [error], or a suppression when there is none (§8.1).
  void _report(
    TrustTaskDocument<Object?> doc,
    RejectReason reason,
    ErrorResponse? error,
    TransportHandler transport,
  ) {
    if (error != null) {
      _emit(DocumentRejected(documentId: doc.id, type: doc.type, error: error));
    } else {
      _emit(
        DocumentSuppressed(
          documentId: doc.id,
          type: doc.type,
          reason: reason,
          consistency: resolveParties(transport, doc).error,
        ),
      );
    }
  }

  /// Seal [received]'s reply into a TSP message back to the authenticated
  /// sender (§6).
  Future<Uint8List> packReply(Received<Object?> received) async {
    final reply = received.reply;
    final peer = received.transport.peer;
    if (reply == null || peer == null) {
      throw StateError('nothing to reply with, or nobody to reply to');
    }
    final receiver = await resolver.resolve(peer);
    return packTrustTask(reply, sender: recipient, receiver: receiver);
  }
}

/// Verifies the document as it arrived rather than as the typed decode would
/// re-encode it, which drops members the generated type does not model and so
/// breaks a valid signature.
///
/// [onRejected] hears of a proof the verifier refused — what it threw, or
/// nulls when it returned false — for [ProofRejected]. A throw still reaches
/// `consumeInbound`, which answers `proofInvalid` as before.
final class _AsReceived implements ProofVerifier {
  const _AsReceived(this.inner, this.json, {required this.onRejected});

  final ProofVerifier inner;
  final Map<String, dynamic> json;
  final void Function(Object? error, StackTrace? stackTrace) onRejected;

  @override
  Future<bool> verify(Map<String, dynamic> doc) async {
    final bool ok;
    try {
      ok = await inner.verify(json);
    } on Object catch (error, stackTrace) {
      onRejected(error, stackTrace);
      rethrow;
    }
    if (!ok) onRejected(null, null);
    return ok;
  }
}
