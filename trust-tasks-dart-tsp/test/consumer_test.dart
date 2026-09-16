import 'dart:typed_data';

import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:ssi/ssi.dart';
import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_tsp/trust_tasks_tsp.dart';

import 'support/fixtures.dart';

void main() {
  late PrivateVid alice;
  late PrivateVid bob;
  late PublicVid bobPub;
  late MapResolver resolver;
  late List<String> calls;
  late TspConsumer consumer;

  setUp(() async {
    alice = await identity(KeyType.ed25519);
    bob = await identity(KeyType.ed25519);
    resolver = await resolverFor([alice, bob]);
    bobPub = await resolver.resolve(bob.id);
    calls = [];
    consumer = TspConsumer(
      recipient: bob,
      resolver: resolver,
      clock: () => fixedNow,
    );
  });

  Future<Uint8List> fromAlice(Map<String, dynamic> doc) =>
      packTrustTask(doc, sender: alice, receiver: bobPub);

  Future<Received<Echo>> receive(Uint8List wire) =>
      consumer.receive<Echo, Echo>(
        wire,
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

  test('in-band identity may be omitted: TSP supplies it (§3)', () async {
    final r = await receive(await fromAlice(echoDoc()));
    expect(r.outcome, isA<Handled<Echo>>());
    expect(r.transport.peer, alice.id);
    expect((r.reply!['payload'] as Map)['text'], 'HELLO');
  });

  test('the reply seals back to alice, who opens it', () async {
    final r = await receive(await fromAlice(echoDoc()));
    final wire = await consumer.packReply(r);
    final back = await unpackTrustTask(wire, recipient: alice, sender: bobPub);
    expect(back.transport.peer, bob.id);
    expect(back.document['type'], '$echoType#response');
    expect(back.document['threadId'], echoDoc()['id']);
  });

  test('a re-forward — resealed, same document — is absorbed with the first '
      'response (§7.1 item 1)', () async {
    final first = await receive(await fromAlice(echoDoc()));
    final again = await receive(await fromAlice(echoDoc()));
    expect(again.outcome, isA<DuplicateOutcome<Echo>>());
    expect(again.reply, first.reply);
    expect(calls, ['hello']);
  });

  test(
    'a different document under the same id is idConflict (§7.1 item 2)',
    () async {
      await receive(await fromAlice(echoDoc()));
      final r = await receive(await fromAlice(echoDoc(text: 'changed')));
      expect(codeOf(r), 'idConflict');
      expect(calls, ['hello']);
    },
  );

  test('an in-band issuer other than the authenticated sender is '
      'identityMismatch, addressed to the sender', () async {
    final r = await receive(
      await fromAlice(echoDoc(issuer: 'did:web:someone-else.example')),
    );
    expect(codeOf(r), 'identityMismatch');
    expect(r.reply!['recipient'], alice.id);
    expect(calls, isEmpty);
  });

  test('an in-band recipient other than this consumer is refused', () async {
    final r = await receive(
      await fromAlice(echoDoc(recipient: 'did:web:elsewhere.example')),
    );
    expect(r.outcome, isA<Rejected<Echo>>());
    expect(calls, isEmpty);
  });

  test('a sender off the allowlist is refused before resolution', () async {
    consumer = TspConsumer(
      recipient: bob,
      resolver: resolver,
      allowedSenders: {'did:web:someone.example'},
      clock: () => fixedNow,
    );
    await expectLater(
      receive(await fromAlice(echoDoc())),
      throwsA(
        isA<TspEnvelopeException>().having(
          (e) => e.failure,
          'failure',
          TspFailure.senderNotAllowed,
        ),
      ),
    );
    expect(calls, isEmpty);
  });

  test('a stale document is refused as expired', () async {
    final stale = echoDoc()..['issuedAt'] = '2026-05-01T00:00:00Z';
    expect(codeOf(await receive(await fromAlice(stale))), 'expired');
  });

  test('a proof with no verifier configured is refused, not ignored', () async {
    final r = await receive(
      await fromAlice(
        echoDoc(
          issuer: alice.id,
          recipient: bob.id,
          proof: {
            'type': 'DataIntegrityProof',
            'cryptosuite': 'eddsa-jcs-2022',
            'verificationMethod': '${alice.id}#k',
            'proofPurpose': 'assertionMethod',
            'created': fixedIssuedAt,
            'proofValue': 'z1',
          },
        ),
      ),
    );
    expect(codeOf(r), 'malformedRequest');
  });

  test('a payload of the wrong shape is malformedRequest', () async {
    final bad = echoDoc()..['payload'] = <String, dynamic>{'text': 7};
    expect(codeOf(await receive(await fromAlice(bad))), 'malformedRequest');
  });

  test(
    'a handler that throws releases the claim, so a re-forward runs again',
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
    },
  );

  test(
    'a store that cannot be consulted fails closed with unavailable',
    () async {
      consumer = TspConsumer(
        recipient: bob,
        resolver: resolver,
        replayGuard: _BrokenGuard(),
        clock: () => fixedNow,
      );
      final r = await receive(await fromAlice(echoDoc()));
      expect(codeOf(r), 'unavailable');
      expect(calls, isEmpty);
    },
  );

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
  ) => Future.error(StateError('store down'));
  @override
  Future<void> recordResponse(String id, Object? response) async {}
  @override
  Future<void> release(String id, String digest) async {}
}
