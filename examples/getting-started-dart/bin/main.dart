// A signed acl/grant/0.3 round trip over HTTPS, both ends in one process — the
// Dart twin of examples/getting-started (Rust).
//
//   dart pub get
//   dart run trust_tasks:generate --lock trust-tasks.lock --out lib/trust_tasks
//   dart run bin/main.dart
//
// This is the worked example behind §3b of GETTING-STARTED.md at the repo
// root. The regions marked `GETTING-STARTED:begin/end` below are extracted
// verbatim into that document, and `scripts/check-getting-started-snippets.mjs`
// fails if the file and the document disagree — so the snippets a newcomer
// copies are code that compiles and runs, not code that once did.
//
// It is laid out as an application is: `trust-tasks.lock` pins the one task it
// speaks, and the generator writes its library into lib/trust_tasks/
// (git-ignored). The transport is `trust_tasks_https`, whose server runs the
// SPEC §7.2 pipeline per request, and the proofs are `trust_tasks_proof`.

import 'dart:convert';
import 'dart:io' as io;

// GETTING-STARTED:begin imports
import 'dart:typed_data';

import 'package:ssi/ssi.dart' as ssi;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_https/io.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

// The task trust-tasks.lock pins, generated into lib/trust_tasks/.
import 'package:trust_tasks_example_getting_started/trust_tasks/acl/grant/v0_3/payload.dart'
    as grant;
// Stands in for a JSON Schema engine — see lib/structural_validator.dart.
import 'package:trust_tasks_example_getting_started/structural_validator.dart';
// GETTING-STARTED:end imports

const serverVid = 'did:web:maintainer.example';

// ── Identity ────────────────────────────────────────────────────────────────
// GETTING-STARTED:begin identity
/// A `did:key` derived from a fixed seed, so both ends agree on the sender's
/// identity with no key exchange. Real deployments load a real key here.
ssi.DidSigner demoIdentity() {
  final keyPair =
      ssi.Ed25519KeyPair.fromSeed(Uint8List.fromList(List.filled(32, 7)));
  final didDoc = ssi.DidKey.generateDocument(keyPair.publicKey);
  return ssi.DidSigner(
    did: didDoc.id,
    didKeyId: didDoc.verificationMethod[0].id,
    keyPair: keyPair,
    signatureScheme: ssi.SignatureScheme.ed25519,
  );
}
// GETTING-STARTED:end identity

// ── Receiver ────────────────────────────────────────────────────────────────
// GETTING-STARTED:begin receiver
/// The consuming end: authenticate the peer, verify the proof, handle the task.
HttpsServer receiver(String aliceDid) {
  return HttpsServer(
    localVid: serverVid,
    // Bearer token → Verifiable Identifier. This is what the SPEC §4.8.1
    // cross-check compares the document's in-band `issuer` against, so it
    // must be the DID the sender actually signs with.
    auth: StaticBearerAuth({'alice': aliceDid}),
    // Without a verifier the server refuses every proof rather than trust it.
    // `forDidKey()` resolves `did:key` offline — no network, no DID document.
    proofVerifier: DataIntegrityProofVerifier.forDidKey(),
    payloadValidator: const StructuralValidator(),
    // The server keeps the duplicate-execution record (§7.2 item 11) itself:
    // an InMemoryReplayGuard unless you pass a shared one as `replayGuard:`.
  )..on<grant.Payload, grant.Response>(
      spec: grant.spec,
      decode: grant.Payload.fromJson,
      encode: (p) => p.toJson(),
      encodeResponse: (r) => r.toJson(),
      handler: (doc, ctx) async {
        final entry = doc.payload.entry;
        print('  [server] acl/grant from ${ctx.authenticatedSender} '
            '→ subject=${entry.subject} role=${entry.role}');
        return grant.Response(entry: entry);
      },
    );
}
// GETTING-STARTED:end receiver

