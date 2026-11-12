// Forward routing along `extends` (#1117), per-type document size bounds
// (#1121) and the observability hook (#1116).
import 'dart:async';
import 'dart:convert';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_https/trust_tasks_https.dart';

import 'support/fixtures.dart';

const echo2Type = 'https://trusttasks.org/spec/example/echo/0.2';
const echo3Type = 'https://trusttasks.org/spec/example/echo/0.3';
const echo4Type = 'https://trusttasks.org/spec/example/echo/0.4';
const bigType = 'https://trusttasks.org/spec/example/big/0.1';
const smallType = 'https://trusttasks.org/spec/example/small/0.1';

/// echo/0.2 extends 0.1 and echo/0.3 extends 0.2; echo/0.4 declares nothing.
const index = {echo2Type: '0.1', echo3Type: '0.2'};

/// A closed 0.1 payload: a later version's extra member is pruned, not refused.
const echoClosedSpec = SpecPolicy(
  typeUri: echoType,
  payloadSchema: '{"type":"object","additionalProperties":false,'
      '"properties":{"text":{"type":"string"}}}',
);

HttpsServer bare({
  ServerEventHook? onEvent,
  int defaultMaxDocumentBytes = maxBodyBytes,
  Map<String, String>? extendsIndex = index,
}) =>
    HttpsServer(
      localVid: server,
      auth: StaticBearerAuth({aliceToken: alice}),
      clock: () => fixedNow,
      onEvent: onEvent,
      defaultMaxDocumentBytes: defaultMaxDocumentBytes,
      extendsIndex: extendsIndex,
    );

extension on HttpsServer {
  void echo(
    SpecPolicy spec, {
    List<String>? seen,
    Future<Echo> Function(TrustTaskDocument<Echo>)? handler,
  }) =>
      on<Echo, Echo>(
        spec: spec,
        decode: Echo.fromJson,
        encode: (p) => p.toJson(),
        encodeResponse: (r) => r.toJson(),
        handler: (doc, ctx) async {
          seen?.add('${spec.typeUri} <- ${doc.type}');
          if (handler != null) return handler(doc);
          return Echo(doc.payload.text.toUpperCase());
        },
      );
}

