// Verifying a signed Trust Task document, inside the §7.2 pipeline.
//
// Run it:
//   dart run example/main.dart
//
// The document below was signed by trust-tasks-proof, the Rust library, with
// eddsa-jcs-2022 under a did:key. Verifying it here needs no network: a did:key
// carries its own public key.

import 'dart:convert';

import 'package:trust_tasks/specs/acl/grant/v0_1/payload.dart' as acl_grant;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

const me = 'did:web:maintainer.example';
const peer = 'did:key:z6MkvDqGT54cXesYGvABpF1UapVNwjCqRcafi4Px6Thv5T3Z';

const signedJson = '''
{
  "id": "urn:uuid:interop-1",
  "type": "https://trusttasks.org/spec/acl/grant/0.1",
  "issuer": "$peer",
  "recipient": "$me",
  "issuedAt": "2026-01-01T00:00:00Z",
  "payload": {"entry": {"role": "admin", "subject": "did:web:alice.example"}},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-16T10:34:46Z",
    "proofPurpose": "assertionMethod",
    "verificationMethod": "$peer#z6MkvDqGT54cXesYGvABpF1UapVNwjCqRcafi4Px6Thv5T3Z",
    "proofValue": "zbBjAshSrFcq3jaNSykHWRSmzNgGPU9DsPYFar8mLy2183ZfUtKTyz8B1nN6WnBdWEkXjT3MiYJvMnHsT1aEMrCK"
  }
}
''';

Future<void> main() async {
  // did:key only, no I/O. Use DataIntegrityProofVerifier.withResolver to accept
  // did:web and other methods.
  final verifier = DataIntegrityProofVerifier.forDidKey();

  Future<void> deliver(String label, Map<String, dynamic> json) async {
    // verifyDetailed says *why* a proof failed. Log it; never send it — the
    // pipeline answers the peer with a bare proofInvalid (SPEC §12.4).
    print('$label: ${await verifier.verifyDetailed(json)}');

    final outcome = await consumeInbound<acl_grant.Payload, acl_grant.Response>(
      transport: const StaticTransport(TransportContext(issuer: peer)),
      spec: acl_grant.spec,
      proofPolicy: ProofPolicy.verify(verifier),
      payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
      checks: notConsequentialChecks(),
      doc: TrustTaskDocument<acl_grant.Payload>.fromJson(
        json,
        (p) => acl_grant.Payload.fromJson(p! as Map<String, dynamic>),
      ),
      myVid: me,
      // Fixed so the example's issuedAt stays inside the freshness window. The
      // verifier keeps its own clock for the proof's `created`; pass `clock:`
      // to forDidKey to pin that too.
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
    print('  -> ${switch (outcome) {
      Handled(:final response) => 'handled, ${response.type}',
      Rejected(:final error) => 'rejected, ${error.payload.code}',
      _ => '$outcome',
    }}');
  }

  final signed = jsonDecode(signedJson) as Map<String, dynamic>;
  await deliver('as signed', signed);

  final escalated = jsonDecode(signedJson) as Map<String, dynamic>;
  (escalated['payload'] as Map<String, dynamic>)['entry'] = {
    'role': 'owner',
    'subject': 'did:web:alice.example',
  };
  await deliver('role changed in transit', escalated);
}
