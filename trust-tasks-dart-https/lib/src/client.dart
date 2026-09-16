import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:trust_tasks/trust_tasks.dart';

import 'binding.dart';

/// Default budget for one exchange, connection included.
const Duration defaultClientTimeout = Duration(seconds: 30);

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
/// `500`. §4.1: this establishes nothing about the document's state. Retry by
/// re-sending the same bytes (SPEC §8.4).
final class HttpStatusException extends HttpsClientException {
  const HttpStatusException(this.httpStatus, this.body);

  final int httpStatus;
  final String body;

  @override
  String toString() => 'HttpStatusException(HTTP $httpStatus)';
}

/// The exchange never produced an answer: a connection failure or [timeout].
/// Like [HttpStatusException], not a statement about the document (§4.1).
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

  final Object cause;

  @override
  String toString() => 'ResponseDecodeException($cause)';
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
  HttpsClient({
    required Uri base,
    this.serverVid,
    this.myVid,
    this.token,
    this.responseVerifier,
    this.timeout = defaultClientTimeout,
    http.Client? httpClient,
  })  : endpoint = _endpoint(base),
        _http = httpClient ?? http.Client(),
        _ownsHttp = httpClient == null;

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

  /// When set, every success response must carry a proof that verifies.
  final ProofVerifier? responseVerifier;

  /// Budget for one exchange.
  final Duration timeout;

  final http.Client _http;
  final bool _ownsHttp;

  static Uri _endpoint(Uri base) {
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
    _throwIfError(reply, stamped);
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
    _throwIfError(reply, stamped);
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

  Future<http.Response> _post(Map<String, dynamic> document) async {
    final headers = <String, String>{
      'content-type': 'application/json',
      'accept': 'application/json',
      if (token != null) 'authorization': 'Bearer $token',
    };
    try {
      return await _http
          .post(endpoint, headers: headers, body: jsonEncode(document))
          .timeout(timeout);
    } on TimeoutException catch (e) {
      throw TransportException(e);
    } on http.ClientException catch (e) {
      throw TransportException(e);
    }
  }

  /// §4: a non-2xx answer with a JSON body is read as a `trust-task-error`
  /// first, and only then treated as a transport failure.
  void _throwIfError<P>(http.Response reply, TrustTaskDocument<P> request) {
    final status = reply.statusCode;
    if (status >= 200 && status < 300) return;

    final contentType = reply.headers['content-type'] ?? '';
    if (contentType.split(';').first.trim().toLowerCase() ==
        'application/json') {
      ErrorResponse? error;
      try {
        final json =
            jsonDecode(utf8.decode(reply.bodyBytes)) as Map<String, dynamic>;
        if (bareTypeUri(json['type'] as String)
            .contains('/trust-task-error/')) {
          error = ErrorResponse.fromJson(
            json,
            (p) => ErrorPayload.fromJson(p! as Map<String, dynamic>),
          );
        }
      } on Object {
        error = null;
      }
      if (error != null) {
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
    throw HttpStatusException(
        status, utf8.decode(reply.bodyBytes, allowMalformed: true));
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
          ResponseBinding.type, type, response.type);
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
