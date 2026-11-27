// Interop with other TSP implementations, on fixed messages. All three carry
// the binding's {type, document} envelope from did:example:alice to
// did:example:bob, sealed with the same parties' keys: alice signs with the
// Ed25519 seed of 32 0x01 bytes (her X25519 key is 32 0x65 bytes), and bob's
// X25519 private key is 32 0x66 (102) bytes. They are the fixtures
// trust-tasks-ts-tsp's test/fixtures.ts uses (#1387), plus a Rev 3 message from
// the affinidi-tsp-go this repository's Go binding required before TSP 1.0.
// HPKE draws a fresh ephemeral key on every seal, so none is reproducible byte
// for byte.
import 'dart:typed_data';

import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_tsp/trust_tasks_tsp.dart';

import 'support/fixtures.dart';

/// A TSP 1.0 (`YTSP-BAA`) Direct message packed by the Rust `affinidi-tsp`
/// 0.5.0 — the crate the Rust `trust-tasks-tsp` binding seals with — through
/// `affinidi_tsp::message::direct::pack(body, MessageType::Direct,
/// "did:example:alice", "did:example:bob", &alice_signing_seed,
/// &bob_x25519_public)`, with `body` the `serde_json` bytes of the envelope.
/// The generating program unpacked it with the same crate before printing it.
const rustTsp10Hex =
    'f8408061348ff81000e41006006469643a6578616d706c653a616c696365e010'
    '056469643a6578616d706c653a626f62e05070dcf39858172f2f081019d8b304'
    '436fcd20facc9141e75b3f6c86ebcb0c958f417d4b830b6e10d4ecbf10e157ab'
    'a5e3c6617f8b29f4be9feb9afa07504ee530c8906b7219e42bc1b37295d3b39f'
    '52bc5c806ad316e1e20a7e8237343c6b14cf72e92172a6f5cc30999beb2a3eff'
    'cb88c18889dff54591796a86b8f9dc82142056d94a965154f737e6115be11a0e'
    'dc7600ced661fbb77ac2de0c8a84f9c87079b2d3aa4284b05dc5005fdc62d8ee'
    'e75ff181d57c0330a2abe170ff8bbb3dc81a63bb2db5fa23289d773b1da9411e'
    '5ac348b26453980b97af243cbe4e768637d2bb3c5092f28cdf84eb600e985cf7'
    '4dc17f3a3ba2d89b97e4a313ee9d26e5b468f0df1bb0561ae1dc35499b840b88'
    'd2b756ca212c93702480ec3a8e5eabc69ffff53b316ad6c7495f596d7681eee3'
    '1ae7894348cae252c4971e4990155645751bbec2e53dfcad96902eb8e607c32a'
    'ad7c64f82017f8a0160400f907598aae27a4762637af72f2f233c9e4b34f2bcd'
    'ea5c60b465fc32720fe0521ae78280fb57dd0268ea35ac08e1dae08152e5ac7e'
    '6116d1c8aad18b1ea0c703';

/// The document [rustTsp10Hex] carries.
const rustTsp10Document = <String, dynamic>{
  'id': 'urn:uuid:00000000-0000-4000-8000-0000000000a1',
  'type': 'https://trusttasks.org/spec/example/echo/0.1',
  'issuedAt': '2026-06-01T12:00:00Z',
  'payload': <String, dynamic>{'text': 'from affinidi-tsp'},
};

/// A TSP Rev 3 (`YTSP-AAC`) Direct message packed by `affinidi-tsp-go`
/// v0.0.0-20260916130837-d32ca7cdde20 — the pseudo-version
/// `trust-tasks-go/tsp` required before TSP 1.0, and the same Rev 3 as the
/// `affinidi_tsp` git commit this package used — with HPKE-Base. It stands for
/// a Go or Dart peer of this repository that has not upgraded. The generating
/// program opened it with the same library first.
const goRev3Hex =
    'f8407d61348ff80002e41006006469643a6578616d706c653a616c696365e010'
    '056469643a6578616d706c653a626f62e0506d05e98823214d48869ab16c103d'
    'a5127bb8a9ae817f0056d92531695cc2ee8a776c43cdaf474e235166b8ab51f1'
    '9703845f86e04d492bf2d1cbb16c2227de3c8fb536b2aa279d41c8703e00ce4e'
    '9d218f82b4357bd855304bbc85752371928a219380cc802c678bf64317c273d0'
    'fe1099a55c342289e348dce91a554b80d3bbf651f5e80c5b041bcf66fc9bc91e'
    'e017fb6121e5e499cf62fc715336768031399dd85677b26159fcc427e4ce41e8'
    'f8667d7b3954f1c3adc9e8559740f7fbf4081db8344b16fd5c8cae1c1b6c5938'
    '7f47f020a8d9c8b5698ffe0fd88b266c0f5f6cb47c32aa544cfac271b970a6de'
    'f8645b728d97ac4c394804e95c61738051b7bdcb7409ccdf264e2c71d6ade965'
    '1e0e740344e83e998d8cb34a3b7cdcd3a1f12556f66292b0f055f93c6ee0ebe7'
    'b479cab7e9cb8ade8fd38257ee0fa481a11f4c7de8713ae13bb3f82017f8a016'
    '04000d3c3aa590b545eb838f579da49ba05918bf6554dd3ef8d8a51aa0890358'
    '2974d78dae43e304e1ac0ad03a3eb4f3d6b8a7111a56e69cf348d45ab64d64a6'
    'a205';

