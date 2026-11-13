// A stand-in for a JSON Schema engine, so this example runs with no dependency
// beyond the Trust Tasks packages.
//
// It understands the subset of JSON Schema 2020-12 the generated payload
// schemas use for their structure — local `$ref`s into `$defs`, `type`,
// `required`, `properties`, `additionalProperties: false`, `const`, `enum`,
// `oneOf`, `items`, `minItems`, `minLength` and `maxLength` — and ignores the
// rest (`pattern`, `format`, `uniqueItems`, …). That is enough to refuse a
// payload with a missing, misspelt or mistyped member.
//
// A real application puts a full Draft 2020-12 engine behind PayloadValidator
// instead — package:json_schema, for example. GETTING-STARTED.md §1, "What you
// bring yourself", says why the runtime bundles none.

import 'dart:convert';

import 'package:trust_tasks/trust_tasks.dart';

typedef _Schema = Map<String, dynamic>;

/// Checks a payload against the generated `payloadSchema` (the subset above).
final class StructuralValidator implements PayloadValidator {
  const StructuralValidator();

  @override
  String? validate(String schemaJson, Object? payload) {
    final schema = jsonDecode(schemaJson) as _Schema;
    final problems = <String>[];
    _check(schema, payload, 'payload', schema, problems);
    return problems.isEmpty ? null : problems.join('; ');
  }
}

bool _typeMatches(Object? type, Object? value) => switch (type) {
      'object' => value is Map,
      'array' => value is List,
      'string' => value is String,
      'boolean' => value is bool,
      'integer' =>
        value is int || (value is double && value == value.truncate()),
      'number' => value is num,
      _ => true,
    };

bool _same(Object? a, Object? b) => jsonEncode(a) == jsonEncode(b);

void _check(
  _Schema schema,
  Object? value,
  String at,
  _Schema root,
  List<String> problems,
) {
  final ref = schema[r'$ref'];
  if (ref is String) {
    final defs = root[r'$defs'];
    final target = ref.startsWith(r'#/$defs/') && defs is Map
        ? defs[ref.substring(r'#/$defs/'.length)]
        : null;
    if (target is! _Schema) {
      problems.add('$at: unsupported \$ref $ref');
      return;
    }
    _check(target, value, at, root, problems);
  }
  final type = schema['type'];
  if (type != null && !_typeMatches(type, value)) {
    problems.add('$at: expected $type');
    return;
  }
  if (schema.containsKey('const') && !_same(schema['const'], value)) {
    problems.add('$at: expected ${jsonEncode(schema['const'])}');
  }
  final enumValues = schema['enum'];
  if (enumValues is List && !enumValues.any((e) => _same(e, value))) {
    problems.add('$at: not one of ${jsonEncode(enumValues)}');
  }
  final oneOf = schema['oneOf'];
  if (oneOf is List) {
    final matching = oneOf.whereType<_Schema>().where((branch) {
      final branchProblems = <String>[];
      _check(branch, value, at, root, branchProblems);
      return branchProblems.isEmpty;
    }).length;
    if (matching != 1) {
      problems
          .add('$at: matches $matching of the oneOf branches, not exactly one');
    }
  }
  if (value is String) {
    final length =
        value.runes.length; // code points, as JSON Schema counts them
    final min = schema['minLength'], max = schema['maxLength'];
    if (min is num && length < min) problems.add('$at: too short');
    if (max is num && length > max) problems.add('$at: too long');
  } else if (value is List) {
    final min = schema['minItems'], items = schema['items'];
    if (min is num && value.length < min) problems.add('$at: too few items');
    if (items is _Schema) {
      for (var i = 0; i < value.length; i++) {
        _check(items, value[i], '$at[$i]', root, problems);
      }
    }
  } else if (value is Map) {
    final properties = schema['properties'];
    final required = schema['required'];
    if (required is List) {
      for (final member in required) {
        if (!value.containsKey(member)) {
          problems.add('$at: missing required member $member');
        }
      }
    }
    for (final MapEntry(:key, value: memberValue) in value.entries) {
      final sub = properties is Map ? properties[key] : null;
      if (sub is _Schema) {
        _check(sub, memberValue, '$at.$key', root, problems);
      } else if (schema['additionalProperties'] == false) {
        problems.add('$at: unknown member $key');
      }
    }
  }
}
