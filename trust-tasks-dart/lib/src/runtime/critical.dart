// SPEC §4.5.2 — namespaces a producer marks critical with `extCritical`.
//
// An `ext` namespace is ignorable by default (§4.5.1 rule 5). A specification
// may let a producer list some of them in a sibling `extCritical` array, and a
// consumer that does not recognize a namespace listed there MUST NOT process
// the document and MUST reject it with `unsupportedExtension` (§4.5.2 rule 4).
// An entry naming a namespace absent from the sibling `ext` is non-conforming
// and is rejected with `malformedRequest` (rule 2).

import 'dart:convert';

import 'codes.dart';
import 'document.dart';

const int _maxDepth = 128;
const int _baseSteps = 100000;
const int _stepsPerValue = 64;
const int _maxEchoedNamespace = 128;
const String _extCritical = 'extCritical';
final RegExp _namespace = RegExp(r'^[a-z][a-z0-9-]*(\.[a-z0-9-]+)+$');

/// Apply SPEC §4.5.2 to [payload], read under [schema] — the payload schema of
/// the specification the document is consumed under ([SpecPolicy.payloadSchema],
/// JSON text), or `null` to read every level. [recognized] is every `ext`
/// namespace this consumer implements.
///
/// `extCritical` exists only at the object levels a specification opts in at,
/// so the payload is walked alongside its schema:
///
/// - a level whose schema declares an `extCritical` property is checked;
/// - a level whose schema closes it (`additionalProperties: false` or
///   `unevaluatedProperties: false`) without declaring it is refused as
///   `malformedRequest` when the member is present — what a schema validator
///   reports, applied even under `AcceptPayloadUnvalidated` so the marking is
///   never dropped silently;
/// - a level the schema leaves open is data (a document a vault is asked to
///   sign, say), and a member named `extCritical` in it is not a marking.
///
/// Every branch of `allOf` / `anyOf` / `oneOf` is followed. With no schema,
/// every level outside an `ext` namespace's own content is checked. The Rust,
/// TypeScript and Go runtimes implement the same walk;
/// `fixtures/consume/ext-critical.json` pins the four together.
///
/// Returns `null`, or the refusal: `unsupportedExtension` for the first
/// critical namespace not recognized, `malformedRequest` for a marking that
/// breaks rules 1 and 2 or sits at a level that does not allow it. A payload
/// with no member named `extCritical` anywhere returns `null` after one pass.
RejectReason? checkCriticalExtensions(
  Object? payload,
  String? schema, [
  Iterable<String> recognized = const [],
]) {
  final values = _countValuesIfMarked(payload);
  if (values == null) return null;
  Object? root;
  if (schema != null && schema.trim().isNotEmpty) {
    try {
      root = jsonDecode(schema);
    } on FormatException {
      root = null;
    }
  }
  final walk = _Walk(recognized.toSet(), root, values);
  try {
    if (root == null) {
      walk.blind(payload, 0);
    } else {
      walk.guided(payload, [root], 0);
    }
    return null;
  } on _Refusal catch (r) {
    return r.reason;
  }
}

class _Refusal implements Exception {
  const _Refusal(this.reason);
  final RejectReason reason;
}

_Refusal _malformed(String message) => _Refusal(
  RejectReason(
    code: StandardCode.malformedRequest,
    message: '$message (SPEC §4.5.2)',
  ),
);

_Refusal _tooComplex() => _malformed(
  'payload is too deeply nested or too complex to check its critical '
  'extensions',
);

int? _countValuesIfMarked(Object? payload) {
  var n = 0;
  var marked = false;
  final stack = <Object?>[payload];
  while (stack.isNotEmpty) {
    final v = stack.removeLast();
    n += 1;
    if (v is Map<dynamic, dynamic>) {
      if (v.containsKey(_extCritical)) marked = true;
      stack.addAll(v.values);
    } else if (v is List) {
      stack.addAll(v);
    }
  }
  return marked ? n : null;
}

Object? _resolve(String ref, Object? root) {
  if (!ref.startsWith('#')) return null;
  final pointer = ref.substring(1);
  if (pointer.isEmpty) return root;
  if (!pointer.startsWith('/')) return null;
  Object? node = root;
  for (final raw in pointer.substring(1).split('/')) {
    final token = raw.replaceAll('~1', '/').replaceAll('~0', '~');
    if (node is Map<dynamic, dynamic> && node.containsKey(token)) {
      node = node[token];
    } else if (node is List) {
      final i = int.tryParse(token);
      if (i == null || i < 0 || i >= node.length) return null;
      node = node[i];
    } else {
      return null;
    }
  }
  return node;
}

Object? _property(Map<dynamic, dynamic> schema, String name) {
  final props = schema['properties'];
  return props is Map<dynamic, dynamic> && props.containsKey(name)
      ? props[name]
      : null;
}

bool _hasProperty(Map<dynamic, dynamic> schema, String name) {
  final props = schema['properties'];
  return props is Map<dynamic, dynamic> && props.containsKey(name);
}

bool _closes(Map<dynamic, dynamic> schema) =>
    schema['additionalProperties'] == false ||
    schema['unevaluatedProperties'] == false;