/// A TSP Rev 2 (`YTSP-AAB`) Direct message, HPKE-Auth sealed, packed by
/// `@openvtc/vti-tsp-js` 0.2.0: #1387's `VTI_TSP_JS_0_2_REV2_HEX`.
const vtiTspJsRev2Hex =
    'f8401061348ff80001e41006006469643a6578616d706c653a616c696365e010'
    '056469643a6578616d706c653a626f625c0000e060674a8d67c84e02956663e9'
    'b8349532d9117d55f25a757553e916663b5067b1c085d81341b8239eaceaa32b'
    'd58cc2708cd5bfc53db86d409e8c1230e01081b0b5bb4e3d4724c66ddc1ef7c1'
    '849ce124148ec3f1a012a638e0c321826e0df592c2dfc64ac622d974bc101260'
    '8670a2b7ca51392a65332cb3c740b5408c8ab9d74a474d07c3a71ffc2f790700'
    '01e17d6b650b38dfb08f5283ecb8647114564bf62207e869afa01491ed0b49dc'
    '0dc2ffe970d12de025743b0595ff1594ab95816421d7e42cec61b55d57c83fa8'
    'be372d85f5c76f1a4b1b661d0695d366ceeb4d6760ec9e5c83a7eff71c959b18'
    'd68f7ff6ea6f64c419ac02493b4a14d6f99d076e1d3140ae44d00c8551d34a54'
    'abd5ff4f310f7e7b198f4a85086ce1dd5cca1bf1c002ed041d527b116f2d350a'
    '54989bfdc65e3597552b29f82016f8a016d010b694a72107551cc44102175cd5'
    '9e4eaf4913ae3460707dac1f634e01bb289f4026f862d60930944ea09b62f2d6'
    '21be936fb429118571de7038951feaa071ef05';

Uint8List fromHex(String hex) => Uint8List.fromList([
  for (var i = 0; i < hex.length; i += 2)
    int.parse(hex.substring(i, i + 2), radix: 16),
]);

List<int> fill(int byte) => List<int>.filled(32, byte);

