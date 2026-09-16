import 'dart:convert';

import 'package:trust_tasks/specs/trust_task_discovery/v0_1/payload.dart'
    as discovery;
import 'package:trust_tasks/trust_tasks.dart';

import 'binding.dart';

/// Largest request body the server reads (SPEC §10.2). Trust Task documents are
/// small, and the body is parsed before the sender is authenticated, so an
/// unbounded read would be a pre-authentication memory exhaustion vector.
const int maxBodyBytes = 256 * 1024;

/// Wire message for a body that is not a Trust Task document at all, and for
/// the suppressed `identityMismatch` answer (§8.1) — the same words for both, so
/// the two cannot be told apart (SPEC §10.4).
const String malformedBodyWireMessage =
    'request body is not a well-formed Trust Task document';

/// Wire message for the attribution gate. See
/// [HttpsServer.requireAttribution].
const String attributionRequiredWireMessage =
    'this request is not transport-authenticated and carries no proof';

/// Wire message for a handler that threw something other than a [Refusal].
const String internalErrorWireMessage = 'internal error';

/// An HTTP request, as far as this binding needs one.
///
/// Framework-agnostic on purpose: `package:trust_tasks_https/io.dart` adapts a
/// `dart:io` request to it, and a shelf, dart_frog or Cloud Functions handler
/// can build one in a few lines.
final class HttpsRequest {
  HttpsRequest({
    required this.method,
    required this.path,
    required Map<String, String> headers,
    required this.body,
  }) : headers = {
          for (final e in headers.entries) e.key.toLowerCase(): e.value,
        };

  final String method;

  /// The request path, without query string.
  final String path;

  /// Header names lower-cased.
  final Map<String, String> headers;

  final List<int> body;
}

/// The server's answer.
final class HttpsReply {
  const HttpsReply(this.status, {this.headers = const {}, this.body});

  /// A JSON body, with `Content-Type: application/json` (§4).
  factory HttpsReply.json(int status, Map<String, dynamic> document) =>
      HttpsReply(
        status,
        headers: const {'content-type': 'application/json'},
        body: utf8.encode(jsonEncode(document)),
      );

  final int status;
  final Map<String, String> headers;

  /// Null for the body-less answers: `202`, `204`, and the transport-level
  /// `404`, `405`, `413` and `415`.
  final List<int>? body;
}

/// What a handler is told about the exchange.
final class RequestContext {
  const RequestContext({
    required this.authenticatedSender,
    required this.local,
    required this.resolved,
    required TrustTaskDocument<Object?> request,
  }) : _request = request;

  /// The VID the bearer token mapped to, if any. Kept apart from [resolved] so
  /// a handler can tell "authenticated by the transport" from "asserted
  /// in-band".
  final String? authenticatedSender;

  /// The server's configured VID.
  final String? local;

  /// The §4.8.1-resolved parties: in-band where present, transport-derived
  /// otherwise.
  final ResolvedParties resolved;

  final TrustTaskDocument<Object?> _request;

  /// A refusal for a handler to throw, addressed to the producer.
  ///
  /// Not for refusals that contest the producer's identity; see [Refusal].
  Refusal refuse(RejectReason reason) =>
      Refusal(rejectWith(_request, newUrnUuid(), toErrorPayload(reason)));
}

typedef _Dispatch = Future<HttpsReply> Function(
  Map<String, dynamic> json,
  _Exchange exchange,
);

/// Per-request state shared between the envelope checks and a route.
final class _Exchange {
  _Exchange(this.transport, this.now);

  final HttpsTransport transport;
  final DateTime now;
}

