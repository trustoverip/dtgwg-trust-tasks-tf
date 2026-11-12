@TestOn('vm')
library;

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_https/io.dart';

import 'support/fixtures.dart';

void main() {
  late HttpServer http;
  late List<String> calls;

  setUp(() async {
    calls = [];
    http = await serve(echoServer(calls: calls));
  });

  tearDown(() => http.close(force: true));

  Uri base() => Uri.parse('http://${http.address.host}:${http.port}');

  test('a real socket round trip, and the retry absorbed', () async {
    final client = HttpsClient(
      base: base(),
      serverVid: server,
      myVid: alice,
      token: aliceToken,
    );
    final doc = TrustTaskDocument<Echo>(
      id: 'urn:uuid:00000000-0000-4000-8000-0000000000d1',
      type: echoType,
      issuedAt: fixedIssuedAt,
      payload: const Echo('over the wire'),
    );
    for (var i = 0; i < 2; i++) {
      final response = await client.send<Echo, Echo>(
        doc,
        encode: (p) => p.toJson(),
        decodeResponse: Echo.fromJson,
      );
      expect(response.payload.text, 'OVER THE WIRE');
    }
    expect(calls, ['over the wire']);
    client.close();
  });

  test('a chunked body past the limit is 413 without buffering it all',
      () async {
    final raw = HttpClient();
    final request = await raw.postUrl(base().resolve('/trust-tasks'));
    request.headers.contentType = ContentType.json;
    request.contentLength = -1; // chunked: no declared length to check
    request.add(utf8.encode('{"pad":"${'x' * (maxBodyBytes + 10)}"}'));
    final response = await request.close();
    expect(response.statusCode, 413);
    await response.drain<void>();
    raw.close(force: true);
  });

  test('reads past 256 KiB when a registered specification declares more',
      () async {
    const bigType = 'https://trusttasks.org/spec/example/big/0.1';
    final big = HttpsServer(
      localVid: server,
      auth: StaticBearerAuth({aliceToken: alice}),
      clock: () => fixedNow,
    )..on<Echo, Echo>(
        spec: const SpecPolicy(typeUri: bigType, maxDocumentBytes: 400 * 1024),
        decode: Echo.fromJson,
        encode: (p) => p.toJson(),
        encodeResponse: (r) => r.toJson(),
        handler: (doc, ctx) async => const Echo('ok'),
      );
    final bigHttp = await serve(big);
    final raw = HttpClient();
    try {
      final request = await raw.postUrl(Uri.parse(
          'http://${bigHttp.address.host}:${bigHttp.port}/trust-tasks'));
      request.headers.contentType = ContentType.json;
      request.headers.set('authorization', 'Bearer $aliceToken');
      request.add(utf8.encode(jsonEncode(
        echoDoc(type: bigType, text: 'x' * (300 * 1024)),
      )));
      final response = await request.close();
      expect(response.statusCode, 200);
      await response.drain<void>();
    } finally {
      raw.close(force: true);
      await bigHttp.close(force: true);
    }
  });
}
