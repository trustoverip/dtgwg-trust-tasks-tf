import 'constants.dart';
import 'document.dart';

// --- git-trust write replies (grant/revoke producers) -----------------------

/// The classification of a `git-trust` write reply.
///
/// [WriteIdempotentSuccess] is load-bearing for redelivery-safe writers: an
/// `already_granted` / `not_granted` rejection means the desired end state
/// already holds, so the write is done, not failed.
sealed class WriteOutcome {
  const WriteOutcome();
}

/// The `#response` document acknowledged the write.
final class WriteSuccess extends WriteOutcome {
  const WriteSuccess();
}

/// Rejected because the end state already holds.
final class WriteIdempotentSuccess extends WriteOutcome {
  const WriteIdempotentSuccess();
}

/// Any other rejection: the machine-readable code and human detail.
final class WriteRejected extends WriteOutcome {
  const WriteRejected(this.code, this.message);
  final String code;
  final String? message;
}

/// How much a caller is willing to infer from a non-conforming peer. The default
/// infers nothing: an outcome is decided from the error `code` alone.
class ReplyPolicy {
  const ReplyPolicy({this.acceptLegacyFreeTextIdempotence = false});

  /// **DEPRECATED — opt-in compatibility only.** Also treat a `taskFailed` whose
  /// free-text `message` contains `already_granted:` or `not_granted:` as
  /// [WriteIdempotentSuccess].
  ///
  /// SPEC §8.2 makes `message` non-normative free text, so deciding an outcome
  /// from it hinges on wording the emitting service may reword, translate or
  /// drop. Enable it only while a specific peer still emits the free-text form;
  /// the correct fix is on the emitting side (send the extended codes SPEC §8.5
  /// provides).
  final bool acceptLegacyFreeTextIdempotence;
}

({String code, String? message}) _errorCodeAndMessage(CapabilityDocument doc) {
  final payload = doc.payload;
  final code = payload['code'];
  final message = payload['message'];
  return (
    code: code is String ? code : 'unknown',
    message: message is String ? message : null,
  );
}

bool _isIdempotentCode(String code) =>
    code == gitTrustAlreadyGrantedCode ||
    code == gitTrustNotGrantedCode ||
    code == gitTrustAlreadyGrantedCodeCamel ||
    code == gitTrustNotGrantedCodeCamel;

/// Classify the reply to a `git-trust/grant` or `git-trust/revoke` write.
///
/// [expectedThreadId] is [correlationThread] of the document you sent. A reply
/// threaded to anything else yields `null` — "not an answer to this request" —
/// rather than an outcome: acting on an uncorrelated reply lets whichever
/// document arrives next decide the fate of a write it has nothing to do with.
///
/// Idempotent success is keyed on the SPEC §8.5 extended error code, never on the
/// free-text `message`; [policy] enables the deprecated compatibility path.
WriteOutcome? classifyGitTrustReply(
  CapabilityDocument doc,
  String expectedThreadId, {
  ReplyPolicy policy = const ReplyPolicy(),
}) {
  // SPEC §4.9: correlation comes first.
  if (!repliesTo(doc, expectedThreadId)) return null;

  final slug = slugOf(doc.type);
  if (slug == 'trust-task-error') {
    final (:code, :message) = _errorCodeAndMessage(doc);
    if (_isIdempotentCode(code)) return const WriteIdempotentSuccess();
    // DEPRECATED: pre-extended-code peers signalled idempotence in the free-text
    // `message` under a bare `taskFailed`. SPEC §8.2 makes `message`
    // non-normative, so this is a string match on a field nobody promised to
    // keep stable — opt-in only.
    if (policy.acceptLegacyFreeTextIdempotence && code == 'taskFailed') {
      final reason = message ?? '';
      if (reason.contains('already_granted:') ||
          reason.contains('not_granted:')) {
        return const WriteIdempotentSuccess();
      }
    }
    return WriteRejected(code, message);
  }
  if (isResponseVariant(doc.type) &&
      (slug == 'git-trust/grant' || slug == 'git-trust/revoke')) {
    return const WriteSuccess();
  }
  return null;
}

// --- governance/capability replies (management UIs) -------------------------

/// One capability entry as rendered by a management UI.
class CapabilitySummary {
  const CapabilitySummary({
    required this.slug,
    required this.version,
    required this.enabled,
    required this.manifest,
    this.title,
    this.enabledAt,
    this.delegate,
  });

  final String slug;
  final String? title;
  final String version;
  final bool enabled;
  final String? enabledAt;
  final String? delegate;

  /// The full manifest, for a detail view.
  final Map<String, dynamic> manifest;
}

/// The classification of a `governance/capability/*` reply.
sealed class CapabilityReply {
  const CapabilityReply();
}

/// A `list` response: the community's capabilities.
final class CapabilityListing extends CapabilityReply {
  const CapabilityListing(this.entries);
  final List<CapabilitySummary> entries;
}

/// An `enable`/`disable` acknowledgement.
final class CapabilityToggled extends CapabilityReply {
  const CapabilityToggled({required this.capability, required this.enabled});
  final String capability;
  final bool enabled;
}

/// A `trust-task-error` document.
final class CapabilityRejected extends CapabilityReply {
  const CapabilityRejected(this.code, this.message);
  final String code;
  final String? message;
}

CapabilitySummary? _summaryOf(Object? entry) {
  if (entry is! Map) return null;
  final manifest = entry['manifest'];
  if (manifest is! Map) return null;
  final capability = manifest['capability'];
  if (capability is! String) return null;
  final version = manifest['version'];
  final title = manifest['title'];
  final enabledAt = entry['enabledAt'];
  final delegate = entry['delegate'];
  return CapabilitySummary(
    slug: capability,
    title: title is String ? title : null,
    version: version is String ? version : '?',
    enabled: entry['enabled'] == true,
    enabledAt: enabledAt is String ? enabledAt : null,
    delegate: delegate is String ? delegate : null,
    manifest: manifest.cast<String, dynamic>(),
  );
}

/// Classify a `governance/capability/*` reply document. `null` when it is not
/// part of this family, or is not threaded to [expectedThreadId].
CapabilityReply? parseCapabilityReply(
  CapabilityDocument doc,
  String expectedThreadId,
) {
  if (!repliesTo(doc, expectedThreadId)) return null;

  final slug = slugOf(doc.type);
  if (slug == 'trust-task-error') {
    final (:code, :message) = _errorCodeAndMessage(doc);
    return CapabilityRejected(code, message);
  }
  if (!isResponseVariant(doc.type)) return null;
  switch (slug) {
    case 'governance/capability/list':
      final raw = doc.payload['capabilities'];
      final entries = <CapabilitySummary>[];
      if (raw is List) {
        for (final e in raw) {
          final s = _summaryOf(e);
          if (s != null) entries.add(s);
        }
      }
      return CapabilityListing(entries);
    case 'governance/capability/enable':
    case 'governance/capability/disable':
      final capability = doc.payload['capability'];
      return CapabilityToggled(
        capability: capability is String ? capability : '',
        enabled: doc.payload['enabled'] == true,
      );
    default:
      return null;
  }
}

/// Parse an inbound envelope body directly into a reply to the request threaded
/// [expectedThreadId] — the entry point for a UI's inbound dispatch, which holds
/// only the body value. `null` when the body is not a `governance/capability/*`
/// reply, or belongs to a different exchange.
CapabilityReply? parseEnvelopeReply(Object? body, String expectedThreadId) {
  final doc = parseEnvelopeDocumentFor(body, expectedThreadId);
  if (doc == null) return null;
  return parseCapabilityReply(doc, expectedThreadId);
}
