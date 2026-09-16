// Smoke test across the seam between the generated libraries and the
// hand-written runtime.
//
// The drift check proves the generator was re-run and the runtime tests prove
// the pipeline behaves, but neither imports a generated library. This does: it
// runs a real specification's Payload, spec and payloadSchemaJson through
// consumeInbound exactly as a consumer would. It is the Dart counterpart of
// `npm run smoke` in trust-tasks-ts and `go test ./smoke` in trust-tasks-go.

import 'dart:convert';

import 'package:test/test.dart';
import 'package:trust_tasks/specs/acl/grant/v0_1/payload.dart' as acl_grant;
import 'package:trust_tasks/trust_tasks.dart';

const me = 'did:web:maintainer.example';
const peer = 'did:web:org.example';

final testNow = DateTime.utc(2026, 1, 1);
String fixedClock() => '2026-01-01T00:00:00Z';

TrustTaskDocument<acl_grant.Payload> request() =>
    TrustTaskDocument<acl_grant.Payload>(
      id: 'req-1',
      type: acl_grant.typeUri,
      issuer: peer,
      recipient: me,
      issuedAt: '2026-01-01T00:00:00Z',
      proof: const Proof(
        type: 'DataIntegrityProof',
        cryptosuite: 'eddsa-jcs-2022',
        verificationMethod: '$peer#key-1',
        created: '2026-01-01T00:00:00Z',
        proofPurpose: 'assertionMethod',
        proofValue: 'z3kg',
      ),
      payload: const acl_grant.Payload(
        entry:
            acl_grant.AclEntry(subject: 'did:web:alice.example', role: 'admin'),
      ),
    );

class _Accepting implements ProofVerifier {
  const _Accepting();
  @override
  Future<bool> verify(Map<String, dynamic> doc) async => true;
}

void main() {
  test('a generated specification drives the runtime end to end', () async {
    final guard = InMemoryReplayGuard();

    Future<ConsumeOutcome<acl_grant.Response>> consume() =>
        consumeInbound<acl_grant.Payload, acl_grant.Response>(
          transport: const StaticTransport(
            TransportContext(issuer: peer, recipient: me),
          ),
          spec: acl_grant.spec,
          proofPolicy: const ProofPolicy.verify(_Accepting()),
          payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
          checks: consequentialChecks(guard),
          doc: request(),
          myVid: me,
          now: testNow,
          newErrorId: () => 'err-1',
          payloadToJson: (p) => p.toJson(),
          clock: fixedClock,
          handler: (accepted, parties) async =>
              respondWith<acl_grant.Payload, acl_grant.Response>(
            accepted,
            'resp-1',
            acl_grant.Response(entry: accepted.payload.entry),
            clock: fixedClock,
          ),
        );

    final first = await consume();
    expect(first, isA<Handled<acl_grant.Response>>());
    expect(
      (first as Handled<acl_grant.Response>).response.type,
      acl_grant.responseTypeUri,
    );

    // §7.2 item 11: the identical resend is absorbed, not executed again.
    expect(await consume(), isA<DuplicateOutcome<acl_grant.Response>>());
  });

  test('the shipped schema is self-contained', () {
    final schema =
        jsonDecode(acl_grant.payloadSchemaJson) as Map<String, dynamic>;
    expect(schema.containsKey(r'$defs'), isTrue);
    expect(acl_grant.spec.payloadSchema, acl_grant.payloadSchemaJson);

    // Every cross-file $ref was inlined at generation time, which is what makes
    // SPEC §7.2 item 2 performable from the package alone.
    bool hasExternalRef(Object? node) {
      if (node is Map) {
        final ref = node[r'$ref'];
        if (ref is String && !ref.startsWith('#')) return true;
        return node.values.any(hasExternalRef);
      }
      if (node is List) return node.any(hasExternalRef);
      return false;
    }

    expect(hasExternalRef(schema), isFalse);
  });

  test('a generated payload round-trips through JSON', () {
    final original = request().payload;
    final decoded = acl_grant.Payload.fromJson(
      jsonDecode(jsonEncode(original.toJson())) as Map<String, dynamic>,
    );
    expect(decoded.entry.subject, original.entry.subject);
    expect(decoded.entry.role, original.entry.role);
    // Absent optional members stay absent rather than becoming null members.
    expect(original.toJson().containsKey('reason'), isFalse);
  });

  test('absent and present-but-empty stay distinguishable on a list member',
      () {
    // `acl`'s `allowedKeys` documents at length that "PRESENT-BUT-EMPTY means
    // authorized on NO keys — the opposite of absent". Dart's nullable list says
    // that natively, where Go needed a pointer-to-slice.
    const absent =
        acl_grant.AclEntry(subject: 'did:web:alice.example', role: 'admin');
    expect(absent.toJson().containsKey('allowedKeys'), isFalse);

    const empty = acl_grant.AclEntry(
      subject: 'did:web:alice.example',
      role: 'admin',
      allowedKeys: <String>[],
    );
    expect(empty.toJson()['allowedKeys'], isEmpty);
    expect(empty.toJson().containsKey('allowedKeys'), isTrue);
  });
}
