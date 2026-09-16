import 'dart:convert';
import 'dart:typed_data';

import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:ssi/ssi.dart';
import 'package:test/test.dart';
import 'package:trust_tasks_tsp/trust_tasks_tsp.dart';

import 'support/fixtures.dart';

Matcher failsWith(TspFailure failure) => throwsA(
  isA<TspEnvelopeException>().having((e) => e.failure, 'failure', failure),
);

void main() {
  final ssi = SsiVidResolver();

  group('round trip', () {
    late PrivateVid alice;
    late PrivateVid bob;
    late PublicVid alicePub;
    late PublicVid bobPub;
    setUp(() async {
      alice = await identity(KeyType.ed25519);
      bob = await identity(KeyType.ed25519);
      alicePub = await ssi.resolve(alice.id);
      bobPub = await ssi.resolve(bob.id);
    });

    test('carries the authenticated sender and recipient', () async {
      final doc = echoDoc();
      final wire = await packTrustTask(doc, sender: alice, receiver: bobPub);
      final unpacked = await unpackTrustTask(
        wire,
        recipient: bob,
        sender: alicePub,
      );
      expect(unpacked.document, doc);
      expect(unpacked.transport.peer, alice.id);
      expect(unpacked.transport.local, bob.id);
    });

    test('the sealed inner payload is exactly the Rust crate\'s envelope '
        'object', () async {
      // trust-tasks-tsp (Rust) seals `{"type": ENVELOPE_TYPE, "document": ...}`
      // and nothing else. Opening a Dart-sealed message and reading the raw
      // ScsPayload pins the two to the same shape, so a document sealed by
      // either opens in the other.
      final doc = echoDoc();
      final wire = await packTrustTask(doc, sender: alice, receiver: bobPub);
      final opened = await Tsp.open(wire, receiver: bob, sender: alicePub);
      final envelope =
          jsonDecode(utf8.decode((opened.payload as ScsPayload).data))
              as Map<String, dynamic>;
      expect(envelope.keys.toSet(), {'type', 'document'});
      expect(envelope['type'], envelopeType);
      expect(envelope['document'], doc);
    });

    test('the sealed payload is confidential to an intermediary', () async {
      final wire = await packTrustTask(
        echoDoc(text: 'secret'),
        sender: alice,
        receiver: bobPub,
      );
      expect(
        utf8.decode(wire, allowMalformed: true),
        isNot(contains('secret')),
      );
      final info = Tsp.peek(wire);
      expect(info.sender, alice.id);
      expect(info.confidential, isTrue);
      expect(advertisedSender(wire), alice.id);
    });
  });

  group('sealing', () {
    late PrivateVid alice;
    late PrivateVid bob;
    late PrivateVid carol;
    late PublicVid alicePub;
    late PublicVid bobPub;
    late PublicVid carolPub;
    setUp(() async {
      alice = await identity(KeyType.ed25519);
      bob = await identity(KeyType.ed25519);
      carol = await identity(KeyType.ed25519);
      alicePub = await ssi.resolve(alice.id);
      bobPub = await ssi.resolve(bob.id);
      carolPub = await ssi.resolve(carol.id);
    });

    test('a message sealed to bob does not open for carol', () async {
      final wire = await packTrustTask(
        echoDoc(),
        sender: alice,
        receiver: bobPub,
      );
      expect(
        unpackTrustTask(wire, recipient: carol, sender: alicePub),
        failsWith(TspFailure.notForThisReceiver),
      );
    });

    test('opening under the wrong sender VID fails', () async {
      final wire = await packTrustTask(
        echoDoc(),
        sender: alice,
        receiver: bobPub,
      );
      expect(
        unpackTrustTask(wire, recipient: bob, sender: carolPub),
        failsWith(TspFailure.notForThisReceiver),
      );
    });

    test('a tampered message does not open', () async {
      final wire = Uint8List.fromList(
        await packTrustTask(echoDoc(), sender: alice, receiver: bobPub),
      );
      wire[wire.length - 1] ^= 0xff;
      expect(
        unpackTrustTask(wire, recipient: bob, sender: alicePub),
        failsWith(TspFailure.notForThisReceiver),
      );
    });

    test('a signed-only message is refused as notSealed', () async {
      final packed = await Tsp.pack(
        sender: alice,
        receiver: bobPub,
        payload: ScsPayload(
          utf8.encode(
            jsonEncode({'type': envelopeType, 'document': echoDoc()}),
          ),
        ),
        scheme: TspScheme.signedOnly,
      );
      expect(
        unpackTrustTask(packed.bytes, recipient: bob, sender: alicePub),
        failsWith(TspFailure.notSealed),
      );
    });

    test('a foreign envelope type is refused, naming the sender', () async {
      final packed = await Tsp.pack(
        sender: alice,
        receiver: bobPub,
        payload: ScsPayload(
          utf8.encode(jsonEncode({'type': 'urn:other', 'document': echoDoc()})),
        ),
        scheme: TspScheme.hpkeBase,
      );
      expect(
        unpackTrustTask(packed.bytes, recipient: bob, sender: alicePub),
        throwsA(
          isA<TspEnvelopeException>()
              .having((e) => e.failure, 'failure', TspFailure.wrongEnvelopeType)
              .having((e) => e.sender, 'sender', alice.id),
        ),
      );
    });

    test(
      'a payload that is not a Trust Task document is invalidBody',
      () async {
        final packed = await Tsp.pack(
          sender: alice,
          receiver: bobPub,
          payload: ScsPayload(
            utf8.encode(
              jsonEncode({
                'type': envelopeType,
                'document': {'x': 1},
              }),
            ),
          ),
          scheme: TspScheme.hpkeBase,
        );
        expect(
          unpackTrustTask(packed.bytes, recipient: bob, sender: alicePub),
          failsWith(TspFailure.invalidBody),
        );
      },
    );

    test('packing to a receiver with no encryption key throws', () async {
      final signOnly = PublicVid(
        id: bobPub.id,
        verificationKey: bobPub.verificationKey,
      );
      expect(
        () => packTrustTask(echoDoc(), sender: alice, receiver: signOnly),
        throwsArgumentError,
      );
    });

    test('non-TSP bytes have no advertised sender', () {
      expect(advertisedSender(Uint8List.fromList(utf8.encode('{}'))), isNull);
    });
  });
}
