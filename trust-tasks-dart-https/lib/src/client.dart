import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:http/http.dart' as http;
import 'package:trust_tasks/trust_tasks.dart';

import 'binding.dart';

/// Default budget for one exchange, connection included.
const Duration defaultClientTimeout = Duration(seconds: 30);

/// Default cap on a response body: 256 KiB, the request cap the servers apply.
/// A trust-task response is one small JSON document.
const int defaultMaxResponseBytes = 256 * 1024;

/// How much of a non-Trust-Task error body [HttpStatusException.body] keeps.
const int _maxErrorBodyChars = 512;

/// Why [HttpsClient.send] did not return a response document.
///
/// Sealed, so a `switch` over it is exhaustive.
sealed class HttpsClientException implements Exception {
  const HttpsClientException();
}

/// The server refused with a `trust-task-error` document. [error] carries the
/// framework code, which is authoritative; [httpStatus] is informative (§4).
final class TrustTaskErrorException extends HttpsClientException {
  const TrustTaskErrorException(this.httpStatus, this.error);

  final int httpStatus;
  final ErrorResponse error;

  @override
  String toString() => 'TrustTaskErrorException(HTTP $httpStatus, '
      '${error.payload.code}: ${error.payload.message})';
}

/// A non-2xx answer with no `trust-task-error` body: a proxy, a `401`, a bare
/// `500`, or a redirect (which the client does not follow). §4.1: this
/// establishes nothing about the document's state. Retry by re-sending the same
/// bytes (SPEC §8.4).
final class HttpStatusException extends HttpsClientException {
  const HttpStatusException(this.httpStatus, this.body);

  final int httpStatus;

  /// The start of the body (at most 512 characters), for diagnostics. Not
  /// included in [toString]: it is whatever the peer chose to send.
  final String body;

  @override
  String toString() => 'HttpStatusException(HTTP $httpStatus)';
}

/// The exchange never produced an answer: a connection or TLS failure, the
/// [HttpsClient.timeout], or any other error raised by the HTTP client. Like
/// [HttpStatusException], not a statement about the document (§4.1).
final class TransportException extends HttpsClientException {
  const TransportException(this.cause);

  final Object cause;

  @override
  String toString() => 'TransportException($cause)';
}

/// The consumer had already accepted this document (§5.1) and returned no
/// result: `202` while the first execution runs, `204` once it has finished
/// with nothing retained. **Not a failure** — the task did not fail, it already
/// happened, or is happening.
final class DuplicateAbsorbedException extends HttpsClientException {
  const DuplicateAbsorbedException(this.httpStatus);

  final int httpStatus;

  /// `202`: the first execution has not finished.
  bool get inFlight => httpStatus == 202;

  @override
  String toString() => 'DuplicateAbsorbedException(HTTP $httpStatus)';
}

/// A 2xx body that is not the document this client can accept.
final class ResponseDecodeException extends HttpsClientException {
  const ResponseDecodeException(this.cause);

  /// The decoding error. Its text may quote the body, so [toString] names
  /// only its type.
  final Object cause;

  @override
  String toString() => 'ResponseDecodeException(${cause.runtimeType})';
}

/// The response body was larger than [HttpsClient.maxResponseBytes]. Reading
/// stopped there; nothing was decoded.
final class ResponseTooLargeException extends HttpsClientException {
  const ResponseTooLargeException(this.limit);

  /// The cap that was exceeded, in bytes.
  final int limit;

  @override
  String toString() => 'ResponseTooLargeException(over $limit bytes)';
}

/// Which binding of a response to its request did not hold.
enum ResponseBinding {
  /// `threadId` is not the request's thread (its `threadId`, else its `id`).
  threadId,

  /// `type` is not the request's Type URI with `#response`.
  type,

  /// `issuer` is not the configured server VID.
  issuer,

  /// `recipient` is not this client's VID.
  recipient,

  /// An error response's `inResponseTo.id` names another document.
  inResponseTo,
}

