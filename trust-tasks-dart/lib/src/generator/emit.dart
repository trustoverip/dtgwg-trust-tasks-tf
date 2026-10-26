// One Dart library from one Trust Tasks bundle.
//
// A port of scripts/build-dart-bindings.mjs — the generator that writes the
// prebuilt tree under lib/specs/ — with its input swapped from the specs/ tree
// to a registry bundle (docs/design/consumer-side-generation.md). Ported, not
// shared: the repository keeps independent generators on purpose, because
// `npm run check-bindings` and this package's equivalence check compare their
// output, and one implementation quoted twice proves nothing. Do not factor it
// together with the .mjs, or with another language's port.
//
// What the bundle changes, and what it cannot carry:
//
//   * No `$ref` resolution and no front-matter parsing. The bundle's schema is
//     already self-contained, and its policy, deprecation note and `extends`
//     are already derived (by the Rust codegen, which builds the bundles).
//
//   * The bundle's schema is key-sorted (RFC 8785). `propertyOrder` restores
//     the authored order of every `properties` object, so the generated
//     classes list their fields — and serialize their members — exactly as the
//     prebuilt ones do. Nothing restores the authored order of `$defs`, so the
//     top-level declarations come out in `$defs`-key order (dependencies first,
//     as before) rather than the order the .mjs happened to splice them in, and
//     the embedded schema text is the bundle's key order. Both are reported by
//     the equivalence check (scripts/check-dart-consumer-generation.mjs).

import 'dart:convert';

/// The runtime library a generated library imports for [SpecPolicy] — by
/// package, never by a relative path into the runtime.
const String runtimeImport = 'package:trust_tasks/trust_tasks.dart';

/* ── Bundle ───────────────────────────────────────────────────────────────── */

/// A bundle this generator cannot use, with the reason.
class BundleError implements Exception {
  BundleError(this.message);
  final String message;
  @override
  String toString() => message;
}

/// The members of a `bundleVersion: 1` bundle this generator reads.
class Bundle {
  Bundle._({
    required this.typeUri,
    required this.slug,
    required this.version,
    required this.status,
    required this.deprecation,
    required this.extendsVersion,
    required this.policy,
    required this.schema,
    required this.responseSchema,
    required this.schemaText,
    required this.responseSchemaText,
  });

  /// The `bundleVersion` this generator reads.
  static const int supportedVersion = 1;

  final String typeUri;
  final String slug;
  final String version;
  final String status;
  final String? deprecation;
  final String? extendsVersion;
  final Map<String, dynamic> policy;

  /// The request payload schema with `propertyOrder` applied — what the types
  /// are emitted from.
  final Map<String, dynamic> schema;

  /// The response document schema with `propertyOrder` applied, or null.
  final Map<String, dynamic>? responseSchema;

  /// The schemas as the bundle carries them, for the embedded JSON constants.
  final String schemaText;
  final String? responseSchemaText;

  bool get isWithdrawn => status == 'withdrawn';

  /// Parse a bundle's bytes (already checked against the lock's digest).
  static Bundle parse(String text) {
    final Object? json;
    try {
      json = jsonDecode(text);
    } on FormatException catch (e) {
      throw BundleError('not JSON: ${e.message}');
    }
    if (json is! Map<String, dynamic>) {
      throw BundleError('not a JSON object');
    }
    final decoded = json;
    final bv = decoded['bundleVersion'];
    if (bv != supportedVersion) {
      throw BundleError(
        'bundleVersion ${jsonEncode(bv)} is not supported '
        '(this generator reads $supportedVersion) — upgrade trust_tasks',
      );
    }
    T need<T>(String key) {
      if (!decoded.containsKey(key)) throw BundleError('no `$key` member');
      final v = decoded[key];
      if (v is! T) throw BundleError('`$key` has the wrong type');
      return v;
    }

    final ext = need<String?>('extends');
    if (ext != null && !RegExp(r'^\d+\.\d+$').hasMatch(ext)) {
      throw BundleError(
          '`extends` must be "MAJOR.MINOR", got ${jsonEncode(ext)}');
    }
    final order = decoded['propertyOrder'] ?? <String, dynamic>{};
    if (order is! Map<String, dynamic>) {
      throw BundleError('`propertyOrder` has the wrong type');
    }
    final schema = need<Map<String, dynamic>>('schema');
    final response = need<Map<String, dynamic>?>('responseSchema');

    return Bundle._(
      typeUri: need<String>('typeUri'),
      slug: need<String>('slug'),
      version: need<String>('version'),
      status: need<String>('status'),
      deprecation: need<String?>('deprecation'),
      extendsVersion: ext,
      policy: need<Map<String, dynamic>>('policy'),
      schemaText: jsonEncode(schema),
      responseSchemaText: response == null ? null : jsonEncode(response),
      schema: _ordered(schema, '/schema', order) as Map<String, dynamic>,
      responseSchema: response == null
          ? null
          : _ordered(response, '/responseSchema', order)
              as Map<String, dynamic>,
    );
  }