/// A Trust Tasks server for the HTTPS binding (`bindings/https/0.2`).
///
/// Register a handler per specification with [on] (or [onAck] for a
/// specification that defines no success response), then hand each request to
/// [handle]. Serve it with `serve` from `package:trust_tasks_https/io.dart`, or
/// adapt [handle] to any other HTTP server.
///
/// ```dart
/// final server = HttpsServer(
///   localVid: 'did:web:maintainer.example',
///   auth: StaticBearerAuth({'s3cret': 'did:web:org.example'}),
/// )..on<acl_grant.Payload, acl_grant.Response>(
///     spec: acl_grant.spec,
///     decode: acl_grant.Payload.fromJson,
///     encode: (p) => p.toJson(),
///     encodeResponse: (r) => r.toJson(),
///     handler: (doc, ctx) async => acl_grant.Response(entry: doc.payload.entry),
///   );
/// ```
///
/// ## Request pipeline
///
/// In this order, which matches `trust-tasks-https` in Rust. Everything that
/// can make the server do work an unauthenticated sender chose — above all
/// resolving a `did:web` inside the proof verifier, an outbound request to a
/// host named in the body — runs after the cheap, local refusals.
///
/// 1. `POST` to `<basePath>/trust-tasks`, else `404` / `405`.
/// 2. `Content-Type: application/json`, else `415`. `text/plain` is a type a
///    browser may POST cross-origin without a preflight; requiring JSON makes
///    the browser refuse on this server's behalf.
/// 3. Body within [maxBodyBytes], else `413`; parses as a Trust Task document,
///    else `400`.
/// 4. Bearer token → transport-authenticated sender.
/// 5. Route lookup: an unregistered `type` is `unsupportedType`.
/// 6. Payload decode; identity cross-check (§4.8.1); expiry and recipient;
///    freshness.
/// 7. Attribution gate — see [requireAttribution].
/// 8. DID-method pre-screen — see [allowedDidMethods].
/// 9. Proof verification, the per-specification policy, the duplicate-execution
///    claim (§7.2 item 11), and the handler — `consumeInbound`.
final class HttpsServer {
  /// [proofVerifier] null means proof-bearing documents are refused with
  /// `malformedRequest` rather than accepted unverified — silently dropping a
  /// producer's proof would mislead it about what the exchange guarantees.
  HttpsServer({
    this.localVid,
    BearerAuthenticator? auth,
    this.proofVerifier,
    this.payloadValidator,
    ReplayGuard? replayGuard,
    this.replayProtection = true,
    this.freshness = FreshnessPolicy.consequential,
    this.requireAttribution = true,
    Set<String>? allowedDidMethods,
    String basePath = '',
    DateTime Function() clock = DateTime.now,
  })  : auth = auth ?? StaticBearerAuth(const {}),
        replayGuard = replayGuard ?? InMemoryReplayGuard(),
        allowedDidMethods = allowedDidMethods == null
            ? null
            : Set.unmodifiable(allowedDidMethods),
        _path = _normalizePath('$basePath$trustTasksPath'),
        _clock = clock;

  /// This server's VID: the transport-authenticated recipient (§3), and what an
  /// in-band `recipient` is checked against.
  final String? localVid;

  /// Maps bearer tokens to VIDs. The default authenticates nobody.
  final BearerAuthenticator auth;

  /// Verifies in-band proofs; see the constructor.
  final ProofVerifier? proofVerifier;

  /// Validates payloads against their schema (§7.2 item 2). Null skips it; the
  /// typed decode still refuses a payload of the wrong shape.
  final PayloadValidator? payloadValidator;

  /// The duplicate-execution record. The default is in-process, which is
  /// correct for one process and **wrong** behind a load balancer: each replica
  /// would accept the same document once (§5.1 item 6). Supply a shared store.
  final ReplayGuard replayGuard;

  /// Keep the §7.2 item 11 record. Defaults to true.
  ///
  /// This binding has no transport freshness (§5), so the record is the only
  /// thing between a captured request body — or an ordinary proxy retry — and a
  /// second execution. Turn it off only where every registered specification
  /// declares repeated execution safe, or for local development.
  final bool replayProtection;

  /// The acceptance window over `issuedAt` / `expiresAt`, and so also how long
  /// each replay record is kept: SPEC §7.2 makes them one bound.
  final FreshnessPolicy freshness;

  /// Refuse a document with **neither** a transport-authenticated sender
  /// **nor** a `proof`, with `proofRequired`. Defaults to true.
  ///
  /// Without it, an unauthenticated `POST` claiming
  /// `"issuer": "did:web:victim.example"` reaches the handler with that string
  /// as the resolved issuer, and nothing downstream can tell it from a genuine
  /// request. §5: "this binding does not permit `proof` to be omitted". Turn it
  /// off only for local development.
  final bool requireAttribution;

  /// When set, a `proof.verificationMethod` whose DID method is not listed
  /// (`{'key'}`, `{'web', 'webvh'}`) is refused before the verifier runs.
  /// Verifying means resolving, and resolving a `did:web` is an outbound request
  /// to a host the sender picked; this bounds where that can go.
  final Set<String>? allowedDidMethods;

  final String _path;
  final DateTime Function() _clock;
  final Map<String, _Dispatch> _routes = {};

