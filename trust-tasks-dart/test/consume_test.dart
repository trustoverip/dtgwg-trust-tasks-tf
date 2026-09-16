// SPEC.md §7.2 conformance tests for the Dart consumer pipeline.
//
// These deliberately mirror the test sets in trust-tasks-rs/src/consume.rs,
// trust-tasks-ts/test/consume.test.ts and trust-tasks-go/trusttasks. The four
// reference implementations must reach the same verdict on the same document;
// where a case exists there and not here, the languages can drift apart without
// anything noticing.

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

const me = 'did:web:maintainer.example';
const peer = 'did:web:org.example';

/// Stands in for a generated library's `spec`.
const requiredSpec = SpecPolicy(
  typeUri: 'https://trusttasks.org/spec/acl/grant/0.1',
  isProofRequired: true,
  isRecipientRequired: true,
);

/// A spec declaring `issuedAtRequirement: REQUIRED` (SPEC §7.3 item 17).
const freshSpec = SpecPolicy(
  typeUri: 'https://trusttasks.org/spec/acl/grant/0.1',
  isRecipientRequired: true,
  isIssuedAtRequired: true,
);

/// A spec that only RECOMMENDS a proof and does not require a recipient.
const relaxedSpec = SpecPolicy(
  typeUri: 'https://trusttasks.org/spec/acl/list/0.1',
);

class Payload {
  const Payload(this.role);
  final String role;
  Map<String, dynamic> toJson() => <String, dynamic>{'role': role};
}

class Response {
  const Response(this.ok);
  final bool ok;
  Map<String, dynamic> toJson() => <String, dynamic>{'ok': ok};
}

const testProof = Proof(
  type: 'DataIntegrityProof',
  cryptosuite: 'eddsa-rdfc-2022',
  verificationMethod: '$peer#key-1',
  created: '2026-01-01T00:00:00Z',
  proofPurpose: 'assertionMethod',
  proofValue: 'z3kg',
);

final testNow = DateTime.utc(2026, 1, 1);
String fixedClock() => '2026-01-01T00:00:00Z';
Object? payloadToJson(Payload p) => p.toJson();

TrustTaskDocument<Payload> doc({
  String? recipient = me,
  String? issuer = peer,
  String? issuedAt,
  String? expiresAt,
  String? parentThreadId,
  Proof? proof,
  Payload payload = const Payload('admin'),
}) =>
    TrustTaskDocument<Payload>(
      id: 'req-1',
      type: requiredSpec.typeUri,
      issuer: issuer,
      recipient: recipient,
      issuedAt: issuedAt,
      expiresAt: expiresAt,
      parentThreadId: parentThreadId,
      proof: proof,
      payload: payload,
    );

class _Verifier implements ProofVerifier {
  const _Verifier(this.result);
  final bool result;
  @override
  Future<bool> verify(Map<String, dynamic> doc) async => result;
}

class _ThrowingVerifier implements ProofVerifier {
  const _ThrowingVerifier();
  @override
  Future<bool> verify(Map<String, dynamic> doc) async =>
      throw StateError('resolver could not reach $peer');
}

/// Stands in for a real JSON Schema engine: refuses any payload missing a member
/// the schema's `required` names. Deliberately not a real engine — these tests
/// assert that the pipeline consults the validator and routes its verdict, not
/// that somebody else's validator is correct.
class _RequiredMembers implements PayloadValidator {
  const _RequiredMembers();
  @override
  String? validate(String schemaJson, Object? payload) {
    final required = RegExp(r'"required":\[(.*?)\]').firstMatch(schemaJson);
    if (required == null) return null;
    final names = RegExp(r'"(\w+)"')
        .allMatches(required.group(1)!)
        .map((m) => m.group(1)!)
        .toList();
    final members = (payload as Map<String, dynamic>? ?? <String, dynamic>{});
    final missing = names.where((n) => !members.containsKey(n)).toList();
    return missing.isEmpty ? null : missing.map((n) => 'missing $n').join('; ');
  }
}

class _ExplodingValidator implements PayloadValidator {
  const _ExplodingValidator();
  @override
  String? validate(String schemaJson, Object? payload) =>
      throw StateError('validator exploded');
}

