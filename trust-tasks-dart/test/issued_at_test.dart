// #970: every instant this library stamps is written in whole seconds, and
// every RFC 3339 precision is still read. Mirrors trust-tasks-rs's
// tests/issued_at_precision.rs, trust-tasks-ts's test/issued-at.test.ts and
// trust-tasks-go's trusttasks/issued_at_test.go.
//
// SPEC §13.1 item 3 calls a full-precision timestamp a fingerprint and lets a
// producer round issuedAt down. Dart has always stamped whole seconds; this
// pins it, so the four libraries cannot drift apart again.

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

final _wholeSeconds = RegExp(r'^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$');

TrustTaskDocument<Map<String, Object?>> _request(String issuedAt) =>
    TrustTaskDocument(
      id: '5b0e8f3a-91c2-8d4e-a6f0-2c7d93b1e4a5',
      type: 'https://real-life.org/trust-tasks/spec/sealed-envelope/0.1',
      issuer: 'did:key:z6MkProducer',
      recipient: 'did:key:z6MkConsumer',
      issuedAt: issuedAt,
      payload: const <String, Object?>{},
    );

void main() {
  group('#970 — instants this library writes', () {
    test('systemClock, rejectWith and respondWith stamp whole seconds', () {
      expect(systemClock(), matches(_wholeSeconds));
      final request = _request('2026-10-27T18:04:11Z');
      final error = rejectWith(
        request,
        'urn:uuid:2',
        const ErrorPayload(code: 'taskFailed', retryable: false),
      );
      expect(error.issuedAt, matches(_wholeSeconds));
      final response = respondWith(request, 'urn:uuid:3', <String, Object?>{});
      expect(response.issuedAt, matches(_wholeSeconds));
    });
  });

  group('#970 — instants this library reads', () {
    test('takes every RFC 3339 precision, and an offset', () {
      final now = DateTime.utc(2026, 10, 27, 18, 4, 12);
      for (final issuedAt in [
        '2026-10-27T18:04:11Z',
        '2026-10-27T18:04:11.5Z',
        '2026-10-27T18:04:11.720Z',
        '2026-10-27T18:04:11.720263Z',
        '2026-10-27T18:04:11.720263396Z',
        '2026-10-27T20:04:11.720+02:00',
      ]) {
        expect(
          validateFreshness(
            _request(issuedAt),
            now,
            FreshnessPolicy.consequential,
          ),
          isNull,
          reason: issuedAt,
        );
      }
    });
  });
}
