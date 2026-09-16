/// Serving an [HttpsServer] with `dart:io`.
///
/// A separate library from `trust_tasks_https.dart` because `dart:io` does not
/// exist on the web, and the client — which a browser or Flutter web app may
/// well want — should not lose the web for the server's sake.
library;

import 'dart:async';
import 'dart:io';

import 'trust_tasks_https.dart';

export 'trust_tasks_https.dart';

/// Default wall-clock budget for one request, body read included. A request
/// still open after this is wedged, and holding it is what a slowloris wants.
const Duration defaultRequestTimeout = Duration(seconds: 30);

/// Bind [address]:[port] and answer every request with [server].
///
/// Plain HTTP unless [securityContext] is given. The binding requires TLS in
/// front of the receiver (§Abstract), which may equally be a reverse proxy.
Future<HttpServer> serve(
  HttpsServer server, {
  Object address = '127.0.0.1',
  int port = 0,
  SecurityContext? securityContext,
  Duration requestTimeout = defaultRequestTimeout,
}) async {
  final http = securityContext == null
      ? await HttpServer.bind(address, port)
      : await HttpServer.bindSecure(address, port, securityContext);
  http.listen((request) {
    unawaited(
      handleIoRequest(server, request, requestTimeout: requestTimeout),
    );
  });
  return http;
}

/// Answer one `dart:io` [request] with [server] — for an application that owns
/// its own [HttpServer] and routes only some paths here.
Future<void> handleIoRequest(
  HttpsServer server,
  HttpRequest request, {
  Duration requestTimeout = defaultRequestTimeout,
}) async {
  final response = request.response;
  try {
    final reply = await _answer(server, request).timeout(requestTimeout);
    response.statusCode = reply.status;
    reply.headers.forEach(response.headers.set);
    final body = reply.body;
    if (body != null) response.add(body);
  } on TimeoutException {
    response.statusCode = HttpStatus.requestTimeout;
  } on Object {
    response.statusCode = HttpStatus.internalServerError;
  }
  try {
    await response.close();
  } on Object {
    // The peer went away; there is nobody left to tell.
  }
}

Future<HttpsReply> _answer(HttpsServer server, HttpRequest request) async {
  // Check the declared length before reading a byte, then enforce the limit
  // while reading, since a chunked body declares none.
  if (request.contentLength > maxBodyBytes) {
    return const HttpsReply(HttpStatus.requestEntityTooLarge);
  }
  final body = <int>[];
  await for (final chunk in request) {
    body.addAll(chunk);
    if (body.length > maxBodyBytes) {
      return const HttpsReply(HttpStatus.requestEntityTooLarge);
    }
  }
  final headers = <String, String>{};
  request.headers.forEach((name, values) => headers[name] = values.join(', '));
  return server.handle(
    HttpsRequest(
      method: request.method,
      path: request.uri.path,
      headers: headers,
      body: body,
    ),
  );
}
