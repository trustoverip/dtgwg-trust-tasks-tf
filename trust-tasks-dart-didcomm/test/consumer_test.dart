import 'package:ssi/ssi.dart';
import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_didcomm/trust_tasks_didcomm.dart';

import 'support/fixtures.dart';

void main() {
  late DidKeyManager alice;
  late DidKeyManager bob;
  late String aliceDid;
  late String bobDid;
  late List<String> calls;
  late DidcommConsumer consumer;

  setUp(() async {
    alice = await party(KeyType.ed25519);
    bob = await party(KeyType.ed25519);
    aliceDid = await didOf(alice);
    bobDid = await didOf(bob);
    calls = [];
    consumer = DidcommConsumer(recipient: bob, clock: () => fixedNow);
  });

  Future<Map<String, dynamic>> fromAlice(Map<String, dynamic> doc) async =>
      packTrustTask(doc, sender: alice, recipient: await bob.getDidDocument());

  Future<Received<Echo>> receive(Map<String, dynamic> envelope) =>
      consumer.receive<Echo, Echo>(
        envelope,
        spec: echoSpec,
        decode: Echo.fromJson,
        encode: (p) => p.toJson(),
        encodeResponse: (r) => r.toJson(),
        handler: (doc, parties) async {
          calls.add(doc.payload.text);
          if (doc.payload.text == 'crash') throw StateError('boom');
          return respondWith<Echo, Echo>(
            doc,
            newUrnUuid(),
            Echo(doc.payload.text.toUpperCase()),
          );
        },
      );

  String? codeOf(Received<Echo> r) =>
      (r.reply?['payload'] as Map<String, dynamic>?)?['code'] as String?;

  test('in-band identity may be omitted: authcrypt supplies it (§3)', () async {
    final r = await receive(await fromAlice(echoDoc()));
    expect(r.outcome, isA<Handled<Echo>>());
    expect(r.transport.peer, aliceDid);
    expect((r.reply!['payload'] as Map)['text'], 'HELLO');
    // respondWith mirrors in-band members, so the response names no parties;
    // packReply still addresses the envelope to the authenticated sender.
    final back = await unpackTrustTask(
      await consumer.packReply(r),
      recipient: alice,
    );
    expect(back.transport.peer, bobDid);
  });

  test('the reply goes back as an authcrypt envelope alice can open', () async {
    final r = await receive(
      await fromAlice(echoDoc(issuer: aliceDid, recipient: bobDid)),
    );
    final wire = await consumer.packReply(r);
    final back = await unpackTrustTask(wire, recipient: alice);
    expect(back.transport.peer, bobDid);
    expect(back.document['type'], '$echoType#response');
    expect(back.document['threadId'], echoDoc()['id']);
    expect(back.message.threadId, echoDoc()['id']);
  });

  test(
      'a mediator redelivery — fresh DIDComm id, same document — is absorbed '
      'with the first response (§6.1 item 1)', () async {
    final first = await receive(await fromAlice(echoDoc()));
    final again = await receive(await fromAlice(echoDoc()));
    expect(again.outcome, isA<DuplicateOutcome<Echo>>());
    expect(again.reply, first.reply);
    expect(calls, ['hello']);
  });

  test('a different document under the same id is idConflict (§6.1 item 2)',
      () async {
    await receive(await fromAlice(echoDoc()));
    final r = await receive(await fromAlice(echoDoc(text: 'changed')));
    expect(codeOf(r), 'idConflict');
    expect(calls, ['hello']);
  });

  test(
      'an in-band issuer other than the authenticated sender is '
      'identityMismatch, addressed to the authenticated sender', () async {
    final r = await receive(
      await fromAlice(echoDoc(issuer: 'did:web:someone-else.example')),
    );
    expect(codeOf(r), 'identityMismatch');
    expect(r.reply!['recipient'], aliceDid);
    expect(calls, isEmpty);
  });

  test('an in-band recipient other than this consumer is refused', () async {
    final r = await receive(
      await fromAlice(echoDoc(recipient: 'did:web:elsewhere.example')),
    );
    expect(r.outcome, isA<Rejected<Echo>>());
    expect(calls, isEmpty);
  });

  test(
      'thid disagreeing with threadId is malformedRequest, not '
      'identityMismatch (§3.1)', () async {
    // Pack with threadId t-1 (so thid = t-1), then read the body back with a
    // different threadId by packing a document whose member disagrees: the
    // simplest honest way is to unpack and alter the document before consume.
    final unpacked = await unpackTrustTask(
      await fromAlice(echoDoc(threadId: 't-1')),
      recipient: bob,
    );
    final altered = UnpackedTrustTask(
      document: {...unpacked.document, 'threadId': 't-2'},
      transport: unpacked.transport,
      message: unpacked.message,
    );
    final r = await consumer.consume<Echo, Echo>(
      altered,
      spec: echoSpec,
      decode: Echo.fromJson,
      encode: (p) => p.toJson(),
      encodeResponse: (r) => r.toJson(),
      handler: (doc, parties) async => null,
    );
    expect(codeOf(r), 'malformedRequest');
  });

  test('a stale document is refused as expired', () async {
    final stale = echoDoc()..['issuedAt'] = '2026-05-01T00:00:00Z';
    final r = await receive(await fromAlice(stale));
    expect(codeOf(r), 'expired');
  });

  test('a proof with no verifier configured is refused, not ignored', () async {
    final r = await receive(
      await fromAlice(
        echoDoc(
          issuer: aliceDid,
          recipient: bobDid,
          proof: {
            'type': 'DataIntegrityProof',
            'cryptosuite': 'eddsa-jcs-2022',
            'verificationMethod': '$aliceDid#k',
            'proofPurpose': 'assertionMethod',
            'created': fixedIssuedAt,
            'proofValue': 'z1',
          },
        ),
      ),
    );
    expect(codeOf(r), 'malformedRequest');
  });

  test('a malformed proof object is malformedRequest, answered to the sender',
      () async {
    final r = await receive(
      await fromAlice(echoDoc()..['proof'] = <String, dynamic>{'type': 1}),
    );
    expect(codeOf(r), 'malformedRequest');
    expect(r.reply!['recipient'], isNull);
    expect(r.transport.peer, aliceDid);
  });

  test('a payload of the wrong shape is malformedRequest', () async {
    final bad = echoDoc()..['payload'] = <String, dynamic>{'text': 7};
    final r = await receive(await fromAlice(bad));
    expect(codeOf(r), 'malformedRequest');
  });

  test('a handler that throws releases the claim, so a redelivery runs again',
      () async {
    await expectLater(
      receive(await fromAlice(echoDoc(text: 'crash'))),
      throwsStateError,
    );
    await expectLater(
      receive(await fromAlice(echoDoc(text: 'crash'))),
      throwsStateError,
    );
    expect(calls, ['crash', 'crash']);
  });

  test('a store that cannot be consulted fails closed with unavailable',
      () async {
    consumer = DidcommConsumer(
      recipient: bob,
      replayGuard: _BrokenGuard(),
      clock: () => fixedNow,
    );
    final r = await receive(await fromAlice(echoDoc()));
    expect(codeOf(r), 'unavailable');
    expect(calls, isEmpty);
  });

  test('a fire-and-forget success has nothing to reply with', () async {
    final r = await consumer.receive<Echo, Object?>(
      await fromAlice(echoDoc()),
      spec: echoSpec,
      decode: Echo.fromJson,
      encode: (p) => p.toJson(),
      encodeResponse: (r) => r,
      handler: (doc, parties) async => null,
    );
    expect(r.outcome, isA<Accepted<Object?>>());
    expect(r.reply, isNull);
    expect(() => consumer.packReply(r), throwsStateError);
  });
}

class _BrokenGuard implements ReplayGuard {
  @override
  Future<ReplayVerdict> claim(
    String id,
    String digest,
    DateTime? retainUntil,
    DateTime now,
  ) =>
      Future.error(StateError('store down'));
  @override
  Future<void> recordResponse(String id, Object? response) async {}
  @override
  Future<void> release(String id, String digest) async {}
}
