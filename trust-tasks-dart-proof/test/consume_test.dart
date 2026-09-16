// The verifier in the place it is meant to be used: behind ProofPolicy.verify,
// inside the §7.2 pipeline.

import 'package:test/test.dart';
import 'package:trust_tasks/specs/acl/grant/v0_1/payload.dart' as acl_grant;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

import 'support/sign.dart';

const me = 'did:web:maintainer.example';

Future<ConsumeOutcome<acl_grant.Response>> consume(
  Map<String, dynamic> json, {
  required String transportIssuer,
}) {
  final doc = TrustTaskDocument<acl_grant.Payload>.fromJson(
    json,
    (payload) => acl_grant.Payload.fromJson(payload! as Map<String, dynamic>),
  );
  return consumeInbound<acl_grant.Payload, acl_grant.Response>(
    transport: StaticTransport(TransportContext(issuer: transportIssuer)),
    spec: acl_grant.spec,
    proofPolicy: ProofPolicy.verify(DataIntegrityProofVerifier.forDidKey()),
    payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
    checks: consequentialChecks(InMemoryReplayGuard()),
    doc: doc,
    myVid: me,
    now: DateTime.parse('2026-01-01T00:00:30Z'),
    newErrorId: () => 'urn:uuid:9b2c1e34-0000-4000-8000-0000000000ff',
    payloadToJson: (p) => p.toJson(),
    handler: (accepted, parties) async =>
        respondWith<acl_grant.Payload, acl_grant.Response>(
      accepted,
      'urn:uuid:9b2c1e34-0000-4000-8000-000000000002',
      acl_grant.Response(entry: accepted.payload.entry),
    ),
  );
}

Map<String, dynamic> grant(String issuer, {String role = 'admin'}) => {
      'id': 'urn:uuid:9b2c1e34-0000-4000-8000-000000000001',
      'type': acl_grant.typeUri,
      'issuer': issuer,
      'recipient': me,
      'issuedAt': '2026-01-01T00:00:00Z',
      'payload': {
        'entry': {'subject': 'did:web:alice.example', 'role': role},
      },
    };

void main() {
  final peer = TestIdentity.ed25519(9);

  test('a correctly signed grant reaches the handler', () async {
    final signed = await peer.sign(
      grant(peer.did),
      created: DateTime.utc(2026),
    );
    final outcome = await consume(signed, transportIssuer: peer.did);
    expect(outcome, isA<Handled<acl_grant.Response>>());
  });

  test('a tampered grant is refused as proofInvalid', () async {
    final signed = await peer.sign(
      grant(peer.did),
      created: DateTime.utc(2026),
    );
    signed['payload'] = grant(peer.did, role: 'owner')['payload'];
    final outcome = await consume(signed, transportIssuer: peer.did);
    expect(outcome, isA<Rejected<acl_grant.Response>>());
    expect(
      (outcome as Rejected<acl_grant.Response>).error.payload.code,
      StandardCode.proofInvalid.value,
    );
  });
}