void main() {
  late PrivateVid alice;
  late PublicVid alicePub;
  late PrivateVid bob;
  late PublicVid bobPub;
  late TspConsumer consumer;
  late List<String> calls;

  setUpAll(() async {
    final aliceSigning = Ed25519SigningKey.fromSeed(fill(0x01));
    final aliceDecryption = X25519DecryptionKey.fromSecret(fill(0x65));
    alice = PrivateVid(
      id: 'did:example:alice',
      signingKey: aliceSigning,
      decryptionKey: aliceDecryption,
    );
    alicePub = PublicVid(
      id: alice.id,
      verificationKey: await aliceSigning.verificationKey(),
      encryptionKey: X25519EncryptionKey(aliceDecryption.publicKey),
    );
    final bobSigning = Ed25519SigningKey.fromSeed(fill(0x02));
    final bobDecryption = X25519DecryptionKey.fromSecret(fill(0x66));
    bob = PrivateVid(
      id: 'did:example:bob',
      signingKey: bobSigning,
      decryptionKey: bobDecryption,
    );
    bobPub = PublicVid(
      id: bob.id,
      verificationKey: await bobSigning.verificationKey(),
      encryptionKey: X25519EncryptionKey(bobDecryption.publicKey),
    );
  });

  setUp(() {
    calls = [];
    consumer = TspConsumer(
      recipient: bob,
      resolver: MapResolver({alice.id: alicePub, bob.id: bobPub}),
      clock: () => fixedNow,
    );
  });

  Future<Received<Echo>> receive(Uint8List wire) =>
      consumer.receive<Echo, Echo>(
        wire,
        spec: echoSpec,
        decode: Echo.fromJson,
        encode: (p) => p.toJson(),
        encodeResponse: (r) => r.toJson(),
        handler: (doc, parties) async {
          calls.add(doc.payload.text);
          return respondWith<Echo, Echo>(doc, newUrnUuid(), const Echo('ok'));
        },
      );

  Matcher failsWith(TspFailure failure) => throwsA(
    isA<TspEnvelopeException>().having((e) => e.failure, 'failure', failure),
  );

  test('seals TSP 1.0', () async {
    expect(tspVersion, '1.0');
    final wire = await packTrustTask(
      rustTsp10Document,
      sender: alice,
      receiver: bobPub,
    );
    final version = Tsp.peek(wire).version;
    expect('${version.major}.${version.minor}', tspVersion);
  });

  test('the fixture keys are the ones the Rust fixture was sealed with', () {
    // alice's Ed25519 public key and bob's X25519 public key, as #1387 and the
    // generating programs record them.
    expect(
      (alicePub.verificationKey as Ed25519VerificationKey).bytes,
      fromHex(
        '8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c',
      ),
    );
    expect(
      (bobPub.encryptionKey! as X25519EncryptionKey).bytes,
      fromHex(
        '219e4d800da968d2a5fcb009c784f4746c7138edb9ee4844b739e830b05cf424',
      ),
    );
  });

  test('a TSP 1.0 message packed by the Rust affinidi-tsp opens', () async {
    final wire = fromHex(rustTsp10Hex);
    expect(advertisedSender(wire), alice.id);
    final unpacked = await unpackTrustTask(
      wire,
      recipient: bob,
      sender: alicePub,
    );
    expect(unpacked.document, rustTsp10Document);
    expect(unpacked.transport.peer, alice.id);
    expect(unpacked.transport.local, bob.id);
  });

  test('consumer: the Rust-packed TSP 1.0 message runs the pipeline and is '
      'answered in TSP 1.0', () async {
    final r = await receive(fromHex(rustTsp10Hex));
    expect(r.outcome, isA<Handled<Echo>>());
    expect(calls, ['from affinidi-tsp']);
    final reply = await consumer.packReply(r);
    expect(Tsp.peek(reply).version.major, 1);
    final back = await unpackTrustTask(reply, recipient: alice, sender: bobPub);
    expect(back.document['type'], '$echoType#response');
  });

  // A peer still on a TSP draft is refused from the version marker, before any
  // key is used or its sender resolved. Its sender cannot be read, so nothing
  // is authenticated and nothing is sent back. TSP has no negotiation.
  for (final (name, hex) in [
    ('Rev 3 from affinidi-tsp-go d32ca7cd', goRev3Hex),
    ('Rev 2 from vti-tsp-js 0.2', vtiTspJsRev2Hex),
  ]) {
    group('a peer on a TSP draft ($name)', () {
      test('is refused unsupportedTspVersion by unpackTrustTask', () async {
        final wire = fromHex(hex);
        expect(advertisedSender(wire), isNull);
        await expectLater(
          unpackTrustTask(wire, recipient: bob, sender: alicePub),
          throwsA(
            isA<TspEnvelopeException>()
                .having(
                  (e) => e.failure,
                  'failure',
                  TspFailure.unsupportedTspVersion,
                )
                .having((e) => e.sender, 'sender', isNull)
                .having((e) => e.detail, 'detail', contains('TSP 1.0')),
          ),
        );
      });

      test('is refused unsupportedTspVersion by the consumer, before the '
          'handler', () async {
        await expectLater(
          receive(fromHex(hex)),
          failsWith(TspFailure.unsupportedTspVersion),
        );
        expect(calls, isEmpty);
      });
    });
  }

  // MINOR never gates a message (TSP 1.0 §3.1): a 1.x message with a MINOR this
  // binding does not know is opened like any other, so one that then fails to
  // open is notForThisReceiver, not a version refusal. Bytes 6..8 are the
  // version code, f8 10 00 for 1.0; the last carries MINOR's low bits. The
  // signature covers the header, so the bumped message no longer verifies.
  test('a TSP 1.x message with an unknown MINOR that does not open is '
      'notForThisReceiver', () async {
    final wire = await packTrustTask(
      rustTsp10Document,
      sender: alice,
      receiver: bobPub,
    );
    expect(wire.sublist(6, 9), [0xf8, 0x10, 0x00]);
    final bumped = Uint8List.fromList(wire)..[8] = 0x05;
    final version = Tsp.peek(bumped).version;
    expect('${version.major}.${version.minor}', '1.5');
    expect(advertisedSender(bumped), alice.id);
    await expectLater(
      unpackTrustTask(bumped, recipient: bob, sender: alicePub),
      failsWith(TspFailure.notForThisReceiver),
    );
    await expectLater(
      receive(bumped),
      failsWith(TspFailure.notForThisReceiver),
    );
    expect(calls, isEmpty);
  });
}
