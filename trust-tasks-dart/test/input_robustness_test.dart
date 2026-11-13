// Received documents that are mistyped or nested too deeply are refused
// `malformedRequest` rather than thrown, and the item-11 digest is over the
// document as received — the shared vectors in fixtures/consume at the
// repository root, which the Rust, TypeScript and Go runtimes run too.

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

const me = 'did:web:maintainer.example';
const peer = 'did:web:org.example';
const spec = SpecPolicy(typeUri: 'https://trusttasks.org/spec/acl/list/0.1');

/// Models only `subject`, as the item-11 vectors require.
class Widget {
  const Widget(this.subject);
  factory Widget.fromJson(Object? json) =>
      Widget((json! as Map<String, dynamic>)['subject'] as String);
  final String subject;
  Map<String, dynamic> toJson() => <String, dynamic>{'subject': subject};
}

String fixedClock() => '2026-01-01T00:00:00Z';

Map<String, dynamic> base() => <String, dynamic>{
      'id': 'req-1',
      'type': spec.typeUri,
      'issuer': peer,
      'recipient': me,
      'issuedAt': '2026-01-01T00:00:00Z',
      'payload': <String, dynamic>{'subject': 'did:web:alice.example'},
    };

/// [depth] levels of lists, the outermost counting as one.
Object nestedLists(int depth) {
  Object v = <Object?>[];
  for (var i = 1; i < depth; i++) {
    v = <Object?>[v];
  }
  return v;
}

Future<ConsumeOutcome<Map<String, dynamic>>> consume(
  Map<String, dynamic> received, {
  ReplayGuard? guard,
  SpecPolicy policy = spec,
  String myVid = me,
  DateTime? now,
}) =>
    consumeInbound<Widget, Map<String, dynamic>>(
      transport: const UnauthenticatedTransport(),
      spec: policy,
      proofPolicy: const ProofPolicy.rejectIfPresent(),
      payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
      checks:
          guard == null ? notConsequentialChecks() : consequentialChecks(guard),
      doc: TrustTaskDocument<Widget>.fromJson(received, Widget.fromJson),
      received: received,
      myVid: myVid,
      now: now ?? DateTime.utc(2026, 1, 1, 0, 0, 30),
      newErrorId: () => 'err-1',
      clock: fixedClock,
      payloadToJson: (p) => p.toJson(),
      responseToJson: (r) => r,
      handler: (accepted, _) async => respondWith(
        accepted,
        'resp-1',
        <String, dynamic>{'ok': true},
        clock: fixedClock,
      ),
    );