  /// A deep copy of [node] (at JSON Pointer [pointer]) with every `properties`
  /// object in its authored order, as `propertyOrder` records it.
  static Object? _ordered(
    Object? node,
    String pointer,
    Map<String, dynamic> order,
  ) {
    if (node is List) {
      return [
        for (var i = 0; i < node.length; i++)
          _ordered(node[i], '$pointer/$i', order),
      ];
    }
    if (node is! Map<String, dynamic>) return node;
    final out = <String, dynamic>{};
    for (final entry in node.entries) {
      out[entry.key] =
          _ordered(entry.value, '$pointer/${_escape(entry.key)}', order);
    }
    final props = out['properties'];
    final names = order[pointer];
    if (props is Map<String, dynamic> && names is List) {
      final reordered = <String, dynamic>{};
      for (final name in names) {
        if (name is String && props.containsKey(name)) {
          reordered[name] = props[name];
        }
      }
      // Anything propertyOrder does not name keeps the bundle's order, after.
      for (final entry in props.entries) {
        reordered.putIfAbsent(entry.key, () => entry.value);
      }
      out['properties'] = reordered;
    }
    return out;
  }

  static String _escape(String key) =>
      key.replaceAll('~', '~0').replaceAll('/', '~1');

  bool _flag(String name, String variant) {
    final group = policy[name];
    if (group is! Map || group[variant] is! bool) {
      throw BundleError('`policy.$name.$variant` is missing or not a boolean');
    }
    return group[variant] as bool;
  }

  bool get _isBearer {
    final b = policy['bearer'];
    if (b is! bool) throw BundleError('`policy.bearer` is not a boolean');
    return b;
  }
}

/* ── Dart identifiers ────────────────────────────────────────────────────── */

/// Split an arbitrary schema name into lowercase word segments.
List<String> _segments(String name) => name
    .replaceAllMapped(RegExp('([a-z0-9])([A-Z])'), (m) => '${m[1]} ${m[2]}')
    .replaceAllMapped(RegExp('([A-Z]+)([A-Z][a-z])'), (m) => '${m[1]} ${m[2]}')
    .split(RegExp('[^A-Za-z0-9]+'))
    .expand((part) => part.split(RegExp(r'\s+')))
    .where((s) => s.isNotEmpty)
    .map((s) => s.toLowerCase())
    .toList();

String _cap(String s) => s[0].toUpperCase() + s.substring(1);

/// `UpperCamelCase`, for a class or extension type.
String _dartClassName(String name) {
  final ident = _segments(name).map(_cap).join();
  if (ident.isEmpty) return 'X';
  return RegExp('^[0-9]').hasMatch(ident) ? 'N$ident' : ident;
}

/// `lowerCamelCase`, for a field, parameter or top-level constant.
String _dartFieldName(String name) {
  final parts = _segments(name);
  if (parts.isEmpty) return 'value';
  var ident = parts.first + parts.skip(1).map(_cap).join();
  if (RegExp('^[0-9]').hasMatch(ident)) ident = 'n$ident';
  return _dartReserved.contains(ident) ? '${ident}Value' : ident;
}

