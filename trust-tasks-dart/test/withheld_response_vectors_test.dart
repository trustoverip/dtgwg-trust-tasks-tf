// A handler may keep its response out of the SPEC §7.2 item 11 record
// (`Delivery.current!.withholdResponse()`); a duplicate of that document then
// comes back to it as a re-delivery (`Delivery.isRedelivery`) instead of being
// answered from the record. Runs the shared vectors in
// `fixtures/consume/withheld-response.json`, which the Rust, TypeScript and Go
// runtimes run too.

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

class Widget {
  const Widget(this.subject);
  factory Widget.fromJson(Object? json) =>
      Widget((json! as Map<String, dynamic>)['subject'] as String);
  final String subject;
  Map<String, dynamic> toJson() => <String, dynamic>{'subject': subject};
}

String fixedClock() => '2026-11-01T00:00:00Z';

/// Hides [InMemoryReplayGuard.recordWithheld], as a guard written before
/// [WithholdingReplayGuard] existed would.
class _PlainGuard implements ReplayGuard {
  final _inner = InMemoryReplayGuard(16);

  @override
  Future<ReplayVerdict> claim(
    String id,
    String digest,
    DateTime? retainUntil,
    DateTime now,
  ) =>
      _inner.claim(id, digest, retainUntil, now);

  @override
  Future<void> recordResponse(String id, Map<String, dynamic>? response) =>
      _inner.recordResponse(id, response);

  @override
  Future<void> release(String id, String digest) => _inner.release(id, digest);
}

void main() {
  final vectors = jsonDecode(
    File('../fixtures/consume/withheld-response.json').readAsStringSync(),
  ) as Map<String, dynamic>;
  final policy = SpecPolicy(typeUri: vectors['spec'] as String);
  final now = DateTime.parse(vectors['now'] as String);
  final documents = (vectors['documents'] as Map).cast<String, String>();

  /// One delivery; `seen` is the signal the handler was called with, null when
  /// it was not called.
  Future<(ConsumeOutcome<Map<String, dynamic>>, String?)> consume(
    ReplayGuard guard,
    String body,
    int index,
    bool withhold,
  ) async {
    final received = jsonDecode(body) as Map<String, dynamic>;
    String? seen;
    final outcome = await consumeInbound<Widget, Map<String, dynamic>>(
      transport: const UnauthenticatedTransport(),
      spec: policy,
      proofPolicy: const ProofPolicy.rejectIfPresent(),
      payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
      checks: consequentialChecks(guard),
      doc: TrustTaskDocument<Widget>.fromJson(received, Widget.fromJson),
      received: received,
      myVid: vectors['consumer'] as String,
      now: now,
      newErrorId: () => 'err-1',
      clock: fixedClock,
      payloadToJson: (p) => p.toJson(),
      responseToJson: (r) => r,
      handler: (accepted, _) async {
        await Future<void>.delayed(Duration.zero);
        final delivery = Delivery.current!;
        seen = delivery.isRedelivery ? 'redelivery' : 'first';
        if (withhold) delivery.withholdResponse();
        return respondWith(
          accepted,
          'resp-$index',
          <String, dynamic>{'delivery': seen},
          clock: fixedClock,
        );
      },
    );
    return (outcome, seen);
  }

  group('withheld-response vectors (fixtures/consume)', () {
    for (final c in (vectors['cases'] as List).cast<Map<String, dynamic>>()) {
      test(c['name'], () async {
        final guard = InMemoryReplayGuard(16);
        final responses = <String?>[];
        final deliveries =
            (c['deliveries'] as List).cast<Map<String, dynamic>>();
        for (var index = 0; index < deliveries.length; index++) {
          final d = deliveries[index];
          final (outcome, seen) = await consume(
            guard,
            documents[d['document'] as String]!,
            index,
            c['withhold'] as bool,
          );
          final got = switch (outcome) {
            Handled() => 'handled',
            DuplicateOutcome() => 'duplicate',
            Rejected(:final error) => error.payload.code,
            _ => outcome.runtimeType.toString(),
          };
          expect(got, d['outcome'], reason: '#$index');
          expect(seen, d['handler'], reason: '#$index: handler');
          switch (outcome) {
            case Handled(:final response):
              expect(response.payload['delivery'], d['handler']);
              responses.add(response.id);
            case DuplicateOutcome(:final priorResponse, :final inFlight):
              expect(inFlight, isFalse);
              expect(
                priorResponse?['id'],
                responses[d['priorResponseOf'] as int],
                reason: '#$index: replays',
              );
              responses.add(null);
            default:
              responses.add(null);
          }
        }
      });
    }

    test('a guard without recordWithheld absorbs the duplicate in silence',
        () async {
      final guard = _PlainGuard();
      final body = documents['original']!;
      expect((await consume(guard, body, 0, true)).$1,
          isA<Handled<Map<String, dynamic>>>());
      final (again, seen) = await consume(guard, body, 1, true);
      expect(again, isA<DuplicateOutcome<Map<String, dynamic>>>());
      final duplicate = again as DuplicateOutcome<Map<String, dynamic>>;
      expect(duplicate.priorResponse, isNull);
      expect(duplicate.inFlight, isFalse);
      expect(seen, isNull);
    });

    test('there is no delivery outside consumeInbound', () {
      expect(Delivery.current, isNull);
    });
  });
}