/// A response that does not belong to the request that was sent.
///
/// A well-formed, correctly signed document is still the wrong answer if it
/// answers a different request or comes from a different party.
final class ResponseMismatchException extends HttpsClientException {
  const ResponseMismatchException(this.binding, this.expected, this.actual);

  final ResponseBinding binding;
  final String expected;
  final String? actual;

  @override
  String toString() =>
      'ResponseMismatchException($binding: expected $expected, got $actual)';
}

/// A [HttpsClient.responseVerifier] is configured and the response's proof is
/// missing ([missing]) or does not verify.
final class ResponseProofException extends HttpsClientException {
  const ResponseProofException({required this.missing});

  final bool missing;

  @override
  String toString() =>
      'ResponseProofException(${missing ? 'missing' : 'invalid'})';
}

/// A non-2xx response carried a `trust-task-error` document that this client,
/// configured with a [HttpsClient.responseVerifier], could not attest: it had
/// no proof, its proof did not verify, or its `issuer` / `recipient` is not the
/// configured server / this client.
///
/// **Untrusted.** [error] is what arrived, kept for diagnostics; anything on
/// the path could have written it. Do not act on its code as the server's
/// verdict — treat the request's outcome as unknown.
final class UnverifiedErrorResponseException extends HttpsClientException {
  const UnverifiedErrorResponseException(
    this.httpStatus,
    this.reason,
    this.error,
  );

  /// The HTTP status code that accompanied it.
  final int httpStatus;

  /// Why it could not be attested.
  final String reason;

  /// The document as received. Unverified.
  final ErrorResponse error;

  /// Names neither the code nor the message: both are unattested.
  @override
  String toString() =>
      'UnverifiedErrorResponseException(HTTP $httpStatus: $reason)';
}

/// A client for the HTTPS binding (`bindings/https/0.2`).
///
/// ```dart
/// final client = HttpsClient(
///   base: Uri.parse('https://maintainer.example'),
///   serverVid: 'did:web:maintainer.example',
///   myVid: 'did:web:org.example',
///   token: 's3cret',
/// );
/// final response = await client.send<acl_grant.Payload, acl_grant.Response>(
///   request,
///   encode: (p) => p.toJson(),
///   decodeResponse: acl_grant.Response.fromJson,
/// );
/// ```
final class HttpsClient {
  /// [base] is the Trust-Task base (§6.1): the request goes to
  /// `<base>/trust-tasks`, with a trailing `/` on [base] ignored.
  ///
  /// Pass [httpClient] to reuse a connection pool, or to substitute a platform
  /// client; this client never closes one it was given.
  ///
  /// [base] must be `https`. A plain `http` base is accepted only on loopback
  /// (`localhost`, `127.0.0.0/8`, `::1`) or with [allowInsecureHttp], and a
  /// [token] is never sent over non-loopback `http`, whatever
  /// [allowInsecureHttp] says: it would cross the network in the clear.
  ///
  /// A [responseVerifier] needs a [serverVid]: a proof only says who signed,
  /// and without a VID to compare the signer to, any signed document would
  /// pass.
  ///
  /// Throws [ArgumentError] for a configuration that breaks these rules.
  HttpsClient({
    required Uri base,
    this.serverVid,
    this.myVid,
    this.token,
    this.responseVerifier,
    this.timeout = defaultClientTimeout,
    this.maxResponseBytes = defaultMaxResponseBytes,
    bool allowInsecureHttp = false,
    http.Client? httpClient,
  })  : endpoint = _endpoint(
          base,
          allowInsecureHttp: allowInsecureHttp,
          hasToken: token != null,
        ),
        _http = httpClient ?? http.Client(),
        _ownsHttp = httpClient == null {
    if (responseVerifier != null && serverVid == null) {
      throw ArgumentError.value(
        serverVid,
        'serverVid',
        'is required when responseVerifier is set',
      );
    }
    if (maxResponseBytes <= 0) {
      throw ArgumentError.value(
        maxResponseBytes,
        'maxResponseBytes',
        'must be positive',
      );
    }
  }

  /// Where requests are posted.
  final Uri endpoint;

