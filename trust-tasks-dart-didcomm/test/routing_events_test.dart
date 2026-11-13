// Forward reading along `extends` (#1117) and the observability hook (#1116).
import 'package:ssi/ssi.dart';
import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_didcomm/trust_tasks_didcomm.dart';

import 'support/fixtures.dart';

const echo2Type = 'https://trusttasks.org/spec/example/echo/0.2';
const echo3Type = 'https://trusttasks.org/spec/example/echo/0.3';

void main() {
  late DidKeyManager alice;
  late DidKeyManager bob;
  late List<ServerEvent> events;
  late DidcommConsumer consumer;

  setUp(() async {
    alice = await party(KeyType.ed25519);
    bob = await party(KeyType.ed25519);
    events = [];
    consumer = DidcommConsumer(
      recipient: bob,
      clock: () => fixedNow,
      onEvent: events.add,
      extendsIndex: const {echo2Type: '0.1'},
    );
  });

  Future<Map<String, dynamic>> fromAlice(Map<String, dynamic> doc) async =>
      packTrustTask(doc, sender: alice, recipient: await bob.getDidDocument());

  Future<Received<Echo>> receive(
    Map<String, dynamic> envelope, {
    bool crash = false,
  }) =>
      consumer.receive<Echo, Echo>(
        envelope,
        spec: echoSpec,
        decode: Echo.fromJson,
        encode: (p) => p.toJson(),
        encodeResponse: (r) => r.toJson(),
        handler: (doc, parties) async {
          if (crash) throw StateError('db down');
          return respondWith<Echo, Echo>(doc, newUrnUuid(), const Echo('ok'));
        },
      );

  String? codeOf(Received<Echo> r) =>
      (r.reply?['payload'] as Map<String, dynamic>?)?['code'] as String?;

  test('a later version declaring extends is read as the implemented one',
      () async {
    final doc = echoDoc()
      ..['type'] = echo2Type
      ..['payload'] = {'text': 'hi', 'addedIn02': true};
    final r = await receive(await fromAlice(doc));
    expect(r.outcome, isA<Handled<Echo>>());
    expect(r.reply!['type'], '$echo2Type#response');
  });

  test('a version that does not declare it is unsupportedVersion', () async {
    final r = await receive(await fromAlice(echoDoc()..['type'] = echo3Type));
    expect(codeOf(r), 'unsupportedVersion');
    expect(
      events.whereType<DocumentRejected>().single.error.payload.code,
      'unsupportedVersion',
    );
  });

  test('a duplicate replies with the recorded JSON', () async {
    final first = await receive(await fromAlice(echoDoc()));
    final again = await receive(await fromAlice(echoDoc()));
    final duplicate = again.outcome as DuplicateOutcome<Echo>;
    expect(duplicate.priorResponse, first.reply);
    expect(duplicate.isError, isFalse);
    expect(again.reply, first.reply);
  });

  test('a handler exception is reported, then propagates as before', () async {
    await expectLater(
      receive(await fromAlice(echoDoc()), crash: true),
      throwsStateError,
    );
    expect(events.whereType<HandlerFailed>().single.error, isA<StateError>());
  });

  test('a hook that throws changes nothing', () async {
    consumer = DidcommConsumer(
      recipient: bob,
      clock: () => fixedNow,
      onEvent: (_) => throw StateError('logger broke'),
    );
    final r = await receive(await fromAlice(echoDoc()..['type'] = echo3Type));
    expect(codeOf(r), 'unsupportedVersion');
  });
}