Future<ConsumeOutcome<Response>> run(
  TrustTaskDocument<Payload> document, {
  SpecPolicy spec = requiredSpec,
  ProofPolicy proofPolicy = const ProofPolicy.verify(_Verifier(true)),
  PayloadPolicy payloadPolicy = const PayloadPolicy.acceptUnvalidated(),
  ConsumeChecks? checks,
  TransportHandler transport = const UnauthenticatedTransport(),
  bool handlerShouldNotRun = false,
  bool handlerReturnsNull = false,
  RejectReason? handlerRefuses,
}) =>
    consumeInbound<Payload, Response>(
      transport: transport,
      spec: spec,
      proofPolicy: proofPolicy,
      payloadPolicy: payloadPolicy,
      checks: checks ?? notConsequentialChecks(),
      doc: document,
      myVid: me,
      now: testNow,
      newErrorId: () => 'err-1',
      payloadToJson: payloadToJson,
      clock: fixedClock,
      handler: (accepted, parties) async {
        if (handlerShouldNotRun) fail('handler must not run');
        if (handlerRefuses != null) {
          throw refuse(accepted, 'err-1', handlerRefuses, clock: fixedClock);
        }
        if (handlerReturnsNull) return null;
        return respondWith<Payload, Response>(
          accepted,
          'resp-1',
          const Response(true),
          clock: fixedClock,
        );
      },
    );

String rejectedCode(ConsumeOutcome<Response> outcome) {
  expect(outcome, isA<Rejected<Response>>(), reason: 'expected a rejection');
  return (outcome as Rejected<Response>).error.payload.code;
}

