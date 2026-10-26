import 'dart:convert';

import 'package:test/test.dart';
import 'package:trust_tasks/specs/acl/grant/v0_2/payload.dart' as acl_grant;
// --- trust-task-discovery 0.1 / 0.2 (superseded) -----------------------------
// Delete these two imports, and the group marked the same way below, when the
// withdrawal batch removes 0.1 and 0.2.
import 'package:trust_tasks/specs/trust_task_discovery/v0_1/payload.dart'
    as discovery_v0_1;
import 'package:trust_tasks/specs/trust_task_discovery/v0_2/payload.dart'
    as discovery_v0_2;
// --- end trust-task-discovery 0.1 / 0.2 -------------------------------------
import 'package:trust_tasks/specs/trust_task_discovery/v0_3/payload.dart'
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
        'a misaddressed document is wrongRecipient even when its issuer '
        'also contradicts the token (§7.2 order, as consumeInbound)', () async {
      final reply = await echoServer().handle(
        post(echoDoc(issuer: mallory, recipient: 'did:web:elsewhere.example')),
      );
      expect(codeOf(reply), 'wrongRecipient');
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
        // acl/grant is here only as a proof-REQUIRED specification.
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
          'entry': {
            'subject': 'did:web:bob.example',
            'role': 'admin',
            'act': {'scope': 'all'},
            'keys': {'scope': 'all'},
            'capabilities': {'scope': 'ceiling'},
          },
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
      final body = bodyOf(reply);
      expect(body['type'], discovery.responseTypeUri);
      final payload = body['payload'] as Map<String, dynamic>;
      final listed = (payload['supportedTypes'] as List).cast<String>();
      expect(listed,
          containsAll(<String>[echoType, notifyType, discovery.typeUri]));
      // Sorted, bare, and nothing that is not a route.
      expect(listed, [...listed]..sort());
      expect(listed.where((t) => t.contains('#')), isEmpty);
      expect(payload['frameworkVersion'], '0.6.0');
    });

    test('a 0.3 answer validates against the 0.3 response schema', () async {
      final s = echoServer()..enableDiscovery(frameworkVersion: '0.6.1');
      final reply = await s.handle(post(query(patterns: ['*'])));
      expect(reply.status, 200);
      final body = bodyOf(reply);
      expect(body['type'], '${discovery.typeUri}#response');
      final payload = body['payload'] as Map<String, dynamic>;
      expect(
          _violations(discovery.responsePayloadSchemaJson, payload), isEmpty);
      // The check is not vacuous: it refuses what the schema refuses.
      for (final bad in <Map<String, dynamic>>[
        {...payload, 'extra': true},
        {...payload, 'frameworkVersion': '0.6'},
        {
          ...payload,
          'acceptanceWindow': {'maxAgeSeconds': 0, 'clockSkewSeconds': 0},
        },
        {
          'supportedTypes': [
            {'type': echoType, 'requiredExt': <String>[]},
          ],
        },
      ]) {
        expect(
            _violations(discovery.responsePayloadSchemaJson, bad), isNotEmpty,
            reason: '$bad');
      }

      // And it reads back through the generated type.
      final read = discovery.Response.fromJson(payload);
      expect(read.supportedTypes, contains(echoType));
      expect(read.frameworkVersion, '0.6.1');
      // echoServer keeps the consequential posture: five minutes, 60 s skew.
      expect(read.acceptanceWindow?.maxAgeSeconds, 300);
      expect(read.acceptanceWindow?.clockSkewSeconds, 60);
    });

    test('a request outside the 0.3 request schema is refused', () async {
      final s = echoServer()..enableDiscovery();
      final ok = query(patterns: ['acl/*']);
      expect(_violations(discovery.payloadSchemaJson, ok['payload']), isEmpty);
      final bad = echoDoc(type: discovery.typeUri)
        ..['payload'] = {'patterns': 'acl/*'};
      expect(codeOf(await s.handle(post(bad))), 'malformedRequest');
    });

    test('patterns filter the 0.3 answer', () async {
      final s = echoServer()..enableDiscovery();
      final reply = await s.handle(post(query(patterns: [_slugOf(echoType)])));
      final payload = bodyOf(reply)['payload'] as Map<String, dynamic>;
      expect(payload['supportedTypes'], [echoType]);
    });

    test('the advertised window is the enforced one, rounded down', () async {
      Future<Map<String, dynamic>> ask(FreshnessPolicy freshness,
          {bool advertise = true}) async {
        final s = echoServer(freshness: freshness)
          ..enableDiscovery(advertiseAcceptanceWindow: advertise);
        return bodyOf(await s.handle(post(query())))['payload']
            as Map<String, dynamic>;
      }

      final window = (await ask(const FreshnessPolicy(
        maxAge: Duration(seconds: 90, milliseconds: 900),
        skew: Duration(seconds: 5, milliseconds: 500),
        requireIssuedAt: true,
      )))['acceptanceWindow'];
      // Never wider than enforced (responder rule 1): fractions round down.
      expect(window, {'maxAgeSeconds': 90, 'clockSkewSeconds': 5});

      // An unbounded window has no 0.3 spelling, so none is stated.
      expect(
        (await ask(FreshnessPolicy.defaults)).containsKey('acceptanceWindow'),
        isFalse,
      );
      // And a deployment may keep it to itself.
      expect(
        (await ask(FreshnessPolicy.consequential, advertise: false))
            .containsKey('acceptanceWindow'),
        isFalse,
      );
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

  // --- trust-task-discovery 0.1 / 0.2 (superseded) -----------------------------
  // Delete this group, and the two imports marked the same way at the top of
  // this file, when the withdrawal batch removes 0.1 and 0.2.
  group('discovery 0.1 / 0.2 (superseded, still answered)', () {
    Map<String, dynamic> query01() => echoDoc(type: discovery_v0_1.typeUri)
      ..['payload'] = <String, dynamic>{};
    Map<String, dynamic> query02() => echoDoc(type: discovery_v0_2.typeUri)
      ..['payload'] = <String, dynamic>{};

    test('0.1 and 0.2 are listed beside 0.3', () async {
      final s = echoServer()..enableDiscovery();
      final payload = bodyOf(await s.handle(post(query01())))['payload']
          as Map<String, dynamic>;
      expect(
          payload['supportedTypes'],
          [
            echoType,
            notifyType,
            discovery_v0_1.typeUri,
            discovery_v0_2.typeUri,
            discovery.typeUri,
          ]..sort());
      // 0.1 admits only MAJOR.MINOR.
      expect(payload['frameworkVersion'], '0.6');
    });

    test('answers 0.2 in 0.2, with the three-part framework release', () async {
      final s = echoServer()..enableDiscovery();
      final reply = await s.handle(post(query02()));
      expect(reply.status, 200);
      final body = bodyOf(reply);
      expect(body['type'], '${discovery_v0_2.typeUri}#response');
      final payload = body['payload'] as Map<String, dynamic>;
      expect(payload['frameworkVersion'], '0.6.0');
      expect(payload['supportedTypes'], contains(echoType));
      // 0.2's response is closed: no acceptanceWindow.
      expect(payload.containsKey('acceptanceWindow'), isFalse);
    });

    test('keeps a PATCH in 0.2 and drops it only in 0.1', () async {
      // One server per request: both documents share echoDoc's id, and the
      // replay record would otherwise refuse the second.
      Future<Map<String, dynamic>> ask(Map<String, dynamic> doc) async {
        final s = echoServer()..enableDiscovery(frameworkVersion: '0.6.1');
        return bodyOf(await s.handle(post(doc)))['payload']
            as Map<String, dynamic>;
      }

      expect((await ask(query01()))['frameworkVersion'], '0.6');
      expect((await ask(query02()))['frameworkVersion'], '0.6.1');
    });
  });
  // --- end trust-task-discovery 0.1 / 0.2 -------------------------------------
}

/// The slug of a bare Type URI.
String _slugOf(String typeUri) => slugFromTypeUri(typeUri);

/// What in [instance] the JSON Schema [schemaJson] refuses, for the subset of
/// JSON Schema the trust-task-discovery schemas use: `type`, `required`,
/// `additionalProperties: false`, `properties`, `items`, `oneOf`, `minItems`,
/// `maxItems`, `uniqueItems`, `minimum`, `minLength`, `pattern`, `format: uri`
/// and local `$ref`. A keyword it does not know fails the check rather than
/// passing silently, so a later schema that needs more cannot slip by.
List<String> _violations(String schemaJson, Object? instance) {
  final root = jsonDecode(schemaJson) as Map<String, dynamic>;
  const known = {
    r'$schema', r'$id', r'$defs', r'$anchor', r'$ref', 'title', //
    'description', 'type', 'required', 'additionalProperties', 'properties',
    'items', 'oneOf', 'minItems', 'maxItems', 'uniqueItems', 'minimum',
    'minLength', 'pattern', 'format',
  };
  final out = <String>[];

  Map<String, dynamic> resolve(String ref) {
    var node = root;
    for (final part in ref.substring(2).split('/')) {
      node = node[part] as Map<String, dynamic>;
    }
    return node;
  }

  bool typeOk(String type, Object? v) => switch (type) {
        'object' => v is Map,
        'array' => v is List,
        'string' => v is String,
        'integer' => v is int,
        'boolean' => v is bool,
        _ => false,
      };

  void check(Map<String, dynamic> schema, Object? v, String at) {
    for (final k in schema.keys) {
      if (!known.contains(k)) out.add('$at: unsupported keyword $k');
    }
    final ref = schema[r'$ref'];
    if (ref is String) check(resolve(ref), v, at);
    final type = schema['type'];
    if (type is String && !typeOk(type, v)) {
      out.add('$at: not $type');
      return;
    }
    final oneOf = schema['oneOf'];
    if (oneOf is List) {
      final matches = oneOf.where((branch) {
        final before = out.length;
        check(branch as Map<String, dynamic>, v, at);
        final ok = out.length == before;
        out.removeRange(before, out.length);
        return ok;
      }).length;
      if (matches != 1) out.add('$at: matches $matches oneOf branches');
    }
    if (v is Map) {
      final props = (schema['properties'] as Map<String, dynamic>?) ?? const {};
      for (final r in (schema['required'] as List?) ?? const []) {
        if (!v.containsKey(r)) out.add('$at: missing $r');
      }
      if (schema['additionalProperties'] == false) {
        for (final k in v.keys) {
          if (!props.containsKey(k)) out.add('$at: unexpected member $k');
        }
      }
      for (final e in props.entries) {
        if (v.containsKey(e.key)) {
          check(e.value as Map<String, dynamic>, v[e.key], '$at/${e.key}');
        }
      }
    }
    if (v is List) {
      final minItems = schema['minItems'];
      final maxItems = schema['maxItems'];
      if (minItems is int && v.length < minItems) out.add('$at: too few');
      if (maxItems is int && v.length > maxItems) out.add('$at: too many');
      if (schema['uniqueItems'] == true && v.toSet().length != v.length) {
        out.add('$at: duplicates');
      }
      final items = schema['items'];
      if (items is Map<String, dynamic>) {
        for (var i = 0; i < v.length; i++) {
          check(items, v[i], '$at/$i');
        }
      }
    }
    if (v is String) {
      final minLength = schema['minLength'];
      if (minLength is int && v.length < minLength) out.add('$at: too short');
      final pattern = schema['pattern'];
      if (pattern is String && !RegExp(pattern).hasMatch(v)) {
        out.add('$at: does not match $pattern');
      }
      if (schema['format'] == 'uri' &&
          !(Uri.tryParse(v)?.isAbsolute ?? false)) {
        out.add('$at: not an absolute URI');
      }
    }
    if (v is int) {
      final minimum = schema['minimum'];
      if (minimum is int && v < minimum) out.add('$at: below $minimum');
    }
  }

  check(root, instance, '');
  return out;
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