void main() {
  group('parseTrustTaskDocument', () {
    final cases = <String, void Function(Map<String, dynamic>)>{
      'an array id': (d) => d['id'] = ['req-1'],
      'a numeric id': (d) => d['id'] = 1,
      'a missing id': (d) => d.remove('id'),
      'a non-string type': (d) => d['type'] = 7,
      'a missing payload': (d) => d.remove('payload'),
      'a list payload': (d) => d['payload'] = ['x'],
      'a non-string issuer': (d) => d['issuer'] = {'id': peer},
      'a non-string recipient': (d) => d['recipient'] = [me],
      'a non-string threadId': (d) => d['threadId'] = 3,
      'a non-string issuedAt': (d) => d['issuedAt'] = 1767225600000,
      'a non-object ceremony': (d) => d['ceremony'] = ['step'],
      'a ceremony missing its step': (d) => d['ceremony'] = {'enactment': 'e'},
      'a non-object proof': (d) => d['proof'] = 'z3kg',
      'a numeric @context': (d) => d['@context'] = 1,
      'a payload the payload type cannot read': (d) =>
          d['payload'] = <String, dynamic>{'subject': 42},
    };
    for (final MapEntry(key: name, value: mutate) in cases.entries) {
      test('refuses $name with malformedRequest, without throwing', () {
        final d = base();
        mutate(d);
        final parsed = parseTrustTaskDocument(d, Widget.fromJson);
        expect(parsed, isA<UnparsableDocument<Widget>>());
        expect(
          (parsed as UnparsableDocument<Widget>).reason.code,
          StandardCode.malformedRequest,
        );
      });
    }

    test('refuses a value that is not an object', () {
      for (final value in <Object?>[null, <Object?>[], 'doc', 3]) {
        expect(
          parseTrustTaskDocument(value, Widget.fromJson),
          isA<UnparsableDocument<Widget>>(),
        );
      }
    });

    test('refuses a document nested beyond maxNestingDepth', () {
      final d = base()
        ..['payload'] = <String, dynamic>{
          'subject': 's',
          'deep': nestedLists(100000),
        };
      expect(
        parseTrustTaskDocument(d, Widget.fromJson),
        isA<UnparsableDocument<Widget>>(),
      );
    });

    test('reads a well-formed document', () {
      final d = base()
        ..['threadId'] = 't-1'
        ..['@context'] = ['https://www.w3.org/ns/credentials/v2']
        ..['ceremony'] = {'enactment': 'e-1', 'step': 's-1'};
      final parsed = parseTrustTaskDocument(d, Widget.fromJson);
      expect(parsed, isA<ParsedDocument<Widget>>());
      expect(
        (parsed as ParsedDocument<Widget>).document.payload.subject,
        'did:web:alice.example',
      );
    });
  });

  group('nesting depth', () {
    test('exceedsNestingDepth counts the outermost value as one level', () {
      expect(exceedsNestingDepth(nestedLists(maxNestingDepth)), isFalse);
      expect(exceedsNestingDepth(nestedLists(maxNestingDepth + 1)), isTrue);
      expect(exceedsNestingDepth('scalar'), isFalse);
    });

    test('consumeInbound accepts $maxNestingDepth levels', () async {
      // document (1) → payload (2) → lists (3..max).
      final d = base()
        ..['payload'] = <String, dynamic>{
          'subject': 's',
          'deep': nestedLists(maxNestingDepth - 2),
        };
      expect(await consume(d), isA<Handled<Map<String, dynamic>>>());
    });

    test(
        'consumeInbound refuses a deeply nested document with malformedRequest '
        'instead of overflowing the stack', () async {
      for (final depth in [maxNestingDepth + 1, 100000]) {
        final d = base()
          ..['extension'] = nestedLists(depth - 1)
          ..['payload'] = <String, dynamic>{'subject': 's'};
        final outcome = await consume(d, guard: InMemoryReplayGuard());
        expect(outcome, isA<Rejected<Map<String, dynamic>>>(),
            reason: '$depth');
        expect(
          (outcome as Rejected<Map<String, dynamic>>).error.payload.code,
          'malformedRequest',
        );
      }
    });
  });

  group('nesting depth before the payload validator', () {
    test(
        'a deep payload is refused before a recursive validator walks it, '
        'and the message names no stack', () async {
      final d = base()
        ..['payload'] = <String, dynamic>{
          'subject': 's',
          'deep': nestedLists(100000),
        };
      var validated = false;
      final outcome =
          await consumeInbound<Map<String, dynamic>, Map<String, dynamic>>(
        transport: const UnauthenticatedTransport(),
        spec: const SpecPolicy(
          typeUri: 'https://trusttasks.org/spec/acl/list/0.1',
          payloadSchema: '{"type":"object"}',
        ),
        proofPolicy: const ProofPolicy.rejectIfPresent(),
        payloadPolicy: PayloadPolicy.validate(
          _RecursiveValidator(() => validated = true),
        ),
        checks: notConsequentialChecks(),
        doc: TrustTaskDocument<Map<String, dynamic>>.fromJson(
          d,
          (p) => p! as Map<String, dynamic>,
        ),
        received: d,
        myVid: me,
        now: DateTime.utc(2026, 1, 1, 0, 0, 30),
        newErrorId: () => 'err-1',
        clock: fixedClock,
        payloadToJson: (p) => p,
        responseToJson: (r) => r,
        handler: (accepted, _) async => null,
      );
      expect(validated, isFalse);
      expect(outcome, isA<Rejected<Map<String, dynamic>>>());
      final payload = (outcome as Rejected<Map<String, dynamic>>).error.payload;
      expect(payload.code, 'malformedRequest');
      expect(payload.message, isNot(contains('Stack')));
    });
  });

  group('canonical numbers', () {
    test('1, 1.0 and 1e0 are one number, as RFC 8785 reads them', () {
      String canon(String json) => canonicalJson(jsonDecode(json));
      for (final (spellings, expected) in [
        (['1', '1.0', '1e0', '10e-1'], '1'),
        (['100', '100.0', '1e2', '1E+2'], '100'),
        (['-0', '-0.0', '0'], '0'),
        (['-7', '-7.0'], '-7'),
        (['1.5', '15e-1'], '1.5'),
      ]) {
        for (final s in spellings) {
          expect(canon(s), expected, reason: s);
        }
      }
      expect(canon('1e20'), '100000000000000000000');
      expect(canon('1e21'), canon('1000000000000000000000'));
    });
  });

  group('shared item-11 vectors (fixtures/consume)', () {
    final vectors = jsonDecode(
      File('../fixtures/consume/item11-digest.json').readAsStringSync(),
    ) as Map<String, dynamic>;
    final policy = SpecPolicy(typeUri: vectors['spec'] as String);
    final now = DateTime.parse(vectors['now'] as String);

    Future<ConsumeOutcome<Map<String, dynamic>>> run(
      ReplayGuard guard,
      String body, {
      DateTime? at,
    }) =>
        consume(
          jsonDecode(body) as Map<String, dynamic>,
          guard: guard,
          policy: policy,
          myVid: vectors['consumer'] as String,
          now: at ?? now,
        );

    for (final c in (vectors['cases'] as List).cast<Map<String, dynamic>>()) {
      test(c['name'], () async {
        final guard = InMemoryReplayGuard();
        expect(
          await run(guard, (c['first'] ?? vectors['first']) as String),
          isA<Handled<Map<String, dynamic>>>(),
        );
        final secondAt = c['secondAt'] as String?;
        final second = await run(
          guard,
          c['second'] as String,
          at: secondAt == null ? null : DateTime.parse(secondAt),
        );
        final got = switch (second) {
          DuplicateOutcome() => 'duplicate',
          Rejected(:final error) => error.payload.code,
          _ => second.runtimeType.toString(),
        };
        expect(got, c['expect']);
      });
    }
  });
}

/// Stands in for a JSON Schema engine that walks the payload recursively.
class _RecursiveValidator implements PayloadValidator {
  _RecursiveValidator(this.onValidate);
  final void Function() onValidate;

  @override
  String? validate(String schemaJson, Object? payload) {
    onValidate();
    int depth(Object? v) => v is List
        ? 1 + v.fold<int>(0, (m, e) => depth(e) > m ? depth(e) : m)
        : v is Map
            ? 1 + v.values.fold<int>(0, (m, e) => depth(e) > m ? depth(e) : m)
            : 0;
    depth(payload);
    return null;
  }
}