List<Object?> _memberSchemas(
  List<Map<dynamic, dynamic>> applicable,
  String name,
) {
  final out = <Object?>[];
  for (final schema in applicable) {
    if (_hasProperty(schema, name)) {
      out.add(_property(schema, name));
      continue;
    }
    // Taken whole: matching the patterns would only narrow the places a
    // marking may be read.
    final patterns = schema['patternProperties'];
    if (patterns is Map<dynamic, dynamic>) out.addAll(patterns.values);
    final additional = schema['additionalProperties'];
    if (additional is Map<dynamic, dynamic>) out.add(additional);
  }
  return out;
}

List<Object?> _elementSchemas(
  List<Map<dynamic, dynamic>> applicable,
  int index,
) {
  final out = <Object?>[];
  for (final schema in applicable) {
    final prefix = schema['prefixItems'];
    if (prefix is List && index < prefix.length) {
      out.add(prefix[index]);
      continue;
    }
    final items = schema['items'];
    if (items is Map<dynamic, dynamic>) out.add(items);
  }
  return out;
}

class _Walk {
  _Walk(this.recognized, this.root, int values)
    : left = _baseSteps + _stepsPerValue * values;

  final Set<String> recognized;
  final Object? root;
  int left;

  void _spend(int depth) {
    if (depth > _maxDepth || left <= 0) throw _tooComplex();
    left -= 1;
  }

  void blind(Object? value, int depth) {
    _spend(depth);
    if (value is List) {
      for (final e in value) {
        blind(e, depth + 1);
      }
    } else if (value is Map<dynamic, dynamic>) {
      if (value.containsKey(_extCritical)) _checkLevel(value);
      for (final entry in value.entries) {
        if (entry.key == 'ext' || entry.key == _extCritical) continue;
        blind(entry.value, depth + 1);
      }
    }
  }

  void guided(Object? value, List<Object?> schemas, int depth) {
    _spend(depth);
    if (value is! Map<dynamic, dynamic> && value is! List) return;
    final applicable = <Map<dynamic, dynamic>>[];
    for (final s in schemas) {
      _expand(s, applicable, depth);
    }
    if (applicable.isEmpty) return;
    if (value is List) {
      for (var i = 0; i < value.length; i++) {
        final children = _elementSchemas(applicable, i);
        if (children.isNotEmpty) guided(value[i], children, depth + 1);
      }
      return;
    }
    final level = value as Map<dynamic, dynamic>;
    if (level.containsKey(_extCritical)) {
      if (applicable.any((s) => _hasProperty(s, _extCritical))) {
        _checkLevel(level);
      } else if (applicable.any(_closes)) {
        throw _malformed(
          '`extCritical` is not allowed at this level of the payload',
        );
      }
    }
    for (final entry in level.entries) {
      if (entry.key == _extCritical || entry.key is! String) continue;
      final children = _memberSchemas(applicable, entry.key as String);
      if (children.isNotEmpty) guided(entry.value, children, depth + 1);
    }
  }

  void _expand(Object? schema, List<Map<dynamic, dynamic>> out, int depth) {
    _spend(depth);
    if (schema is! Map<dynamic, dynamic>) return;
    if (out.any((s) => identical(s, schema))) return;
    out.add(schema);
    final ref = schema[r'$ref'];
    if (ref is String && root != null) {
      final target = _resolve(ref, root);
      if (target != null) _expand(target, out, depth + 1);
    }
    for (final keyword in const ['allOf', 'anyOf', 'oneOf']) {
      final branches = schema[keyword];
      if (branches is List) {
        for (final b in branches) {
          _expand(b, out, depth + 1);
        }
      }
    }
  }

  /// Rules 1 and 2, then rule 4, at one object level.
  void _checkLevel(Map<dynamic, dynamic> level) {
    final entries = level[_extCritical];
    if (entries is! List) throw _malformed('`extCritical` must be an array');
    if (entries.isEmpty) {
      throw _malformed('`extCritical` must name at least one namespace');
    }
    final ext = level['ext'];
    if (ext is! Map<dynamic, dynamic>) {
      throw _malformed(
        '`extCritical` is present where there is no sibling `ext` object',
      );
    }
    final seen = <String>[];
    for (final entry in entries) {
      if (entry is! String) {
        throw _malformed('every `extCritical` entry must be a string');
      }
      if (!_namespace.hasMatch(entry)) {
        throw _malformed(
          'every `extCritical` entry must be a reverse-DNS namespace',
        );
      }
      if (seen.contains(entry)) {
        throw _malformed('`extCritical` entries must be unique');
      }
      seen.add(entry);
      if (!ext.containsKey(entry)) {
        throw _malformed(
          'every `extCritical` entry must name a namespace present in the '
          'sibling `ext`',
        );
      }
    }
    for (final name in seen) {
      if (!recognized.contains(name)) {
        final echoed = name.length > _maxEchoedNamespace
            ? name.substring(0, _maxEchoedNamespace)
            : name;
        throw _Refusal(
          RejectReason(
            code: StandardCode.unsupportedExtension,
            message: 'unsupported critical extension: $echoed',
          ),
        );
      }
    }
  }
}
