import 'dart:convert';

import 'package:didcomm/didcomm.dart';
import 'package:ssi/ssi.dart';
import 'package:test/test.dart';
import 'package:trust_tasks_didcomm/trust_tasks_didcomm.dart';

import 'support/fixtures.dart';

Matcher failsWith(EnvelopeFailure failure) => throwsA(
      isA<EnvelopeException>().having((e) => e.failure, 'failure', failure),
    );

void main() {
  for (final keyType in [KeyType.p256, KeyType.ed25519]) {
    group('$keyType', () {
      late DidKeyManager alice;
      late DidKeyManager bob;
      setUp(() async {
        alice = await party(keyType);
        bob = await party(keyType);
      });

      test('round trip: the authenticated sender and the recipient', () async {
        final doc = echoDoc();
        final wire = await packTrustTask(
          doc,
          sender: alice,
          recipient: await bob.getDidDocument(),
        );
        final unpacked = await unpackTrustTask(wire, recipient: bob);
        expect(unpacked.document, doc);
        expect(unpacked.transport.peer, await didOf(alice));
        expect(unpacked.transport.local, await didOf(bob));
        expect(unpacked.message.type.toString(), envelopeType);
      });
    });
  }

  group('with P-256 parties', () {
    late DidKeyManager alice;
    late DidKeyManager bob;
    late DidKeyManager carol;
    setUp(() async {
      alice = await party(KeyType.p256);
      bob = await party(KeyType.p256);
      carol = await party(KeyType.p256);
    });

    Future<Map<String, dynamic>> packFor(
      DidManager to,
      Map<String, dynamic> doc,
    ) async =>
        packTrustTask(doc, sender: alice, recipient: await to.getDidDocument());

    test('thid follows threadId, falling back to the document id (§3.1)',
        () async {
      var unpacked = await unpackTrustTask(
        await packFor(bob, echoDoc()),
        recipient: bob,
      );
      expect(unpacked.message.threadId, echoDoc()['id']);
      expect(unpacked.message.parentThreadId, isNull);

      unpacked = await unpackTrustTask(
        await packFor(bob, echoDoc(threadId: 't-1', parentThreadId: 'p-1')),
        recipient: bob,
      );
      expect(unpacked.message.threadId, 't-1');
      expect(unpacked.message.parentThreadId, 'p-1');
    });

    test('two packs of one document carry different DIDComm ids', () async {
      final a =
          await unpackTrustTask(await packFor(bob, echoDoc()), recipient: bob);
      final b =
          await unpackTrustTask(await packFor(bob, echoDoc()), recipient: bob);
      expect(a.message.id, isNot(b.message.id));
    });

    test('an envelope for someone else does not open', () async {
      expect(
        unpackTrustTask(await packFor(bob, echoDoc()), recipient: carol),
        failsWith(EnvelopeFailure.undecryptable),
      );
    });

    test('a tampered ciphertext does not open', () async {
      final wire = await packFor(bob, echoDoc());
      final ct = wire['ciphertext'] as String;
      wire['ciphertext'] = '${ct[0] == 'A' ? 'B' : 'A'}${ct.substring(1)}';
      expect(
        unpackTrustTask(wire, recipient: bob),
        failsWith(EnvelopeFailure.undecryptable),
      );
    });

    test('anoncrypt authenticates no sender (§4)', () async {
      final message = PlainTextMessage(
        id: 'm-1',
        type: Uri.parse(envelopeType),
        to: [await didOf(bob)],
        body: echoDoc(),
      );
      final anon = await EncryptedMessage.pack(
        message,
        keyType: KeyType.p256,
        recipientDidDocuments: [await bob.getDidDocument()],
        keyWrappingAlgorithm: KeyWrappingAlgorithm.ecdhEs,
        encryptionAlgorithm: EncryptionAlgorithm.a256gcm,
      );
      expect(
        unpackTrustTask(
          jsonDecode(jsonEncode(anon)) as Map<String, dynamic>,
          recipient: bob,
        ),
        failsWith(EnvelopeFailure.unauthenticatedSender),
      );
    });

    test('plaintext and signed-only authenticate no sender (§4)', () async {
      final message = PlainTextMessage(
        id: 'm-1',
        type: Uri.parse(envelopeType),
        from: await didOf(alice),
        to: [await didOf(bob)],
        body: echoDoc(),
      );
      expect(
        unpackTrustTask(message.toJson(), recipient: bob),
        failsWith(EnvelopeFailure.unauthenticatedSender),
      );
      final aliceDoc = await alice.getDidDocument();
      final signed = await SignedMessage.pack(
        message,
        signer: await alice.getSigner(aliceDoc.assertionMethod.first.id),
      );
      expect(
        unpackTrustTask(
          jsonDecode(jsonEncode(signed)) as Map<String, dynamic>,
          recipient: bob,
        ),
        failsWith(EnvelopeFailure.unauthenticatedSender),
      );
    });

    test('a sender off the allowlist is refused before decryption', () async {
      final wire = await packFor(bob, echoDoc());
      expect(advertisedSender(wire), await didOf(alice));
      expect(
        unpackTrustTask(
          wire,
          recipient: bob,
          allowedSenders: {await didOf(carol)},
        ),
        failsWith(EnvelopeFailure.senderNotAllowed),
      );
      final ok = await unpackTrustTask(
        wire,
        recipient: bob,
        allowedSenders: {await didOf(alice)},
      );
      expect(ok.transport.peer, await didOf(alice));
    });

    test('a foreign DIDComm type is refused, naming the sender', () async {
      final aliceDoc = await alice.getDidDocument();
      final bobDoc = await bob.getDidDocument();
      final keyId =
          aliceDoc.matchKeysInKeyAgreement(otherDidDocuments: [bobDoc]).first;
      final message = PlainTextMessage(
        id: 'm-1',
        type: Uri.parse('https://didcomm.org/basicmessage/2.0/message'),
        from: aliceDoc.id,
        to: [bobDoc.id],
        body: echoDoc(),
      );
      final wire = await EncryptedMessage.packWithAuthentication(
        message,
        keyPair: await alice.getKeyPairByDidKeyId(keyId),
        didKeyId: keyId,
        recipientDidDocuments: [bobDoc],
      );
      expect(
        unpackTrustTask(
          jsonDecode(jsonEncode(wire)) as Map<String, dynamic>,
          recipient: bob,
        ),
        throwsA(isA<EnvelopeException>()
            .having(
                (e) => e.failure, 'failure', EnvelopeFailure.wrongEnvelopeType)
            .having((e) => e.sender, 'sender', aliceDoc.id)),
      );
    });

    test('a body that is not a Trust Task document is invalidBody', () async {
      expect(
        unpackTrustTask(await packFor(bob, {'hello': 'world'}), recipient: bob),
        failsWith(EnvelopeFailure.invalidBody),
      );
    });
  });
}