/// `lower_snake_case`, for a file or directory name (the `file_names` lint).
String dartFileName(String name) {
  final s = _segments(name).join('_');
  return s.isEmpty ? 'x' : s;
}

/// The genuinely reserved words; built-in identifiers are legal field names.
const Set<String> _dartReserved = {
  'assert', 'break', 'case', 'catch', 'class', 'const', 'continue', //
  'default', 'do', 'else', 'enum', 'extends', 'false', 'final', 'finally',
  'for', 'if', 'in', 'is', 'new', 'null', 'rethrow', 'return', 'super',
  'switch', 'this', 'throw', 'true', 'try', 'var', 'void', 'while', 'with',
};

/// Quote [s] as a Dart single-quoted string literal, escaping `$` too.
String dartString(String s) {
  final out = StringBuffer("'");
  for (final cp in s.runes) {
    if (cp == 0x27) {
      out.write(r"\'");
    } else if (cp == 0x5c) {
      out.write(r'\\');
    } else if (cp == 0x24) {
      out.write(r'\$');
    } else if (cp == 0x0a) {
      out.write(r'\n');
    } else if (cp == 0x0d) {
      out.write(r'\r');
    } else if (cp == 0x09) {
      out.write(r'\t');
    } else if (cp < 0x20 || cp == 0x7f) {
      out.write('\\x${cp.toRadixString(16).padLeft(2, '0')}');
    } else {
      out.writeCharCode(cp);
    }
  }
  out.write("'");
  return out.toString();
}

/// Render [text] as a Dart doc comment, indented by [indent].
List<String> _docComment(Object? text, [String indent = '']) {
  if (text is! String || text.trim().isEmpty) return const [];
  final words = text.replaceAll(RegExp(r'\s+'), ' ').trim().split(' ');
  final lines = <String>[];
  var line = '';
  for (final word in words) {
    if (line.length + word.length + 1 > 84 && line != '') {
      lines.add(line.trimRight());
      line = '';
    }
    line += '$word ';
  }
  if (line.trim() != '') lines.add(line.trimRight());
  // `[` starts a doc reference and `<` an HTML tag, as far as Dartdoc cares.
  return [
    for (final l in lines)
      '$indent/// ${l.replaceAll('[', r'\[').replaceAll(']', r'\]').replaceAll('<', '&lt;').replaceAll('>', '&gt;')}',
  ];
}

/* ── Schema -> Dart type emission ────────────────────────────────────────── */

/// JavaScript truthiness, which the .mjs generator's checks are written in.
bool _truthy(Object? v) =>
    v != null && v != false && v != 0 && v != '' && !(v is double && v.isNaN);

Map<String, dynamic> _obj(Object? node) =>
    node is Map<String, dynamic> ? node : const <String, dynamic>{};

/// Merge plain-object `allOf` branches into the node; leave it alone otherwise.
Map<String, dynamic> _mergeAllOf(Map<String, dynamic> node) {
  final allOf = node['allOf'];
  if (allOf is! List) return node;
  final merged = Map<String, dynamic>.of(node)..remove('allOf');
  for (final branch in allOf) {
    if (branch is! Map<String, dynamic> || branch.containsKey(r'$ref')) {
      return node;
    }
    merged['properties'] = <String, dynamic>{
      ..._obj(merged['properties']),
      ..._obj(branch['properties']),
    };
    final req = <Object?>{
      ...(merged['required'] is List ? merged['required'] as List : const []),
      ...(branch['required'] is List ? branch['required'] as List : const []),
    };
    merged['required'] = req.toList();
    if (branch['type'] != null) merged['type'] ??= branch['type'];
    if (branch['description'] != null) {
      merged['description'] ??= branch['description'];
    }
  }
  return merged;
}

/// The declared type(s) of [node], with `null` stripped and reported apart.
({List<Object?> types, bool nullable}) _typeOf(Object? node) {
  final raw = _obj(node)['type'];
  if (raw == null) return (types: const [], nullable: false);
  final list = raw is List ? raw : [raw];
  return (
    types: list.where((t) => t != 'null').toList(),
    nullable: list.contains('null'),
  );
}

