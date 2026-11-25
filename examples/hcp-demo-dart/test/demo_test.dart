// The demo's own tests: hermetic (no network, no files outside the examples
// directory), deterministic (fixed keys, ids and clock), and run with
// `dart test`. ../hcp-demo-ts/SCENARIO.md "Tests every port has" lists them;
// every port has the same ones.

import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks_example_hcp_demo/demo.dart';
import 'package:trust_tasks_example_hcp_demo/trust_tasks/hcp/entries/query/v0_1/payload.dart'
    as query;
import 'package:trust_tasks_example_hcp_demo/trust_tasks/hcp/grants/request/v0_1/payload.dart'
    as grant_request;
import 'package:trust_tasks_example_hcp_demo/trust_tasks/hcp/grants/revoked/v0_1/payload.dart'
    as revoked;

/// The golden transcript every language's port prints, owned by the TypeScript
/// demo. Compared with, never copied.
final File golden = File('../hcp-demo-ts/transcript.golden.txt');

const String errorType = 'https://trusttasks.org/spec/trust-task-error/0.6';

/// The grant's token, as the provider minted it.
const String token = 'hcp_at_demo_0001';
const String requestType = '${grant_request.typeUri}#request';
const String queryType = '${query.typeUri}#request';

Map<String, dynamic> payloadOf(Map<String, dynamic> reply) =>
    reply['payload'] as Map<String, dynamic>;

String codeOf(Map<String, dynamic> reply) => payloadOf(reply)['code'] as String;

/// A world after the whole demo has run.
Future<(World, DemoResult)> afterDemo() async {
  final world = await World.create();
  return (world, await runDemo(world));
}

/// A world after steps 1 and 2: the grant is issued and active.
Future<(World, ({String threadId, String grantId}))> afterGrant() async {
  final world = await World.create();
  await stepDiscover(world);
  return (world, await stepGrantRequest(world));
}