  /// The server's VID. Stamped as `recipient` on a request that has none, and
  /// required as the `issuer` of every success response.
  final String? serverVid;

  /// This client's VID. Stamped as `issuer` on a request that has none, and
  /// required as the `recipient` of every success response.
  final String? myVid;

  /// The bearer token (§3).
  final String? token;

  /// When set, every success response must carry a proof that verifies, over
  /// the body exactly as received.
  ///
  /// **Error responses are held to it too.** A `trust-task-error` is thrown as
  /// [TrustTaskErrorException] only when its proof verifies and its `issuer` /
  /// `recipient` are [serverVid] / [myVid], as a success response's must be.
  /// Anything else — an unsigned error, a bad proof, an error from another
  /// party — is [UnverifiedErrorResponseException]: the document is kept for
  /// diagnostics, but nothing about it is attested, and it must not be read as
  /// the server's answer.
  final ProofVerifier? responseVerifier;

  /// Budget for one exchange, reading the response included.
  final Duration timeout;

  /// The largest response body read, in bytes; a larger one is
  /// [ResponseTooLargeException].
  final int maxResponseBytes;

  final http.Client _http;
  final bool _ownsHttp;

  static bool _sameResource(Uri a, Uri b) =>
      a.scheme == b.scheme &&
      a.host.toLowerCase() == b.host.toLowerCase() &&
      a.port == b.port &&
      a.path == b.path;

  static bool _isLoopback(String host) {
    final h = host.toLowerCase();
    if (h == 'localhost' || h.endsWith('.localhost')) return true;
    if (h == '::1' || h == '[::1]') return true;
    final octets = h.split('.');
    return octets.length == 4 &&
        octets.first == '127' &&
        octets.every((o) => int.tryParse(o) != null);
  }

  static Uri _endpoint(
    Uri base, {
    required bool allowInsecureHttp,
    required bool hasToken,
  }) {
    final scheme = base.scheme.toLowerCase();
    if (scheme == 'http' && !_isLoopback(base.host)) {
      if (hasToken) {
        throw ArgumentError.value(
          base,
          'base',
          'is plain http: a bearer token is only sent over https '
              '(or to loopback)',
        );
      }
      if (!allowInsecureHttp) {
        throw ArgumentError.value(
          base,
          'base',
          'is plain http: use https, or pass allowInsecureHttp: true',
        );
      }
    } else if (scheme != 'https' && scheme != 'http') {
      throw ArgumentError.value(base, 'base', 'is not an https URL');
    }
    var path = base.path;
    while (path.endsWith('/')) {
      path = path.substring(0, path.length - 1);
    }
    return base.replace(path: '$path$trustTasksPath');
  }

  /// Send [request] and return the `#response` document.
  ///
  /// `issuer`, `recipient` and `issuedAt` are filled from [myVid], [serverVid]
  /// and the clock where the request leaves them unset. Throws a
  /// [HttpsClientException] for anything but a response that belongs to this
  /// request.
  Future<TrustTaskDocument<R>> send<P, R>(
    TrustTaskDocument<P> request, {
    required Object? Function(P payload) encode,
    required R Function(Map<String, dynamic> json) decodeResponse,
  }) async {
    final stamped = _stamp(request);
    final reply = await _post(stamped.toJson(encode));

    if (reply.statusCode == 202 || reply.statusCode == 204) {
      throw DuplicateAbsorbedException(reply.statusCode);
    }
    await _throwIfError(reply, stamped);
    if (reply.bodyBytes.isEmpty) {
      throw DuplicateAbsorbedException(reply.statusCode);
    }

    final Map<String, dynamic> json;
    final TrustTaskDocument<R> response;
    try {
      json = jsonDecode(utf8.decode(reply.bodyBytes)) as Map<String, dynamic>;
      response = TrustTaskDocument<R>.fromJson(
        json,
        (p) => decodeResponse(p! as Map<String, dynamic>),
      );
    } on Object catch (e) {
      throw ResponseDecodeException(e);
    }

    _checkBinding(stamped, response);
    await _verify(json);
    return response;
  }