typedef _Conv = String Function(String expr);

/// A Dart type plus how to read it from and write it to a decoded JSON value.
class _Desc {
  const _Desc(this.type, this.from, this.to, [this.cast]);
  final String type;
  final _Conv from;
  final _Conv to;

  /// The target of a plain `as` cast, when the conversion is exactly that.
  final String? cast;
}

String _identity(String e) => e;

class _Emitter {
  final List<String> decls = [];
  final Set<String> names = {};

  String reserve(String preferred) {
    var name = preferred;
    var n = 2;
    while (names.contains(name)) {
      name = '$preferred${n++}';
    }
    names.add(name);
    return name;
  }
}

_Desc _string() => _Desc('String', (e) => '$e as String', _identity, 'String');

_Desc _dartTypeFor(
  _Emitter em,
  Object? input,
  String hint,
  Map<String, _Desc> defNames,
) {
  if (input is! Map<String, dynamic>) {
    return const _Desc('Object?', _identity, _identity);
  }
  final ref = input[r'$ref'];
  if (ref is String) {
    return defNames[ref] ?? const _Desc('Object?', _identity, _identity);
  }

  final node = _mergeAllOf(input);
  final types = _typeOf(node).types;

  if (types.length > 1 ||
      (types.isEmpty &&
          (_truthy(node['oneOf']) ||
              _truthy(node['anyOf']) ||
              _truthy(node['not'])))) {
    final branches = node['oneOf'] ?? node['anyOf'];
    if (types.isEmpty &&
        branches is List &&
        branches.isNotEmpty &&
        branches.every((b) =>
            b is Map &&
            (b['type'] == 'string' ||
                _truthy(b['enum']) ||
                _truthy(b['pattern'])))) {
      return _string();
    }
    return const _Desc('Object?', _identity, _identity);
  }

  final type = types.isEmpty ? null : types.first;

  if (type == 'string') {
    final values = node['enum'];
    if (values is List &&
        values.isNotEmpty &&
        values.every((v) => v is String)) {
      return _declareExtensionType(em, node, hint);
    }
    return _string();
  }
  if (type == 'integer') {
    return _Desc('int', (e) => '$e as int', _identity, 'int');
  }
  if (type == 'number') {
    return _Desc('double', (e) => '($e as num).toDouble()', _identity);
  }
  if (type == 'boolean') {
    return _Desc('bool', (e) => '$e as bool', _identity, 'bool');
  }

  if (type == 'array') {
    final item =
        _dartTypeFor(em, node['items'] ?? true, '${hint}Item', defNames);
    final itemTo = item.to('e');
    return _Desc(
      'List<${item.type}>',
      (e) => '($e as List<dynamic>).map((e) => ${item.from('e')}).toList()',
      itemTo == 'e' ? _identity : (e) => '$e.map((e) => $itemTo).toList()',
    );
  }

  if (type == 'object') {
    final props = node['properties'];
    if (props == null || (props is Map && props.isEmpty)) {
      final ap = node['additionalProperties'];
      if (ap is Map<String, dynamic>) {
        final value = _dartTypeFor(em, ap, '${hint}Value', defNames);
        final valueTo = value.to('v');
        return _Desc(
          'Map<String, ${value.type}>',
          (e) =>
              '($e as Map<String, dynamic>).map((k, v) => MapEntry(k, ${value.from('v')}))',
          valueTo == 'v'
              ? _identity
              : (e) => '$e.map((k, v) => MapEntry(k, $valueTo))',
        );
      }
      return _Desc(
        'Map<String, dynamic>',
        (e) => '$e as Map<String, dynamic>',
        _identity,
        'Map<String, dynamic>',
      );
    }
    return _declareClass(em, node, hint, defNames);
  }

  return const _Desc('Object?', _identity, _identity);
}

