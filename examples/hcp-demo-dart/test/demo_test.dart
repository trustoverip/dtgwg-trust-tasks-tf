// The demo's own tests: hermetic (no network, no files outside the examples
// directory), deterministic (fixed keys, ids and clock), and run with
// `dart test`.

import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks_example_hcp_demo/demo.dart';
import 'package:trust_tasks_example_hcp_demo/trust_tasks/hcp/entries/query/v0_1/payload.dart'
    as query;

/// The golden transcript every language's port prints, owned by the TypeScript
/// demo. Compared with, never copied.
final File golden = File('../hcp-demo-ts/transcript.golden.txt');

String codeOf(Map<String, dynamic> reply) =>
    (reply['payload'] as Map<String, dynamic>)['code'] as String;

/// A world after the demo has run, with a query payload that would otherwise
/// be well-formed.
Future<(World, Map<String, dynamic>)> afterDemo() async {
  final world = await World.create();
  await runDemo(world);
  const payload = query.Payload(
    query: 'Anything else?',
    purpose: purpose,
    accessToken: managementToken,
  );
  return (world, payload.toJson());
}

void main() {
  test('the five steps print exactly the golden transcript '
      "(shared by every language's port)", () async {
    final result = await runDemo();
    expect(result.transcript, golden.readAsStringSync());
  });

  test(
    'a query after revoke fails with hcp:invalidToken (HCP 6.2.2)',
    () async {
      final result = await runDemo();
      expect(
        result.postRevokeError['type'],
        'https://trusttasks.org/spec/trust-task-error/0.6',
      );
      expect(codeOf(result.postRevokeError), 'hcp:invalidToken');
    },
  );

  test(
    'a document altered after signing is refused with proofInvalid',
    () async {
      final (world, payload) = await afterDemo();
      final (c, p) = (world.contextClient, world.provider);
      final signed = await world.request(
        c,
        p,
        '${query.typeUri}#request',
        payload,
      );
      final tampered = {
        ...signed,
        'payload': {...payload, 'query': 'Tell me everything.'},
      };
      final reply = await world.exchange(c, p, tampered);
      expect(codeOf(reply), 'proofInvalid');
    },
  );

  test('a correctly signed document that fails its schema is refused with '
      'malformedRequest', () async {
    final (world, payload) = await afterDemo();
    final (c, p) = (world.contextClient, world.provider);
    final noPurpose = {...payload}..remove('purpose');
    final reply = await world.exchange(
      c,
      p,
      await world.request(c, p, '${query.typeUri}#request', noPurpose),
    );
    expect(codeOf(reply), 'malformedRequest');
  });
}
