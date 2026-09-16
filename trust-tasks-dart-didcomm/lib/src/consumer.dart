import 'package:ssi/ssi.dart';
import 'package:trust_tasks/trust_tasks.dart';

import 'envelope.dart';

/// What [DidcommConsumer.receive] made of one envelope.
final class Received<R> {
  const Received({
    required this.outcome,
    required this.transport,
    required this.reply,
  });

  /// The §7.2 outcome, exactly as `consumeInbound` defines it.
  final ConsumeOutcome<R> outcome;

  /// Who sent it (authenticated) and who received it.
  final DidcommTransport transport;

  /// The document to send back, as JSON, or null when nothing is to be sent:
  /// a fire-and-forget success, a suppressed `identityMismatch` (§8.1), or a
  /// duplicate with no retained response (§6.1 item 4). Pack it with
  /// [DidcommConsumer.packReply].
  final Map<String, dynamic>? reply;
}

/// The guarded inbound path for the DIDComm binding: unpack, check the thread
/// headers (§3.1), then run SPEC §7.2 with the duplicate-execution record of
/// item 11 and the freshness bound it needs — both **on by default**.
///
/// They matter most on this transport. §6 records that DIDComm guarantees no
/// freshness and that a mediator "can drop, delay, reorder, and re-deliver", so
/// an ordinary queue retry — no attacker involved — executes a consequential
/// task twice unless the consumer keeps the record. The record is keyed on the
/// document `id`, never the DIDComm `id` or `thid`: a redelivery is a fresh
/// DIDComm message carrying the same document (§6.1 item 1).
///
/// **One consumer per process, not one per message.** The guard *is* the
/// record; a consumer built per envelope remembers nothing. Replicas share a
/// store through [replayGuard].
final class DidcommConsumer {
  DidcommConsumer({
    required this.recipient,
    this.allowedSenders,
    ReplayGuard? replayGuard,
    this.replayProtection = true,
    this.freshness = FreshnessPolicy.consequential,
    this.proofVerifier,
    this.payloadValidator,
    DateTime Function() clock = DateTime.now,
  })  : replayGuard = replayGuard ?? InMemoryReplayGuard(),
        _clock = clock;

  /// The identity envelopes are addressed to, and replies are sent from.
  final DidManager recipient;

  /// When set, only these sender DIDs are opened; see [unpackTrustTask].
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
  /// `malformedRequest` rather than ignoring the proof. Authcrypt already
  /// authenticates the sender, so a proof may be omitted over this binding
  /// (§5) — unless the specification requires one.
  final ProofVerifier? proofVerifier;

  /// Validates payloads against their schema (§7.2 item 2).
  final PayloadValidator? payloadValidator;

  final DateTime Function() _clock;

  /// Open [envelope] and run it through §7.2 for the specification [spec].
  ///
  /// [decode] reads the request payload and [encode] writes it back (for the
  /// replay digest). [encodeResponse] writes [handler]'s response payload. The
  /// handler returns the response document — build it with `respondWith` — or
  /// null for a specification that defines no success response, or throws a
  /// `Refusal`.
  ///
  /// Throws [EnvelopeException] for an envelope that never reached the pipeline
  /// (§4): nothing can be sent back for those, except for
  /// [EnvelopeFailure.invalidBody], where [EnvelopeException.sender] is known.
  Future<Received<R>> receive<P, R>(
    Map<String, dynamic> envelope, {
    required SpecPolicy spec,
    required P Function(Map<String, dynamic> json) decode,
    required Object? Function(P payload) encode,
    required Object? Function(R payload) encodeResponse,
    required Handler<P, R> handler,
  }) async {
    final unpacked = await unpackTrustTask(
      envelope,
      recipient: recipient,
      allowedSenders: allowedSenders,
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
      return Received<R>(
        outcome: error == null ? Suppressed<R>(reason) : Rejected<R>(error),
        transport: transport,
        reply: error?.toJson((p) => p.toJson()),
      );
    }

    // unpackTrustTask guarantees string `id` and `type`, so even a body whose
    // other members fail to parse can be answered: §4 makes it
    // `malformedRequest`, and the sender is already authenticated.
    TrustTaskDocument<Object?> envelopeDoc;
    try {
      envelopeDoc = TrustTaskDocument<Object?>.fromJson(json, (p) => p);
    } on Object {
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

    final thread = checkThreadHeaders(unpacked.message, json);
    if (thread != null) return refused(envelopeDoc, thread);

    final TrustTaskDocument<P> doc;
    try {
      doc = TrustTaskDocument<P>.fromJson(
        json,
        (p) => decode(p! as Map<String, dynamic>),
      );
    } on Object {
      return refused(
        envelopeDoc,
        const RejectReason(
          code: StandardCode.malformedRequest,
          message: 'payload does not match the specification',
        ),
      );
    }

    final verifier = proofVerifier;
    final validator = payloadValidator;
    final ConsumeOutcome<R> outcome;
    try {
      outcome = await consumeInbound<P, R>(
        transport: transport,
        spec: spec,
        proofPolicy: verifier == null
            ? const ProofPolicy.rejectIfPresent()
            : ProofPolicy.verify(_AsReceived(verifier, json)),
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
        handler: handler,
      );
    } on Object {
      // The handler threw something that is not a Refusal. Release the claim
      // before the error propagates: holding it would absorb every redelivery
      // of this document in silence until the record expires, and nobody can
      // say whether the effect happened.
      if (replayProtection) {
        try {
          await replayGuard.release(doc.id, documentDigest(doc, encode));
        } on Object {
          /* best-effort */
        }
      }
      rethrow;
    }

    final reply = switch (outcome) {
      Handled(:final response) => response.toJson(encodeResponse),
      Rejected(:final error) => error.toJson((p) => p.toJson()),
      Accepted() || Suppressed() => null,
      DuplicateOutcome(:final priorResponse) => switch (priorResponse) {
          final TrustTaskDocument<R> d => d.toJson(encodeResponse),
          final Map<String, dynamic> m => m,
          _ => null,
        },
    };
    return Received<R>(outcome: outcome, transport: transport, reply: reply);
  }

  /// Pack [reply] into an authcrypt envelope back to the authenticated sender
  /// (§4: "addressed to the verified `sender_kid` of the originating
  /// message").
  ///
  /// [resolver] resolves the sender's DID document; the default handles
  /// `did:key`, `did:peer`, `did:web` and `did:webvh`.
  Future<Map<String, dynamic>> packReply(
    Received<Object?> received, {
    DidResolver? resolver,
  }) async {
    final reply = received.reply;
    final peer = received.transport.peer;
    if (reply == null || peer == null) {
      throw StateError('nothing to reply with, or nobody to reply to');
    }
    final recipientDoc =
        await (resolver ?? UniversalDIDResolver.defaultResolver)
            .resolveDid(peer);
    return packTrustTask(reply, sender: recipient, recipient: recipientDoc);
  }
}

/// Verifies the document as it arrived rather than as the typed decode would
/// re-encode it, which drops members the generated type does not model and so
/// breaks a valid signature.
final class _AsReceived implements ProofVerifier {
  const _AsReceived(this.inner, this.json);

  final ProofVerifier inner;
  final Map<String, dynamic> json;

  @override
  Future<bool> verify(Map<String, dynamic> doc) => inner.verify(json);
}
