// Two agents exchanging a Trust Task over DIDComm v2.1, in one process.
//
// Run it:
//   dart run example/main.dart
//
// Alice asks Bob what he supports (trust-task-discovery). The envelope is
// authcrypt, so Bob learns who sent it from the cryptography, not from anything
// Alice wrote — which is why her document can leave `issuer` out. (It names
// Bob as `recipient` because discovery requires that in-band.) Then the same
// document arrives a second time, as a mediator redelivery would, and Bob
// absorbs it instead of executing it again (bindings/didcomm/0.2 §6.1).

import 'package:ssi/ssi.dart';
import 'package:trust_tasks/specs/trust_task_discovery/v0_1/payload.dart'
    as discovery;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_didcomm/trust_tasks_didcomm.dart';

Future<DidKeyManager> newAgent() async {
  final wallet = PersistentWallet(InMemoryKeyStore());
  final manager = DidKeyManager(wallet: wallet, store: InMemoryDidStore());
  await wallet.generateKey(keyId: 'key-1', keyType: KeyType.ed25519);
  await manager.addVerificationMethod('key-1');
  return manager;
}

Future<void> main() async {
  final alice = await newAgent();
  final bob = await newAgent();
  final aliceDid = (await alice.getDidDocument()).id;
  final bobDid = (await bob.getDidDocument()).id;

  // One consumer for the life of the process: its replay guard is the record.
  final bobConsumer = DidcommConsumer(recipient: bob);

  final query = TrustTaskDocument<discovery.Payload>(
    id: newUrnUuid(),
    type: discovery.typeUri,
    recipient: bobDid,
    issuedAt: systemClock(),
    payload: const discovery.Payload(),
  );

  for (final delivery in ['delivery', 'redelivery']) {
    // A redelivery is a fresh DIDComm message around the same document.
    final envelope = await packTrustTask(
      query.toJson((p) => p.toJson()),
      sender: alice,
      recipient: await bob.getDidDocument(),
    );

    final received =
        await bobConsumer.receive<discovery.Payload, discovery.Response>(
      envelope,
      spec: discovery.spec,
      decode: discovery.Payload.fromJson,
      encode: (p) => p.toJson(),
      encodeResponse: (r) => r.toJson(),
      handler: (doc, parties) async {
        print(
            'bob: handling a query from ${parties.issuer == aliceDid ? 'alice' : parties.issuer}');
        return respondWith<discovery.Payload, discovery.Response>(
          doc,
          newUrnUuid(),
          const discovery.Response(
            supportedTypes: [discovery.typeUri],
            frameworkVersion: '0.5',
          ),
        );
      },
    );

    print('bob: $delivery -> ${received.outcome.runtimeType}');
    if (received.reply == null) continue;

    // Back to Alice, authcrypt, addressed to the DID that authenticated.
    final replyEnvelope = await bobConsumer.packReply(received);
    final reply = await unpackTrustTask(
      replyEnvelope,
      recipient: alice,
      allowedSenders: {bobDid},
    );
    final payload = reply.document['payload'] as Map<String, dynamic>;
    print('alice: ${reply.document['type']} from ${reply.transport.peer}');
    print('alice: supportedTypes ${payload['supportedTypes']}');
  }
}
