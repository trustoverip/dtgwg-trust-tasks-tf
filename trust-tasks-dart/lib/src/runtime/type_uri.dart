/// Parsed Trust Task *Type URI* — every form SPEC.md §6.1 defines.
///
/// The public registry's canonical form, and the two shapes a third-party
/// specification (SPEC.md §6.5) may use under any other authority:
///
/// ```text
/// https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>          public registry
/// <prefix>/spec/<slug>/<MAJOR.MINOR>                        path form (https, DID URL, …)
/// urn:<NID>:<prefix>:spec:<slug-segments>:<MAJOR.MINOR>     URN form
/// ```
///
/// each optionally followed by `#request` or `#response` (§4.4.1). A Type URI
/// whose scheme is `urn` must use the URN form, and every other scheme the path
/// form; `http` is refused in every form. In the URN form the slug's segments
/// are `:`-separated (`urn:example:tasks:spec:payments:settle:0.1` names the
/// slug `payments/settle`).
///
/// A Type URI is a name compared by its whole string (§6.1), so the authority is
/// kept verbatim and is part of equality: the same slug under two authorities is
/// two specifications.
///
/// Hand-written. Mirrors `type_uri.rs` in trust-tasks-rs, rule for rule — the
/// two must accept and refuse the same strings.
library;

/// The `#request` / `#response` fragment of a Type URI (SPEC.md §4.4.1).
///
/// An `enum` is right here, unlike for wire codes: the parser refuses any other
/// fragment, so there is no unrecognised value to carry through.
enum TypeUriVariant {
  /// `#request` — or no fragment, which is semantically equivalent.
  request,

  /// `#response` — the success-response variant of the same specification.
  response,
}

/// Why a string failed to parse as a [TypeUri].
enum TypeUriError {
  /// No scheme, a malformed one, or `http`, which SPEC §6.1 forbids in every
  /// form.
  scheme,

  /// Neither the path form `<prefix>/spec/<slug>/<major.minor>` nor (for a
  /// `urn:`) the URN form `urn:<nid>:<prefix>:spec:<seg>:…:<major.minor>`.
  shape,

  /// The slug fails the §6.1 grammar.
  slug,

  /// The slug is in the reserved `trust-task` / `trust-ceremony` namespace
  /// (§6.1) and is not a framework-defined slug.
  reservedSlug,

  /// The version segment is not `MAJOR.MINOR` (§5.1).
  version,

  /// A fragment other than `#request` or `#response`.
  fragment,
}

/// Thrown by [TypeUri.parse] for a string that is not a Trust Task Type URI.
final class TypeUriFormatException extends FormatException {
  TypeUriFormatException(this.kind, String message, String source)
      : super(message, source);

  /// Which rule the string broke.
  final TypeUriError kind;
}

/// A parsed Trust Task *Type URI*.
///
/// [toString] reproduces the parsed string exactly; `==` and [hashCode] compare
/// every component, the authority included.
final class TypeUri {
  const TypeUri._(
    this.authority,
    this.isUrn,
    this.slug,
    this.major,
    this.minor,
    this.variant,
  );

  /// A path-form Type URI from its components:
  /// `<authority>/spec/<slug>/<major>.<minor>`.
  ///
  /// [authority] is everything before `/spec/` — the scheme, host and any path
  /// prefix, or a DID. A trailing `/` is dropped. Throws
  /// [TypeUriFormatException] for a malformed slug, and [ArgumentError] for a
  /// negative version component.
  factory TypeUri.pathForm(
    String authority,
    String slug,
    int major,
    int minor, {
    TypeUriVariant? variant,
  }) {
    _validateSlug(slug, slug);
    _checkVersion(major, minor);
    return TypeUri._(
      _trimEnd(authority, '/'),
      false,
      slug,
      major,
      minor,
      variant,
    );
  }

  /// A URN-form Type URI from its components:
  /// `<prefix>:spec:<slug segments>:<major>.<minor>`, where [prefix] is
  /// `urn:<NID>:<prefix>`, e.g. `urn:example:tasks`. A trailing `:` is
  /// dropped.
  factory TypeUri.urnForm(
    String prefix,
    String slug,
    int major,
    int minor, {
    TypeUriVariant? variant,
  }) {
    _validateSlug(slug, slug);
    _checkVersion(major, minor);
    return TypeUri._(_trimEnd(prefix, ':'), true, slug, major, minor, variant);
  }

  /// A Type URI in the public registry, `https://trusttasks.org`.
  factory TypeUri.canonical(String slug, int major, int minor) =>
      TypeUri.pathForm(registryAuthority, slug, major, minor);

  /// The public registry's authority.
  static const String registryAuthority = 'https://trusttasks.org';

  /// Parse [source] as a Type URI in any §6.1 form.
  ///
  /// Throws [TypeUriFormatException] when it is not one.
  factory TypeUri.parse(String source) => _parse(source);

  /// [TypeUri.parse], returning `null` instead of throwing.
  static TypeUri? tryParse(String source) {
    try {
      return _parse(source);
    } on TypeUriFormatException {
      return null;
    }
  }