  /// Send [request] to a fire-and-forget specification (SPEC §4.4.1), whose
  /// consumer answers success with `204 No Content`.
  ///
  /// Returns normally on `204`, and on `202` (a duplicate whose first execution
  /// is still running — accepted all the same). Throws a
  /// [HttpsClientException] otherwise.
  Future<void> sendAck<P>(
    TrustTaskDocument<P> request, {
    required Object? Function(P payload) encode,
  }) async {
    final stamped = _stamp(request);
    final reply = await _post(stamped.toJson(encode));
    if (reply.statusCode == 202 || reply.statusCode == 204) return;
    await _throwIfError(reply, stamped);
    // A 2xx with a body: the framework's optional empty `{}` #response, which
    // a fire-and-forget consumer may send as a courtesy (§4.1, `responded`).
  }

  /// Close the underlying HTTP client, if this client created it.
  void close() {
    if (_ownsHttp) _http.close();
  }

  TrustTaskDocument<P> _stamp<P>(TrustTaskDocument<P> request) =>
      request.copyWith(
        issuer: request.issuer ?? myVid,
        recipient: request.recipient ?? serverVid,
        issuedAt: request.issuedAt ?? systemClock(),
      );

  /// POST [document] and read the reply, within [timeout] and
  /// [maxResponseBytes]. Redirects are not followed: a `3xx` comes back as
  /// itself, so the signed document is never re-sent to another URL. Every
  /// error the HTTP client raises — TLS included — is a [TransportException].
  Future<http.Response> _post(Map<String, dynamic> document) async {
    final request = http.Request('POST', endpoint)
      ..followRedirects = false
      ..headers.addAll({
        'content-type': 'application/json',
        'accept': 'application/json',
        if (token != null) 'authorization': 'Bearer $token',
      })
      ..body = jsonEncode(document);
    try {
      return await _exchange(request);
    } on HttpsClientException {
      rethrow;
    } on Object catch (e) {
      throw TransportException(e);
    }
  }

  Future<http.Response> _exchange(http.Request request) async {
    final clock = Stopwatch()..start();
    final streamed = await _http.send(request).timeout(timeout);
    // A platform client that follows redirects anyway (a browser's fetch)
    // reports where it ended up; an answer from anywhere else is not this
    // endpoint's.
    final finalUrl = streamed is http.BaseResponseWithUrl
        ? (streamed as http.BaseResponseWithUrl).url
        : null;
    if (finalUrl != null && !_sameResource(finalUrl, endpoint)) {
      unawaited(streamed.stream.listen(null).cancel());
      throw TransportException(
        StateError('the response came from another URL after a redirect'),
      );
    }
    final declared = streamed.contentLength;
    if (declared != null && declared > maxResponseBytes) {
      unawaited(streamed.stream.listen(null).cancel());
      throw ResponseTooLargeException(maxResponseBytes);
    }
    final bytes = BytesBuilder(copy: false);
    final done = Completer<void>();
    late final StreamSubscription<List<int>> sub;
    sub = streamed.stream.listen(
      (chunk) {
        if (bytes.length + chunk.length > maxResponseBytes) {
          unawaited(sub.cancel());
          if (!done.isCompleted) {
            done.completeError(ResponseTooLargeException(maxResponseBytes));
          }
          return;
        }
        bytes.add(chunk);
      },
      onError: (Object e, StackTrace st) {
        if (!done.isCompleted) done.completeError(e, st);
      },
      onDone: () {
        if (!done.isCompleted) done.complete();
      },
      cancelOnError: true,
    );
    final remaining = timeout - clock.elapsed;
    await done.future.timeout(
      remaining.isNegative ? Duration.zero : remaining,
      onTimeout: () {
        // Stop reading: the connection is released rather than left
        // draining a body nobody will look at.
        unawaited(sub.cancel());
        throw TimeoutException('reading the response', timeout);
      },
    );
    return http.Response.bytes(
      bytes.takeBytes(),
      streamed.statusCode,
      headers: streamed.headers,
      request: request,
    );
  }

