// The shared RFC 3339 vectors in fixtures/consume/timestamps.json at the
// repository root, which the Rust, Go and TypeScript runtimes also run: every
// field of issuedAt / expiresAt must be in range. DateTime.tryParse rolls an
// out-of-range field over instead of refusing it.

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/src/runtime/rfc3339.dart';

void main() {
  final vectors = jsonDecode(
    File('../fixtures/consume/timestamps.json').readAsStringSync(),
  ) as Map<String, dynamic>;

  group('shared RFC 3339 vectors (fixtures/consume/timestamps.json)', () {
    for (final v in (vectors['valid'] as List).cast<Map<String, dynamic>>()) {
      test('reads ${v['value']} (${v['why']})', () {
        final at = parseRfc3339(v['value'] as String);
        expect(at, isNotNull);
        expect(at!.millisecondsSinceEpoch, v['epochMs']);
      });
    }
    for (final v in (vectors['invalid'] as List).cast<Map<String, dynamic>>()) {
      test('refuses ${v['value']} (${v['why']})', () {
        expect(parseRfc3339(v['value'] as String), isNull);
      });
    }
  });
}
