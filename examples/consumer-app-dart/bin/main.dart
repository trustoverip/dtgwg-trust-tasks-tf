// An application that speaks three Trust Tasks and generates only those.
//
// `trust-tasks.lock` pins acl/grant/0.2, keys/list/0.2 and vault/get/0.3; their
// registry bundles are vendored under `trust-tasks/`. The libraries for exactly
// those three are generated into lib/trust_tasks/ (git-ignored):
//
//   dart pub get
//   dart run trust_tasks:generate --lock trust-tasks.lock --out lib/trust_tasks
//   dart run bin/main.dart
//
// The generator checks every bundle against its locked digest first. Nothing
// here imports `package:trust_tasks/specs/...`: the runtime comes from the
// package, the task types from this application.
//
// It plays both parties in one process: a producer builds a request and
// serializes it; a consumer parses the bytes, applies the version's policy
// through consumeInbound, and answers.

import 'dart:convert';

import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_example_consumer_app/trust_tasks/acl/grant/v0_2/payload.dart'
    as grant;
import 'package:trust_tasks_example_consumer_app/trust_tasks/extends_index.dart'
    as locked;
import 'package:trust_tasks_example_consumer_app/trust_tasks/keys/list/v0_2/payload.dart'
    as keys_list;
import 'package:trust_tasks_example_consumer_app/trust_tasks/vault/get/v0_3/payload.dart'
    as vault_get;

const consumer = 'did:web:vta.example';
const producer = 'did:web:root-admin.example';
final now = DateTime.utc(2026, 1, 1);
String clock() => '2026-01-01T00:00:00Z';

void check(bool ok, String what) {
  if (!ok) throw StateError('example failed: $what');
}

Future<void> main() async {
  // The policy each bundle declares is on the generated constants.
  check(keys_list.typeUri == 'https://trusttasks.org/spec/keys/list/0.2',
      'keys/list Type URI');
  check(grant.spec.isProofRequired, 'acl/grant requires a proof');
  check(!keys_list.spec.isProofRequired, 'keys/list does not');
  check(vault_get.spec.isRecipientRequired, 'vault/get requires a recipient');
  // None of the three declares `extends`.
  check(locked.extendsIndex.isEmpty, 'empty extends index');

  // 1. keys/list — produced, serialized, consumed, answered.
  final request = TrustTaskDocument<keys_list.Payload>(
    id: 'urn:uuid:00000000-0000-4000-8000-000000000001',
    type: keys_list.typeUri,
    issuer: producer,
    recipient: consumer,
    issuedAt: clock(),
    payload: const keys_list.Payload(
      status: keys_list.KeyStatus.active,
      limit: 10,
    ),
  );
  final wire = jsonEncode(request.toJson((p) => p.toJson()));

  final outcome = await consumeInbound<keys_list.Payload, keys_list.Response>(
    transport: const StaticTransport(TransportContext(issuer: producer)),
    spec: keys_list.spec,
    proofPolicy: const ProofPolicy.rejectIfPresent(),
    payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
    checks: notConsequentialChecks(),
    doc: TrustTaskDocument<keys_list.Payload>.fromJson(
      jsonDecode(wire) as Map<String, dynamic>,
      (p) => keys_list.Payload.fromJson(p! as Map<String, dynamic>),
    ),
    myVid: consumer,
    now: now,
    newErrorId: () => 'urn:uuid:reply-error',
    payloadToJson: (p) => p.toJson(),
    clock: clock,
    handler: (accepted, parties) async {
      check(accepted.payload.status == keys_list.KeyStatus.active,
          'closed value read back');
      check(accepted.payload.limit == 10, 'limit read back');
      return respondWith<keys_list.Payload, keys_list.Response>(
        accepted,
        'urn:uuid:reply-keys',
        const keys_list.Response(keys: []),
        clock: clock,
      );
    },
  );
  check(outcome is Handled<keys_list.Response>, 'keys/list handled');
  final response = (outcome as Handled<keys_list.Response>).response;
  print('keys/list  -> ${response.type}');
  check(response.type == keys_list.responseTypeUri, 'response type');
  check(response.threadId == request.id, 'response threads the request');

  // The response crosses the wire too, and reads back as the generated type.
  final reply = TrustTaskDocument<keys_list.Response>.fromJson(
    jsonDecode(jsonEncode(response.toJson((r) => r.toJson())))
        as Map<String, dynamic>,
    (p) => keys_list.Response.fromJson(p! as Map<String, dynamic>),
  );
  check(reply.payload.keys.isEmpty, 'empty key list round-trips');

  // 2. acl/grant — the specification REQUIRES a proof, so a proofless
  //    document is refused before any handler runs.
  final unsigned = TrustTaskDocument<grant.Payload>(
    id: 'urn:uuid:00000000-0000-4000-8000-000000000003',
    type: grant.typeUri,
    issuer: producer,
    recipient: consumer,
    issuedAt: clock(),
    payload: const grant.Payload(
      entry: grant.AclEntry(
        subject: 'did:web:alice.example',
        role: 'admin',
        act: {'scope': 'all'},
        capabilities: {'scope': 'ceiling'},
        keys: {'scope': 'all'},
      ),
    ),
  );
  final refused = await consumeInbound<grant.Payload, grant.Response>(
    transport: const StaticTransport(TransportContext(issuer: producer)),
    spec: grant.spec,
    proofPolicy: const ProofPolicy.rejectIfPresent(),
    payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
    checks: notConsequentialChecks(),
    doc: unsigned,
    myVid: consumer,
    now: now,
    newErrorId: () => 'urn:uuid:reply-grant',
    payloadToJson: (p) => p.toJson(),
    clock: clock,
    handler: (accepted, parties) async =>
        throw StateError('a proofless acl/grant reached the handler'),
  );
  check(refused is Rejected<grant.Response>, 'acl/grant refused');
  final code = (refused as Rejected<grant.Response>).error.payload.code;
  print('acl/grant  -> refused, code=$code');
  check(code == StandardCode.proofRequired.value, 'proofRequired');

  // 3. vault/get — a payload round-trips through JSON.
  const get = vault_get.Payload(id: 'entry-42');
  final back = vault_get.Payload.fromJson(
      jsonDecode(jsonEncode(get.toJson())) as Map<String, dynamic>);
  check(back.id == 'entry-42', 'vault/get payload round-trips');
  print('vault/get  -> payload round-trips');

  // 4. A version this application did not lock is refused, not guessed at.
  final other = readForVersion<keys_list.Payload>(
    {
      'id': 'urn:uuid:00000000-0000-4000-8000-000000000004',
      'type': 'https://trusttasks.org/spec/keys/list/0.1',
      'payload': <String, dynamic>{},
    },
    keys_list.spec,
    (p) => keys_list.Payload.fromJson(p! as Map<String, dynamic>),
    index: locked.extendsIndex,
  );
  check(other is VersionRefused<keys_list.Payload>, 'keys/list 0.1 refused');
  print('keys/list/0.1 -> refused (not locked)');
}
