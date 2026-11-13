// SPEC §12.4: the specification's declared `maxDocumentBytes`, held against
// each document the consumer opens (#1121).
import 'dart:convert';

import 'package:ssi/ssi.dart';
import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_didcomm/trust_tasks_didcomm.dart';

import 'support/fixtures.dart';

const echo2Type = 'https://trusttasks.org/spec/example/echo/0.2';

int sizeOf(Map<String, dynamic> doc) => utf8.encode(jsonEncode(doc)).length;

void main() {
  late DidKeyManager alice;
  late DidKeyManager bob;
  late String aliceDid;
  late DidcommConsumer consumer;
  late int handled;

  // The bound is exactly the size of the default echo document from alice.
  late Map<String, dynamic> atBound;
  late SpecPolicy bounded;

  setUp(() async {
    alice = await party(KeyType.ed25519);
    bob = await party(KeyType.ed25519);
    aliceDid = await didOf(alice);
    handled = 0;
    consumer = DidcommConsumer(
      recipient: bob,
      clock: () => fixedNow,
      extendsIndex: const {echo2Type: '0.1'},
    );
    atBound = echoDoc(text: 'hello');
    bounded = SpecPolicy(typeUri: echoType, maxDocumentBytes: sizeOf(atBound));
  });

  Future<Map<String, dynamic>> fromAlice(Map<String, dynamic> doc) async =>
      packTrustTask(doc, sender: alice, recipient: await bob.getDidDocument());

  Future<Received<Echo>> receive(Map<String, dynamic> wire) =>
      consumer.receive<Echo, Echo>(
        wire,
        spec: bounded,
        decode: Echo.fromJson,
        encode: (p) => p.toJson(),
        encodeResponse: (r) => r.toJson(),
        handler: (doc, parties) async {
          handled++;
          return respondWith<Echo, Echo>(doc, newUrnUuid(), const Echo('ok'));
        },
      );

  test('a document at its declared bound is accepted', () async {
    final r = await receive(await fromAlice(atBound));
    expect(r.outcome, isA<Handled<Echo>>());
    expect(handled, 1);
  });

  test(
    'one byte over is malformedRequest to the authenticated sender, '
    'with no record written and no handler run',
    () async {
      final over = echoDoc(text: 'hello!');
      expect(sizeOf(over), sizeOf(atBound) + 1);
      final r = await receive(await fromAlice(over));
      final error = (r.outcome as Rejected<Echo>).error;
      expect(error.payload.code, 'malformedRequest');
      expect(error.payload.message, contains('size bound'));
      // Packed back to the authcrypt sender (packReply), and never
      // addressed to anyone else in-band.
      expect(r.transport.peer, aliceDid);
      expect(error.recipient, anyOf(isNull, aliceDid));
      expect(handled, 0);

      // The id was not burned: the same id, within the bound, is a fresh
      // document — not an idConflict against a record of the refused one.
      final again = await receive(await fromAlice(atBound));
      expect(again.outcome, isA<Handled<Echo>>());
      expect(handled, 1);
    },
  );

  test(
    'a later version read forward is not held to the implemented bound',
    () async {
      final later = echoDoc(text: 'hello')
        ..['type'] = echo2Type
        ..['payload'] = {'text': 'hello', 'addedIn02': 'x' * 64};
      expect(sizeOf(later), greaterThan(sizeOf(atBound)));
      final r = await receive(await fromAlice(later));
      expect(r.outcome, isA<Handled<Echo>>());
      expect(r.reply!['type'], '$echo2Type#response');
    },
  );

  test('a specification that declares no bound sets none', () async {
    final big = echoDoc(text: 'x' * 100000);
    final r = await consumer.receive<Echo, Echo>(
      await fromAlice(big),
      spec: echoSpec,
      decode: Echo.fromJson,
      encode: (p) => p.toJson(),
      encodeResponse: (r) => r.toJson(),
      handler: (doc, parties) async =>
          respondWith<Echo, Echo>(doc, newUrnUuid(), const Echo('ok')),
    );
    expect(r.outcome, isA<Handled<Echo>>());
  });
}
