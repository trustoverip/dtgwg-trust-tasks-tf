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
    await sendEcho(client, request()).catchError(
      (Object _) => request(),
    );
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
        isA<DuplicateAbsorbedException>()
            .having((e) => e.inFlight, 'inFlight', isFalse),
      ),
    );
  });

  test('a non-JSON non-2xx is HttpStatusException', () async {
    final client = clientFor(
      MockClient((_) async => http.Response('bad gateway', 502)),
    );
    await expectLater(
      sendEcho(client, request()),
      throwsA(isA<HttpStatusException>()
          .having((e) => e.httpStatus, 'status', 502)),
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
        throwsA(isA<ResponseMismatchException>().having(
          (e) => e.binding,
          'binding',
          ResponseBinding.inResponseTo,
        )),
      );
    });
  });

  test('a response verifier requires a proof on the response', () async {
    final client = clientFor(
      inProcess(echoServer()),
      responseVerifier: FakeVerifier(),
    );
    await expectLater(
      sendEcho(client, request()),
      throwsA(isA<ResponseProofException>()
          .having((e) => e.missing, 'missing', isTrue)),
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