  /// Register [handler] for the specification [spec] describes.
  ///
  /// [decode] reads the request payload, [encode] writes it back (for the
  /// replay digest and proof verification), and [encodeResponse] writes what
  /// [handler] returns, which the server wraps in the `#response` document.
  /// Throw `ctx.refuse(...)` from the handler to refuse.
  void on<P, R>({
    required SpecPolicy spec,
    required P Function(Map<String, dynamic> json) decode,
    required Object? Function(P payload) encode,
    required Object? Function(R payload) encodeResponse,
    required Future<R> Function(TrustTaskDocument<P> doc, RequestContext ctx)
        handler,
  }) {
    _routes[routingKey(spec.typeUri)] = (json, exchange) => _dispatch<P, R>(
          json: json,
          exchange: exchange,
          spec: spec,
          decode: decode,
          encode: encode,
          encodeResponse: encodeResponse,
          handler: (doc, ctx) async => respondWith<P, R>(
            doc,
            newUrnUuid(),
            await handler(doc, ctx),
          ),
        );
  }

  /// Register [handler] for a fire-and-forget specification — one that defines
  /// no success response (SPEC §4.4.1). Success is answered `204 No Content`.
  void onAck<P>({
    required SpecPolicy spec,
    required P Function(Map<String, dynamic> json) decode,
    required Object? Function(P payload) encode,
    required Future<void> Function(TrustTaskDocument<P> doc, RequestContext ctx)
        handler,
  }) {
    _routes[routingKey(spec.typeUri)] =
        (json, exchange) => _dispatch<P, Object?>(
              json: json,
              exchange: exchange,
              spec: spec,
              decode: decode,
              encode: encode,
              encodeResponse: (r) => r,
              handler: (doc, ctx) async {
                await handler(doc, ctx);
                return null;
              },
            );
  }

  /// Answer `trust-task-discovery/0.1` (SPEC §11) with every Type URI this
  /// server has a handler for, read at request time.
  ///
  /// A discovery response enumerates the route table, and SPEC §10 says a
  /// responder SHOULD authenticate the discoverer. So by default a caller with
  /// no transport-authenticated sender is refused with `permissionDenied`; pass
  /// [public] when the supported set is genuinely public.
  void enableDiscovery({bool public = false, String frameworkVersion = '0.5'}) {
    on<discovery.Payload, discovery.Response>(
      spec: discovery.spec,
      decode: discovery.Payload.fromJson,
      encode: (p) => p.toJson(),
      encodeResponse: (r) => r.toJson(),
      handler: (doc, ctx) async {
        if (!public && ctx.authenticatedSender == null) {
          // The generic wording any permission failure uses: a
          // discovery-specific message would confirm discovery is installed.
          throw ctx.refuse(
            const RejectReason(
              code: StandardCode.permissionDenied,
              message: 'permission denied',
            ),
          );
        }
        final patterns = doc.payload.patterns ?? const <String>[];
        final supported = <String>[
          for (final key in _routes.keys.toList()..sort())
            if (!key.contains('#') && _queryMatches(patterns, key)) key,
        ];
        return discovery.Response(
          supportedTypes: supported,
          frameworkVersion: frameworkVersion,
        );
      },
    );
  }

  /// Answer one request. Never throws.
  Future<HttpsReply> handle(HttpsRequest request) async {
    if (_normalizePath(request.path) != _path) return const HttpsReply(404);
    if (request.method.toUpperCase() != 'POST') {
      return const HttpsReply(405, headers: {'allow': 'POST'});
    }
    if (!_isJson(request.headers['content-type'])) {
      return const HttpsReply(415);
    }
    if (request.body.length > maxBodyBytes) return const HttpsReply(413);

    final Map<String, dynamic> json;
    final TrustTaskDocument<Object?> envelope;
    try {
      json = jsonDecode(utf8.decode(request.body)) as Map<String, dynamic>;
      envelope = TrustTaskDocument<Object?>.fromJson(json, (p) => p);
    } on Object {
      return _malformedBody();
    }

    String? peer;
    final header = request.headers['authorization'];
    if (header != null) {
      final token = _bearer(header);
      if (token != null) {
        try {
          peer = await auth.resolve(token);
        } on Object {
          // An authenticator that fails has authenticated nobody.
          peer = null;
        }
      }
    }

    final exchange = _Exchange(
      HttpsTransport(local: localVid, peer: peer),
      _clock(),
    );

    final route = _routes[routingKey(envelope.type)];
    if (route == null) {
      return _reject(
        exchange.transport,
        envelope,
        const RejectReason(
          code: StandardCode.unsupportedType,
          message: 'no handler is registered for this type',
        ),
      );
    }
    return route(json, exchange);
  }