void main() {
  group('forward routing along extends (#1117)', () {
    test('a later version declaring extends reaches the implemented handler',
        () async {
      final seen = <String>[];
      final s = bare()..echo(echoClosedSpec, seen: seen);
      final doc = echoDoc(type: echo2Type)
        ..['payload'] = {'text': 'hi', 'addedIn02': true};
      final reply = await s.handle(post(doc));
      expect(reply.status, 200);
      expect(seen, ['$echoType <- $echo2Type']);
      // Answered in the version the producer used (SPEC §5.2).
      expect(bodyOf(reply)['type'], '$echo2Type#response');
      expect((bodyOf(reply)['payload'] as Map)['text'], 'HI');
    });

    test('the chain is followed to the highest registered version', () async {
      final seen = <String>[];
      final s = bare()
        ..echo(echoSpec, seen: seen)
        ..echo(const SpecPolicy(typeUri: echo2Type), seen: seen);
      expect((await s.handle(post(echoDoc(type: echo3Type)))).status, 200);
      expect(seen, ['$echo2Type <- $echo3Type']);
    });

    test('an exact registration wins over the chain', () async {
      final seen = <String>[];
      final s = bare()
        ..echo(echoSpec, seen: seen)
        ..echo(const SpecPolicy(typeUri: echo2Type), seen: seen);
      expect((await s.handle(post(echoDoc(type: echo2Type)))).status, 200);
      expect(seen, ['$echo2Type <- $echo2Type']);
    });

    test('another version of a registered specification is unsupportedVersion',
        () async {
      final s = bare()..echo(echoSpec);
      final reply = await s.handle(post(echoDoc(type: echo4Type)));
      expect(reply.status, 422);
      expect(codeOf(reply), 'unsupportedVersion');
      // An earlier version is not read forward either.
      final s2 = bare()..echo(const SpecPolicy(typeUri: echo2Type));
      expect(
        codeOf(await s2.handle(post(echoDoc(type: echoType)))),
        'unsupportedVersion',
      );
    });

    test('without the declaration in the index it is unsupportedVersion',
        () async {
      final s = bare(extendsIndex: const {})..echo(echoSpec);
      expect(
        codeOf(await s.handle(post(echoDoc(type: echo2Type)))),
        'unsupportedVersion',
      );
    });

    test('another specification is still unsupportedType', () async {
      final s = bare()..echo(echoSpec);
      final reply = await s.handle(
        post(echoDoc(type: 'https://trusttasks.org/spec/example/other/0.2')),
      );
      expect(codeOf(reply), 'unsupportedType');
      // And the same slug under another authority is another specification.
      expect(
        codeOf(await s.handle(
          post(echoDoc(type: 'https://example.org/spec/example/echo/0.2')),
        )),
        'unsupportedType',
      );
    });

    test('a #response document is never routed to a request handler', () async {
      final s = bare()..echo(echoSpec);
      expect(
        codeOf(await s.handle(post(echoDoc(type: '$echo2Type#response')))),
        'unsupportedType',
      );
    });

    test('a forward resend is absorbed over the document as received',
        () async {
      final calls = <String>[];
      final s = bare()..echo(echoClosedSpec, seen: calls);
      final doc = echoDoc(type: echo2Type)
        ..['payload'] = {'text': 'once', 'addedIn02': 1};
      expect((await s.handle(post(doc))).status, 200);
      expect((await s.handle(post(doc))).status, 200);
      expect(calls, hasLength(1));
      // Differing only in a member the implemented version prunes is still a
      // different document under the same id (SPEC §7.2 item 11).
      final changed = echoDoc(type: echo2Type)
        ..['payload'] = {'text': 'once', 'addedIn02': 2};
      expect(codeOf(await s.handle(post(changed))), 'idConflict');
    });
  });

  group('per-type document size bounds (#1121)', () {
    String padded(int bytes) => 'x' * bytes;

    test('the body limit is the largest registered bound, floored', () {
      final s = bare();
      expect(s.bodyLimitBytes, maxBodyBytes);
      s.echo(const SpecPolicy(typeUri: smallType, maxDocumentBytes: 2048));
      expect(s.bodyLimitBytes, maxBodyBytes, reason: 'never under the floor');
      s.echo(const SpecPolicy(typeUri: bigType, maxDocumentBytes: 400 * 1024));
      expect(s.bodyLimitBytes, 400 * 1024);
      expect(bare(defaultMaxDocumentBytes: 1024).bodyLimitBytes, 1024);
    });

    test('a document within a declared bound over 256 KiB is accepted',
        () async {
      final s = bare()
        ..echo(
            const SpecPolicy(typeUri: bigType, maxDocumentBytes: 400 * 1024));
      final doc = echoDoc(type: bigType, text: padded(300 * 1024));
      final body = utf8.encode(jsonEncode(doc));
      expect(body.length, greaterThan(maxBodyBytes));
      expect((await s.handle(post(doc))).status, 200);
    });

    test('a document over its own bound is malformedRequest', () async {
      final s = bare()
        ..echo(const SpecPolicy(typeUri: bigType, maxDocumentBytes: 400 * 1024))
        ..echo(const SpecPolicy(typeUri: smallType, maxDocumentBytes: 2048))
        ..echo(echoSpec);
      // Over the small type's declared bound.
      final small = await s.handle(
        post(echoDoc(type: smallType, text: padded(4096))),
      );
      expect(small.status, 400);
      expect(codeOf(small), 'malformedRequest');
      expect(
        (bodyOf(small)['payload'] as Map)['message'],
        documentTooLargeWireMessage,
      );
      // A type declaring nothing keeps the default, even though the server
      // now reads larger bodies for the big type.
      final undeclared = await s.handle(
        post(echoDoc(text: padded(300 * 1024))),
      );
      expect(codeOf(undeclared), 'malformedRequest');
      // Over the largest bound is refused before parsing.
      final huge = await s.handle(
        post(echoDoc(type: bigType, text: padded(401 * 1024))),
      );
      expect(huge.status, 413);
    });

    test('a forward document is held to the implemented version\'s bound',
        () async {
      final s = bare()
        ..echo(const SpecPolicy(typeUri: echoType, maxDocumentBytes: 2048));
      final reply = await s.handle(
        post(echoDoc(type: echo2Type, text: padded(4096))),
      );
      expect(codeOf(reply), 'malformedRequest');
    });
  });

  group('observability hook (#1116)', () {
    test('a handler exception is reported, and answered internalError',
        () async {
      final events = <ServerEvent>[];
      final s = bare(onEvent: events.add)
        ..echo(echoSpec, handler: (_) async => throw StateError('db down'));
      final reply = await s.handle(post(echoDoc()));
      expect(reply.status, 500);
      expect(codeOf(reply), 'internalError');
      final failed = events.whereType<HandlerFailed>().single;
      expect(failed.error, isA<StateError>());
      expect(failed.type, echoType);
      expect(failed.documentId, echoDoc()['id']);
      expect(events.whereType<InternalFailure>(), isEmpty);
      // The exception text stays off the wire (SPEC §12.4).
      expect(utf8.decode(reply.body!), isNot(contains('db down')));
    });

    test('a suppressed identity mismatch is reported with the disagreement',
        () async {
      final events = <ServerEvent>[];
      final s = bare(onEvent: events.add)..echo(echoSpec);
      // No bearer token, and addressed to somebody else: §8.1 suppresses.
      final reply = await s.handle(
        post(echoDoc(recipient: mallory, proof: proof(alice)), token: null),
      );
      expect(reply.status, 400);
      final suppressed = events.whereType<DocumentSuppressed>().single;
      expect(suppressed.consistency?.party, 'recipient');
      expect(suppressed.consistency?.inBand, mallory);
    });

    test('a malformed body, a failing authenticator and a refusal', () async {
      final events = <ServerEvent>[];
      final s = HttpsServer(
        localVid: server,
        auth: _ThrowingAuth(),
        clock: () => fixedNow,
        onEvent: events.add,
        requireAttribution: false,
      )..echo(echoSpec);
      await s.handle(post('not json'));
      expect(events.single, isA<DocumentMalformed>());
      events.clear();

      final reply = await s.handle(
        post(echoDoc(type: 'https://trusttasks.org/spec/example/x/0.1')),
      );
      expect(codeOf(reply), 'unsupportedType');
      expect(events.first, isA<AuthenticatorFailed>());
      final rejected = events.whereType<DocumentRejected>().single;
      expect(rejected.error.payload.code, 'unsupportedType');
    });

    test('a refused proof is reported with what the verifier threw', () async {
      final events = <ServerEvent>[];
      final s = HttpsServer(
        localVid: server,
        auth: StaticBearerAuth({aliceToken: alice}),
        proofVerifier: _ThrowingVerifier(),
        allowAnyDidMethod: true,
        clock: () => fixedNow,
        onEvent: events.add,
      )..echo(echoSpec);
      final reply = await s.handle(post(echoDoc(proof: proof(alice))));
      expect(codeOf(reply), 'proofInvalid');
      final rejected = events.whereType<ProofRejected>().single;
      expect(rejected.error, isA<FormatException>());
      expect(utf8.decode(reply.body!), isNot(contains('did:web:resolver')));
    });

    test('a hook that throws, or fails asynchronously, changes no answer',
        () async {
      for (final hook in <ServerEventHook>[
        (_) => throw StateError('logger broke'),
        (_) async => throw StateError('logger broke later'),
        (_) => Future<void>.error(StateError('rejected future')),
      ]) {
        final s = bare(onEvent: hook)
          ..echo(echoSpec, handler: (_) async => throw StateError('db down'));
        final crash = await s.handle(post(echoDoc()));
        expect(crash.status, 500);
        expect(codeOf(crash), 'internalError');
        final ok = bare(onEvent: hook)..echo(echoSpec);
        expect((await ok.handle(post(echoDoc()))).status, 200);
        expect(codeOf(await ok.handle(post('not json'))), 'malformedRequest');
      }
      // Let any asynchronous failure surface inside the test's zone, where it
      // would fail the test if it escaped.
      await Future<void>.delayed(Duration.zero);
    });

    test('without a hook, nothing changes', () async {
      final s = bare()..echo(echoSpec);
      expect((await s.handle(post(echoDoc()))).status, 200);
    });
  });
}

final class _ThrowingAuth implements BearerAuthenticator {
  @override
  FutureOr<String?> resolve(String token) =>
      throw StateError('introspection endpoint down');
}

final class _ThrowingVerifier implements ProofVerifier {
  @override
  Future<bool> verify(Map<String, dynamic> doc) async =>
      throw const FormatException('did:web:resolver.example unreachable');
}
