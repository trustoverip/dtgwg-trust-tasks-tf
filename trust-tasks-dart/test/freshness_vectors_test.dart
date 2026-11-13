// The shared acceptance-window vectors in fixtures/consume/freshness.json at
// the repository root, which the Rust, TypeScript and Go runtimes also run:
// item 4's expiry check, then the freshness bound, then the record expiry, so
// that the four runtimes reach the same verdict on the same document and keep
// its replay record for the same time (SPEC §7.2, *Bounding the record*).

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

Duration? _ms(Object? v) => v == null ? null : Duration(milliseconds: v as int);

FreshnessPolicy _policy(Map<String, dynamic> v) => FreshnessPolicy(
      skew: _ms(v['skewMs'])!,
      maxAge: _ms(v['maxAgeMs']),
      expiryHorizon: _ms(v['expiryHorizonMs']),
      requireIssuedAt: v['requireIssuedAt'] as bool? ?? false,
    );

/// The members a vector names, for comparing two policies.
List<Object?> _members(FreshnessPolicy p) =>
    [p.skew, p.maxAge, p.expiryHorizon, p.requireIssuedAt];

void main() {
  final vectors = jsonDecode(
    File('../fixtures/consume/freshness.json').readAsStringSync(),
  ) as Map<String, dynamic>;
  final policies = vectors['policies'] as Map<String, dynamic>;
  final cases = (vectors['cases'] as List).cast<Map<String, dynamic>>();

  group('shared freshness vectors (fixtures/consume/freshness.json)', () {
    test('default and consequential are the library\'s own policies', () {
      expect(
        _members(_policy(policies['default'] as Map<String, dynamic>)),
        _members(FreshnessPolicy.defaults),
      );
      expect(
        _members(_policy(policies['consequential'] as Map<String, dynamic>)),
        _members(FreshnessPolicy.consequential),
      );
    });

    test('there are cases', () => expect(cases, isNotEmpty));

    for (final c in cases) {
      test(c['name'] as String, () {
        final policy =
            _policy(policies[c['policy'] as String] as Map<String, dynamic>);
        final now = DateTime.parse(c['now'] as String);
        final doc = TrustTaskDocument<Object?>.fromJson({
          'id': 'req-1',
          'type': 'https://trusttasks.org/spec/acl/show/0.2',
          'payload': <String, dynamic>{},
          if (c.containsKey('issuedAt')) 'issuedAt': c['issuedAt'],
          if (c.containsKey('expiresAt')) 'expiresAt': c['expiresAt'],
        }, (p) => p);

        final reason =
            validateBasic(doc, now, '') ?? validateFreshness(doc, now, policy);
        expect(reason?.code.value ?? 'accept', c['verdict'], reason: '$reason');
        if (reason != null) return;

        final expected = c['recordExpiry'];
        expect(
          recordExpiry(doc, policy, now)?.toUtc(),
          expected == null ? isNull : DateTime.parse(expected as String),
          reason: 'record expiry',
        );
      });
    }
  });
}