void main() {
  group('§7.2 pipeline', () {
    test('runs the handler when every check passes', () async {
      final outcome = await run(doc(proof: testProof));
      expect(outcome, isA<Handled<Response>>());
      final response = (outcome as Handled<Response>).response;
      expect(response.id, 'resp-1');
      // §4.4.1 — the response carries the #response fragment...
      expect(response.type, '${requiredSpec.typeUri}#response');
      // ...the parties swap...
      expect(response.issuer, me);
      expect(response.recipient, peer);
      // ...and §4.9 continues the thread from the request's id.
      expect(response.threadId, 'req-1');
    });

    test('item 4 — rejects an expired document', () async {
      final outcome = await run(
        doc(proof: testProof, expiresAt: '2025-01-01T00:00:00Z'),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'expired');
    });

    test('item 4 — treats the expiry instant itself as expired', () async {
      final outcome = await run(
        doc(
          proof: testProof,
          issuedAt: '2025-12-31T23:00:00Z',
          expiresAt: '2026-01-01T00:00:00Z',
        ),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'expired');
    });

    test('item 4 — rejects a malformed expiresAt', () async {
      final outcome = await run(
        doc(proof: testProof, expiresAt: 'not-a-timestamp'),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'malformedRequest');
    });

    test('item 5a — wrong recipient routes to the original issuer', () async {
      final outcome = await run(
        doc(proof: testProof, recipient: 'did:web:someone-else.example'),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'wrongRecipient');
      expect((outcome as Rejected<Response>).error.recipient, peer);
    });

    test('item 5b — recipient REQUIRED but absent in-band', () async {
      final outcome = await run(
        doc(proof: testProof, recipient: null),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'malformedRequest');
    });

    test('item 7 — proof REQUIRED by the spec but absent', () async {
      final outcome = await run(doc(), handlerShouldNotRun: true);
      expect(rejectedCode(outcome), 'proofRequired');
    });

    test('item 7 — a RECOMMENDED spec accepts a proofless document', () async {
      final outcome = await run(doc(recipient: null), spec: relaxedSpec);
      expect(outcome, isA<Handled<Response>>());
    });

    test('§7.3 item 17 — issuedAt REQUIRED but absent', () async {
      final outcome =
          await run(doc(), spec: freshSpec, handlerShouldNotRun: true);
      expect(rejectedCode(outcome), 'malformedRequest');
      expect(
        (outcome as Rejected<Response>).error.payload.message,
        contains('§7.3 item 17'),
      );
    });

    test('§7.3 item 17 — passes once the document carries issuedAt', () async {
      final outcome =
          await run(doc(issuedAt: '2026-01-01T00:00:00Z'), spec: freshSpec);
      expect(outcome, isA<Handled<Response>>());
    });

    test('item 7 — a failing verifier maps to proofInvalid', () async {
      final outcome = await run(
        doc(proof: testProof),
        proofPolicy: const ProofPolicy.verify(_Verifier(false)),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'proofInvalid');
    });

    test('item 7 — a verifier that throws is a failure, not a crash', () async {
      final outcome = await run(
        doc(proof: testProof),
        proofPolicy: const ProofPolicy.verify(_ThrowingVerifier()),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'proofInvalid');
      // §10.4: the verifier's own text must not reach the wire.
      final message = (outcome as Rejected<Response>).error.payload.message!;
      expect(message, proofInvalidWireMessage);
      expect(message, isNot(contains('resolver')));
    });

    test('item 7 — rejectIfPresent does not leak configuration', () async {
      final outcome = await run(
        doc(proof: testProof),
        proofPolicy: const ProofPolicy.rejectIfPresent(),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'malformedRequest');
      expect(
        (outcome as Rejected<Response>).error.payload.message,
        proofNotAcceptedByPolicy,
      );
    });

    test('item 7 — acceptUnverified passes a proof-bearing document', () async {
      final outcome = await run(
        doc(proof: testProof),
        proofPolicy: const ProofPolicy.acceptUnverified(),
      );
      expect(outcome, isA<Handled<Response>>());
    });

    test('item 8 — proof with no recipient on a non-bearer spec', () async {
      final outcome = await run(
        doc(recipient: null, proof: testProof),
        spec: relaxedSpec,
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'malformedRequest');
      expect(
        (outcome as Rejected<Response>).error.payload.message,
        contains('audience binding'),
      );
    });

    test('item 8 — a bearer spec is exempt from audience binding', () async {
      const bearer = SpecPolicy(
        typeUri: 'https://trusttasks.org/spec/acl/list/0.1',
        isBearer: true,
      );
      final outcome =
          await run(doc(recipient: null, proof: testProof), spec: bearer);
      expect(outcome, isA<Handled<Response>>());
    });
  });

  group('§4.8.1 party resolution', () {
    test('fills absent in-band members from the transport', () async {
      ResolvedParties? seen;
      await consumeInbound<Payload, Response>(
        transport: const StaticTransport(
          TransportContext(issuer: peer, recipient: me),
        ),
        spec: relaxedSpec,
        proofPolicy: const ProofPolicy.acceptUnverified(),
        payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
        checks: notConsequentialChecks(),
        doc: doc(issuer: null, recipient: null),
        myVid: me,
        now: testNow,
        newErrorId: () => 'err-1',
        payloadToJson: payloadToJson,
        handler: (accepted, parties) async {
          seen = parties;
          return respondWith<Payload, Response>(
            accepted,
            'resp-1',
            const Response(true),
            clock: fixedClock,
          );
        },
      );
      expect(seen!.issuer, peer);
      expect(seen!.recipient, me);
    });

    test('item 6 — in-band and transport disagreeing is identityMismatch',
        () async {
      final outcome = await run(
        doc(proof: testProof),
        transport: const StaticTransport(
          TransportContext(issuer: 'did:web:impostor.example', recipient: me),
        ),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'identityMismatch');
      final error = (outcome as Rejected<Response>).error;
      // §8.1: addressed to the transport-authenticated sender.
      expect(error.recipient, 'did:web:impostor.example');
      // §8.1 / §10.4: the wire message names neither value.
      expect(error.payload.message, isNot(contains('impostor')));
      expect(error.payload.message, isNot(contains(peer)));
      // §8.2: the originating id is withheld under identityMismatch.
      expect(error.payload.inResponseTo!.id, isNull);
      expect(error.payload.inResponseTo!.typeUri, requiredSpec.typeUri);
    });

    test('§8.1 — identityMismatch with no transport sender is suppressed',
        () async {
      final outcome = await run(
        doc(proof: testProof),
        transport: const StaticTransport(
          TransportContext(recipient: 'did:web:other.example'),
        ),
        handlerShouldNotRun: true,
      );
      expect(outcome, isA<Suppressed<Response>>());
      expect(
        (outcome as Suppressed<Response>).reason.code,
        StandardCode.identityMismatch,
      );
    });
  });

  group('§8.2 inResponseTo and §4.9.2 parentThreadId', () {
    test('names the document the error reports on', () async {
      final outcome = await run(doc(proof: testProof, recipient: null),
          handlerShouldNotRun: true);
      final about = (outcome as Rejected<Response>).error.payload.inResponseTo!;
      expect(about.typeUri, requiredSpec.typeUri);
      expect(about.id, 'req-1');
    });

    test('error responses carry the declared error Type URI', () async {
      final outcome = await run(doc(proof: testProof, recipient: null),
          handlerShouldNotRun: true);
      final error = (outcome as Rejected<Response>).error;
      expect(error.type, trustTaskErrorTypeUri);
      expect(isErrorResponse(error), isTrue);
    });

    test('the parent thread carries onto both response kinds', () async {
      final handled =
          await run(doc(proof: testProof, parentThreadId: 'parent-1'));
      expect(
          (handled as Handled<Response>).response.parentThreadId, 'parent-1');

      final rejected = await run(
        doc(proof: testProof, recipient: null, parentThreadId: 'parent-1'),
        handlerShouldNotRun: true,
      );
      expect((rejected as Rejected<Response>).error.parentThreadId, 'parent-1');
    });

    test('omits the member entirely when there is no parent', () {
      final response = respondWith<Payload, Response>(
        doc(),
        'resp-1',
        const Response(true),
        clock: fixedClock,
      );
      expect(
        response.toJson((r) => r.toJson()).containsKey('parentThreadId'),
        isFalse,
      );
    });
  });

  group('§7.2 item 2 — payload schema validation', () {
    const withSchema = SpecPolicy(
      typeUri: 'https://trusttasks.org/spec/acl/list/0.1',
      payloadSchema: '{"type":"object","required":["role"]}',
    );
    const demandsMore = SpecPolicy(
      typeUri: 'https://trusttasks.org/spec/acl/list/0.1',
      payloadSchema: '{"type":"object","required":["role","scope"]}',
    );

    test('accepts a conforming payload', () async {
      final outcome = await run(
        doc(recipient: null),
        spec: withSchema,
        payloadPolicy: const PayloadPolicy.validate(_RequiredMembers()),
      );
      expect(outcome, isA<Handled<Response>>());
    });

    test('rejects a payload missing a REQUIRED member, before the handler',
        () async {
      final outcome = await run(
        doc(recipient: null),
        spec: demandsMore,
        payloadPolicy: const PayloadPolicy.validate(_RequiredMembers()),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'malformedRequest');
      expect(
        (outcome as Rejected<Response>).error.payload.message,
        contains('missing scope'),
      );
    });

    test('a validator that throws is a failure, not a crash', () async {
      final outcome = await run(
        doc(recipient: null),
        spec: withSchema,
        payloadPolicy: const PayloadPolicy.validate(_ExplodingValidator()),
        handlerShouldNotRun: true,
      );
      expect(rejectedCode(outcome), 'malformedRequest');
    });

    test('acceptUnvalidated really does skip the check', () async {
      final outcome = await run(
        doc(recipient: null),
        spec: demandsMore,
        payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
      );
      expect(outcome, isA<Handled<Response>>());
    });

    test('is a no-op when the spec carries no schema', () async {
      final outcome = await run(
        doc(recipient: null),
        spec: relaxedSpec,
        payloadPolicy: const PayloadPolicy.validate(_ExplodingValidator()),
      );
      expect(outcome, isA<Handled<Response>>());
    });
  });

  group('handler outcomes', () {
    test('a handler that returns nothing yields accepted', () async {
      final outcome = await run(doc(recipient: null),
          spec: relaxedSpec, handlerReturnsNull: true);
      expect(outcome, isA<Accepted<Response>>());
    });

    test('a handler refusal passes through verbatim', () async {
      final outcome = await run(
        doc(recipient: null),
        spec: relaxedSpec,
        handlerRefuses: const RejectReason(
          code: StandardCode.permissionDenied,
          message: 'not on the approver list',
        ),
      );
      expect(rejectedCode(outcome), 'permissionDenied');
      expect((outcome as Rejected<Response>).error.recipient, peer);
    });
  });

  group('§7.1 / §7.2 unrecognized members', () {
    test('unrecognized top-level members survive a round trip', () {
      final wire = <String, dynamic>{
        'id': 'req-1',
        'type': requiredSpec.typeUri,
        'payload': <String, dynamic>{'role': 'admin'},
        'x-vendor-hint': <String, dynamic>{'keep': 'me'},
      };
      final parsed = TrustTaskDocument<Payload>.fromJson(
        wire,
        (p) => Payload((p as Map<String, dynamic>)['role'] as String),
      );
      expect(parsed.extra['x-vendor-hint'], <String, dynamic>{'keep': 'me'});
      final again = parsed.toJson(payloadToJson);
      expect(again['x-vendor-hint'], <String, dynamic>{'keep': 'me'});
    });
  });
}