/// An extension type over String for a closed set of values — not an enum,
/// which would throw on a value from a newer MINOR (SPEC §5.2).
_Desc _declareExtensionType(
  _Emitter em,
  Map<String, dynamic> node,
  String hint,
) {
  final name = em.reserve(hint);
  final lines = <String>[
    ..._docComment(node['description'] ??
        '$name is a closed set of string values defined by this specification\'s schema.'),
    '///',
    ..._docComment(
      'An extension type rather than an enum: a value from a newer MINOR of this '
      'specification must not crash the parse (SPEC §5.2), and an enum would throw on one. '
      'Compare against the constants below, and treat anything else as unrecognised.',
    ),
    'extension type const $name(String value) {',
  ];
  final seen = <String>{};
  final constNames = <String>[];
  for (final value in (node['enum'] as List).cast<String>()) {
    var constName = _dartFieldName(value);
    while (seen.contains(constName)) {
      constName += '_';
    }
    seen.add(constName);
    constNames.add(constName);
    lines.add('  static const $name $constName = $name(${dartString(value)});');
  }
  lines
    ..add('')
    ..add('  /// Every value this specification\'s schema permits.')
    ..add(
        '  static const List<$name> values = <$name>[${constNames.join(', ')}];')
    ..add('}');
  em.decls.add(lines.join('\n'));
  return _Desc(name, (e) => '$name($e as String)', (e) => '$e.value');
}

String _nullable(String type, bool isRequired) =>
    isRequired || type.endsWith('?') ? type : '$type?';

class _Field {
  _Field(this.wireName, this.fieldName, this.desc, this.isRequired, this.doc);
  final String wireName;
  final String fieldName;
  final _Desc desc;
  final bool isRequired;
  final Object? doc;
}

_Desc _declareClass(
  _Emitter em,
  Map<String, dynamic> node,
  String hint,
  Map<String, _Desc> defNames,
) {
  final name = em.reserve(hint);
  final required = node['required'] is List
      ? (node['required'] as List).toSet()
      : const <Object?>{};
  final fields = <_Field>[];
  final fieldNames = <String>{};

  for (final entry in (node['properties'] as Map<String, dynamic>).entries) {
    final wireName = entry.key;
    final member = entry.value;
    var fieldName = _dartFieldName(wireName);
    while (fieldNames.contains(fieldName)) {
      fieldName += '_';
    }
    fieldNames.add(fieldName);

    final memberNode = _obj(member);
    final nullable = _typeOf(memberNode).nullable;
    final desc =
        _dartTypeFor(em, member, '$name${_dartClassName(wireName)}', defNames);
    final isRequired = required.contains(wireName) && !nullable;
    fields.add(_Field(
        wireName, fieldName, desc, isRequired, memberNode['description']));
  }

  final lines = <String>[
    ..._docComment(node['description'] ??
        node['title'] ??
        '$name, generated from its schema.'),
    'class $name {',
    '  const $name({',
    for (final f in fields)
      '    ${f.isRequired ? 'required ' : ''}this.${f.fieldName},',
    '  });',
    '',
    '  /// Read this payload from a decoded JSON object.',
    '  factory $name.fromJson(Map<String, dynamic> json) => $name(',
  ];
  for (final f in fields) {
    final access = 'json[${dartString(f.wireName)}]';
    final String expr;
    if (f.isRequired) {
      expr = f.desc.from(access);
    } else if (f.desc.cast != null) {
      expr = '$access as ${f.desc.cast}?';
    } else if (f.desc.from(access) == access) {
      expr = access;
    } else {
      expr = '$access == null ? null : ${f.desc.from(access)}';
    }
    lines.add('        ${f.fieldName}: $expr,');
  }
  lines
    ..add('      );')
    ..add('');

  var first = true;
  for (final f in fields) {
    final doc = _docComment(f.doc, '  ');
    if (!first && doc.isNotEmpty) lines.add('');
    first = false;
    lines
      ..addAll(doc)
      ..add('  final ${_nullable(f.desc.type, f.isRequired)} ${f.fieldName};');
  }
  lines
    ..add('')
    ..add('  /// Serialize to a JSON-encodable map, omitting absent members.')
    ..add('  Map<String, dynamic> toJson() => <String, dynamic>{');
  for (final f in fields) {
    if (f.isRequired) {
      lines
          .add('        ${dartString(f.wireName)}: ${f.desc.to(f.fieldName)},');
    } else {
      final value = f.desc.to('${f.fieldName}!');
      lines.add(
          '        if (${f.fieldName} != null) ${dartString(f.wireName)}: $value,');
    }
  }
  lines
    ..add('      };')
    ..add('}');

  em.decls.add(lines.join('\n'));
  return _Desc(
    name,
    (e) => '$name.fromJson($e as Map<String, dynamic>)',
    (e) => '$e.toJson()',
  );
}

