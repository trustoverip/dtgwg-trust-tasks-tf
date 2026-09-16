import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:trust_tasks/trust_tasks.dart';

/// The binding's stable identifier (`bindings/tsp/0.1` §1).
const String bindingUri = _bindingUri;

// Named privately as well, because [TspTransport.bindingUri] shadows the public
// constant inside that class.
const String _bindingUri = 'https://trusttasks.org/binding/tsp/0.1';

/// The `type` member of the envelope object a TSP message carries (§1, §2). A
/// TSP VID is a framework VID, so — unlike DIDComm — no identifier is
/// transformed on the way in or out.
const String envelopeType = 'https://trusttasks.org/binding/tsp/0.1/envelope';

/// The [TransportHandler] for one TSP exchange: [peer] is the VID TSP
/// authenticated as the sender, [local] the VID that opened the message (§3).
final class TspTransport implements TransportHandler {
  const TspTransport({this.local, this.peer});

  final String? local;
  final String? peer;

  @override
  String get bindingUri => _bindingUri;

  @override
  TransportContext deriveParties() =>
      TransportContext(issuer: peer, recipient: local);
}

/// Why a TSP message was refused before a document could be produced.
///
/// TSP has no anonymous sender mode, so a message that opens at all carries a
/// verified sender (§2). None of these can be answered with a
/// `trust-task-error`: [notForThisReceiver], [unauthenticated] and
/// [notSealed] leave no opened, authenticated document to reply about, and
/// [wrongEnvelopeType] / [invalidBody] are refused at the binding layer (§4).
enum TspFailure {
  /// Not a TSP message, or one this VID cannot open (bad framing, wrong
  /// receiver, failed signature or HPKE authentication).
  notForThisReceiver,

  /// The sender is not on the allowlist. Refused before the message is opened,
  /// so an unlisted sender's VID is never resolved.
  senderNotAllowed,

  /// The message opened but is not HPKE-sealed (a signed-only message). §2
  /// requires the payload to be sealed as well as signed.
  notSealed,

  /// The message is routed or nested and this VID is an intermediary, not the
  /// final recipient — the document is not opened here (§5.2). Forward it at
  /// the TSP layer.
  notFinalRecipient,

  /// TSP surfaced no authenticated sender VID.
  unauthenticated,

  /// The envelope object's `type` is not [envelopeType] (§2 item 2).
  wrongEnvelopeType,

  /// The payload is not a Trust Task envelope object, or its `document` is not
  /// a Trust Task document.
  invalidBody,
}

/// A TSP message refused by [unpackTrustTask].
final class TspEnvelopeException implements Exception {
  const TspEnvelopeException(this.failure, this.detail, {this.sender});

  final TspFailure failure;

  /// For logs. It can name VIDs; do not send it to the peer.
  final String detail;

  /// The authenticated sender, where authentication got that far.
  final String? sender;

  @override
  String toString() => 'TspEnvelopeException($failure): $detail';
}

/// A Trust Task document taken out of a TSP message.
final class UnpackedTrustTask {
  const UnpackedTrustTask({required this.document, required this.transport});

  /// The Trust Task document, as JSON — exactly as the sender serialised it,
  /// which is what a proof over it was computed on.
  final Map<String, dynamic> document;

  /// The authenticated sender and the opening recipient.
  final TspTransport transport;
}

/// Seal [document] into a Direct TSP message from [sender] to [receiver] (§2,
/// §5.1), returning the wire bytes.
///
/// The message payload is the envelope object with `type` [envelopeType] and
/// `document` set to [document], sealed with HPKE authenticated encryption and
/// signed from [sender]'s VID. [receiver] must carry an encryption key; a
/// signed-only receiver cannot be sealed to.
Future<Uint8List> packTrustTask(
  Map<String, dynamic> document, {
  required PrivateVid sender,
  required PublicVid receiver,
}) async {
  if (receiver.encryptionKey == null) {
    throw ArgumentError.value(
      receiver.id,
      'receiver',
      'has no encryption key; this binding seals every message (§2)',
    );
  }
  final envelope = <String, dynamic>{
    'type': envelopeType,
    'document': document,
  };
  final packed = await Tsp.pack(
    sender: sender,
    receiver: receiver,
    payload: ScsPayload(utf8.encode(jsonEncode(envelope))),
    scheme: TspScheme.hpkeBase,
  );
  return packed.bytes;
}

