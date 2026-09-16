// A Trust Tasks server and client talking over a real socket.
//
// Run it:
//   dart run example/main.dart
//
// The server answers trust-task-discovery (SPEC §11); the client asks it what
// it supports, then re-sends the identical document, which the server absorbs
// as a retry rather than executing twice (bindings/https/0.2 §5.1).

import 'package:trust_tasks/specs/trust_task_discovery/v0_1/payload.dart'
    as discovery;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_https/io.dart';

const serverVid = 'did:web:maintainer.example';
const clientVid = 'did:web:org.example';

Future<void> main() async {
  final server = HttpsServer(
    localVid: serverVid,
    // A static map is for demos (§3). Production verifies a JWT, or looks the
    // token up somewhere the deployment controls.
    auth: StaticBearerAuth({'org-token': clientVid}),
  )..enableDiscovery();

  // Plain HTTP on loopback. A deployment terminates TLS in front of this.
  final http = await serve(server);
  print('serving on http://127.0.0.1:${http.port}/trust-tasks');

  final client = HttpsClient(
    base: Uri.parse('http://127.0.0.1:${http.port}'),
    serverVid: serverVid,
    myVid: clientVid,
    token: 'org-token',
  );

  final query = TrustTaskDocument<discovery.Payload>(
    id: newUrnUuid(),
    type: discovery.typeUri,
    // Stamped once, so the resend below is byte-for-byte the same document —
    // which is what SPEC §8.4 calls a retry.
    issuedAt: systemClock(),
    payload: const discovery.Payload(),
  );

  for (final attempt in ['first send', 'retry']) {
    try {
      final response = await client.send<discovery.Payload, discovery.Response>(
        query,
        encode: (p) => p.toJson(),
        decodeResponse: discovery.Response.fromJson,
      );
      print('$attempt: ${response.payload.supportedTypes}');
    } on HttpsClientException catch (e) {
      // Sealed: a switch over it is exhaustive.
      final summary = switch (e) {
        TrustTaskErrorException(:final error) =>
          'refused: ${error.payload.code}',
        DuplicateAbsorbedException() => 'already accepted',
        HttpStatusException(:final httpStatus) => 'HTTP $httpStatus',
        TransportException() => 'no answer',
        ResponseDecodeException() ||
        ResponseMismatchException() ||
        ResponseProofException() =>
          'unacceptable response: $e',
      };
      print('$attempt: $summary');
    }
  }

  client.close();
  await http.close();
}
