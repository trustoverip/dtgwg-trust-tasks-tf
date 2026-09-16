import 'dart:math';

import 'package:trust_tasks/trust_tasks.dart';

import 'constants.dart';

/// A capability document: a Trust Task whose payload is an open JSON object.
typedef CapabilityDocument = TrustTaskDocument<Map<String, dynamic>>;

final _rand = Random.secure();

/// A fresh document `id` (a UUID v4 URN). One per *attempt* — never reused
/// across attempts; see [newAttempt].
String _freshId() {
  final b = List<int>.generate(16, (_) => _rand.nextInt(256));
  b[6] = (b[6] & 0x0f) | 0x40;
  b[8] = (b[8] & 0x3f) | 0x80;
  String hex(int i) => b[i].toRadixString(16).padLeft(2, '0');
  final s = StringBuffer('urn:uuid:');
  for (var i = 0; i < 16; i++) {
    if (i == 4 || i == 6 || i == 8 || i == 10) s.write('-');
    s.write(hex(i));
  }
  return s.toString();
}

String _now() => DateTime.now().toUtc().toIso8601String();

/// Build a capability Trust Task addressed [issuerDid] → [recipientDid].
///
/// Mints a fresh `id` and stamps `issuedAt` on every call, so each built
/// document is a new attempt in the sense of SPEC §8.4. To re-send one you have
/// already built, see [newAttempt] — the choice between resending the identical
/// document (a §8.4 retry the consumer absorbs) and minting a new one is
/// enforced by the consumer's §7.2 item-11 record.
CapabilityDocument buildDocument(
  String issuerDid,
  String recipientDid,
  String typeUri,
  Map<String, dynamic> payload,
) =>
    TrustTaskDocument<Map<String, dynamic>>(
      id: _freshId(),
      type: typeUri,
      issuer: issuerDid,
      recipient: recipientDid,
      issuedAt: _now(),
      payload: payload,
    );

/// A **new attempt** at the request [previous] carried: the same addressing,
/// type and payload under a fresh `id`, a fresh `issuedAt`, and no `proof`.
///
/// This is the counterpart of a SPEC §8.4 retry, and the two are not
/// interchangeable. A **retry** is a bit-for-bit resend of [previous] (same
/// `id`) that the consumer's §7.2 item-11 record absorbs. A **new attempt** is a
/// different document — anything that changes the bytes makes it one, including a
/// re-stamped `issuedAt` or a re-signed `proof` over identical content — and MUST
/// carry a fresh `id`, or the consumer rejects it with `idConflict`.
///
/// `proof` is cleared because it committed to the previous `id` and `issuedAt`;
/// sign the returned document before sending it. Where [previous] opened its own
/// exchange (no `threadId`), the new attempt opens a *new* one — wait on
/// [correlationThread] of the returned document, not of [previous].
CapabilityDocument newAttempt(CapabilityDocument previous) =>
    TrustTaskDocument<Map<String, dynamic>>(
      id: _freshId(),
      type: previous.type,
      payload: previous.payload,
      threadId: previous.threadId,
      parentThreadId: previous.parentThreadId,
      ceremony: previous.ceremony,
      issuer: previous.issuer,
      recipient: previous.recipient,
      issuedAt: _now(),
      expiresAt: previous.expiresAt,
      context: previous.context,
      extra: previous.extra,
    );

/// Build a `governance/capability/list` request (status `all`).
CapabilityDocument buildListDocument(String issuerDid, String vtcDid) =>
    buildDocument(issuerDid, vtcDid, capabilityListType, {'status': 'all'});

/// Build a `governance/capability/enable` or `/disable` request. On enable,
/// `config.authority` defaults to the community's own DID.
CapabilityDocument buildToggleDocument(
  String issuerDid,
  String vtcDid,
  String slug,
  String version, {
  required bool enable,
}) =>
    enable
        ? buildDocument(issuerDid, vtcDid, capabilityEnableType, {
            'capability': slug,
            'version': version,
            'config': {'authority': vtcDid},
          })
        : buildDocument(
            issuerDid, vtcDid, capabilityDisableType, {'capability': slug});

/// Build a `git-trust/grant`: grant [subjectDid] commit-signing trust for
/// [resource] (an org or `org/repo` slug).
CapabilityDocument buildGitTrustGrant(
  String authorityDid,
  String registryDid,
  String subjectDid,
  String resource,
) =>
    buildDocument(authorityDid, registryDid, gitTrustGrantType, {
      'subject': subjectDid,
      'resource': resource,
    });

/// Build a `git-trust/revoke`. A null [reason] is omitted.
CapabilityDocument buildGitTrustRevoke(
  String authorityDid,
  String registryDid,
  String subjectDid,
  String resource, {
  String? reason,
}) =>
    buildDocument(authorityDid, registryDid, gitTrustRevokeType, {
      'subject': subjectDid,
      'resource': resource,
      if (reason != null) 'reason': reason,
    });

// --- correlation + envelope parsing -----------------------------------------

/// The slug of a Type URI, or `null` when it is not a Trust Task Type URI.
String? slugOf(String typeUri) {
  try {
    return slugFromTypeUri(typeUri);
  } on ArgumentError {
    return null;
  }
}

/// Whether a Type URI names the success-response variant.
bool isResponseVariant(String typeUri) => typeUri.endsWith('#response');

/// The thread an exchange started by [doc] is correlated by: its own `threadId`,
/// or its `id` where it opens the exchange (SPEC §4.9's fallback). Hold this from
/// the moment you send a request; every reply-classifying function wants it as
/// `expectedThreadId`.
String correlationThread(CapabilityDocument doc) => doc.threadId ?? doc.id;

/// Whether [reply] is threaded to [expectedThreadId] — SPEC §4.9 correlation,
/// and the precondition for acting on any reply. A reply with no `threadId`
/// matches nothing.
bool repliesTo(CapabilityDocument reply, String expectedThreadId) =>
    reply.threadId != null && reply.threadId == expectedThreadId;

/// Coerce a JSON body into a [CapabilityDocument], or `null` when it is not one.
CapabilityDocument? _asDocument(Object? body) {
  if (body is! Map) return null;
  try {
    return TrustTaskDocument<Map<String, dynamic>>.fromJson(
      body.cast<String, dynamic>(),
      (p) => (p as Map).cast<String, dynamic>(),
    );
  } catch (_) {
    return null;
  }
}

/// Parse a DIDComm envelope body into `(threadId, document)`, or `null` when the
/// body is not a threaded Trust Task document.
///
/// The `threadId` is a **dispatch key, not a check**: it says which outstanding
/// request this document belongs to, not that it is a reply to anything you sent.
/// Correlate before acting — with [parseEnvelopeDocumentFor] or your own
/// outstanding map.
(String, CapabilityDocument)? parseEnvelopeDocument(Object? body) {
  final doc = _asDocument(body);
  final thid = doc?.threadId;
  if (doc == null || thid == null) return null;
  return (thid, doc);
}

/// Parse a DIDComm envelope body into the document it carries, only if that
/// document is threaded to [expectedThreadId]. `null` covers both "not a Trust
/// Task document" and "belongs to another exchange".
CapabilityDocument? parseEnvelopeDocumentFor(
  Object? body,
  String expectedThreadId,
) {
  final parsed = parseEnvelopeDocument(body);
  if (parsed == null) return null;
  return repliesTo(parsed.$2, expectedThreadId) ? parsed.$2 : null;
}