/// Every `#/$defs/<name>` that [node] reaches.
Set<String> _internalRefsOf(Object? node, [Set<String>? out]) {
  out ??= <String>{};
  if (node is List) {
    for (final item in node) {
      _internalRefsOf(item, out);
    }
  } else if (node is Map) {
    final ref = node[r'$ref'];
    if (ref is String && ref.startsWith(r'#/$defs/')) {
      out.add(ref.substring(r'#/$defs/'.length));
    }
    for (final value in node.values) {
      _internalRefsOf(value, out);
    }
  }
  return out;
}

/// The `$defs` ordered so a definition follows everything it references.
List<String> _topologicalDefOrder(Map<String, dynamic> defs) {
  final names = defs.keys.toList();
  final known = names.toSet();
  final deps = {
    for (final name in names)
      name: _internalRefsOf(defs[name])
          .where((r) => known.contains(r) && r != name)
          .toList(),
  };
  final order = <String>[];
  final state = {for (final n in names) n: 0}; // 0 new, 1 active, 2 done
  void visit(String name) {
    if (state[name] != 0) return;
    state[name] = 1;
    for (final dep in deps[name]!) {
      visit(dep);
    }
    state[name] = 2;
    order.add(name);
  }

  names.forEach(visit);
  return order;
}

_Desc _emitNamed(
  _Emitter em,
  Object? node,
  String target,
  Map<String, _Desc> defNames,
) {
  em.names.remove(target);
  final produced = _dartTypeFor(em, node, target, defNames);
  em.names.add(target);
  return produced;
}

/// Declare a Dart type for every `$defs` entry, then for the root.
({String rootName, String? responseName}) _emitDeclarations(
  _Emitter em,
  Map<String, dynamic> schema,
  String rootHint,
) {
  final defs = _obj(schema[r'$defs']);
  final defNames = <String, _Desc>{};
  final pending = <String, String>{};

  for (final defName in defs.keys) {
    final target = em.reserve(_dartClassName(defName));
    pending[defName] = target;
    defNames['#/\$defs/$defName'] = _Desc(
      target,
      (e) => '$target.fromJson($e as Map<String, dynamic>)',
      (e) => '$e.toJson()',
    );
  }
  final rootName = em.reserve(rootHint);

  for (final defName in _topologicalDefOrder(defs)) {
    final defNode = defs[defName];
    final target = pending[defName]!;
    final produced = _emitNamed(em, defNode, target, defNames);
    if (produced.type != target) {
      em.decls.add([
        ..._docComment(_obj(defNode)['description'] ??
            '$target, defined by this specification\'s schema.'),
        'typedef $target = ${produced.type};',
      ].join('\n'));
      defNames['#/\$defs/$defName'] =
          _Desc(target, produced.from, produced.to, produced.cast);
    } else {
      defNames['#/\$defs/$defName'] = produced;
    }
  }

  final produced = _emitNamed(em, schema, rootName, defNames);
  if (produced.type != rootName) {
    em.decls.add([
      ..._docComment(
          schema['description'] ?? '$rootName, this specification\'s payload.'),
      'typedef $rootName = ${produced.type};',
    ].join('\n'));
  }
  return (rootName: rootName, responseName: pending['Response']);
}

