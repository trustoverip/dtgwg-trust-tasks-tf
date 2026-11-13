// SPEC §4.5.2: a namespace marked critical is understood or refused. Runs the
// shared vectors in fixtures/consume/ext-critical.json at the repository root,
// which the Rust, TypeScript and Go runtimes also run, and drives the marking
// through consumeInbound.

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

const _marked = 'https://trusttasks.org/spec/conformance/marked/0.1';

/// Models only `subject`, so the typed reading drops the marking.
class _Subject {
  const _Subject(this.subject);
  final String subject;
  Map<String, dynamic> toJson() => {'subject': subject};
}

void main() {
  final schema =
      File('../fixtures/consume/ext-critical.schema.json').readAsStringSync();
  final vectors = jsonDecode(
    File('../fixtures/consume/ext-critical.json').readAsStringSync(),
  ) as Map<String, dynamic>;
  final cases = (vectors['cases'] as List).cast<Map<String, dynamic>>();

  group('shared SPEC §4.5.2 vectors (fixtures/consume/ext-critical.json)', () {
    test('there are cases', () => expect(cases, isNotEmpty));
    for (final c in cases) {
      test(c['name'] as String, () {
        final reason = checkCriticalExtensions(
          c['payload'],
          c['schemaless'] == true ? null : schema,
          (c['recognized'] as List).cast<String>(),
        );
        expect(reason == null ? 'ok' : reason.code.toString(), c['expect']);
      });
    }
  });

  test(
      'an unrecognized critical namespace is refused before the handler '
      'and the record', () async {
    final now = DateTime.now().toUtc();
    final received = <String, dynamic>{
      'id': 'urn:uuid:00000000-0000-4000-8000-0000000000c1',
      'type': _marked,
      'issuedAt': '${now.toIso8601String().split('.').first}Z',
      'payload': {
        'subject': 's',
        'ext': {'org.example.a': 1},
        'extCritical': ['org.example.a'],
      },
    };
    final guard = InMemoryReplayGuard();
    var runs = 0;

    Future<ConsumeOutcome<Map<String, dynamic>>> consume(
      List<String> recognized,
    ) =>
        consumeInbound<_Subject, Map<String, dynamic>>(
          transport: const UnauthenticatedTransport(),
          spec: SpecPolicy(typeUri: _marked, payloadSchema: schema),
          proofPolicy: const ProofPolicy.acceptUnverified(),
          payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
          checks:
              consequentialChecks(guard).withRecognizedExtensions(recognized),
          doc: TrustTaskDocument<_Subject>.fromJson(
            received,
            (p) => _Subject((p as Map)['subject'] as String),
          ),
          received: received,
          myVid: 'did:web:consumer.example',
          now: now,
          newErrorId: () => 'err-1',
          payloadToJson: (p) => p.toJson(),
          responseToJson: (r) => r,
          handler: (accepted, parties) async {
            runs += 1;
            return respondWith<_Subject, Map<String, dynamic>>(
                accepted, 'resp-1', {
              'ok': true,
            });
          },
        );

    final refused = await consume(const []);
    expect(refused, isA<Rejected<Map<String, dynamic>>>());
    final error = (refused as Rejected<Map<String, dynamic>>).error;
    expect(error.payload.code.toString(), 'unsupportedExtension');
    expect(error.payload.retryable, isFalse);
    expect(runs, 0, reason: 'the handler must not run');

    // The refusal wrote no record: the same document, consumed by a consumer
    // that implements the namespace, runs.
    final handled = await consume(const ['org.example.a']);
    expect(handled, isA<Handled<Map<String, dynamic>>>());
    expect(runs, 1);
  });
}