void main() {
  test('1. the six steps print exactly the golden transcript '
      "(shared by every language's port)", () async {
    final result = await runDemo();
    expect(result.transcript, golden.readAsStringSync());
  });

  test(
    '2. step 5, a query after revoke, fails with hcp:invalidToken (HCP 6.2.2)',
    () async {
      final result = await runDemo();
      expect(result.postRevokeError['type'], errorType);
      expect(codeOf(result.postRevokeError), 'hcp:invalidToken');
    },
  );

  test(
    '3. a document altered after signing is refused with proofInvalid',
    () async {
      final (world, _) = await afterDemo();
      final (c, p) = (world.contextClient, world.provider);
      final signed = await world.request(
        c,
        p,
        queryType,
        queryPayload(managementToken),
      );
      final tampered = {
        ...signed,
        'payload': {...payloadOf(signed), 'query': 'Tell me everything.'},
      };
      final reply = await world.exchange(c, p, tampered);
      expect(codeOf(reply), 'proofInvalid');
    },
  );

  test('4. a correctly signed document that fails its schema is refused with '
      'malformedRequest', () async {
    final (world, _) = await afterDemo();
    final (c, p) = (world.contextClient, world.provider);
    final noPurpose = queryPayload(managementToken)..remove('purpose');
    final reply = await world.exchange(
      c,
      p,
      await world.request(c, p, queryType, noPurpose),
    );
    expect(codeOf(reply), 'malformedRequest');
  });

  test('5. after the notice the client holds no token and has recorded the '
      'notice; a later attempt is acknowledged, not acted on', () async {
    final (world, result) = await afterDemo();
    final (p, c) = (world.provider, world.contextClient);
    final grantId = result.grantId;
    final notice = result.notice;
    expect(world.wallet.containsKey(grantId), isFalse);
    expect(world.processedNotices, {payloadOf(notice)['noticeId']: grantId});
    // A later delivery attempt: a new document, the same threadId and payload
    // (noticeId included).
    final ack = await world.exchange(
      p,
      c,
      await world.request(
        p,
        c,
        '${revoked.typeUri}#request',
        payloadOf(notice),
        threadId: notice['threadId'] as String,
      ),
    );
    expect(ack['type'], revoked.responseTypeUri);
    expect(payloadOf(ack), isEmpty);
    expect(world.processedNotices, hasLength(1));
  });

  test(
    "6. the notice alone makes the client drop the grant's token and copies",
    () async {
      final (world, (threadId: _, :grantId)) = await afterGrant();
      await stepRevoke(world, grantId);
      expect(world.wallet.containsKey(grantId), isTrue);
      await stepNotice(world, grantId);
      expect(world.wallet.containsKey(grantId), isFalse);
    },
  );

  test("7. the grant's token presented from another VID is refused with "
      'permissionDenied', () async {
    final (world, _) = await afterGrant();
    final (m, p) = (world.management, world.provider);
    final reply = await world.exchange(
      m,
      p,
      await world.request(m, p, queryType, queryPayload(token)),
    );
    expect(reply['type'], errorType);
    expect(codeOf(reply), 'permissionDenied');
  });

  test('8. a grant request from an issuer that is not a registered client is '
      'refused with permissionDenied', () async {
    final (world, _) = await afterDemo();
    final (m, p) = (world.management, world.provider);
    final reply = await world.exchange(
      m,
      p,
      await world.request(m, p, requestType, ask),
    );
    expect(codeOf(reply), 'permissionDenied');
    expect(world.held, hasLength(1));
  });

  test('9. a resubmission whose payload differs from the held request is '
      'refused with malformedRequest', () async {
    final (world, (:threadId, grantId: _)) = await afterGrant();
    final (c, p) = (world.contextClient, world.provider);
    final reply = await world.exchange(
      c,
      p,
      await world.request(c, p, requestType, {
        ...ask,
        'expiresIn': 3600,
      }, threadId: threadId),
    );
    expect(codeOf(reply), 'malformedRequest');
  });

  test('10. within the hour after approval a resubmission gets the same '
      'grant, never a second one', () async {
    final (world, (:threadId, :grantId)) = await afterGrant();
    final (c, p) = (world.contextClient, world.provider);
    // The same payload with its members in another order: the provider
    // compares JCS forms.
    final reordered = <String, dynamic>{
      'subjectHint': ask['subjectHint'],
      'expiresIn': ask['expiresIn'],
      'access': ask['access'],
    };
    final reply = await world.exchange(
      c,
      p,
      await world.request(c, p, requestType, reordered, threadId: threadId),
    );
    expect(reply['type'], grant_request.responseTypeUri);
    expect((payloadOf(reply)['grant'] as Map)['grantId'], grantId);
    expect(world.grants, hasLength(1));
  });

  test(
    '11. a resubmission after the hour is refused with permissionDenied',
    () async {
      final (world, (:threadId, grantId: _)) = await afterGrant();
      final (c, p) = (world.contextClient, world.provider);
      world.clock.advance(afterApprovalWindow + 1);
      final reply = await world.exchange(
        c,
        p,
        await world.request(c, p, requestType, ask, threadId: threadId),
      );
      expect(codeOf(reply), 'permissionDenied');
    },
  );

  test('12. a resubmission once the grant is revoked is refused with '
      'permissionDenied, and no grant is issued', () async {
    final (world, result) = await afterDemo();
    final (c, p) = (world.contextClient, world.provider);
    final reply = await world.exchange(
      c,
      p,
      await world.request(c, p, requestType, ask, threadId: result.threadId),
    );
    expect(codeOf(reply), 'permissionDenied');
    expect(world.grants, hasLength(1));
  });

  test('13. a reply is used only if it comes from the party asked, on the '
      'same thread, and is signed unless the library built it', () async {
    final world = await World.create();
    final (p, c) = (world.provider, world.contextClient);
    Map<String, dynamic> reply(String code, {bool signed = false}) => {
      'id': 'urn:uuid:00000000-0000-4000-8000-0000000000f0',
      'type': errorType,
      'issuer': p.vid,
      'recipient': c.vid,
      'issuedAt': '2026-11-25T10:00:01Z',
      'threadId': 't',
      'payload': {'code': code},
      // Presence only: the proof itself is verified separately.
      if (signed) 'proof': {'type': 'DataIntegrityProof'},
    };
    // A library-built error, unsigned.
    checkReply(reply('proofInvalid'), p, 't');
    checkReply(reply('permissionDenied', signed: true), p, 't');
    expect(
      () => checkReply(reply('permissionDenied'), p, 't'),
      throwsA(predicate((e) => '$e'.contains('no proof'))),
    );
    expect(
      () => checkReply(reply('proofInvalid'), c, 't'),
      throwsA(predicate((e) => '$e'.contains('issuer'))),
    );
    expect(
      () => checkReply(reply('proofInvalid'), p, 'other'),
      throwsA(predicate((e) => '$e'.contains('threadId'))),
    );
  });
}
