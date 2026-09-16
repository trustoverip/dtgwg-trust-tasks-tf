import 'dart:convert';

import 'package:test/test.dart';
import 'package:trust_tasks/specs/acl/grant/v0_1/payload.dart' as acl_grant;
import 'package:trust_tasks/specs/trust_task_discovery/v0_1/payload.dart'
    as discovery;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_https/trust_tasks_https.dart';

import 'support/fixtures.dart';

void main() {
  group('carriage (§2)', () {
    test('POST /trust-tasks with a registered type reaches the handler',
        () async {
      final calls = <String>[];
      final reply = await echoServer(calls: calls).handle(post(echoDoc()));
      expect(reply.status, 200);
      expect(reply.headers['content-type'], 'application/json');
      final body = bodyOf(reply);
      expect(body['type'], '$echoType#response');
      expect(body['threadId'], echoDoc()['id']);
      expect(body['issuer'], server);
      expect(body['recipient'], alice);
      expect((body['payload'] as Map)['text'], 'HELLO');
      expect(calls, ['hello']);
    });

    test('another path is 404, another method 405', () async {
      final s = echoServer();
      expect((await s.handle(post(echoDoc(), path: '/other'))).status, 404);
      final get = await s.handle(post(echoDoc(), method: 'GET'));
      expect(get.status, 405);
      expect(get.headers['allow'], 'POST');
    });

    test('a trailing slash on the path is ignored; a base path is honoured',
        () async {
      expect(
        (await echoServer().handle(post(echoDoc(), path: '/trust-tasks/')))
            .status,
        200,
      );
      final based = echoServer(basePath: '/tt/');
      expect(
        (await based.handle(post(echoDoc(), path: '/tt/trust-tasks'))).status,
        200,
      );
      expect((await based.handle(post(echoDoc()))).status, 404);
    });

    test('a non-JSON content type is 415, parameters are tolerated', () async {
      final s = echoServer();
      expect(
        (await s.handle(post(echoDoc(), contentType: 'text/plain'))).status,
        415,
      );
      expect(
        (await s.handle(
          post(echoDoc(), contentType: 'application/json; charset=utf-8'),
        ))
            .status,
        200,
      );
    });

    test('an oversized body is 413', () async {
      final big = echoDoc(text: 'x' * (maxBodyBytes + 1));
      expect((await echoServer().handle(post(big))).status, 413);
    });

    test('a body that is not a Trust Task document is malformedRequest/400',
        () async {
      for (final body in ['not json', '[]', '{"type": 1}']) {
        final reply = await echoServer().handle(post(body));
        expect(reply.status, 400, reason: body);
        expect(codeOf(reply), 'malformedRequest');
        expect(
          (bodyOf(reply)['payload'] as Map)['message'],
          malformedBodyWireMessage,
        );
      }
    });

    test('an unregistered type is unsupportedType/422', () async {
      final reply = await echoServer().handle(
        post(echoDoc(type: 'https://trusttasks.org/spec/example/other/0.1')),
      );
      expect(reply.status, 422);
      expect(codeOf(reply), 'unsupportedType');
    });

    test('a #response document does not reach the request handler', () async {
      final reply =
          await echoServer().handle(post(echoDoc(type: '$echoType#response')));
      expect(codeOf(reply), 'unsupportedType');
    });

    test('#request routes like the bare Type URI', () async {
      final reply =
          await echoServer().handle(post(echoDoc(type: '$echoType#request')));
      expect(reply.status, 200);
    });

    test('a payload of the wrong shape is malformedRequest without detail',
        () async {
      final doc = echoDoc()..['payload'] = {'text': 42};
      final reply = await echoServer().handle(post(doc));
      expect(reply.status, 400);
      expect(codeOf(reply), 'malformedRequest');
      expect(jsonEncode(bodyOf(reply)), isNot(contains('String')));
    });
  });

  group('identity (§3)', () {
    test('the bearer token fills an absent issuer', () async {
      String? seen;
      final s = HttpsServer(
        localVid: server,
        auth: StaticBearerAuth({aliceToken: alice}),
        clock: () => fixedNow,
      )..on<Echo, Echo>(
          spec: echoSpec,
          decode: Echo.fromJson,
          encode: (p) => p.toJson(),
          encodeResponse: (r) => r.toJson(),
          handler: (doc, ctx) async {
            seen = ctx.resolved.issuer;
            expect(ctx.authenticatedSender, alice);
            return doc.payload;
          },
        );
      final reply = await s.handle(post(echoDoc(issuer: null)));
      expect(reply.status, 200);
      expect(seen, alice);
    });

    test(
        'an in-band issuer contradicting the token is identityMismatch, '
        'addressed to the authenticated sender', () async {
      final reply = await echoServer().handle(post(echoDoc(issuer: mallory)));
      expect(reply.status, 422);
      expect(codeOf(reply), 'identityMismatch');
      expect(bodyOf(reply)['recipient'], alice);
    });

    test(
        'a mismatch with no authenticated sender is indistinguishable from '
        'a body that did not parse', () async {
      // The server's own VID stands as the transport recipient, so a
      // contradicting in-band recipient is a mismatch with no sender to route
      // to — which §8.1 suppresses.
      final reply = await echoServer(requireAttribution: false).handle(
        post(echoDoc(recipient: 'did:web:elsewhere.example'), token: null),
      );
      expect(reply.status, 400);
      expect(codeOf(reply), 'malformedRequest');
      expect(
        (bodyOf(reply)['payload'] as Map)['message'],
        malformedBodyWireMessage,
      );
      expect(bodyOf(reply).containsKey('recipient'), isFalse);
    });

    test('an unknown token authenticates nobody', () async {
      final reply = await echoServer().handle(post(echoDoc(), token: 'wrong'));
      expect(codeOf(reply), 'proofRequired');
    });
  });

  group('attribution gate', () {
    test('no token and no proof is refused before the handler', () async {
      final calls = <String>[];
      final reply =
          await echoServer(calls: calls).handle(post(echoDoc(), token: null));
      expect(reply.status, 422);
      expect(codeOf(reply), 'proofRequired');
      expect(calls, isEmpty);
    });

    test('a proof alone satisfies it', () async {
      final reply = await echoServer(verifier: FakeVerifier())
          .handle(post(echoDoc(proof: proof(alice)), token: null));
      expect(reply.status, 200);
    });

    test('can be turned off', () async {
      final reply = await echoServer(requireAttribution: false)
          .handle(post(echoDoc(), token: null));
      expect(reply.status, 200);
    });

    test('an expired document is refused as expired, not as unattributed',
        () async {
      final reply = await echoServer().handle(
        post(echoDoc(issuedAt: '2026-05-01T00:00:00Z'), token: null),
      );
      expect(codeOf(reply), 'expired');
    });
  });

  group('proofs', () {
    test('with no verifier, a proof is refused rather than ignored', () async {
      final reply =
          await echoServer().handle(post(echoDoc(proof: proof(alice))));
      expect(reply.status, 400);
      expect(codeOf(reply), 'malformedRequest');
    });

    test('a failing proof is proofInvalid/422 with a constant message',
        () async {
      final reply = await echoServer(verifier: FakeVerifier())
          .handle(post(echoDoc(proof: proof(alice, value: 'bad'))));
      expect(reply.status, 422);
      expect(codeOf(reply), 'proofInvalid');
      expect(
        (bodyOf(reply)['payload'] as Map)['message'],
        proofInvalidWireMessage,
      );
    });

    test('the verifier sees the document as it arrived', () async {
      // A member the typed payload does not model would be dropped by a
      // decode-and-re-encode; the signature is over the original.
      Map<String, dynamic>? seen;
      final s = echoServer(verifier: _Capture((doc) => seen = doc));
      final doc = echoDoc(proof: proof(alice));
      (doc['payload'] as Map<String, dynamic>)['fromANewerMinor'] = true;
      final reply = await s.handle(post(doc));
      expect(reply.status, 200);
      expect((seen!['payload'] as Map)['fromANewerMinor'], isTrue);
    });

    test('allowedDidMethods refuses before the verifier runs', () async {
      final verifier = FakeVerifier();
      final s = echoServer(verifier: verifier, allowedDidMethods: {'key'});
      final reply = await s.handle(post(echoDoc(proof: proof(alice))));
      expect(codeOf(reply), 'proofInvalid');
      expect(verifier.calls, 0);
    });

    test('a proof-REQUIRED specification refuses a proofless document',
        () async {
      final s = HttpsServer(
        localVid: server,
        auth: StaticBearerAuth({aliceToken: alice}),
        clock: () => fixedNow,
      )..on<acl_grant.Payload, acl_grant.Response>(
          spec: acl_grant.spec,
          decode: acl_grant.Payload.fromJson,
          encode: (p) => p.toJson(),
          encodeResponse: (r) => r.toJson(),
          handler: (doc, ctx) async =>
              acl_grant.Response(entry: doc.payload.entry),
        );
      final doc = echoDoc(type: acl_grant.typeUri)
        ..['payload'] = {
          'entry': {'subject': 'did:web:bob.example', 'role': 'admin'},
        };
      final reply = await s.handle(post(doc));
      expect(codeOf(reply), 'proofRequired');
    });
  });

  group('duplicate execution (§5.1)', () {
    test(
        'a bit-for-bit resend is answered with the first response and not '
        're-executed', () async {
      final calls = <String>[];
      final s = echoServer(calls: calls);
      final first = await s.handle(post(echoDoc()));
      final second = await s.handle(post(echoDoc()));
      expect(second.status, 200);
      expect(bodyOf(second), bodyOf(first));
      expect(calls, ['hello']);
    });

    test('a different document under the same id is idConflict/409', () async {
      final s = echoServer();
      await s.handle(post(echoDoc()));
      final reply = await s.handle(post(echoDoc(text: 'changed')));
      expect(reply.status, 409);
      expect(codeOf(reply), 'idConflict');
    });

    test('a fire-and-forget duplicate is 204 and not re-executed', () async {
      final calls = <String>[];
      final s = echoServer(calls: calls);
      final doc = echoDoc(type: notifyType);
      expect((await s.handle(post(doc))).status, 204);
      expect((await s.handle(post(doc))).status, 204);
      expect(calls, ['notify:hello']);
    });

    test('a duplicate still executing is 202 with no body', () async {
      final guard = _InFlightGuard();
      final s = HttpsServer(
        localVid: server,
        auth: StaticBearerAuth({aliceToken: alice}),
        replayGuard: guard,
        clock: () => fixedNow,
      )..on<Echo, Echo>(
          spec: echoSpec,
          decode: Echo.fromJson,
          encode: (p) => p.toJson(),
          encodeResponse: (r) => r.toJson(),
          handler: (doc, ctx) async => doc.payload,
        );
      final reply = await s.handle(post(echoDoc()));
      expect(reply.status, 202);
      expect(reply.body, isNull);
    });

    test('a refusal that is not retryable is replayed with its own status',
        () async {
      final calls = <String>[];
      final s = echoServer(calls: calls);
      final first = await s.handle(post(echoDoc(text: 'refuse')));
      final second = await s.handle(post(echoDoc(text: 'refuse')));
      expect(first.status, 403);
      expect(second.status, 403);
      expect(codeOf(second), 'permissionDenied');
      expect(calls, ['refuse']);
    });

    test('a handler that throws releases the claim, so a resend runs again',
        () async {
      final calls = <String>[];
      final s = echoServer(calls: calls);
      final first = await s.handle(post(echoDoc(text: 'crash')));
      expect(first.status, 500);
      expect(codeOf(first), 'internalError');
      await s.handle(post(echoDoc(text: 'crash')));
      expect(calls, ['crash', 'crash']);
    });

    test('a store that cannot be consulted fails closed with unavailable/503',
        () async {
      final calls = <String>[];
      final s = HttpsServer(
        localVid: server,
        auth: StaticBearerAuth({aliceToken: alice}),
        replayGuard: _BrokenGuard(),
        clock: () => fixedNow,
      )..on<Echo, Echo>(
          spec: echoSpec,
          decode: Echo.fromJson,
          encode: (p) => p.toJson(),
          encodeResponse: (r) => r.toJson(),
          handler: (doc, ctx) async {
            calls.add('ran');
            return doc.payload;
          },
        );
      final reply = await s.handle(post(echoDoc()));
      expect(reply.status, 503);
      expect((bodyOf(reply)['payload'] as Map)['retryable'], isTrue);
      expect(calls, isEmpty);
    });

    test('turning the record off re-executes a resend', () async {
      final calls = <String>[];
      final s = echoServer(calls: calls, replayProtection: false);
      await s.handle(post(echoDoc()));
      await s.handle(post(echoDoc()));
      expect(calls, ['hello', 'hello']);
    });
  });

  group('discovery (§7)', () {
    Map<String, dynamic> query({List<String>? patterns}) =>
        echoDoc(type: discovery.typeUri)
          ..['payload'] = {if (patterns != null) 'patterns': patterns};

    test('lists every registered request type', () async {
      final s = echoServer()..enableDiscovery();
      final reply = await s.handle(post(query()));
      expect(reply.status, 200);
      final payload = bodyOf(reply)['payload'] as Map<String, dynamic>;
      expect(
          payload['supportedTypes'],
          [
            echoType,
            notifyType,
            discovery.typeUri,
          ]..sort());
    });

    test('filters by slug pattern', () async {
      final s = echoServer()..enableDiscovery();
      final reply = await s.handle(post(query(patterns: ['example/*'])));
      final payload = bodyOf(reply)['payload'] as Map<String, dynamic>;
      expect(payload['supportedTypes'], [echoType, notifyType]);
    });

    test('refuses an unauthenticated discoverer unless public', () async {
      final private = echoServer()..enableDiscovery();
      final doc = query()..['proof'] = proof(alice);
      final withVerifier = echoServer(verifier: FakeVerifier())
        ..enableDiscovery();
      expect(
        codeOf(await withVerifier.handle(post(doc, token: null))),
        'permissionDenied',
      );
      expect((await private.handle(post(query()))).status, 200);

      final public = echoServer(verifier: FakeVerifier())
        ..enableDiscovery(public: true);
      expect((await public.handle(post(doc, token: null))).status, 200);
    });
  });
}

class _Capture implements ProofVerifier {
  _Capture(this.onVerify);
  final void Function(Map<String, dynamic>) onVerify;
  @override
  Future<bool> verify(Map<String, dynamic> doc) async {
    onVerify(doc);
    return true;
  }
}

class _InFlightGuard implements ReplayGuard {
  @override
  Future<ReplayVerdict> claim(
    String id,
    String digest,
    DateTime? retainUntil,
    DateTime now,
  ) async =>
      const Duplicate(inFlight: true);
  @override
  Future<void> recordResponse(String id, Object? response) async {}
  @override
  Future<void> release(String id, String digest) async {}
}

class _BrokenGuard implements ReplayGuard {
  @override
  Future<ReplayVerdict> claim(
    String id,
    String digest,
    DateTime? retainUntil,
    DateTime now,
  ) =>
      Future.error(StateError('redis://10.0.0.5 unreachable'));
  @override
  Future<void> recordResponse(String id, Object? response) async {}
  @override
  Future<void> release(String id, String digest) async {}
}
