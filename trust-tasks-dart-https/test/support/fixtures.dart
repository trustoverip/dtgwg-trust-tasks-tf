import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_https/trust_tasks_https.dart';

const server = 'did:web:maintainer.example';
const alice = 'did:web:alice.example';
const mallory = 'did:web:mallory.example';
const aliceToken = 'alice-token';

const echoType = 'https://trusttasks.org/spec/example/echo/0.1';
const notifyType = 'https://trusttasks.org/spec/example/notify/0.1';

/// A request/response specification that requires nothing.
const echoSpec = SpecPolicy(typeUri: echoType);

/// A fire-and-forget specification (no success response).
const notifySpec = SpecPolicy(typeUri: notifyType);

class Echo {
  const Echo(this.text);
  factory Echo.fromJson(Map<String, dynamic> json) =>
      Echo(json['text'] as String);
  final String text;
  Map<String, dynamic> toJson() => {'text': text};
}

final fixedNow = DateTime.utc(2026, 6, 1, 12);
const fixedIssuedAt = '2026-06-01T12:00:00Z';

/// Accepts a proof whose proofValue is 'good'.
class FakeVerifier implements ProofVerifier {
  int calls = 0;
  @override
  Future<bool> verify(Map<String, dynamic> doc) async {
    calls++;
    return (doc['proof'] as Map<String, dynamic>)['proofValue'] == 'good';
  }
}

Map<String, dynamic> proof(String issuer, {String value = 'good'}) => {
      'type': 'DataIntegrityProof',
      'cryptosuite': 'eddsa-jcs-2022',
      'verificationMethod': '$issuer#key-1',
      'proofPurpose': 'assertionMethod',
      'created': fixedIssuedAt,
      'proofValue': value,
    };

Map<String, dynamic> echoDoc({
  String id = 'urn:uuid:00000000-0000-4000-8000-000000000001',
  String text = 'hello',
  String? issuer = alice,
  String? recipient = server,
  String? issuedAt = fixedIssuedAt,
  String type = echoType,
  Map<String, dynamic>? proof,
}) =>
    {
      'id': id,
      'type': type,
      if (issuer != null) 'issuer': issuer,
      if (recipient != null) 'recipient': recipient,
      if (issuedAt != null) 'issuedAt': issuedAt,
      'payload': <String, dynamic>{'text': text},
      if (proof != null) 'proof': proof,
    };

HttpsServer echoServer({
  ProofVerifier? verifier,
  Set<String>? allowedDidMethods,
  bool requireAttribution = true,
  bool replayProtection = true,
  String basePath = '',
  List<String>? calls,
}) {
  return HttpsServer(
    localVid: server,
    auth: StaticBearerAuth({aliceToken: alice}),
    proofVerifier: verifier,
    allowedDidMethods: allowedDidMethods,
    requireAttribution: requireAttribution,
    replayProtection: replayProtection,
    basePath: basePath,
    clock: () => fixedNow,
  )
    ..on<Echo, Echo>(
      spec: echoSpec,
      decode: Echo.fromJson,
      encode: (p) => p.toJson(),
      encodeResponse: (r) => r.toJson(),
      handler: (doc, ctx) async {
        calls?.add(doc.payload.text);
        if (doc.payload.text == 'refuse') {
          throw ctx.refuse(
            const RejectReason(
              code: StandardCode.permissionDenied,
              message: 'no',
            ),
          );
        }
        if (doc.payload.text == 'crash') throw StateError('boom');
        return Echo(doc.payload.text.toUpperCase());
      },
    )
    ..onAck<Echo>(
      spec: notifySpec,
      decode: Echo.fromJson,
      encode: (p) => p.toJson(),
      handler: (doc, ctx) async => calls?.add('notify:${doc.payload.text}'),
    );
}

HttpsRequest post(
  Object? body, {
  String? token = aliceToken,
  String contentType = 'application/json',
  String path = '/trust-tasks',
  String method = 'POST',
}) =>
    HttpsRequest(
      method: method,
      path: path,
      headers: {
        'Content-Type': contentType,
        if (token != null) 'Authorization': 'Bearer $token',
      },
      body: utf8.encode(body is String ? body : jsonEncode(body)),
    );

Map<String, dynamic> bodyOf(HttpsReply reply) =>
    jsonDecode(utf8.decode(reply.body!)) as Map<String, dynamic>;

String codeOf(HttpsReply reply) =>
    (bodyOf(reply)['payload'] as Map<String, dynamic>)['code'] as String;

/// An http.Client that hands every request to [server] in-process.
http.Client inProcess(HttpsServer server) => MockClient((request) async {
      final reply = await server.handle(
        HttpsRequest(
          method: request.method,
          path: request.url.path,
          headers: request.headers,
          body: request.bodyBytes,
        ),
      );
      return http.Response.bytes(
        reply.body ?? const [],
        reply.status,
        headers: reply.headers,
      );
    });
