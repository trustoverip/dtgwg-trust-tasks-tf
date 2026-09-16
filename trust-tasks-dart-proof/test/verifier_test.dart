import 'dart:convert';
import 'dart:io';

import 'package:ssi/ssi.dart' as ssi;
import 'package:test/test.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

import 'support/sign.dart';

Map<String, dynamic> grant(String issuer, {String role = 'admin'}) => {
      'id': 'urn:uuid:9b2c1e34-0000-4000-8000-000000000001',
      'type': 'https://trusttasks.org/spec/acl/grant/0.1',
      'issuer': issuer,
      'recipient': 'did:web:maintainer.example',
      'issuedAt': '2026-01-01T00:00:00Z',
      'payload': {
        'entry': {'subject': 'did:web:alice.example', 'role': role},
      },
    };

Map<String, dynamic> copy(Map<String, dynamic> doc) =>
    jsonDecode(jsonEncode(doc)) as Map<String, dynamic>;

void main() {
  final verifier = DataIntegrityProofVerifier.forDidKey();

  group('interop', () {
    // Signed by trust-tasks-proof's `sign_trust_task` (Rust, eddsa-jcs-2022,
    // Ed25519 seed [7; 32]). The one test here that no Dart code produced, so
    // the one that shows the two libraries agree on the bytes being signed.
    final rustSigned = jsonDecode(
      File('test/fixtures/trust-tasks-proof-eddsa-jcs-2022.json')
          .readAsStringSync(),
    ) as Map<String, dynamic>;

    test('verifies a document signed by trust-tasks-proof (Rust)', () async {
      final result = await verifier.verifyDetailed(copy(rustSigned));
      expect(result.isValid, isTrue, reason: '$result');
    });

    test('rejects the Rust document once its payload is changed', () async {
      final doc = copy(rustSigned);
      (doc['payload'] as Map<String, dynamic>)['entry'] = {
        'role': 'owner',
        'subject': 'did:web:alice.example',
      };
      final result = await verifier.verifyDetailed(doc);
      expect(result.failure, ProofFailureKind.signatureInvalid);
    });
  });

  for (final make in [
    () => TestIdentity.ed25519(1),
    () => TestIdentity.p256(1),
    () => TestIdentity.p384(1),
  ]) {
    final id = make();
    group('${id.cryptosuite} (${id.signer.signatureScheme.name})', () {
      test('verifies its own signature', () async {
        final signed = await id.sign(grant(id.did));
        expect(await verifier.verify(signed), isTrue);
        expect(
          (await verifier.verifyDetailed(signed)).isValid,
          isTrue,
        );
      });

      test('rejects a tampered payload', () async {
        final signed = await id.sign(grant(id.did));
        final tampered = copy(signed);
        tampered['payload'] = grant(id.did, role: 'owner')['payload'];
        final result = await verifier.verifyDetailed(tampered);
        expect(result.failure, ProofFailureKind.signatureInvalid);
      });

      test('rejects a tampered proof configuration', () async {
        final signed = await id.sign(grant(id.did));
        (signed['proof'] as Map<String, dynamic>)['created'] =
            '2020-01-01T00:00:00Z';
        final result = await verifier.verifyDetailed(signed);
        expect(result.failure, ProofFailureKind.signatureInvalid);
      });

      test('does not mutate the document it verifies', () async {
        final signed = await id.sign(grant(id.did));
        final before = jsonEncode(signed);
        await verifier.verify(signed);
        expect(jsonEncode(signed), before);
      });
    });
  }

  group('issuer binding', () {
    test('rejects a valid signature under a different issuer', () async {
      // The attack the binding exists for: sign with your own key, claim to
      // be someone else. The signature is fine; the claim is not.
      final attacker = TestIdentity.ed25519(2);
      final victim = TestIdentity.ed25519(3);
      final signed = await attacker.sign(grant(victim.did));
      final result = await verifier.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.issuerMismatch);
      expect(await verifier.verify(signed), isFalse);
    });

    test('rejects a proof with no issuer to bind it to', () async {
      final id = TestIdentity.ed25519(1);
      final signed = await id.sign(grant(id.did)..remove('issuer'));
      final result = await verifier.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.issuerMismatch);
    });

    test('compares by exact string, without normalization', () async {
      final id = TestIdentity.ed25519(1);
      final signed = await id.sign(grant(id.did));
      signed['issuer'] = '${id.did} ';
      final result = await verifier.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.issuerMismatch);
    });
  });

  group('malformed proofs', () {
    late Map<String, dynamic> signed;
    setUp(() async {
      final id = TestIdentity.ed25519(1);
      signed = await id.sign(grant(id.did));
    });

    Map<String, dynamic> proof() => signed['proof'] as Map<String, dynamic>;

    test('no proof', () async {
      final result = await verifier.verifyDetailed(signed..remove('proof'));
      expect(result.failure, ProofFailureKind.malformedProof);
    });

    test('proof is not an object', () async {
      signed['proof'] = 'z123';
      final result = await verifier.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.malformedProof);
    });

    test('wrong proof type', () async {
      proof()['type'] = 'Ed25519Signature2020';
      final result = await verifier.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.malformedProof);
    });

    for (final member in [
      'cryptosuite',
      'verificationMethod',
      'proofPurpose',
      'proofValue',
    ]) {
      test('missing $member', () async {
        proof().remove(member);
        final result = await verifier.verifyDetailed(signed);
        expect(result.failure, ProofFailureKind.malformedProof);
      });
    }

    test('an undecodable proofValue fails rather than throws', () async {
      proof()['proofValue'] = 'not-multibase!';
      final result = await verifier.verifyDetailed(signed);
      expect(result.isValid, isFalse);
    });
  });

  group('created', () {
    final id = TestIdentity.ed25519(1);

    test('refuses a created with no zone designator', () async {
      // What ssi's own generators emit today (affinidi-ssi-dart#305). The
      // specification says to read it as UTC; DateTime.parse reads it as local
      // time. Refusing is the only reading that cannot be wrong, and it is
      // what trust-tasks-proof does.
      final signed = await id.sign(grant(id.did));
      final p = signed['proof'] as Map<String, dynamic>;
      p['created'] = (p['created'] as String).replaceFirst('Z', '');
      final result = await verifier.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.malformedProof);
    });

    test('accepts an explicit offset', () async {
      final signed = await id.sign(
        grant(id.did),
        createdText: '2026-01-01T09:30:00+09:30',
      );
      expect(await verifier.verify(signed), isTrue);
    });

    test('applies the offset when checking for a future created', () async {
      // 23:30-01:00 is 00:30Z the next day: in the future of a 00:00Z clock,
      // though the local digits alone would put it half an hour in the past.
      final now = DateTime.utc(2026, 1, 2);
      final strict = DataIntegrityProofVerifier.forDidKey(clock: () => now);
      final signed = await id.sign(
        grant(id.did),
        createdText: '2026-01-01T23:30:00-01:00',
      );
      final result = await strict.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.malformedProof);
    });

    test('refuses a created beyond the clock-skew allowance', () async {
      final now = DateTime.utc(2026, 6, 1, 12);
      final strict = DataIntegrityProofVerifier.forDidKey(clock: () => now);
      final future = await id.sign(
        grant(id.did),
        created: now.add(const Duration(minutes: 5)),
      );
      final result = await strict.verifyDetailed(future);
      expect(result.failure, ProofFailureKind.malformedProof);
    });

    test('tolerates a created within the allowance', () async {
      final now = DateTime.utc(2026, 6, 1, 12);
      final lenient = DataIntegrityProofVerifier.forDidKey(clock: () => now);
      final skewed = await id.sign(
        grant(id.did),
        created: now.add(const Duration(seconds: 30)),
      );
      expect(await lenient.verify(skewed), isTrue);
    });
  });

  group('cryptosuites', () {
    test('an instance can narrow the accepted set', () async {
      final edOnly = DataIntegrityProofVerifier.forDidKey(
          cryptosuites: {'eddsa-jcs-2022'});
      final p256 = TestIdentity.p256(1);
      final result =
          await edOnly.verifyDetailed(await p256.sign(grant(p256.did)));
      expect(result.failure, ProofFailureKind.unsupportedCryptosuite);
    });

    test('an unknown cryptosuite in a proof is unsupported', () async {
      final id = TestIdentity.ed25519(1);
      final signed = await id.sign(grant(id.did));
      (signed['proof'] as Map<String, dynamic>)['cryptosuite'] =
          'eddsa-rdfc-2022';
      final result = await verifier.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.unsupportedCryptosuite);
    });

    test('asking for a cryptosuite this package lacks is a programming error',
        () {
      expect(
        () => DataIntegrityProofVerifier.forDidKey(cryptosuites: {'bbs-2023'}),
        throwsArgumentError,
      );
    });
  });

  group('resolution', () {
    test('forDidKey refuses other DID methods without touching the network',
        () async {
      final id = TestIdentity.ed25519(1);
      // Sign as did:key, then relabel both issuer and verificationMethod as a
      // did:web. The binding holds; resolution must be what refuses it.
      final signed = await id.sign(grant(id.did));
      const web = 'did:web:unreachable.invalid';
      signed['issuer'] = web;
      final p = signed['proof'] as Map<String, dynamic>;
      p['verificationMethod'] = '$web#key-1';
      final result = await verifier.verifyDetailed(signed);
      expect(result.failure, ProofFailureKind.signatureInvalid);
      expect(result.detail, contains('did:key only'));
    });

    test('withResolver uses the resolver it is given', () async {
      final id = TestIdentity.ed25519(1);
      final seen = <String>[];
      final custom = DataIntegrityProofVerifier.withResolver(
        _RecordingResolver(seen),
      );
      expect(await custom.verify(await id.sign(grant(id.did))), isTrue);
      expect(seen, [id.did]);
    });
  });
}

final class _RecordingResolver implements ssi.DidResolver {
  _RecordingResolver(this.seen);

  final List<String> seen;

  @override
  Future<ssi.DidDocument> resolveDid(String did) async {
    seen.add(did);
    return ssi.DidKey.resolve(did);
  }
}