void _markDeprecated(_Emitter em, List<String> names, String note) {
  final annotation = '@Deprecated(${dartString(note)})';
  for (final name in names) {
    final decl = RegExp(
        '^(class|typedef|extension type const) ${RegExp.escape(name)}\\b');
    for (var i = 0; i < em.decls.length; i++) {
      final lines = em.decls[i].split('\n');
      final at = lines.indexWhere(decl.hasMatch);
      if (at < 0) continue;
      lines.insert(at, annotation);
      em.decls[i] = lines.join('\n');
    }
  }
}

/* ── File assembly ───────────────────────────────────────────────────────── */

const _nullableLint =
    '// ignore: unnecessary_nullable_for_final_variable_declarations';

List<String> _emitTail(Bundle b, bool hasResponse) {
  // The identity is the bundle's own Type URI, never rebuilt from the slug: a
  // third-party task (SPEC §6.5) lives under another authority, in any §6.1
  // form. For a registry bundle the two agree (the driver checks).
  final uri = b.typeUri;
  final out = <String>[
    [
      '/// Why this specification version is deprecated, or null when it is current.',
      '///',
      '/// Set when the version is retired, or superseded by a later version of the',
      '/// same slug at an equal or higher maturity (SPEC "Superseded and Withdrawn',
      '/// Versions"). A deprecated version is still supported; it is removed from',
      '/// this package once withdrawn.',
      if (b.deprecation != null) _nullableLint,
      'const String? deprecation = ${b.deprecation == null ? 'null' : dartString(b.deprecation!)};',
    ].join('\n'),
    [
      "/// The Trust Task type URI this library's [Payload] is carried under.",
      'const String typeUri = ${dartString(uri)};',
    ].join('\n'),
    [
      '/// The `MAJOR.MINOR` this version declares it extends (SPEC §5.2, "Declared',
      '/// extension"), or null when it declares none.',
      '///',
      '/// Only the direct declaration. A consumer implementing that version SHOULD',
      '/// accept a document of this one by ignoring the payload members it does not',
      '/// recognise; [readForVersion] does that, walking the whole chain through',
      '/// `package:trust_tasks/specs/extends_index.dart`. Named `extendsVersion`',
      '/// because `extends` is a reserved word.',
      if (b.extendsVersion != null) _nullableLint,
      'const String? extendsVersion = ${b.extendsVersion == null ? 'null' : dartString(b.extendsVersion!)};',
    ].join('\n'),
  ];

  if (hasResponse) {
    out.add([
      '/// The success-response form of [typeUri] (SPEC §4.4.1).',
      'const String responseTypeUri = ${dartString('$uri#response')};',
    ].join('\n'));
  }

  out.add([
    "/// This specification's payload schema, as JSON text.",
    '///',
    '/// SPEC §7.2 item 2 is performed against this. It ships with the library rather',
    '/// than only as a file under specs/ because a Dart class carries none of the',
    "/// schema's constraints — not minLength, not pattern, not the oneOf mutual",
    r'/// exclusion — so without it every such rule is unenforced. Cross-file \$refs are',
    '/// already inlined, so it needs no resolver.',
    'const String payloadSchemaJson = ${dartString(b.schemaText)};',
  ].join('\n'));
  if (b.responseSchemaText != null) {
    out.add([
      '/// As [payloadSchemaJson], for the success-response variant.',
      'const String responsePayloadSchemaJson = ${dartString(b.responseSchemaText!)};',
    ].join('\n'));
  }

  String block(String uriConst, String variant, String schemaConst) => [
        '  typeUri: $uriConst,',
        '  isBearer: ${b._isBearer},',
        '  isProofRequired: ${b._flag('proofRequired', variant)},',
        '  isRecipientRequired: ${b._flag('recipientRequired', variant)},',
        '  isIssuedAtRequired: ${b._flag('issuedAtRequired', variant)},',
        '  payloadSchema: $schemaConst,',
      ].join('\n');

  out.add([
    '/// The SPEC §7.2 policy for the request variant, taken from this',
    "/// specification's front matter.",
    '///',
    '/// Pass it to [consumeInbound] — items 5b, 7 and 8 are per-specification and',
    '/// cannot be derived from the document alone, and item 2 needs the schema it',
    '/// carries.',
    'const SpecPolicy spec = SpecPolicy(',
    block('typeUri', 'request', 'payloadSchemaJson'),
    ');',
  ].join('\n'));

  if (hasResponse) {
    out.add([
      '/// The SPEC §7.2 policy for the success-response variant.',
      '///',
      '/// `isRecipientRequired` tracks the *issuer* party\'s requirement, because a',
      '/// response swaps the parties (§7.3 item 5).',
      'const SpecPolicy responseSpec = SpecPolicy(',
      block(
        'responseTypeUri',
        'response',
        b.responseSchemaText != null ? 'responsePayloadSchemaJson' : 'null',
      ),
      ');',
    ].join('\n'));
  }
  return out;
}

