import 'dart:convert';
import 'dart:math';

import 'package:didcomm/didcomm.dart';
import 'package:ssi/ssi.dart';
import 'package:trust_tasks/trust_tasks.dart';

/// The binding's stable identifier (`bindings/didcomm/0.2` §1).
const String bindingUri = _bindingUri;

// Named privately as well, because [DidcommTransport.bindingUri] shadows the
// public constant inside that class.
const String _bindingUri = 'https://trusttasks.org/binding/didcomm/0.2';

/// The DIDComm message `type` that carries a Trust Task document (§1). It did
/// not change with the binding's `0.1` → `0.2` minor; only the binding
/// identifier did.
const String envelopeType =
    'https://trusttasks.org/binding/didcomm/0.1/envelope';

/// The wrappings that authenticate a sender: authcrypt, optionally signed
/// inside, optionally anoncrypted outside for a mediator. Anoncrypt alone,
/// signed-only and plaintext are refused (§2, §4).
const List<MessageWrappingType> authenticatedWrappings = [
  MessageWrappingType.authcryptPlaintext,
  MessageWrappingType.authcryptSignPlaintext,
  MessageWrappingType.anoncryptAuthcryptPlaintext,
];

/// The [TransportHandler] for one DIDComm exchange: [peer] is the DID the
/// authcrypt authenticated, [local] the DID that unpacked it (§3).
final class DidcommTransport implements TransportHandler {
  const DidcommTransport({this.local, this.peer});

  final String? local;
  final String? peer;

  @override
  String get bindingUri => _bindingUri;

  @override
  TransportContext deriveParties() =>
      TransportContext(issuer: peer, recipient: local);
}

/// Why an envelope was refused before a document could be produced.
///
/// Except for [invalidBody], none of these reach the framework pipeline, and
/// none can be answered with a `trust-task-error`: there is no authenticated
/// sender to address one to (§4, `errored`).
enum EnvelopeFailure {
  /// Not a DIDComm message this agent can open: not JSON, not a JWE, or
  /// decryption failed.
  undecryptable,

  /// Anoncrypt, signed-only or plaintext: no authenticated sender (§4).
  unauthenticatedSender,

  /// The authcrypt `skid` carries no `#fragment`, so it names no key.
  unqualifiedSenderKid,

  /// The sender is not on the allowlist. Refused before decryption, so an
  /// unlisted sender's DID is never resolved.
  senderNotAllowed,

  /// The DIDComm `type` is not [envelopeType] (§2 item 2).
  wrongEnvelopeType,

  /// The body is not a Trust Task document. The sender *is* authenticated here,
  /// so [EnvelopeException.sender] is set and a caller may answer with
  /// `malformedRequest` (§4).
  invalidBody,
}

/// An envelope refused by [unpackTrustTask].
final class EnvelopeException implements Exception {
  const EnvelopeException(this.failure, this.detail, {this.sender});

  final EnvelopeFailure failure;

  /// For logs. It can name DIDs; do not send it to the peer.
  final String detail;

  /// The authenticated sender, where authentication got that far.
  final String? sender;

  @override
  String toString() => 'EnvelopeException($failure): $detail';
}

/// A Trust Task document taken out of an authcrypt envelope.
final class UnpackedTrustTask {
  const UnpackedTrustTask({
    required this.document,
    required this.transport,
    required this.message,
  });

  /// The Trust Task document, as JSON — exactly as the sender serialised it,
  /// which is what a proof over it was computed on.
  final Map<String, dynamic> document;

  /// The authenticated sender and the unpacking recipient.
  final DidcommTransport transport;

  /// The DIDComm message that carried it, for its headers.
  final PlainTextMessage message;
}