  /// Everything before `/spec/` (path form) or `:spec:` (URN form), verbatim.
  final String authority;

  /// Whether this is the URN form.
  final bool isUrn;

  /// The slug, `/`-separated in either form — e.g. `acl/grant`.
  final String slug;

  /// The MAJOR version.
  final int major;

  /// The MINOR version.
  final int minor;

  /// The fragment, if any. None is equivalent to [TypeUriVariant.request]
  /// (§4.4.1).
  final TypeUriVariant? variant;

  /// `MAJOR.MINOR`.
  String get version => '$major.$minor';

  /// Whether this names the success-response variant.
  bool get isResponse => variant == TypeUriVariant.response;

  /// Whether this is in the public registry's authority, path form.
  bool get isRegistry => !isUrn && authority == registryAuthority;

  /// This URI with the fragment removed — the form a schema is dereferenced by.
  TypeUri get bare => withVariant(null);

  /// This URI with the fragment set to [v], or removed when [v] is null.
  TypeUri withVariant(TypeUriVariant? v) =>
      TypeUri._(authority, isUrn, slug, major, minor, v);

  /// This URI with the fragment set to `#response`.
  TypeUri withResponse() => withVariant(TypeUriVariant.response);

  /// This URI at `major.minor` — same authority, form, slug and variant. How a
  /// declared-extension chain (SPEC §5.2), whose links name only a version, is
  /// walked without leaving the authority it started under.
  TypeUri withVersion(int major, int minor) {
    _checkVersion(major, minor);
    return TypeUri._(authority, isUrn, slug, major, minor, variant);
  }

  /// Whether [other] names a version of the same specification — the same
  /// authority, form and slug, whatever the version and variant. A slug is
  /// unique only within its authority (§6.1), so this, never the slug alone,
  /// decides whether two Type URIs are versions of one specification.
  bool isSameSpecification(TypeUri other) =>
      authority == other.authority &&
      isUrn == other.isUrn &&
      slug == other.slug;

  @override
  String toString() {
    final base = isUrn
        ? '$authority:spec:${slug.replaceAll('/', ':')}:$major.$minor'
        : '$authority/spec/$slug/$major.$minor';
    return switch (variant) {
      null => base,
      TypeUriVariant.request => '$base#request',
      TypeUriVariant.response => '$base#response',
    };
  }

  @override
  bool operator ==(Object other) =>
      other is TypeUri &&
      isSameSpecification(other) &&
      major == other.major &&
      minor == other.minor &&
      variant == other.variant;

  @override
  int get hashCode =>
      Object.hash(authority, isUrn, slug, major, minor, variant);
}

/* ── internals ──────────────────────────────────────────────────────────── */

final RegExp _schemeRe = RegExp(r'^[A-Za-z][A-Za-z0-9+\-.]*$');

/// One slug segment (§6.1): a lowercase letter, then lowercase letters and
/// digits, single hyphens between them, no trailing hyphen.
final RegExp _segmentRe = RegExp(r'^[a-z][a-z0-9]*(-[a-z0-9]+)*$');

/// A version component: a non-negative integer without a leading zero.
final RegExp _versionPartRe = RegExp(r'^(0|[1-9][0-9]*)$');

/// The largest component trust-tasks-rs accepts (`u32`).
const int _maxVersionPart = 0xFFFFFFFF;

