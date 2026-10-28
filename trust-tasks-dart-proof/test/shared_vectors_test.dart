// The registry's shared proof vectors (fixtures/proof at the repository root),
// which every library's verifier runs, so a document one implementation signs
// is checked by all four. They include a document whose timestamps a typed
// parser would re-spell (SPEC §4.7, "What a consumer verifies"): it must
// verify, and its re-spelled twin must not.
import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

void main() {
  const dir = '../fixtures/proof/';
  final manifest = jsonDecode(
    File('${dir}manifest.json').readAsStringSync(),
  ) as Map<String, dynamic>;
  final verifier = DataIntegrityProofVerifier.forDidKey();

  for (final entry
      in (manifest['entries'] as List).cast<Map<String, dynamic>>()) {
    final file = entry['file'] as String;
    final expect_ = entry['expect'] as String;
    test('shared vector: $file is $expect_', () async {
      // Every suite this verifier implements; an entry for any other suite
      // fails rather than being skipped silently.
      expect(
        DataIntegrityProofVerifier.supportedCryptosuites,
        contains(entry['cryptosuite']),
      );
      final doc = jsonDecode(
        File('$dir$file').readAsStringSync(),
      ) as Map<String, dynamic>;
      expect(await verifier.verify(doc), expect_ == 'valid');
    });
  }
}