/// Pack [document] into an authcrypt envelope from [sender] to [recipient]
/// (§2), returning the JWE as JSON.
///
/// `thid` is set from the document's `threadId`, falling back to its `id`, and
/// `pthid` from `parentThreadId` — the headers follow the members, never the
/// reverse (§3.1). The sender's key-agreement key is the first one whose curve
/// the recipient also has.
Future<Map<String, dynamic>> packTrustTask(
  Map<String, dynamic> document, {
  required DidManager sender,
  required DidDocument recipient,
}) async {
  final senderDoc = await sender.getDidDocument();
  final keyIds =
      senderDoc.matchKeysInKeyAgreement(otherDidDocuments: [recipient]);
  if (keyIds.isEmpty) {
    throw ArgumentError.value(
      recipient.id,
      'recipient',
      'shares no key-agreement curve with ${senderDoc.id}',
    );
  }
  final id = document['id'];
  final threadId = document['threadId'] ?? id;
  final parentThreadId = document['parentThreadId'];

  final message = PlainTextMessage(
    id: _uuid(),
    type: Uri.parse(envelopeType),
    from: senderDoc.id,
    to: [recipient.id],
    threadId: threadId is String ? threadId : null,
    parentThreadId: parentThreadId is String ? parentThreadId : null,
    createdTime: DateTime.now().toUtc(),
    body: document,
  );

  final encrypted = await EncryptedMessage.packWithAuthentication(
    message,
    keyPair: await sender.getKeyPairByDidKeyId(keyIds.first),
    didKeyId: keyIds.first,
    recipientDidDocuments: [recipient],
  );
  return jsonDecode(jsonEncode(encrypted)) as Map<String, dynamic>;
}

/// The sender DID an authcrypt envelope *claims*, read from its protected
/// header without decrypting anything. Unauthenticated until the envelope is
/// opened — use it only to decide whether to open it.
String? advertisedSender(Map<String, dynamic> envelope) {
  try {
    final header = jsonDecode(
      utf8.decode(
        base64Url.decode(base64Url.normalize(envelope['protected'] as String)),
      ),
    ) as Map<String, dynamic>;
    final skid = header['skid'];
    if (skid is! String || !skid.contains('#')) return null;
    return skid.substring(0, skid.indexOf('#'));
  } on Object {
    return null;
  }
}