  Future<HttpsReply> _dispatch<P, R>({
    required Map<String, dynamic> json,
    required _Exchange exchange,
    required SpecPolicy spec,
    required P Function(Map<String, dynamic> json) decode,
    required Object? Function(P payload) encode,
    required Object? Function(R payload) encodeResponse,
    required Future<TrustTaskDocument<R>?> Function(
      TrustTaskDocument<P> doc,
      RequestContext ctx,
    ) handler,
  }) async {
    final transport = exchange.transport;
    final now = exchange.now;

    final TrustTaskDocument<P> doc;
    try {
      doc = TrustTaskDocument<P>.fromJson(
        json,
        (p) => decode(p! as Map<String, dynamic>),
      );
    } on Object {
      // The decoder's own words name members and types — this consumer's
      // internal layout (SPEC §10.4). The category is what a producer can act on.
      return _reject(
        transport,
        TrustTaskDocument<Object?>.fromJson(json, (p) => p),
        const RejectReason(
          code: StandardCode.malformedRequest,
          message: 'payload does not match the specification',
        ),
      );
    }

    // The cheap, local checks, in the Rust server's order, so that an
    // unattributable document that is also stale or misaddressed is refused
    // for the same reason in both. consumeInbound repeats them below; they are
    // pure, so the repeat changes nothing.
    final resolution = resolveParties(transport, doc);
    if (resolution.error != null) {
      return _reject(transport, doc, identityMismatchReason(resolution.error!));
    }
    final basic = validateBasic(doc, now, localVid ?? '');
    if (basic != null) return _reject(transport, doc, basic);
    final fresh = validateFreshness(doc, now, freshness);
    if (fresh != null) return _reject(transport, doc, fresh);

    if (requireAttribution && transport.peer == null && doc.proof == null) {
      return _reject(
        transport,
        doc,
        const RejectReason(
          code: StandardCode.proofRequired,
          message: attributionRequiredWireMessage,
        ),
      );
    }

    final allowed = allowedDidMethods;
    final proof = doc.proof;
    if (allowed != null &&
        proof != null &&
        !_didMethodAllowed(proof.verificationMethod, allowed)) {
      return _reject(
        transport,
        doc,
        // Does not name the accepted methods: that is deployment configuration,
        // and echoing it would let a probe fingerprint the fleet.
        const RejectReason(
          code: StandardCode.proofInvalid,
          message: proofInvalidWireMessage,
        ),
      );
    }

    final ctx = RequestContext(
      authenticatedSender: transport.peer,
      local: localVid,
      resolved: resolution.parties!,
      request: TrustTaskDocument<Object?>.fromJson(json, (p) => p),
    );

    final verifier = proofVerifier;
    final validator = payloadValidator;
    final ConsumeOutcome<R> outcome;
    try {
      outcome = await consumeInbound<P, R>(
        transport: transport,
        spec: spec,
        proofPolicy: verifier == null
            ? const ProofPolicy.rejectIfPresent()
            : ProofPolicy.verify(_AsReceived(verifier, json)),
        payloadPolicy: validator == null
            ? const PayloadPolicy.acceptUnvalidated()
            : PayloadPolicy.validate(validator),
        checks: ConsumeChecks(
          freshness: freshness,
          replay: replayProtection
              ? ReplayPolicy.guarded(replayGuard)
              : const ReplayPolicy.notConsequential(),
        ),
        doc: doc,
        myVid: localVid ?? '',
        now: now,
        newErrorId: newUrnUuid,
        payloadToJson: encode,
        handler: (accepted, parties) => handler(accepted, ctx),
      );
    } on Object {
      // The handler threw something that is not a Refusal. Nothing may be
      // remembered as executed on its behalf: holding the claim would absorb
      // every honest resend in silence until the record expires.
      if (replayProtection) {
        try {
          await replayGuard.release(doc.id, documentDigest(doc, encode));
        } on Object {
          /* best-effort */
        }
      }
      return _reject(
        transport,
        doc,
        const RejectReason(
          code: StandardCode.internalError,
          message: internalErrorWireMessage,
        ),
      );
    }

    switch (outcome) {
      case Handled(:final response):
        return HttpsReply.json(200, response.toJson(encodeResponse));
      case Accepted():
        return const HttpsReply(204);
      case Rejected(:final error):
        return _errorReply(error);
      case Suppressed():
        // §8.1 says emit nothing; HTTP cannot. Answer exactly as for a body
        // that did not parse, so a prober cannot learn the identity was
        // contested (SPEC §10.4).
        return _malformedBody();
      case DuplicateOutcome(:final priorResponse, :final inFlight):
        // §5.1 item 4 and the §4 table: the first execution's result under the
        // status it earned, else 202 while that execution runs, else 204.
        final prior = switch (priorResponse) {
          final TrustTaskDocument<R> d => d.toJson(encodeResponse),
          final Map<String, dynamic> m => m,
          _ => null,
        };
        if (prior != null) {
          return _isErrorDocument(prior)
              ? HttpsReply.json(_statusOf(prior), prior)
              : HttpsReply.json(200, prior);
        }
        return HttpsReply(inFlight ? 202 : 204);
    }
  }

