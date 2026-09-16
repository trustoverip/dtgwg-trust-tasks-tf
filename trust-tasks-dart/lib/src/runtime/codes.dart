/// Error codes — SPEC.md §8.3 (standard) and §8.5 (extended).
///
/// Hand-written. Mirrors `error.rs` in trust-tasks-rs, `_runtime/codes.ts` in
/// @openvtc/trust-tasks and `trusttasks/codes.go` in trust-tasks-go, including
/// their acceptance of both the framework 0.2 lowerCamelCase spellings and the
/// frozen 0.1 snake_case ones, so a 0.2 consumer can still read an error
/// response from a 0.1 peer.
library;

/// A framework-defined standard error code (SPEC.md §8.3).
///
/// ## Why an extension type and not an enum
///
/// A Dart `enum` would give exhaustive switches and would **throw** on a value
/// it does not know. Error codes arrive from peers, and SPEC §5.2 requires a
/// consumer to tolerate a newer MINOR — so a peer emitting a code added after
/// this release would crash the parse rather than fall through to the §8.5
/// handling the spec prescribes. That is the wrong failure for a wire value.
///
/// An extension type over String is zero-cost, erases to String, keeps `==` and
/// the named constants below, and represents an unrecognised code without
/// complaint. This puts Dart alongside Rust's `#[non_exhaustive]` enum and Go's
/// named string type. TypeScript's closed union is the odd one out, and adding a
/// standard code is a breaking change only there.
///
/// Narrow a wire value with [isStandardCode] and keep a fallback branch:
///
/// ```dart
/// final code = payload.code;
/// if (isStandardCode(code)) {
///   // a §8.3 code, in canonical spelling
/// } else {
///   // a §8.5 extended code
/// }
/// ```
extension type const StandardCode(String value) {
  static const StandardCode malformedRequest = StandardCode('malformedRequest');
  static const StandardCode unsupportedType = StandardCode('unsupportedType');
  static const StandardCode unsupportedVersion =
      StandardCode('unsupportedVersion');
  static const StandardCode expired = StandardCode('expired');
  static const StandardCode proofRequired = StandardCode('proofRequired');
  static const StandardCode proofInvalid = StandardCode('proofInvalid');
  static const StandardCode permissionDenied = StandardCode('permissionDenied');
  static const StandardCode wrongRecipient = StandardCode('wrongRecipient');
  static const StandardCode identityMismatch = StandardCode('identityMismatch');
  static const StandardCode idConflict = StandardCode('idConflict');
  static const StandardCode cancelled = StandardCode('cancelled');
  static const StandardCode taskFailed = StandardCode('taskFailed');
  static const StandardCode unavailable = StandardCode('unavailable');
  static const StandardCode internalError = StandardCode('internalError');

  /// Every §8.3 code, in the order SPEC.md declares them.
  static const List<StandardCode> values = <StandardCode>[
    malformedRequest,
    unsupportedType,
    unsupportedVersion,
    expired,
    proofRequired,
    proofInvalid,
    permissionDenied,
    wrongRecipient,
    identityMismatch,
    idConflict,
    cancelled,
    taskFailed,
    unavailable,
    internalError,
  ];
}

final Set<String> _standard = <String>{
  for (final c in StandardCode.values) c.value,
};

/// Framework 0.1 snake_case spellings, mapped to their 0.2 lowerCamelCase form.
/// `expired` and `unavailable` are single words and unchanged.
const Map<String, String> _legacyStandard = <String, String>{
  'malformed_request': 'malformedRequest',
  'unsupported_type': 'unsupportedType',
  'unsupported_version': 'unsupportedVersion',
  'proof_required': 'proofRequired',
  'proof_invalid': 'proofInvalid',
  'permission_denied': 'permissionDenied',
  'wrong_recipient': 'wrongRecipient',
  'identity_mismatch': 'identityMismatch',
  'id_conflict': 'idConflict',
  'canceled': 'cancelled',
  'task_failed': 'taskFailed',
  'internal_error': 'internalError',
};

/// Normalize a wire `code` to its canonical 0.2 spelling when it is a standard
/// code, or return it unchanged.
///
/// A consumer comparing a received code against [StandardCode.values] must
/// normalize first, or a 0.1 peer's `proof_required` reads as an unrecognized
/// extended code and falls through to `taskFailed` (§8.5), losing the meaning.
String normalizeCode(String code) {
  if (_standard.contains(code)) return code;
  return _legacyStandard[code] ?? code;
}

