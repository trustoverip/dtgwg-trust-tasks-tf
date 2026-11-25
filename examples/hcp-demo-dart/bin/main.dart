// The HCP-over-Trust-Tasks demo: run the five steps and print every document.
//
//   dart pub get
//   dart run trust_tasks:generate --lock trust-tasks.lock --out lib/trust_tasks
//   dart run bin/main.dart
//
// A DEMONSTRATION, not a reference provider. See README.md and
// ../hcp-demo-ts/SCENARIO.md.

import 'dart:io';

import 'package:trust_tasks_example_hcp_demo/demo.dart';

Future<void> main() async {
  final result = await runDemo();
  stdout.write(result.transcript);
  final code = (result.postRevokeError['payload'] as Map)['code'];
  if (code != 'hcp:invalidToken') {
    throw StateError('expected hcp:invalidToken after revoke, got $code');
  }
  print(
    '\nhcp-demo-dart: discover, grant request (held, then granted), query, '
    'revoke, notice and post-revoke refusal all OK',
  );
}
