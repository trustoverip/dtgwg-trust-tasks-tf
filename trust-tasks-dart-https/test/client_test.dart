import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_https/trust_tasks_https.dart';

import 'support/fixtures.dart';

TrustTaskDocument<Echo> request({
  String id = 'urn:uuid:00000000-0000-4000-8000-0000000000c1',
  String text = 'hello',
  String type = echoType,
}) =>
    TrustTaskDocument<Echo>(
      id: id,
      type: type,
      issuedAt: fixedIssuedAt,
      payload: Echo(text),
    );

HttpsClient clientFor(
  http.Client transport, {
  String base = 'https://maintainer.example',
  ProofVerifier? responseVerifier,
}) =>
    HttpsClient(
      base: Uri.parse(base),
      serverVid: server,
      myVid: alice,
      token: aliceToken,
      responseVerifier: responseVerifier,
      httpClient: transport,
    );

Future<TrustTaskDocument<Echo>> sendEcho(
  HttpsClient client,
  TrustTaskDocument<Echo> doc,
) =>
    client.send<Echo, Echo>(
      doc,
      encode: (p) => p.toJson(),
      decodeResponse: Echo.fromJson,
    );

void main() {
  test('posts to <base>/trust-tasks with JSON and the bearer token', () async {
    late http.Request seen;
    final client = clientFor(
      MockClient((r) async {
        seen = r;
        return inProcessReply(r);
      }),
      base: 'https://maintainer.example/tt/',
    );
    await sendEcho(client, request()).catchError((Object _) => request());
    expect(seen.url.toString(), 'https://maintainer.example/tt/trust-tasks');
    expect(seen.headers['content-type'], startsWith('application/json'));
    expect(seen.headers['authorization'], 'Bearer $aliceToken');
    final body = jsonDecode(seen.body) as Map<String, dynamic>;
    expect(body['issuer'], alice);
    expect(body['recipient'], server);
  });

  test('round trip against the server returns the typed response', () async {
    final client = clientFor(inProcess(echoServer()));
    final response = await sendEcho(client, request());
    expect(response.payload.text, 'HELLO');
    expect(response.type, '$echoType#response');
  });

  test('a refusal surfaces as TrustTaskErrorException with the code', () async {
    final client = clientFor(inProcess(echoServer()));
    await expectLater(
      sendEcho(client, request(text: 'refuse')),
      throwsA(
        isA<TrustTaskErrorException>()
            .having((e) => e.httpStatus, 'status', 403)
            .having((e) => e.error.payload.code, 'code', 'permissionDenied'),
      ),
    );
  });

  test('a resend of a fire-and-forget document is accepted', () async {
    final client = clientFor(inProcess(echoServer()));
    final doc = request(type: notifyType);
    await client.sendAck<Echo>(doc, encode: (p) => p.toJson());
    await client.sendAck<Echo>(doc, encode: (p) => p.toJson());
  });

  test('a 204 to send() is a duplicate absorbed, not a failure', () async {
    final client = clientFor(MockClient((_) async => http.Response('', 204)));
    await expectLater(
      sendEcho(client, request()),
      throwsA(
        isA<DuplicateAbsorbedException>().having(
          (e) => e.inFlight,
          'inFlight',
          isFalse,
        ),
      ),
    );
  });

  test('a non-JSON non-2xx is HttpStatusException', () async {
    final client = clientFor(
      MockClient((_) async => http.Response('bad gateway', 502)),
    );
    await expectLater(
      sendEcho(client, request()),
      throwsA(
        isA<HttpStatusException>().having((e) => e.httpStatus, 'status', 502),
      ),
    );
  });

  test('a connection failure is TransportException', () async {
    final client = clientFor(
      MockClient((_) async => throw http.ClientException('refused')),
    );
    await expectLater(
      sendEcho(client, request()),
      throwsA(isA<TransportException>()),
    );
  });

  group('response binding', () {
    Future<Object?> answerWith(
      Map<String, dynamic> Function(Map<String, dynamic>) edit,
    ) async {
      final client = clientFor(
        MockClient((r) async {
          final real = await inProcessReply(r);
          final body = jsonDecode(real.body) as Map<String, dynamic>;
          return http.Response(
            jsonEncode(edit(body)),
            200,
            headers: {'content-type': 'application/json'},
          );
        }),
      );
      try {
        await sendEcho(client, request());
        return null;
      } on ResponseMismatchException catch (e) {
        return e.binding;
      }
    }

    test('wrong thread', () async {
      expect(
        await answerWith((b) => {...b, 'threadId': 'urn:uuid:other'}),
        ResponseBinding.threadId,
      );
    });

    test('wrong type', () async {
      expect(
        await answerWith((b) => {...b, 'type': '$notifyType#response'}),
        ResponseBinding.type,
      );
    });

    test('wrong issuer', () async {
      expect(
        await answerWith((b) => {...b, 'issuer': mallory}),
        ResponseBinding.issuer,
      );
    });

    test('wrong recipient', () async {
      expect(
        await answerWith((b) => {...b, 'recipient': mallory}),
        ResponseBinding.recipient,
      );
    });

    test('an error response about another document', () async {
      final client = clientFor(
        MockClient((r) async {
          final real = await inProcessReply(r);
          final body = jsonDecode(real.body) as Map<String, dynamic>;
          (body['payload'] as Map)['inResponseTo'] = {
            'typeUri': echoType,
            'id': 'urn:uuid:someone-else',
          };
          return http.Response(
            jsonEncode(body),
            real.statusCode,
            headers: real.headers,
          );
        }),
      );
      await expectLater(
        sendEcho(client, request(text: 'refuse')),
        throwsA(
          isA<ResponseMismatchException>().having(
            (e) => e.binding,
            'binding',
            ResponseBinding.inResponseTo,
          ),
        ),
      );
    });
  });

  group('transport limits', () {
    test('a plain http base is refused unless loopback or allowed', () {
      expect(
        () => HttpsClient(base: Uri.parse('http://maintainer.example')),
        throwsArgumentError,
      );
      expect(
        HttpsClient(
          base: Uri.parse('http://maintainer.example'),
          allowInsecureHttp: true,
        ).endpoint.toString(),
        'http://maintainer.example/trust-tasks',
      );
      for (final base in [
        'http://127.0.0.1:8080',
        'http://localhost:3000',
        'http://[::1]:3000',
      ]) {
        HttpsClient(base: Uri.parse(base), token: aliceToken).close();
      }
    });

    test('a bearer token is never sent over non-loopback http', () {
      expect(
        () => HttpsClient(
          base: Uri.parse('http://maintainer.example'),
          token: aliceToken,
          allowInsecureHttp: true,
        ),
        throwsArgumentError,
      );
    });

    test('a response verifier needs a server VID', () {
      expect(
        () => HttpsClient(
          base: Uri.parse('https://maintainer.example'),
          responseVerifier: FakeVerifier(),
        ),
        throwsArgumentError,
      );
    });

    test('redirects are not followed', () async {
      late http.BaseRequest seen;
      final client = clientFor(
        MockClient((r) async {
          seen = r;
          return http.Response(
            '',
            307,
            headers: {'location': 'https://elsewhere.example/trust-tasks'},
          );
        }),
      );
      await expectLater(
        sendEcho(client, request()),
        throwsA(
          isA<HttpStatusException>().having((e) => e.httpStatus, 'status', 307),
        ),
      );
      expect(seen.followRedirects, isFalse);
    });

    test('a body over maxResponseBytes is refused', () async {
      final client = HttpsClient(
        base: Uri.parse('https://maintainer.example'),
        serverVid: server,
        myVid: alice,
        maxResponseBytes: 1024,
        httpClient: MockClient.streaming((r, _) async {
          return http.StreamedResponse(
            Stream.fromIterable(
              List.generate(8, (_) => List<int>.filled(512, 0x20)),
            ),
            200,
            headers: {'content-type': 'application/json'},
          );
        }),
      );
      await expectLater(
        sendEcho(client, request()),
        throwsA(
          isA<ResponseTooLargeException>().having(
            (e) => e.limit,
            'limit',
            1024,
          ),
        ),
      );
    });

    test('a declared Content-Length over the cap is refused unread', () async {
      final client = HttpsClient(
        base: Uri.parse('https://maintainer.example'),
        serverVid: server,
        myVid: alice,
        maxResponseBytes: 16,
        httpClient: MockClient((_) async => http.Response('x' * 64, 200)),
      );
      await expectLater(
        sendEcho(client, request()),
        throwsA(isA<ResponseTooLargeException>()),
      );
    });

    test('any client error is a TransportException', () async {
      final client = clientFor(
        MockClient((_) async => throw const FormatException('handshake')),
      );
      await expectLater(
        sendEcho(client, request()),
        throwsA(isA<TransportException>()),
      );
    });

    test('error text does not quote the response body', () async {
      final secretish = 'BODY-${'z' * 600}';
      final client = clientFor(
        MockClient((_) async => http.Response(secretish, 502)),
      );
      final error = await sendEcho(
        client,
        request(),
      ).then<Object?>((_) => null, onError: (Object e) => e);
      expect(error, isA<HttpStatusException>());
      expect(error.toString(), isNot(contains('BODY-')));
      expect(
        (error! as HttpStatusException).body.length,
        lessThanOrEqualTo(512),
      );

      final decoding = clientFor(
        MockClient((_) async => http.Response('{"BODY-', 200)),
      );
      final decodeError = await sendEcho(
        decoding,
        request(),
      ).then<Object?>((_) => null, onError: (Object e) => e);
      expect(decodeError, isA<ResponseDecodeException>());
      expect(decodeError.toString(), isNot(contains('BODY-')));
    });
  });

  group('error responses with a response verifier', () {
    const requestId = 'urn:uuid:00000000-0000-4000-8000-0000000000e1';

    /// The echo server's genuine refusal of [requestId], as JSON.
    Future<Map<String, dynamic>> refusal() async => bodyOf(
          await echoServer().handle(
            post(echoDoc(id: requestId, text: 'refuse')),
          ),
        );

    HttpsClient answering(
      Map<String, dynamic> body, {
      ProofVerifier? verifier,
    }) =>
        clientFor(
          MockClient(
            (_) async => http.Response(
              jsonEncode(body),
              403,
              headers: {'content-type': 'application/json'},
            ),
          ),
          responseVerifier: verifier ?? FakeVerifier(),
        );

    Future<Object?> outcome(HttpsClient client) => sendEcho(
          client,
          request(id: requestId, text: 'refuse'),
        ).then<Object?>((_) => null, onError: (Object e) => e);

    test('an unsigned error is unverified, not the server answer', () async {
      final error = await outcome(answering(await refusal()));
      expect(
        error,
        isA<UnverifiedErrorResponseException>()
            .having((e) => e.httpStatus, 'status', 403)
            .having((e) => e.reason, 'reason', contains('no proof'))
            .having(
              (e) => e.error.payload.code,
              'code (kept for diagnostics)',
              'permissionDenied',
            ),
      );
      expect(error.toString(), isNot(contains('permissionDenied')));
    });

    test('an error whose proof does not verify is unverified', () async {
      final body = await refusal()
        ..['proof'] = proof(server, value: 'bad');
      expect(
        await outcome(answering(body)),
        isA<UnverifiedErrorResponseException>()
            .having((e) => e.reason, 'reason', contains('did not verify')),
      );
    });

    test('a verifier that throws leaves the error unverified', () async {
      final body = await refusal()
        ..['proof'] = proof(server);
      expect(
        await outcome(answering(body, verifier: _Throwing())),
        isA<UnverifiedErrorResponseException>(),
      );
    });

    test('a valid error from another party is unverified', () async {
      final body = await refusal()
        ..['issuer'] = mallory
        ..['proof'] = proof(mallory);
      expect(
        await outcome(answering(body)),
        isA<UnverifiedErrorResponseException>()
            .having((e) => e.reason, 'reason', contains('issuer')),
      );
    });

    test('a valid error addressed to someone else is unverified', () async {
      final body = await refusal()
        ..['recipient'] = mallory
        ..['proof'] = proof(server);
      expect(
        await outcome(answering(body)),
        isA<UnverifiedErrorResponseException>()
            .having((e) => e.reason, 'reason', contains('addressed')),
      );
    });

    test('a signed error from the server is its answer', () async {
      final body = await refusal()
        ..['proof'] = proof(server);
      expect(
        await outcome(answering(body)),
        isA<TrustTaskErrorException>()
            .having((e) => e.httpStatus, 'status', 403)
            .having((e) => e.error.payload.code, 'code', 'permissionDenied'),
      );
    });

    test('the proof is verified over the body as received', () async {
      // A member the typed model does not carry, and an offset spelled
      // `+00:00`: a re-serialised document would differ from what was signed.
      final body = await refusal()
        ..['issuedAt'] = '2026-06-01T12:00:00+00:00'
        ..['extension'] = {'kept': true}
        ..['proof'] = proof(server);
      Map<String, dynamic>? seen;
      final error = await outcome(
        answering(body, verifier: _Capture((doc) => seen = doc)),
      );
      expect(error, isA<TrustTaskErrorException>());
      expect(seen, body);
    });

    test('without a verifier, an unsigned error is still returned', () async {
      final client = clientFor(
        MockClient(
          (_) async => http.Response(
            jsonEncode(await refusal()),
            403,
            headers: {'content-type': 'application/json'},
          ),
        ),
      );
      expect(await outcome(client), isA<TrustTaskErrorException>());
    });
  });

  test('a response verifier requires a proof on the response', () async {
    final client = clientFor(
      inProcess(echoServer()),
      responseVerifier: FakeVerifier(),
    );
    await expectLater(
      sendEcho(client, request()),
      throwsA(
        isA<ResponseProofException>().having(
          (e) => e.missing,
          'missing',
          isTrue,
        ),
      ),
    );
  });
}

final _server = echoServer();

Future<http.Response> inProcessReply(http.BaseRequest r) async {
  final req = r as http.Request;
  final reply = await _server.handle(
    HttpsRequest(
      method: req.method,
      path: '/trust-tasks',
      headers: req.headers,
      body: req.bodyBytes,
    ),
  );
  return http.Response.bytes(
    reply.body ?? const [],
    reply.status,
    headers: reply.headers,
  );
}

class _Throwing implements ProofVerifier {
  @override
  Future<bool> verify(Map<String, dynamic> doc) async =>
      throw StateError('resolver unavailable');
}

class _Capture implements ProofVerifier {
  _Capture(this.onVerify);
  final void Function(Map<String, dynamic> doc) onVerify;
  @override
  Future<bool> verify(Map<String, dynamic> doc) async {
    onVerify(doc);
    return true;
  }
}
