// Two VIDs exchanging a Trust Task over TSP, in one process.
//
// Run it:
//   dart run example/main.dart
//
// Alice seals a discovery query to Bob. TSP authenticates and encrypts in one
// step, so Bob learns who sent it from the cryptography, not from anything in
// the document — which is why Alice's document can leave `issuer` out. The same
// message then arrives a second time, as an intermediary re-forward would, and
// Bob absorbs it instead of executing it again (bindings/tsp/0.1 §7.1).

import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:ssi/ssi.dart';
import 'package:trust_tasks/specs/trust_task_discovery/v0_1/payload.dart'
    as discovery;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_tsp/trust_tasks_tsp.dart';

Future<PrivateVid> newVid() async {
  final wallet = PersistentWallet(InMemoryKeyStore());
  final manager = DidKeyManager(wallet: wallet, store: InMemoryDidStore());
  // TSP signing is Ed25519; the HPKE key agreement is derived from it.
  await wallet.generateKey(keyId: 'key-1', keyType: KeyType.ed25519);
  await manager.addVerificationMethod('key-1');
  return manager.toTspPrivateVid();
}

Future<void> main() async {
  final alice = await newVid();
  final bob = await newVid();
  final resolver = SsiVidResolver();
  final bobPub = await resolver.resolve(bob.id);

  // One consumer for the life of the process: its replay guard is the record.
  final bobConsumer = TspConsumer(recipient: bob, resolver: resolver);

  final query = TrustTaskDocument<discovery.Payload>(
    id: newUrnUuid(),
    type: discovery.typeUri,
    recipient: bob.id, // discovery requires an in-band recipient
    issuedAt: systemClock(),
    payload: const discovery.Payload(),
  );

  for (final delivery in ['delivery', 're-forward']) {
    // A re-forward is the same document resealed with fresh TSP material.
    final wire = await packTrustTask(
      query.toJson((p) => p.toJson()),
      sender: alice,
      receiver: bobPub,
    );

    final received = await bobConsumer
        .receive<discovery.Payload, discovery.Response>(
          wire,
          spec: discovery.spec,
          decode: discovery.Payload.fromJson,
          encode: (p) => p.toJson(),
          encodeResponse: (r) => r.toJson(),
          handler: (doc, parties) async {
            print(
              'bob: handling a query from ${parties.issuer == alice.id ? 'alice' : parties.issuer}',
            );
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

    // Back to Alice, sealed to the VID that authenticated.
    final reply = await unpackTrustTask(
      await bobConsumer.packReply(received),
      recipient: alice,
      sender: bobPub,
    );
    final payload = reply.document['payload'] as Map<String, dynamic>;
    print(
      'alice: ${reply.document['type']} from ${reply.transport.peer == bob.id ? 'bob' : reply.transport.peer}',
    );
    print('alice: supportedTypes ${payload['supportedTypes']}');
  }
}