/// The library path for [slug] at [version], relative to the output directory:
/// every segment snake_cased (`change-role` → `change_role`), the version as
/// `v0_1`, the file `payload.dart` — the layout of the prebuilt `lib/specs/`.
///
/// A task from a third-party [source] sits under a top-level directory named
/// for the source, snake_cased like a slug segment: `acme/payments/settle/v1_0`.
String libraryPathFor(String slug, String version, {String? source}) {
  final dirs = [if (source != null) source, ...slug.split('/')]
      .map((p) => dartFileName(p.startsWith('_') ? p.substring(1) : p));
  return [...dirs, 'v${version.replaceAll('.', '_')}', 'payload.dart']
      .join('/');
}

/// The unformatted source of the library for [bundle]; [source] names where
/// the bundle came from, for the header.
String generateLibrary(Bundle bundle, {required String source}) {
  // The .mjs emits declarations from the request schema, whose `$defs` hold
  // `Response`; it emits a response variant when that definition exists.
  final hasResponse = _obj(bundle.schema[r'$defs']).containsKey('Response');
  if (hasResponse != (bundle.responseSchemaText != null)) {
    throw BundleError(
      '`responseSchema` is ${bundle.responseSchemaText == null ? 'null' : 'present'} '
      'but the schema ${hasResponse ? 'defines' : 'does not define'} \$defs.Response',
    );
  }
  final em = _Emitter();
  final names = _emitDeclarations(em, bundle.schema, 'Payload');
  final deprecation = bundle.deprecation;
  if (deprecation != null) {
    _markDeprecated(
      em,
      [names.rootName, if (names.responseName != null) names.responseName!],
      deprecation,
    );
  }
  final tail = _emitTail(bundle, hasResponse);

  final header = <String>[
    '// Code generated by `dart run trust_tasks:generate`. DO NOT EDIT.',
    '// Source: $source',
    if (deprecation != null)
      '// ignore_for_file: deprecated_member_use_from_same_package',
    '',
    "import '$runtimeImport';",
  ];
  final body = [...em.decls, ...tail];
  return '${[...header, '', ...body.join('\n\n').split('\n')].join('\n')}\n';
}

/// The library mapping every locked bare request Type URI that declares
/// `extends` to the version it extends — pass it as `index:` to
/// `readForVersion` / `extensionChain` / `extendsTo`.
String generateExtendsIndex(List<(String, String)> entries) {
  final sorted = [...entries]..sort((a, b) => a.$1.compareTo(b.$1));
  return [
    '// Code generated by `dart run trust_tasks:generate`. DO NOT EDIT.',
    '// Source: the `extends` of every bundle in trust-tasks.lock.',
    '',
    '/// Every Trust Task version that declares `extends` (SPEC §5.2, "Declared',
    '/// extension"), keyed by its bare request Type URI, mapped to the `MAJOR.MINOR`',
    '/// it extends. A version that declares none is absent.',
    '///',
    "/// One link per entry; a chain is followed by looking the extended version's",
    '/// own Type URI up again, which `readForVersion` in the runtime does.',
    'library;',
    '',
    '/// See the library documentation.',
    'const Map<String, String> extendsIndex = <String, String>{',
    for (final (uri, v) in sorted) '  ${dartString(uri)}: ${dartString(v)},',
    '};',
    '',
  ].join('\n');
}