/// Open an authcrypt envelope addressed to [recipient] and take out the Trust
/// Task document (§2 consumer items 1–3, and §3.1's thread check is left to
/// the pipeline, which can answer it).
///
/// With [allowedSenders], an envelope whose `skid` names anyone else is refused
/// **before** decryption: opening it means resolving the sender's DID, which
/// for `did:web` is a fetch from a host the sender picked.
///
/// Throws [EnvelopeException].
Future<UnpackedTrustTask> unpackTrustTask(
  Map<String, dynamic> envelope, {
  required DidManager recipient,
  Set<String>? allowedSenders,
}) async {
  if (allowedSenders != null) {
    final claimed = advertisedSender(envelope);
    if (claimed == null) {
      throw const EnvelopeException(
        EnvelopeFailure.unauthenticatedSender,
        'no authcrypt skid to check against the allowlist',
      );
    }
    if (!allowedSenders.contains(claimed)) {
      throw EnvelopeException(
        EnvelopeFailure.senderNotAllowed,
        'sender $claimed is not on this consumer\'s allowlist',
      );
    }
  }

  List<DidcommMessage> layers = const [];
  final PlainTextMessage message;
  try {
    // Every wrapping is let through here and judged below from the layers
    // actually found, rather than by reading didcomm's error text.
    message = await DidcommMessage.unpackToPlainTextMessage(
      message: envelope,
      recipientDidManager: recipient,
      expectedMessageWrappingTypes: MessageWrappingType.values,
      onUnpacked: ({required foundMessages, required foundSigners}) {
        layers = foundMessages;
      },
    );
  } on Object catch (e) {
    // Not a DIDComm message, not for this recipient, a failed key unwrap or
    // AEAD tag, or a `from` / `to` that disagrees with the JWE's own headers.
    throw EnvelopeException(EnvelopeFailure.undecryptable, '$e');
  }

  final wrapping = MessageWrappingType.findFromMessages(layers);
  if (wrapping == null || !authenticatedWrappings.contains(wrapping)) {
    throw EnvelopeException(
      EnvelopeFailure.unauthenticatedSender,
      'wrapping ${wrapping?.name ?? 'unrecognised'} authenticates no sender',
    );
  }

  // The sender is the skid of the ECDH-1PU layer: the key whose public half
  // opened the key wrap. didcomm has already checked the message's `from`
  // against it.
  String? skid;
  for (final layer in layers) {
    if (layer is! EncryptedMessage) continue;
    final header = _protectedHeader(layer.protected);
    if (header?['alg'] is String &&
        (header!['alg'] as String).startsWith('ECDH-1PU')) {
      skid = header['skid'] as String?;
    }
  }
  if (skid == null) {
    throw const EnvelopeException(
      EnvelopeFailure.unauthenticatedSender,
      'no ECDH-1PU layer',
    );
  }
  final hash = skid.indexOf('#');
  if (hash <= 0) {
    // Never downgrade to "no sender": the pipeline would then trust the
    // in-band issuer with the transport cross-check skipped.
    throw EnvelopeException(
      EnvelopeFailure.unqualifiedSenderKid,
      'sender kid $skid carries no verification-method fragment',
    );
  }
  final sender = skid.substring(0, hash);
  if (allowedSenders != null && !allowedSenders.contains(sender)) {
    throw EnvelopeException(
      EnvelopeFailure.senderNotAllowed,
      'authenticated sender $sender is not on this consumer\'s allowlist',
    );
  }

  if (message.type.toString() != envelopeType) {
    throw EnvelopeException(
      EnvelopeFailure.wrongEnvelopeType,
      'DIDComm type ${message.type}',
      sender: sender,
    );
  }
  final body = message.body;
  if (body == null || body['id'] is! String || body['type'] is! String) {
    throw EnvelopeException(
      EnvelopeFailure.invalidBody,
      'body is not a Trust Task document',
      sender: sender,
    );
  }

  final local = (await recipient.getDidDocument()).id;
  return UnpackedTrustTask(
    document: body,
    transport: DidcommTransport(local: local, peer: sender),
    message: message,
  );
}

/// §3.1: where a DIDComm thread header and its framework member are *both*
/// present they must agree. Returns the refusal, or null.
///
/// Scoped to both-present because DIDComm's `thid` defaults to the message `id`
/// and the framework's `threadId` to the document `id` — different identifier
/// spaces, so an unconditional comparison would refuse conforming exchanges.
RejectReason? checkThreadHeaders(
  PlainTextMessage message,
  Map<String, dynamic> document,
) {
  for (final (header, member, transport) in [
    ('thid', 'threadId', message.threadId),
    ('pthid', 'parentThreadId', message.parentThreadId),
  ]) {
    final inBand = document[member];
    if (transport != null && inBand is String && transport != inBand) {
      return RejectReason(
        code: StandardCode.malformedRequest,
        message: 'DIDComm $header disagrees with the document\'s $member '
            '(bindings/didcomm/0.2 §3.1)',
      );
    }
  }
  return null;
}

Map<String, dynamic>? _protectedHeader(String protected) {
  try {
    return jsonDecode(
            utf8.decode(base64Url.decode(base64Url.normalize(protected))))
        as Map<String, dynamic>;
  } on Object {
    return null;
  }
}

final Random _random = Random.secure();

String _uuid() {
  final b = List<int>.generate(16, (_) => _random.nextInt(256));
  b[6] = (b[6] & 0x0f) | 0x40;
  b[8] = (b[8] & 0x3f) | 0x80;
  String hex(int from, int to) => [
        for (var i = from; i < to; i++) b[i].toRadixString(16).padLeft(2, '0'),
      ].join();
  return '${hex(0, 4)}-${hex(4, 6)}-${hex(6, 8)}-${hex(8, 10)}-${hex(10, 16)}';
}

/// A fresh `urn:uuid:` identifier, for response and error documents.
String newUrnUuid() => 'urn:uuid:${_uuid()}';