TypeUri _parse(String s) {
  TypeUriFormatException err(TypeUriError kind, String message) =>
      TypeUriFormatException(kind, message, s);

  final hash = s.indexOf('#');
  final withoutFragment = hash < 0 ? s : s.substring(0, hash);
  final TypeUriVariant? variant;
  if (hash < 0) {
    variant = null;
  } else {
    final fragment = s.substring(hash + 1);
    variant = switch (fragment) {
      'request' => TypeUriVariant.request,
      'response' => TypeUriVariant.response,
      _ => throw err(
          TypeUriError.fragment,
          'only #request and #response fragments are permitted '
          '(got #$fragment)',
        ),
    };
  }

  // SPEC §6.1: any absolute URI, but never `http` — an `http` Type URI would
  // normalise a transport-downgrade path for a consumer that dereferences it
  // under §6.2 content negotiation, so it is refused in every form.
  final colon = withoutFragment.indexOf(':');
  final scheme = colon < 0 ? '' : withoutFragment.substring(0, colon);
  final lower = scheme.toLowerCase();
  if (!_schemeRe.hasMatch(scheme) || lower == 'http') {
    throw err(
      TypeUriError.scheme,
      'type URI must have a scheme other than http (SPEC §6.1)',
    );
  }
  TypeUriFormatException shape() => err(
        TypeUriError.shape,
        'type URI must be <prefix>/spec/<slug>/<major.minor>, or '
        'urn:<nid>:<prefix>:spec:<slug-segments>:<major.minor>',
      );

  final String authority;
  final String slug;
  final String version;
  final bool urn;
  if (lower == 'urn') {
    // `urn:<NID>:<prefix>:spec:<seg>:…:<MAJOR.MINOR>`. The slug is the
    // segments between the first `:spec:` and the version.
    final at = withoutFragment.indexOf(':spec:');
    if (at < 0) throw shape();
    final prefix = withoutFragment.substring(0, at);
    final rest = withoutFragment.substring(at + ':spec:'.length);
    // `urn`, the NID, and a namespace-specific prefix: both non-empty.
    final parts = prefix.split(':');
    if (parts.length < 3 || parts[1].isEmpty) throw shape();
    if (parts.sublist(2).join(':').isEmpty) throw shape();
    if (rest.contains('/')) throw shape();
    final last = rest.lastIndexOf(':');
    if (last < 0) throw shape();
    authority = prefix;
    slug = rest.substring(0, last).replaceAll(':', '/');
    version = rest.substring(last + 1);
    urn = true;
  } else {
    // `<prefix>/spec/<slug>/<MAJOR.MINOR>` — an https authority (with an
    // optional path prefix), a DID URL, or any other scheme's prefix.
    final at = withoutFragment.indexOf('/spec/');
    if (at < 0) throw shape();
    final prefix = withoutFragment.substring(0, at);
    final afterScheme = prefix.substring(scheme.length + 1);
    if (_trimStart(afterScheme, '/').isEmpty ||
        (lower == 'https' && !afterScheme.startsWith('//'))) {
      throw shape();
    }
    final rest = withoutFragment.substring(at + '/spec/'.length);
    // The version is the final '/' segment; everything before it is the slug.
    final last = rest.lastIndexOf('/');
    if (last < 0) throw shape();
    authority = prefix;
    slug = rest.substring(0, last);
    version = rest.substring(last + 1);
    urn = false;
  }

  _validateSlug(slug, s);
  final (major, minor) = _parseVersion(version, s);
  return TypeUri._(authority, urn, slug, major, minor, variant);
}

(int, int) _parseVersion(String v, String source) {
  TypeUriFormatException bad() => TypeUriFormatException(
        TypeUriError.version,
        'version "$v" must be MAJOR.MINOR per SPEC §5.1',
        source,
      );
  final dot = v.indexOf('.');
  if (dot < 0) throw bad();
  final maj = v.substring(0, dot);
  final min = v.substring(dot + 1);
  if (!_versionPartRe.hasMatch(maj) || !_versionPartRe.hasMatch(min)) {
    throw bad();
  }
  final major = int.tryParse(maj);
  final minor = int.tryParse(min);
  if (major == null ||
      minor == null ||
      major > _maxVersionPart ||
      minor > _maxVersionPart) {
    throw bad();
  }
  return (major, minor);
}

/// A version string from an `extends` declaration (`M.K`), or null when it
/// is not one. Shared with the extension walker so both read versions alike;
/// internal — the barrel hides it.
(int, int)? parseTypeUriVersion(String v) {
  try {
    return _parseVersion(v, v);
  } on TypeUriFormatException {
    return null;
  }
}

void _validateSlug(String slug, String source) {
  if (slug.isEmpty || !slug.split('/').every(_segmentRe.hasMatch)) {
    throw TypeUriFormatException(
      TypeUriError.slug,
      'slug "$slug" does not satisfy SPEC §6.1 grammar',
      source,
    );
  }
  if (_isReservedNamespace(slug) && !_allowedFrameworkSlugs.contains(slug)) {
    throw TypeUriFormatException(
      TypeUriError.reservedSlug,
      'slug "$slug" is reserved per SPEC §6.1',
      source,
    );
  }
}

/// SPEC §6.1 reserves `^trust-(task|ceremony)($|-|/)` — both halves.
bool _isReservedNamespace(String slug) {
  final first = slug.split('/').first;
  for (final stem in const ['trust-task', 'trust-ceremony']) {
    if (first == stem || first.startsWith('$stem-')) return true;
  }
  return false;
}

/// The framework slugs permitted inside the reservation. A third
/// hand-maintained copy, beside trust-tasks-rs's `is_allowed_framework_slug`
/// and `#/properties/slug/anyOf[1]/enum` in `specs/spec.meta.schema.json`;
/// `test/type_uri_test.dart` fails when this one drifts from the latter.
const Set<String> _allowedFrameworkSlugs = <String>{
  'trust-task',
  'trust-task-error',
  'trust-task-ok',
  'trust-task-next-step',
  'trust-task-discovery',
  'trust-task-control',
  'trust-ceremony-receipt',
};

void _checkVersion(int major, int minor) {
  if (major < 0 || major > _maxVersionPart) {
    throw RangeError.range(major, 0, _maxVersionPart, 'major');
  }
  if (minor < 0 || minor > _maxVersionPart) {
    throw RangeError.range(minor, 0, _maxVersionPart, 'minor');
  }
}

String _trimEnd(String s, String c) {
  var end = s.length;
  while (end > 0 && s[end - 1] == c) {
    end--;
  }
  return s.substring(0, end);
}

String _trimStart(String s, String c) {
  var start = 0;
  while (start < s.length && s[start] == c) {
    start++;
  }
  return s.substring(start);
}
