import 'dart:convert';

import 'package:test/test.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

import 'support/sign.dart';

// The bounds the Go, Rust and TypeScript proof packages apply too: at most 175
// base58 characters, decoding to at most 128 bytes.
const maxChars = 175;
const maxBytes = 128;

Map<String, dynamic> grant(String issuer) => {
      'id': 'urn:uuid:9b2c1e34-0000-4000-8000-000000000002',
      'type': 'https://trusttasks.org/spec/acl/grant/0.1',
      'issuer': issuer,
      'recipient': 'did:web:maintainer.example',
      'issuedAt': '2026-01-01T00:00:00Z',
      'payload': {
        'entry': {'subject': 'did:web:alice.example', 'role': 'admin'},
      },
    };

Map<String, dynamic> copy(Map<String, dynamic> doc) =>
    jsonDecode(jsonEncode(doc)) as Map<String, dynamic>;

void main() {
  final verifier = DataIntegrityProofVerifier.forDidKey();
  late Map<String, dynamic> signed;

  setUpAll(() async {
    final id = TestIdentity.ed25519(3);
    signed = await id.sign(grant(id.did));
  });

  Map<String, dynamic> withProofValue(String value) {
    final doc = copy(signed);
    (doc['proof'] as Map<String, dynamic>)['proofValue'] = value;
    return doc;
  }

  Map<String, dynamic> withDidKey(String multibase) {
    final doc = copy(signed);
    final did = 'did:key:$multibase';
    doc['issuer'] = did;
    (doc['proof'] as Map<String, dynamic>)['verificationMethod'] =
        '$did#$multibase';
    return doc;
  }

  test('a document within the bounds still verifies', () async {
    expect((await verifier.verifyDetailed(signed)).isValid, isTrue);
  });

  group('a value over the bounds is refused before ssi decodes it', () {
    final cases = {
      'one character too long': 'z${'2' * (maxChars + 1)}',
      'within the length, one byte too large': 'z${'z' * maxChars}',
      'too many leading zero bytes': 'z${'1' * (maxBytes + 1)}',
    };
    cases.forEach((name, value) {
      test('proofValue: $name', () async {
        final r = await verifier.verifyDetailed(withProofValue(value));
        expect(r.failure, ProofFailureKind.signatureInvalid);
        expect(r.detail, startsWith('proofValue: base58 value'));
      });
      test('did:key: $name', () async {
        final r = await verifier.verifyDetailed(withDidKey(value));
        expect(r.failure, ProofFailureKind.signatureInvalid);
        expect(r.detail, startsWith('verificationMethod: base58 value'));
      });
    });
  });

  // A megabyte-sized multibase value is refused without decoding it, so what a
  // verificationMethod or proofValue costs does not grow with its length.
  test('a 1 MiB proofValue or did:key is refused quickly', () async {
    final huge = 'z${'2' * (1 << 20)}';
    // `z6Mk` is the Ed25519 multikey prefix, which ssi decodes in full.
    final hugeKey = 'z6Mk${'2' * (1 << 20)}';
    final docs = [withProofValue(huge), withDidKey(hugeKey)];
    final sw = Stopwatch()..start();
    for (var i = 0; i < 100; i++) {
      for (final doc in docs) {
        final r = await verifier.verifyDetailed(doc);
        expect(r.failure, ProofFailureKind.signatureInvalid);
      }
    }
    // Decoding a value this size would take minutes; refusing it on its
    // length takes milliseconds. The bound sits between the two with room for
    // a slow or busy CI runner.
    expect(sw.elapsed, lessThan(const Duration(seconds: 10)),
        reason: 'refusing 200 oversized proofs took ${sw.elapsed}');
  });
}
