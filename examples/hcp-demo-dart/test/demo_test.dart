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
  test('1. the eight steps print exactly the golden transcript '
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
    final before = world.held.length;
    final reply = await world.exchange(
      m,
      p,
      await world.request(m, p, requestType, ask),
    );
    expect(codeOf(reply), 'permissionDenied');
    expect(world.held, hasLength(before));
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
      'grant with a new token, never a second grant', () async {
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
    final granted = payloadOf(reply);
    expect((granted['grant'] as Map)['grantId'], grantId);
    expect(world.grants, hasLength(1));
    // A newly issued token under the same grant: the provider keeps no copy
    // of the one it sent, only digests.
    final second = granted['accessToken'] as String;
    expect(second, 'hcp_at_demo_0002');
    expect(granted['tokenExpiresAt'], (granted['grant'] as Map)['expiresAt']);
    expect(world.grants[0].tokenDigests, {
      tokenDigest(token),
      tokenDigest(second),
    });
    // Both tokens are good until the grant ends, and neither is once it is
    // revoked.
    Future<Map<String, dynamic>> send(String t) async => world.exchange(
      c,
      p,
      await world.request(c, p, queryType, queryPayload(t)),
    );
    for (final t in [token, second]) {
      expect((await send(t))['type'], query.responseTypeUri);
    }
    await stepRevoke(world, grantId);
    for (final t in [token, second]) {
      expect(codeOf(await send(t)), 'hcp:invalidToken');
    }
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

  test('14. another registered client resubmitting on a held thread is '
      'refused with permissionDenied, and the hold is unchanged', () async {
    final world = await World.create();
    final (c, m, p) = (world.contextClient, world.management, world.provider);
    // A second registered client, so the check on the hold's issuer is
    // reached (rule 1 passes).
    world.registeredClients[m.vid] = (
      name: "Alex's wallet",
      operator: const grant_request.Named(
        id: 'wallet.example',
        name: 'Wallet Example Ltd',
      ),
    );
    final first = await world.request(c, p, requestType, ask);
    final held = await world.exchange(c, p, first);
    expect(
      held['type'],
      'https://trusttasks.org/spec/trust-task-next-step/0.1',
    );
    final threadId = first['threadId'] as String;
    world.personApproves(threadId);
    final reply = await world.exchange(
      m,
      p,
      await world.request(m, p, requestType, ask, threadId: threadId),
    );
    expect(reply['type'], errorType);
    expect(codeOf(reply), 'permissionDenied');
    final hold = world.held[threadId]!;
    expect(hold.issuer, c.vid);
    expect(hold.state, 'approved');
    expect(world.grants, isEmpty);
  });

  test("15. the hold's hint is read against NextStepHint, and a hint that "
      'fails is ignored as if absent', () {
    expect(retryAfterOf({'retryAfter': 1}), 1);
    expect(retryAfterOf({'retryAfter': 3600}), 3600);
    expect(retryAfterOf(null), defaultRetryAfter);
    for (final hint in <Object?>[
      {'retryAfter': 2.5},
      {'retryAfter': 0},
      {'retryAfter': '5'},
      {'retryAfter': 3601},
      {'retryAfter': 1, 'extra': true},
      <String, Object?>{},
      null,
      5,
    ]) {
      expect(retryAfterOf(hint), defaultRetryAfter, reason: '$hint');
    }
  });

  test(
    "16. the grant's token lasts as long as the grant (no refresh in 0.1)",
    () async {
      final (world, _) = await afterGrant();
      expect(world.grants, hasLength(1));
      expect(world.grants[0].tokenExpiresAt, world.grants[0].expiresAt);
      expect(world.grants[0].expiresAt, '2026-11-25T14:00:06Z');
    },
  );

  Map<String, dynamic> denial(Map<String, dynamic> refused) => {
    'code': 'permissionDenied',
    'message': 'The request is not authorized.',
    'retryable': false,
    'inResponseTo': payloadOf(refused)['inResponseTo'],
  };

  test(
    '17. after a decline the next resubmission is refused with the fixed '
    'permissionDenied, the thread never reopens, and no grant is issued',
    () async {
      final (world, result) = await afterDemo();
      final (c, p) = (world.contextClient, world.provider);
      final (:threadId, :refused) = result.declined;
      expect(refused['type'], errorType);
      expect(payloadOf(refused), denial(refused));
      expect(world.held[threadId]!.state, 'declined');
      expect(world.grants, hasLength(1)); // step 2's, revoked
      world.personApproves(threadId); // a later approval changes nothing
      expect(world.held[threadId]!.state, 'declined');
      final again = await world.exchange(
        c,
        p,
        await world.request(c, p, requestType, ask, threadId: threadId),
      );
      expect(codeOf(again), 'permissionDenied');
      expect(world.grants, hasLength(1));
    },
  );

  test("18. once the hold's lifetime passes undecided the thread is refused "
      'with the fixed permissionDenied and never reopened; a new thread is put '
      'to the person again', () async {
    final (world, result) = await afterDemo();
    final (:threadId, :newThreadId, :refused) = result.expiredHold;
    expect(refused, hasLength(2));
    for (final r in refused) {
      expect(r['type'], errorType);
      expect(payloadOf(r), denial(r));
    }
    expect(world.held[threadId]!.state, 'expired');
    expect(world.held[newThreadId]!.state, 'pending');
    expect(world.grants, hasLength(1)); // step 2's, revoked

    // The lifetime counts from the held request's issuedAt: still pending
    // when it has not passed.
    final fresh = await World.create();
    final (c, p) = (fresh.contextClient, fresh.provider);
    final first = await fresh.request(c, p, requestType, ask);
    await fresh.exchange(c, p, first);
    final firstThread = first['threadId'] as String;
    // The resubmission is issued exactly holdLifetime after the request.
    fresh.clock.advance(holdLifetime - 2);
    final held = await fresh.exchange(
      c,
      p,
      await fresh.request(c, p, requestType, ask, threadId: firstThread),
    );
    expect(
      held['type'],
      'https://trusttasks.org/spec/trust-task-next-step/0.1',
    );
    expect(fresh.held[firstThread]!.state, 'pending');
  });

  test("19. a copy's useUntil is the grant's expiresAt where the grant ends "
      'inside the hour', () async {
    final (world, _) = await afterGrant();
    final (c, p) = (world.contextClient, world.provider);
    // The grant ends at 14:00:06; the query is issued at 13:30:08.
    world.clock.advance(12600);
    final reply = await world.exchange(
      c,
      p,
      await world.request(c, p, queryType, queryPayload(token)),
    );
    expect(reply['type'], query.responseTypeUri);
    final entries = payloadOf(reply)['entries'] as List;
    expect(entries, isNotEmpty);
    for (final entry in entries) {
      // Not now plus 1 hour, 14:30:08.
      expect((entry as Map)['useUntil'], '2026-11-25T14:00:06Z');
    }
  });
}
