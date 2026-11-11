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

  group('proofPurpose', () {
    for (final purpose in signingPurposes) {
      test('$purpose verifies', () async {
        final id = TestIdentity.ed25519(1);
        final signed = await id.sign(grant(id.did), proofPurpose: purpose);
        final result = await verifier.verifyDetailed(signed);
        expect(result.isValid, isTrue, reason: '$result');
      });
    }

    for (final purpose in [
      'keyAgreement',
      'assertion',
      'AUTHENTICATION',
      'authentication ',
    ]) {
      test('"$purpose" is malformedProof', () async {
        final id = TestIdentity.ed25519(1);
        final signed = await id.sign(grant(id.did), proofPurpose: purpose);
        final result = await verifier.verifyDetailed(signed);
        expect(result.failure, ProofFailureKind.malformedProof);
      });
    }
  });

  group('expires', () {
    final now = DateTime.utc(2026, 6, 1, 12);
    final fixed = DataIntegrityProofVerifier.forDidKey(clock: () => now);
    final id = TestIdentity.ed25519(1);
    Future<Map<String, dynamic>> withExpires(Object expires) => id.sign(
          grant(id.did),
          createdText: '2026-06-01T11:00:00Z',
          proofPurpose: 'authentication',
          extra: {'expires': expires},
        );

    for (final expires in [
      '2026-06-01T12:00:01Z',
      '2026-06-01T12:00:00Z',
      '2028-02-29T00:00:00Z',
      '2027-01-01T00:00:00+14:00',
      '2027-01-01T00:00:00-12:59',
      '2027-01-01T23:59:59.999999Z',
    ]) {
      test('$expires verifies', () async {
        final result = await fixed.verifyDetailed(await withExpires(expires));
        expect(result.isValid, isTrue, reason: '$result');
      });
    }

    final refused = <String, Object>{
      'passed': '2026-06-01T11:59:59Z',
      'passed, offset': '2026-06-01T13:59:59+02:00',
      'no zone': '2027-01-01T00:00:00',
      'not a string': 1767225600,
      'not a date': 'tomorrow',
      'date only': '2027-01-01',
      'impossible month': '2027-13-01T00:00:00Z',
      'February 30': '2027-02-30T00:00:00Z',
      'February 29, not a leap year': '2027-02-29T00:00:00Z',
      'hour 24': '2027-01-01T24:00:00Z',
      'minute 60': '2027-01-01T00:60:00Z',
      'leap second': '2027-01-01T00:00:60Z',
      'offset +24:00': '2027-01-01T00:00:00+24:00',
      'offset +23:60': '2027-01-01T00:00:00+23:60',
      'offset +14:01': '2027-01-01T00:00:00+14:01',
      'offset -99:00': '2027-01-01T00:00:00-99:00',
      'empty': '',
    };
    refused.forEach((name, expires) {
      test('$name is malformedProof', () async {
        final result = await fixed.verifyDetailed(await withExpires(expires));
        expect(result.failure, ProofFailureKind.malformedProof);
      });
    });
  });

  // A resolved document lists the issuer's key under some relationships and
  // not others; the proof verifies only under one that lists it.
  group('verification relationship (withResolver)', () {
    const did = 'did:web:issuer.example';
    final id = TestIdentity.ed25519(1);
    final multibase =
        (ssi.DidKey.resolve(id.did).verificationMethod.first as dynamic)
            .publicKeyMultibase as String;

    Map<String, dynamic> method(String vmId, {String controller = did}) => {
          'id': vmId,
          'type': 'Multikey',
          'controller': controller,
          'publicKeyMultibase': multibase,
        };

    Future<ProofVerification> verifyAgainst(
      Map<String, dynamic> document, {
      String purpose = 'authentication',
      String vm = '$did#key-1',
    }) async {
      final signed = await id.sign(
        grant(did),
        proofPurpose: purpose,
        verificationMethod: vm,
      );
      final v = DataIntegrityProofVerifier.withResolver(_Fixed({
        '@context': [
          'https://www.w3.org/ns/did/v1',
          'https://w3id.org/security/multikey/v1',
        ],
        ...document,
      }));
      return v.verifyDetailed(signed);
    }

    test('a key listed under the purpose verifies', () async {
      final result = await verifyAgainst({
        'id': did,
        'verificationMethod': [method('$did#key-1')],
        'authentication': ['$did#key-1'],
      });
      expect(result.isValid, isTrue, reason: '$result');
    });

    test('a relative reference verifies', () async {
      final result = await verifyAgainst({
        'id': did,
        'verificationMethod': [method('$did#key-1')],
        'authentication': ['#key-1'],
      });
      expect(result.isValid, isTrue, reason: '$result');
    });

    test('a key listed only under keyAgreement is refused', () async {
      final result = await verifyAgainst({
        'id': did,
        'verificationMethod': [method('$did#key-1')],
        'keyAgreement': ['$did#key-1'],
      });
      expect(result.failure, ProofFailureKind.signatureInvalid);
      expect(result.detail, contains('not listed under authentication'));
    });

    test('a key listed under another relationship is refused', () async {
      final result = await verifyAgainst(
        {
          'id': did,
          'verificationMethod': [method('$did#key-1')],
          'authentication': ['$did#key-1'],
        },
        purpose: 'assertionMethod',
      );
      expect(result.failure, ProofFailureKind.signatureInvalid);
      expect(result.detail, contains('not listed under assertionMethod'));
    });

    test('a key listed only under verificationMethod is refused', () async {
      final result = await verifyAgainst({
        'id': did,
        'verificationMethod': [method('$did#key-1')],
      });
      expect(result.failure, ProofFailureKind.signatureInvalid);
      expect(result.detail, contains('not listed under authentication'));
    });

    test('a key whose controller is another DID is refused', () async {
      final result = await verifyAgainst({
        'id': did,
        'verificationMethod': [
          method('$did#key-1', controller: 'did:web:other.example'),
        ],
        'authentication': ['$did#key-1'],
      });
      expect(result.failure, ProofFailureKind.signatureInvalid);
      expect(result.detail, contains('controller'));
    });

    test('an embedded key whose controller is another DID is refused',
        () async {
      final result = await verifyAgainst({
        'id': did,
        'verificationMethod': [method('$did#key-1')],
        'authentication': [
          method('$did#key-1', controller: 'did:web:other.example'),
        ],
      });
      expect(result.failure, ProofFailureKind.signatureInvalid);
      expect(result.detail, contains('controller'));
    });

    test('an embedded key with the issuer as controller verifies', () async {
      final result = await verifyAgainst({
        'id': did,
        'verificationMethod': [method('$did#key-1')],
        'authentication': [method('$did#key-1')],
      });
      expect(result.isValid, isTrue, reason: '$result');
    });

    test('a document for another DID is refused', () async {
      final result = await verifyAgainst({
        'id': 'did:web:other.example',
        'verificationMethod': [method('$did#key-1')],
        'authentication': ['$did#key-1'],
      });
      expect(result.failure, ProofFailureKind.signatureInvalid);
      expect(result.detail, contains("document's id"));
    });

    test('a fragment match on a method with a different id is refused',
        () async {
      // ssi matches a method by fragment suffix alone; the relationship check
      // names the exact id.
      final result = await verifyAgainst({
        'id': did,
        'verificationMethod': [method('did:web:other.example#key-1')],
        'authentication': ['did:web:other.example#key-1'],
      });
      expect(result.failure, ProofFailureKind.signatureInvalid);
      expect(result.detail, contains('not listed under authentication'));
    });
  });
}

final class _Fixed implements ssi.DidResolver {
  _Fixed(this.document);

  final Map<String, dynamic> document;

  @override
  Future<ssi.DidDocument> resolveDid(String did) async =>
      ssi.DidDocument.fromJson(document);
}
