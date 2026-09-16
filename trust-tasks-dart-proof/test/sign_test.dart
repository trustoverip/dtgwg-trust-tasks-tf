import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:ssi/ssi.dart' as ssi;
import 'package:test/test.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

import 'support/sign.dart';

Map<String, dynamic> grant(String issuer) => {
      'id': 'urn:uuid:9b2c1e34-0000-4000-8000-000000000001',
      'type': 'https://trusttasks.org/spec/acl/grant/0.1',
      'issuer': issuer,
      'recipient': 'did:web:maintainer.example',
      'issuedAt': '2026-01-01T00:00:00Z',
      'payload': {
        'entry': {'subject': 'did:web:alice.example', 'role': 'admin'},
      },
    };

void main() {
  final verifier = DataIntegrityProofVerifier.forDidKey();

  group('interop', () {
    test('reproduces the Rust signature byte for byte', () async {
      // Ed25519 is deterministic, so the same key over the same document and
      // proof configuration must give the same proofValue. The fixture was
      // signed by trust-tasks-proof with seed [7; 32]; if the two libraries
      // disagreed about a single canonicalized byte, this would differ.
      final rust = jsonDecode(
        File('test/fixtures/trust-tasks-proof-eddsa-jcs-2022.json')
            .readAsStringSync(),
      ) as Map<String, dynamic>;
      final rustProof = rust['proof'] as Map<String, dynamic>;

      final key = TestIdentity.ed25519(7);
      expect(key.did, rust['issuer']);

      final dart = await signTrustTask(
        rust,
        key.signer,
        created: DateTime.parse(rustProof['created'] as String),
      );
      expect(dart['proof'], rustProof);
    });
  });

  for (final make in [
    () => TestIdentity.ed25519(1),
    () => TestIdentity.p256(1),
    () => TestIdentity.p384(1),
  ]) {
    final id = make();
    group(id.signer.signatureScheme.name, () {
      test('emits what the verifier accepts', () async {
        final signed = await signTrustTask(grant(id.did), id.signer);
        expect(await verifier.verifyDetailed(signed), ProofVerification.ok);
        expect(
          (signed['proof'] as Map<String, dynamic>)['cryptosuite'],
          id.cryptosuite,
        );
      });
    });
  }

  test('created is UTC with a Z, whatever the input zone', () async {
    final id = TestIdentity.ed25519(1);
    final signed = await signTrustTask(
      grant(id.did),
      id.signer,
      created: DateTime.parse('2026-01-01T09:30:00+09:30'),
    );
    expect(
      (signed['proof'] as Map<String, dynamic>)['created'],
      '2026-01-01T00:00:00Z',
    );
    final fractional = await signTrustTask(
      grant(id.did),
      id.signer,
      created: DateTime.utc(2026, 1, 1, 0, 0, 0, 120),
    );
    expect(
      (fractional['proof'] as Map<String, dynamic>)['created'],
      '2026-01-01T00:00:00.120Z',
    );
    final byDefault = await signTrustTask(grant(id.did), id.signer);
    expect(
      (byDefault['proof'] as Map<String, dynamic>)['created'],
      endsWith('Z'),
    );
  });

  test('replaces an existing proof rather than signing over it', () async {
    final id = TestIdentity.ed25519(1);
    final once = await signTrustTask(grant(id.did), id.signer);
    final twice = await signTrustTask(once, id.signer);
    expect(await verifier.verify(twice), isTrue);
    expect(
      (twice['proof'] as Map<String, dynamic>).containsKey('proof'),
      isFalse,
    );
  });

  test('does not mutate the document it is given', () async {
    final id = TestIdentity.ed25519(1);
    final doc = grant(id.did);
    final before = jsonEncode(doc);
    await signTrustTask(doc, id.signer);
    expect(jsonEncode(doc), before);
  });

  group('refuses a document that could never verify', () {
    final id = TestIdentity.ed25519(1);

    test('no issuer', () {
      expect(
        () => signTrustTask(grant(id.did)..remove('issuer'), id.signer),
        throwsA(isA<SignException>()
            .having((e) => e.kind, 'kind', SignFailureKind.missingIssuer)),
      );
    });

    test('an issuer the signer does not control', () {
      expect(
        () => signTrustTask(grant(TestIdentity.ed25519(2).did), id.signer),
        throwsA(isA<SignException>()
            .having((e) => e.kind, 'kind', SignFailureKind.issuerMismatch)),
      );
    });

    test('a cryptosuite the key cannot produce', () {
      expect(
        () => signTrustTask(
          grant(id.did),
          id.signer,
          cryptosuite: 'ecdsa-jcs-2019',
        ),
        throwsA(isA<SignException>()
            .having((e) => e.kind, 'kind', SignFailureKind.unsupportedKey)),
      );
    });

    test('a key type with no JCS cryptosuite', () {
      final keyPair =
          ssi.Secp256k1KeyPair.fromSeed(Uint8List.fromList(List.filled(32, 1)));
      final doc = ssi.DidKey.generateDocument(keyPair.publicKey);
      final signer = ssi.DidSigner(
        did: doc.id,
        didKeyId: doc.verificationMethod[0].id,
        keyPair: keyPair,
        signatureScheme: ssi.SignatureScheme.ecdsa_secp256k1_sha256,
      );
      expect(
        () => signTrustTask(grant(doc.id), signer),
        throwsA(isA<SignException>()
            .having((e) => e.kind, 'kind', SignFailureKind.unsupportedKey)),
      );
    });
  });
}
