# trust_tasks_https

The [Trust Tasks](https://trusttasks.org) HTTPS transport binding
([`bindings/https/0.2`](https://trusttasks.org/binding/https/0.2)) for Dart: a
typed client, and a server that runs the SPEC §7.2 consumer pipeline on every
request. Built on [`package:trust_tasks`](https://pub.dev/packages/trust_tasks);
the Dart counterpart of the Rust
[`trust-tasks-https`](https://crates.io/crates/trust-tasks-https) crate, and
interoperable with it in both directions.

```console
dart pub add trust_tasks trust_tasks_https
```

Requires Dart 3.3 or later.

## Client

```dart
import 'package:trust_tasks_https/trust_tasks_https.dart';

final client = HttpsClient(
  base: Uri.parse('https://maintainer.example'), // POSTs to <base>/trust-tasks
  serverVid: 'did:web:maintainer.example',
  myVid: 'did:web:org.example',
  token: 's3cret',
);

final response = await client.send<acl_grant.Payload, acl_grant.Response>(
  request,
  encode: (p) => p.toJson(),
  decodeResponse: acl_grant.Response.fromJson,
);
```

`send` fills in `issuer`, `recipient` and `issuedAt` where the request leaves
them unset. It returns the `#response` document only when that document belongs
to this request: same thread, the request's type with `#response`, issued by
`serverVid`, addressed to `myVid`. Anything else throws a subclass of the sealed
`HttpsClientException`:

| Exception | Meaning |
|---|---|
| `TrustTaskErrorException` | The server refused with a `trust-task-error`; the code is authoritative |
| `UnverifiedErrorResponseException` | With a `responseVerifier`: a `trust-task-error` without a valid proof from `serverVid` to `myVid`. **Untrusted** — kept for diagnostics, not the server's answer; the outcome is unknown |
| `DuplicateAbsorbedException` | `202`/`204`: the server had already accepted this document. **Not a failure** |
| `HttpStatusException` | A non-2xx with no error document — a proxy, a `401`, a redirect. Says nothing about the task |
| `TransportException` | No answer at all — connection, TLS, timeout or any other client error. Retry by re-sending the same bytes (SPEC §8.4) |
| `ResponseMismatchException` | A response, but not to this request, or not from this server |
| `ResponseTooLargeException` | The body exceeded `maxResponseBytes` (256 KiB by default); reading stopped there |
| `ResponseDecodeException`, `ResponseProofException` | A response this client cannot accept |

Use `sendAck` for a specification that defines no success response. Pass
`responseVerifier` to require a verified proof on every response; it requires
`serverVid`. The proof is verified over the body as received, and error
responses are held to it too: an error without a valid proof from the
configured server comes back as `UnverifiedErrorResponseException`, not as the
server's answer. The client uses `package:http`, so it runs on the web and in
Flutter.

The base must be `https`. A plain `http` base is accepted on loopback, or
elsewhere with `allowInsecureHttp: true` — but a `token` is never sent over
non-loopback `http`. Redirects are not followed (the signed document is never
re-posted to another URL), the response body is capped at `maxResponseBytes`,
and `timeout` covers reading the body as well as connecting. Exception
messages do not quote the response body.

## Server

```dart
import 'package:trust_tasks_https/io.dart';

final server = HttpsServer(
  localVid: 'did:web:maintainer.example',
  auth: StaticBearerAuth({'s3cret': 'did:web:org.example'}),
  proofVerifier: DataIntegrityProofVerifier.forDidKey(), // trust_tasks_proof
)
  ..on<acl_grant.Payload, acl_grant.Response>(
    spec: acl_grant.spec,
    decode: acl_grant.Payload.fromJson,
    encode: (p) => p.toJson(),
    encodeResponse: (r) => r.toJson(),
    handler: (doc, ctx) async {
      if (!mayGrant(ctx.resolved.issuer)) {
        throw ctx.refuse(const RejectReason(
          code: StandardCode.permissionDenied,
          message: 'not an administrator',
        ));
      }
      return acl_grant.Response(entry: await grant(doc.payload.entry));
    },
  )
  ..enableDiscovery();

await serve(server, port: 8443, securityContext: tls);
```

`HttpsServer.handle` takes a plain `HttpsRequest` and returns an `HttpsReply`,
so the server is not tied to `dart:io`: `package:trust_tasks_https/io.dart`
adapts it to `dart:io`, and shelf, dart_frog or a Cloud Functions handler need
only a few lines. `io.dart` is a separate library so that the client keeps
running on the web.

### What every request goes through

In the same order as the Rust server, so that the two refuse the same document
for the same reason, and so that nothing an unauthenticated sender picks —
above all resolving a `did:web` during proof verification — runs before the
cheap local checks:

1. `POST <base>/trust-tasks`, `Content-Type: application/json`, at most
   `bodyLimitBytes` (`413` otherwise): the largest `maxDocumentBytes` among
   the registered specifications, never under `defaultMaxDocumentBytes`
   (256 KiB unless you pass another).
2. The bearer token is mapped to the sender's VID.
3. Routing. A registered `type` reaches its handler. A later version whose
   declared `extends` chain (SPEC §5.2) reaches a registered version is read
   as the highest such version and answered as `<received type>#response`;
   any other version of a registered specification is `unsupportedVersion`,
   and anything else `unsupportedType`.
4. The document is within its specification's `maxDocumentBytes` — or
   `defaultMaxDocumentBytes` where it declares none — else `malformedRequest`
   (SPEC §12.4: never refused within the bound, SHOULD be above it).
5. The payload decodes; the in-band identity agrees with the token (§4.8.1);
   expiry, recipient and the freshness window hold.
6. **Attribution:** a request with neither a recognised token nor a `proof` is
   refused. §5 of the binding does not let a proof be omitted, and without this
   anyone could claim any `issuer`.
7. With a `proofVerifier`, `proof.verificationMethod` is screened by DID
   method: by default only `did:key` passes (`defaultAllowedDidMethods`),
   since resolving any other method can mean a fetch from a host the sender
   names. Widen it with `allowedDidMethods: {'key', 'web'}`, or lift it with
   `allowAnyDidMethod: true` for a verifier whose resolver bounds its own
   egress.
8. The proof verifies — or, with no `proofVerifier`, a proof is refused rather
   than silently ignored.
9. The specification's own policy, then the **duplicate-execution claim**: a
   byte-identical resend gets the first response back without running the
   handler again, a different document under the same `id` is `idConflict`, and
   a store that cannot be consulted fails closed with `unavailable`.

Statuses follow the binding's §4 table. The codes that concern identity —
`proofRequired`, `proofInvalid`, `identityMismatch`, `wrongRecipient` — all
share `422`, and a contested identity with no authenticated sender is answered
exactly like an unparseable body, so none of them works as an oracle.

**Behind a load balancer, pass a shared `replayGuard`.** The default record is
in-process; two replicas would each accept the same document once (§5.1).

**Pass `onEvent` to see what the wire does not say.** A handler that throws is
answered `internalError`, a suppressed refusal looks like an unparseable body,
and a proof failure is a constant `proofInvalid` — on purpose (SPEC §12.4). The
hook receives a `ServerEvent` (from `package:trust_tasks`) for each: the
handler's exception and stack, the identity disagreement, what the verifier
threw, an authenticator that failed. It never changes an answer, and anything
it throws is dropped:

```dart
final server = HttpsServer(
  localVid: 'did:web:maintainer.example',
  onEvent: (event) => switch (event) {
    HandlerFailed(:final error, :final stackTrace) =>
      log.severe('handler threw', error, stackTrace),
    _ => log.fine(event),
  },
);
```

## Interoperability

Checked against the Rust crate's own demos, with `trust_tasks_proof` signing
and verifying:

- a Dart client sending a signed `acl/grant` to the Rust `server_demo` gets the
  response; the retry gets the same response without the handler running
  again, and a document whose signature no longer matches is refused with
  `proofInvalid`;
- the Rust `client_demo` sending a signed `acl/grant` to a Dart server gets the
  response, including payload members the Dart types model only as optional.

## License

Apache-2.0.
