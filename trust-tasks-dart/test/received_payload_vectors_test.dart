// The shared vectors in fixtures/consume/received-payload.json at the
// repository root, which the Rust, TypeScript and Go runtimes also run: with
// `received` passed, consumeInbound validates the payload as received — where
// the typed decode reads a forbidden null as absent — and a later version read
// forward along `extends` pruned, as readForVersion reads it (SPEC §7.2 item 2).

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/specs/acl/update/v0_3/payload.dart' as acl_update;
import 'package:trust_tasks/trust_tasks.dart';

/// The implemented version as a generated type reads it: an optional `role`
/// that is null whether absent or `null`, and written only when non-null —
/// the shape of acl/update 0.3's `purposes`.
class Grant {
  const Grant(this.subject, this.role);
  final String subject;
  final String? role;

  static Grant fromJson(Object? json) {
    final m = json as Map<String, dynamic>;
    return Grant(m['subject'] as String, m['role'] as String?);
  }

  Map<String, dynamic> toJson() =>
      {'subject': subject, if (role != null) 'role': role};
}

/// Records the payload it is handed and checks the vector schema by hand: an
/// object, `subject` present, every member named in `properties`, every member
/// a string.
class _Recording implements PayloadValidator {
  Object? seen;
  @override
  String? validate(String schemaJson, Object? payload) {
    seen = payload;
    if (payload is! Map) return 'not an object';
    if (!payload.containsKey('subject')) return 'missing subject';
    for (final entry in payload.entries) {
      if (entry.key != 'subject' && entry.key != 'role') {
        return 'unknown member ${entry.key}';
      }
      if (entry.value is! String) return '${entry.key} is not a string';
    }
    return null;
  }
}

String _clock() => '2026-08-01T12:00:30Z';

void main() {
  final vectors = jsonDecode(
    File('../fixtures/consume/received-payload.json').readAsStringSync(),
  ) as Map<String, dynamic>;
  final implemented = vectors['implements'] as Map<String, dynamic>;
  final spec = SpecPolicy(
    typeUri: implemented['typeUri'] as String,
    payloadSchema: implemented['payloadSchema'] as String,
  );
  final index =
      (vectors['extends'] as Map<String, dynamic>).cast<String, String>();
  final consumer = vectors['consumer'] as String;
  final now = DateTime.parse(vectors['now'] as String);
  final cases = (vectors['cases'] as List).cast<Map<String, dynamic>>();

  /// Reads [document] as the implemented version and consumes it, with or
  /// without `received`; the verdict and the payload validated.
  Future<(String, Object?)> consume(String document,
      {bool passReceived = true}) async {
    final received = jsonDecode(document) as Map<String, dynamic>;
    final read =
        readForVersion<Grant>(received, spec, Grant.fromJson, index: index);
    if (read is! VersionAccepted<Grant>) fail('readForVersion refused');
    final validator = _Recording();
    final outcome = await consumeInbound<Grant, Object?>(
      transport: const UnauthenticatedTransport(),
      spec: spec,
      proofPolicy: const ProofPolicy.acceptUnverified(),
      payloadPolicy: PayloadPolicy.validate(validator),
      checks: notConsequentialChecks(),
      doc: read.doc,
      myVid: consumer,
      now: now,
      newErrorId: () => 'urn:uuid:error',
      payloadToJson: (p) => p.toJson(),
      responseToJson: (r) => r,
      clock: _clock,
      received: passReceived ? read.received : null,
      extendsIndex: index,
      handler: (accepted, parties) async => respondWith<Grant, Object?>(
          accepted, 'urn:uuid:answer', const <String, dynamic>{},
          clock: _clock),
    );
    final verdict = switch (outcome) {
      Handled<Object?>() => 'accept',
      Rejected<Object?>(:final error) => error.payload.code,
      _ => outcome.runtimeType.toString(),
    };
    return (verdict, validator.seen);
  }

  group(
      'shared received-payload vectors (fixtures/consume/received-payload.json)',
      () {
    test('has cases', () => expect(cases, isNotEmpty));
    for (final c in cases) {
      test(c['name'] as String, () async {
        final (verdict, validated) = await consume(c['document'] as String);
        expect(verdict, c['verdict']);
        expect(validated, equals(c['validated']));
      });
    }
  });

  test('without received, only the re-encoding is validated', () async {
    // The forbidden null is gone from the typed re-encoding, so it passes.
    const document = '{"id":"urn:uuid:7b0c2f1e-0000-4000-8000-0000000000a1",'
        '"type":"https://example.com/spec/vector/grant/0.1",'
        '"issuer":"did:example:alice","recipient":"did:example:bob",'
        '"issuedAt":"2026-08-01T12:00:00Z",'
        '"payload":{"subject":"did:example:carol","role":null}}';
    expect((await consume(document)).$1, 'malformedRequest');
    final (verdict, validated) = await consume(document, passReceived: false);
    expect(verdict, 'accept');
    expect(validated, equals({'subject': 'did:example:carol'}));
  });

  test('acl/update 0.3 "purposes": null is refused when received is passed',
      () async {
    // `purposes` has no null form. The generated type reads a null as absent
    // and `toJson` omits it; the received payload still carries it. The
    // documents carry no proof, which acl/update requires, so the one without
    // the null clears item 2 and is refused proofRequired.
    Future<String> run(Map<String, dynamic> payload) async {
      final received = <String, dynamic>{
        'id': 'urn:uuid:5f0c2a4e-8d1b-4c3a-9e2f-00000000a1c1',
        'type': acl_update.spec.typeUri,
        'issuer': 'did:web:producer.example',
        'recipient': 'did:web:consumer.example',
        'issuedAt': '2026-11-01T00:00:00Z',
        'payload': payload,
      };
      final doc = TrustTaskDocument<acl_update.Payload>.fromJson(received,
          (p) => acl_update.Payload.fromJson(p as Map<String, dynamic>));
      expect(doc.payload.toJson().containsKey('purposes'), isFalse);
      final outcome = await consumeInbound<acl_update.Payload, Object?>(
        transport: const UnauthenticatedTransport(),
        spec: acl_update.spec,
        proofPolicy: const ProofPolicy.acceptUnverified(),
        payloadPolicy: const PayloadPolicy.validate(_PurposesNull()),
        checks: notConsequentialChecks(),
        doc: doc,
        myVid: 'did:web:consumer.example',
        now: DateTime.utc(2026, 11, 1, 0, 0, 30),
        payloadToJson: (p) => p.toJson(),
        responseToJson: (r) => r,
        received: received,
        handler: (accepted, parties) async => null,
      );
      return switch (outcome) {
        Rejected<Object?>(:final error) => error.payload.code,
        _ => outcome.runtimeType.toString(),
      };
    }

    expect(await run({'subject': 'did:example:alice', 'purposes': null}),
        'malformedRequest');
    expect(
        await run({'subject': 'did:example:alice'}), isNot('malformedRequest'));
  });
}

/// Stands in for a JSON Schema engine on acl/update 0.3, whose `purposes` has
/// no null form: refuses a null `purposes`.
class _PurposesNull implements PayloadValidator {
  const _PurposesNull();
  @override
  String? validate(String schemaJson, Object? payload) => payload is Map &&
          payload.containsKey('purposes') &&
          payload['purposes'] == null
      ? 'purposes: null is not an object'
      : null;
}