/// The sender VID a TSP message *names*, read from the envelope without
/// verifying anything. Unauthenticated until [unpackTrustTask] opens the
/// message — use it only to decide whether to open it (for an allowlist, or to
/// resolve the sender's VID first).
String? advertisedSender(Uint8List wire) {
  if (!Tsp.looksLikeTsp(wire)) return null;
  try {
    return Tsp.peek(wire).sender;
  } on Object {
    return null;
  }
}

/// Open a Direct TSP message addressed to [recipient] and take out the Trust
/// Task document. [sender] is the resolved public VID whose signature and HPKE
/// authentication are checked; it must be the VID the message names.
///
/// Throws [TspEnvelopeException].
Future<UnpackedTrustTask> unpackTrustTask(
  Uint8List wire, {
  required PrivateVid recipient,
  required PublicVid sender,
}) async {
  final TspMessage message;
  try {
    message = await Tsp.open(wire, receiver: recipient, sender: sender);
  } on TspReceiverException catch (e) {
    throw TspEnvelopeException(TspFailure.notForThisReceiver, '$e');
  } on TspSenderException catch (e) {
    // The message names a different sender than the one resolved.
    throw TspEnvelopeException(TspFailure.notForThisReceiver, '$e');
  } on TspSignatureException catch (e) {
    throw TspEnvelopeException(TspFailure.notForThisReceiver, '$e');
  } on TspDecryptionException catch (e) {
    throw TspEnvelopeException(TspFailure.notForThisReceiver, '$e');
  } on Object catch (e) {
    throw TspEnvelopeException(TspFailure.notForThisReceiver, '$e');
  }

  // §2: the payload MUST be sealed, not merely signed. A signed-only message
  // authenticates the sender but leaves the document in the clear, which is not
  // the carriage this binding defines.
  if (!message.scheme.confidential) {
    throw TspEnvelopeException(
      TspFailure.notSealed,
      'message is ${message.scheme.wireName}, not HPKE-sealed',
    );
  }

  final payload = message.payload;
  if (payload is! ScsPayload) {
    // A routed or nested envelope whose inner message this VID is not meant to
    // open, or a TSP control message — not a Trust Task the consumer runs.
    throw TspEnvelopeException(
      payload is HopPayload
          ? TspFailure.notFinalRecipient
          : TspFailure.invalidBody,
      'payload is ${payload.typeCode}, not a Trust Task envelope',
      sender: message.sender,
    );
  }

  final Map<String, dynamic> envelope;
  try {
    envelope = jsonDecode(utf8.decode(payload.data)) as Map<String, dynamic>;
  } on Object {
    throw TspEnvelopeException(
      TspFailure.invalidBody,
      'payload is not a JSON envelope object',
      sender: message.sender,
    );
  }
  if (envelope['type'] != envelopeType) {
    throw TspEnvelopeException(
      TspFailure.wrongEnvelopeType,
      'envelope type ${envelope['type']}',
      sender: message.sender,
    );
  }
  final document = envelope['document'];
  if (document is! Map<String, dynamic> ||
      document['id'] is! String ||
      document['type'] is! String) {
    throw TspEnvelopeException(
      TspFailure.invalidBody,
      'envelope carries no Trust Task document',
      sender: message.sender,
    );
  }

  return UnpackedTrustTask(
    document: document,
    // The authenticated sender is surfaced verbatim: a TSP VID is a framework
    // VID (§3). `message.receiver` is null only for the NULL-VID form, which
    // this binding does not send; fall back to the opening VID.
    transport: TspTransport(
      local: message.receiver ?? recipient.id,
      peer: message.sender,
    ),
  );
}

final Random _random = Random.secure();

/// A fresh `urn:uuid:` identifier (UUID version 4), for response and error
/// documents.
String newUrnUuid() {
  final b = List<int>.generate(16, (_) => _random.nextInt(256));
  b[6] = (b[6] & 0x0f) | 0x40;
  b[8] = (b[8] & 0x3f) | 0x80;
  String hex(int from, int to) => [
    for (var i = from; i < to; i++) b[i].toRadixString(16).padLeft(2, '0'),
  ].join();
  return 'urn:uuid:${hex(0, 4)}-${hex(4, 6)}-${hex(6, 8)}-${hex(8, 10)}-'
      '${hex(10, 16)}';
}