  HttpsReply _reject<P>(
    HttpsTransport transport,
    TrustTaskDocument<P> doc,
    RejectReason reason,
  ) {
    final error = reject(transport, doc, newUrnUuid(), reason);
    return error == null ? _malformedBody() : _errorReply(error);
  }

  HttpsReply _errorReply(ErrorResponse error) => HttpsReply.json(
        statusForCode(error.payload.code),
        error.toJson((p) => p.toJson()),
      );

  HttpsReply _malformedBody() => HttpsReply.json(
        400,
        ErrorResponse(
          id: newUrnUuid(),
          type: trustTaskErrorTypeUri,
          issuedAt: systemClock(),
          payload: const ErrorPayload(
            code: 'malformedRequest',
            retryable: false,
            message: malformedBodyWireMessage,
          ),
        ).toJson((p) => p.toJson()),
      );
}

/// Verifies the document as it arrived, not as the typed decode would write it
/// back. A payload member the generated type does not model — one from a newer
/// MINOR (§5.2) — is dropped by decoding, and a signature over the original
/// bytes would then fail for a document that is perfectly valid. The Rust server
/// verifies the untyped document for the same reason.
final class _AsReceived implements ProofVerifier {
  const _AsReceived(this.inner, this.json);

  final ProofVerifier inner;
  final Map<String, dynamic> json;

  @override
  Future<bool> verify(Map<String, dynamic> doc) => inner.verify(json);
}

bool _isErrorDocument(Map<String, dynamic> doc) {
  final type = doc['type'];
  return type is String && bareTypeUri(type).contains('/trust-task-error/');
}

int _statusOf(Map<String, dynamic> errorDoc) {
  final payload = errorDoc['payload'];
  final code = payload is Map ? payload['code'] : null;
  return code is String ? statusForCode(code) : 500;
}

/// `application/json`, parameters ignored. A missing header is not JSON: the
/// binding makes `Content-Type` a MUST, and guessing would reopen the
/// preflight-free path the check exists to close.
bool _isJson(String? contentType) {
  if (contentType == null) return false;
  return contentType.split(';').first.trim().toLowerCase() ==
      'application/json';
}

String? _bearer(String header) {
  final space = header.indexOf(' ');
  if (space < 0) return null;
  if (header.substring(0, space).toLowerCase() != 'bearer') return null;
  final token = header.substring(space + 1).trim();
  return token.isEmpty ? null : token;
}

/// `/a/b/` and `/a/b` are the same path (§6.1: a trailing `/` is ignored).
String _normalizePath(String path) {
  var p = path.isEmpty ? '/' : path;
  if (!p.startsWith('/')) p = '/$p';
  while (p.length > 1 && p.endsWith('/')) {
    p = p.substring(0, p.length - 1);
  }
  return p.replaceAll(RegExp('/{2,}'), '/');
}

/// A DID URL (`did:<method>:<rest>`) whose method is in [allowed]. Anything that
/// is not a DID URL fails — an `https:` verification method would hand the
/// resolver an arbitrary URL.
bool _didMethodAllowed(String verificationMethod, Set<String> allowed) {
  if (!verificationMethod.startsWith('did:')) return false;
  final rest = verificationMethod.substring(4);
  final colon = rest.indexOf(':');
  if (colon <= 0 || colon == rest.length - 1) return false;
  return allowed.contains(rest.substring(0, colon));
}

/// SPEC §11 slug-glob matching: `*`, `<prefix>/*`, or an exact slug; an empty
/// pattern list matches everything.
bool _queryMatches(List<String> patterns, String typeUri) {
  if (patterns.isEmpty) return true;
  final String slug;
  try {
    slug = slugFromTypeUri(typeUri);
  } on ArgumentError {
    return false;
  }
  for (final pattern in patterns) {
    if (pattern == '*') return true;
    if (pattern.endsWith('/*')) {
      final prefix = pattern.substring(0, pattern.length - 2);
      if (prefix.isNotEmpty && slug.startsWith('$prefix/')) return true;
    } else if (pattern == slug) {
      return true;
    }
  }
  return false;
}
