// The server keys the item-11 record on the body as received, and answers a
// body nested too deeply to walk with malformedRequest rather than an internal
// error.

import 'dart:convert';

import 'package:test/test.dart';

import 'support/fixtures.dart';

/// [echoDoc] as JSON text with an `extension` member nested [depth] lists
/// deep — written as text, since encoding it would overflow `jsonEncode`.
String deepBody(int depth) {
  final doc = jsonEncode(echoDoc());
  return '${doc.substring(0, doc.length - 1)},'
      '"extension":${'[' * depth}${']' * depth}}';
}

void main() {
  test(
      'a payload member the handler type does not model makes a different '
      'document under the same id: idConflict/409', () async {
    final calls = <String>[];
    final s = echoServer(calls: calls);
    expect((await s.handle(post(echoDoc()))).status, 200);
    final extended = echoDoc()
      ..['payload'] = <String, dynamic>{'text': 'hello', 'note': 'added'};
    final reply = await s.handle(post(extended));
    expect(reply.status, 409);
    expect(codeOf(reply), 'idConflict');
    expect(calls, ['hello']);
  });

  test('a deeply nested body is malformedRequest/400, not an internal error',
      () async {
    final calls = <String>[];
    final s = echoServer(calls: calls);
    final reply = await s.handle(post(deepBody(100000)));
    expect(reply.status, 400);
    expect(codeOf(reply), 'malformedRequest');
    expect(calls, isEmpty);
  });
}