/// Whether `code` is a standard §8.3 code, in either casing.
bool isStandardCode(String code) => _standard.contains(normalizeCode(code));

/// The local part of an extended code: a lowercase letter, then letters of
/// either case, digits, or underscores.
///
/// Both casings are accepted so framework 0.2 lowerCamelCase locals
/// (`documentRevoked`) and frozen 0.1 snake_case locals (`document_revoked`)
/// parse under one rule. SPEC §4.10 item 4 SHOULDs lowerCamelCase for new
/// specifications; only the first character is required to be lowercase.
final RegExp _localRe = RegExp(r'^[a-z][A-Za-z0-9_]*$');

/// One path segment of a slug: lowercase, hyphen-separated (§6.1).
final RegExp _segmentRe = RegExp(r'^[a-z][a-z0-9]*(-[a-z0-9]+)*$');

bool _validNamespace(String namespace) =>
    namespace.isNotEmpty && namespace.split('/').every(_segmentRe.hasMatch);

const String _typeUriPrefix = 'https://trusttasks.org/spec/';

/// Build an extended error code under a specification's own slug (SPEC §8.5).
///
/// [typeUri] is normally a generated library's `typeUri`, so the namespace
/// cannot drift from the type's identity. The `#response` fragment is stripped:
/// an error raised while handling a response still belongs to the bare slug.
///
/// Throws [ArgumentError] if the derived slug or [local] is malformed.
String extendedCode(String typeUri, String local) {
  final slug = slugFromTypeUri(typeUri);
  if (!_localRe.hasMatch(local)) {
    throw ArgumentError.value(
      local,
      'local',
      'must match $_localRe (a lowercase first character, then letters, '
          'digits or underscores)',
    );
  }
  return '$slug:$local';
}

/// Build an extended error code under a *family namespace* — a proper path
/// prefix of the specification's slug (SPEC §8.5 rule 2).
///
/// For a condition whose meaning is defined once across a family rather than per
/// specification, such as `did-management:unknownDomain` on every
/// `did-management/*` task. Prefer [extendedCode] otherwise: a family namespace
/// claims the condition means the same thing across every sibling.
///
/// [namespace] is checked against the slug derived from [typeUri] rather than
/// taken on trust, so §8.5's prefix rule holds by construction. A sibling's slug
/// is rejected — it shares a prefix but is not itself one, which is exactly the
/// confusion §8.5 forbids.
String familyCode(String typeUri, String namespace, String local) {
  final slug = slugFromTypeUri(typeUri);
  final segs = slug.split('/');
  final permitted = <String>[
    for (var i = 0; i < segs.length; i++) segs.take(i + 1).join('/'),
  ];
  if (!permitted.contains(namespace)) {
    throw ArgumentError.value(
      namespace,
      'namespace',
      'is neither the slug "$slug" nor a path prefix of it (SPEC §8.5 rule 2); '
          'permitted: ${permitted.join(', ')}',
    );
  }
  if (!_localRe.hasMatch(local)) {
    throw ArgumentError.value(local, 'local', 'must match $_localRe');
  }
  return '$namespace:$local';
}

/// The slug of a Type URI, with any `#request` / `#response` fragment removed.
String slugFromTypeUri(String typeUri) {
  if (!typeUri.startsWith(_typeUriPrefix)) {
    throw ArgumentError.value(typeUri, 'typeUri', 'not a Trust Task Type URI');
  }
  var rest = typeUri.substring(_typeUriPrefix.length);
  final hash = rest.indexOf('#');
  if (hash >= 0) rest = rest.substring(0, hash);
  // The trailing segment is the MAJOR.MINOR version; everything before it is
  // the slug.
  final parts = rest.split('/');
  if (parts.length < 2) {
    throw ArgumentError.value(typeUri, 'typeUri', 'carries no slug');
  }
  final slug = parts.take(parts.length - 1).join('/');
  if (!_validNamespace(slug)) {
    throw ArgumentError.value(typeUri, 'typeUri', 'yielded an invalid slug');
  }
  return slug;
}