  /// §4: a non-2xx answer with a JSON body is read as a `trust-task-error`
  /// first, and only then treated as a transport failure.
  Future<void> _throwIfError<P>(
    http.Response reply,
    TrustTaskDocument<P> request,
  ) async {
    final status = reply.statusCode;
    if (status >= 200 && status < 300) return;

    final contentType = reply.headers['content-type'] ?? '';
    if (contentType.split(';').first.trim().toLowerCase() ==
        'application/json') {
      ErrorResponse? error;
      Map<String, dynamic>? received;
      try {
        final json =
            jsonDecode(utf8.decode(reply.bodyBytes)) as Map<String, dynamic>;
        if (bareTypeUri(json['type'] as String)
            .contains('/trust-task-error/')) {
          error = ErrorResponse.fromJson(
            json,
            (p) => ErrorPayload.fromJson(p! as Map<String, dynamic>),
          );
          received = json;
        }
      } on Object {
        error = null;
      }
      if (error != null) {
        // With a response verifier configured, an error is the server's
        // answer only once it is attested as one: a valid proof, from the
        // configured server, to this client. Otherwise anything on the path
        // could answer every request with an unsigned refusal.
        final reason = await _attestError(error, received!);
        if (reason != null) {
          throw UnverifiedErrorResponseException(status, reason, error);
        }
        final reported = error.payload.inResponseTo?.id;
        if (reported != null && reported != request.id) {
          throw ResponseMismatchException(
            ResponseBinding.inResponseTo,
            request.id,
            reported,
          );
        }
        throw TrustTaskErrorException(status, error);
      }
    }
    final text = utf8.decode(reply.bodyBytes, allowMalformed: true);
    throw HttpStatusException(
      status,
      text.length > _maxErrorBodyChars
          ? text.substring(0, _maxErrorBodyChars)
          : text,
    );
  }

  void _checkBinding<P, R>(
    TrustTaskDocument<P> request,
    TrustTaskDocument<R> response,
  ) {
    final thread = request.threadId ?? request.id;
    if (response.threadId != thread) {
      throw ResponseMismatchException(
        ResponseBinding.threadId,
        thread,
        response.threadId,
      );
    }
    final type = '${bareTypeUri(request.type)}#response';
    if (response.type != type) {
      throw ResponseMismatchException(
        ResponseBinding.type,
        type,
        response.type,
      );
    }
    final server = serverVid;
    if (server != null && response.issuer != server) {
      throw ResponseMismatchException(
        ResponseBinding.issuer,
        server,
        response.issuer,
      );
    }
    final me = myVid;
    if (me != null && response.recipient != me) {
      throw ResponseMismatchException(
        ResponseBinding.recipient,
        me,
        response.recipient,
      );
    }
  }

  /// With a [responseVerifier], why [error] is not attested as the server's
  /// answer, or null when it is: it must carry a proof that verifies over
  /// [received] — the body as it arrived — and name [serverVid] as `issuer`
  /// and [myVid] as `recipient` (each where configured). Without a verifier
  /// there is nothing to attest and every error passes.
  Future<String?> _attestError(
    ErrorResponse error,
    Map<String, dynamic> received,
  ) async {
    final verifier = responseVerifier;
    if (verifier == null) return null;
    if (received['proof'] == null) {
      return 'the error response carried no proof';
    }
    final server = serverVid;
    if (server != null && error.issuer != server) {
      return "the error response's issuer is not the configured server";
    }
    final me = myVid;
    if (me != null && error.recipient != me) {
      return 'the error response is not addressed to this client';
    }
    var ok = false;
    try {
      ok = await verifier.verify(received);
    } on Object {
      ok = false;
    }
    return ok ? null : "the error response's proof did not verify";
  }

  Future<void> _verify(Map<String, dynamic> json) async {
    final verifier = responseVerifier;
    if (verifier == null) return;
    if (json['proof'] == null) {
      throw const ResponseProofException(missing: true);
    }
    var ok = false;
    try {
      ok = await verifier.verify(json);
    } on Object {
      ok = false;
    }
    if (!ok) throw const ResponseProofException(missing: false);
  }
}