// ── Sender ──────────────────────────────────────────────────────────────────
// GETTING-STARTED:begin sender
/// The producing end: build the document, sign it, send it, read the reply.
Future<TrustTaskDocument<grant.Payload>> sender(
  ssi.DidSigner signer,
  Uri serverUrl,
) async {
  // `myVid` must be the *signing* identity: the server cross-checks the
  // in-band `issuer` against whatever its bearer token maps to (§4.8.1).
  final client = HttpsClient(
    base: serverUrl,
    serverVid: serverVid,
    myVid: signer.did,
    token: 'alice',
  );

  // acl/grant/0.3 states every authority axis explicitly — there is no
  // default and no implicit form. Carol may act everywhere, may use no keys,
  // holds the full ceiling of the `moderator` role, and may use what she
  // reads for any purpose (`any` never includes `modelTraining`).
  const payload = grant.Payload(
    entry: grant.AclEntry(
      subject: 'did:web:carol.example',
      role: 'moderator',
      act: {'scope': 'all'},
      keys: {'scope': 'none'},
      capabilities: {'scope': 'ceiling'},
      purposes: {'scope': 'any'},
      label: 'Carol — content moderation',
    ),
    reason: 'onboarding moderator',
  );

  // TRAP: party members and `issuedAt` must be set BEFORE signing.
  // `HttpsClient.send` fills in any that are still unset — and a member filled
  // in after the fact is a member the proof does not cover, which the server
  // rejects as `proofInvalid`.
  final unsigned = TrustTaskDocument<grant.Payload>(
    id: newUrnUuid(),
    type: grant.typeUri,
    issuer: signer.did,
    recipient: serverVid,
    issuedAt: systemClock(), // whole seconds, UTC
    payload: payload,
  );
  final request = TrustTaskDocument<grant.Payload>.fromJson(
    await signTrustTask(unsigned.toJson((p) => p.toJson()), signer),
    (p) => grant.Payload.fromJson(p! as Map<String, dynamic>),
  );

  print('  [client] POST /trust-tasks  id=${request.id}');
  try {
    final resp = await client.send<grant.Payload, grant.Response>(
      request,
      encode: (p) => p.toJson(),
      decodeResponse: grant.Response.fromJson,
    );
    print('  [client] ← ${resp.type}\n'
        '           threadId=${resp.threadId} '
        'entry: subject=${resp.payload.entry.subject} '
        'role=${resp.payload.entry.role}');
  } on TrustTaskErrorException catch (e) {
    print('  [client] ← HTTP ${e.httpStatus} trust-task-error: '
        'code=${e.error.payload.code} retryable=${e.error.payload.retryable}');
  } finally {
    client.close();
  }
  return request;
}
// GETTING-STARTED:end sender

void check(bool ok, String what) {
  if (!ok) throw StateError('getting-started-dart: $what');
}

/// POST a document as-is; the HTTP status and the parsed body.
Future<(int, Map<String, dynamic>?)> post(Uri serverUrl, Object doc) async {
  final http = io.HttpClient();
  try {
    final req = await http.postUrl(serverUrl.resolve('/trust-tasks'));
    req.headers
      ..contentType = io.ContentType.json
      ..set(io.HttpHeaders.authorizationHeader, 'Bearer alice');
    req.write(jsonEncode(doc));
    final res = await req.close();
    final text = await res.transform(utf8.decoder).join();
    return (
      res.statusCode,
      text.isEmpty ? null : jsonDecode(text) as Map<String, dynamic>,
    );
  } finally {
    http.close();
  }
}

String? codeOf(Map<String, dynamic>? body) =>
    (body?['payload'] as Map<String, dynamic>?)?['code'] as String?;

Future<void> main() async {
  final signer = demoIdentity();
  final http = await serve(receiver(signer.did));
  final serverUrl = Uri.parse('http://127.0.0.1:${http.port}');
  print('  [server] listening on $serverUrl/trust-tasks (vid $serverVid)');

  try {
    // 1. The round trip the document walks through.
    final request = await sender(signer, serverUrl);
    final sent = request.toJson((p) => p.toJson());

    // 2. The same bytes again: §7.2 item 11 absorbs the retry and returns the
    //    first execution's response — the handler does not run a second time.
    final (status, retry) = await post(serverUrl, sent);
    check(status == 200, 'retry: HTTP $status');
    check(retry?['type'] == grant.responseTypeUri, 'retry: response type');
    check(retry?['threadId'] == request.id, 'retry: threadId');
    print('  [client] retry absorbed: same response, handler not re-run');

    // 3. A payload changed after signing: the proof no longer covers it.
    final tampered = jsonDecode(jsonEncode(sent)) as Map<String, dynamic>;
    tampered['id'] = newUrnUuid();
    ((tampered['payload'] as Map)['entry'] as Map)['role'] = 'owner';
    final (forgedStatus, forged) = await post(serverUrl, tampered);
    check(forgedStatus == 422 && codeOf(forged) == 'proofInvalid',
        'tampered: HTTP $forgedStatus $forged');
    print('  [client] tampered copy refused: proofInvalid');

    // 4. A payload the schema refuses, correctly signed: item 2 catches it.
    final invalid = jsonDecode(jsonEncode(sent)) as Map<String, dynamic>
      ..remove('proof')
      ..['id'] = newUrnUuid()
      ..['issuedAt'] = systemClock();
    ((invalid['payload'] as Map)['entry'] as Map).remove('purposes');
    final (badStatus, bad) =
        await post(serverUrl, await signTrustTask(invalid, signer));
    check(badStatus == 400 && codeOf(bad) == 'malformedRequest',
        'schema: HTTP $badStatus $bad');
    print('  [client] payload missing `purposes` refused: malformedRequest');

    print('getting-started-dart: round trip, retry, tamper and schema checks '
        'all OK');
  } finally {
    await http.close();
  }
}
