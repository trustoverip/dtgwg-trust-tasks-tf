// The shared forward-routing vectors in fixtures/consume/extends-routing.json
// at the repository root, which the Rust, TypeScript and Go runtimes also run:
// a consumer that implements M.K reads a later version whose declared
// `extends` chain reaches it as M.K, pruned, and answers it as the received
// version; anything else is unsupportedVersion or unsupportedType
// (SPEC §5.2 "Declared extension", §7.2 item 3).

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/src/runtime/extension.dart' show typeRefusalFor;
import 'package:trust_tasks/trust_tasks.dart';

void main() {
  final vectors = jsonDecode(
    File('../fixtures/consume/extends-routing.json').readAsStringSync(),
  ) as Map<String, dynamic>;
  final implemented = vectors['implements'] as Map<String, dynamic>;
  final spec = SpecPolicy(
    typeUri: implemented['typeUri'] as String,
    payloadSchema: implemented['payloadSchema'] as String,
  );
  final index =
      (vectors['extends'] as Map<String, dynamic>).cast<String, String>();
  final cases = (vectors['cases'] as List).cast<Map<String, dynamic>>();

  group(
      'shared extends-routing vectors (fixtures/consume/extends-routing.json)',
      () {
    for (final c in cases) {
      test(c['name'] as String, () {
        final received =
            jsonDecode(c['document'] as String) as Map<String, dynamic>;
        final before = jsonEncode(received);
        final read = readForVersion<Object?>(
          received,
          spec,
          (p) => p,
          index: index,
        );
        // The document as received is never modified: a proof and the item-11
        // digest are over it.
        expect(jsonEncode(received), before);

        final verdict = c['verdict'] as String;
        switch (read) {
          case VersionRefused(:final reason):
            expect(reason.code.toString(), verdict);
          case VersionAccepted(:final doc, :final forward):
            expect(verdict, 'accept');
            expect(doc.payload, c['payload']);
            expect(forward, c['extended']);
            expect(doc.type, received['type']);
            final answer =
                respondWith<Object?, Object?>(doc, 'urn:uuid:answer', null);
            expect(answer.type, c['answerType']);
        }

        // consumeInbound's own item-3 check agrees with the reader.
        final refusal =
            typeRefusalFor(received['type'] as String, spec, index: index);
        expect(refusal?.code.toString() ?? 'accept', verdict);
      });
    }
  });
}
