import 'dart:async';
import 'dart:math';

import 'package:trust_tasks/trust_tasks.dart';

/// The binding's stable identifier (`bindings/https/0.2` §1). It never appears
/// on the wire; HTTPS has no envelope to carry it.
const String bindingUri = _bindingUri;

// Named privately as well, because [HttpsTransport.bindingUri] shadows the
// public constant inside that class.
const String _bindingUri = 'https://trusttasks.org/binding/https/0.2';

/// The request path, relative to the Trust-Task base (§2, §6.1).
const String trustTasksPath = '/trust-tasks';

/// The [TransportHandler] for one HTTPS exchange.
///
/// On the server, [local] is the server's configured VID and [peer] the VID
/// its bearer token mapped to. On the client, [local] is the client's VID and
/// [peer] the configured server VID. Either may be null: a null [peer] means the
/// transport authenticated nobody, and the §4.8.1 fallback then rests entirely
/// on the document's in-band identity.
final class HttpsTransport implements TransportHandler {
  const HttpsTransport({this.local, this.peer});

  final String? local;
  final String? peer;

  @override
  String get bindingUri => _bindingUri;

  @override
  TransportContext deriveParties() =>
      TransportContext(issuer: peer, recipient: local);
}

/// Resolves a bearer token to the VID it authenticates (§3).
///
/// Deployment-defined: a static map in a test, a JWT verified against an
/// issuer-controlled JWKS in production. Return null for a token that
/// authenticates nobody. May be asynchronous — a database or introspection
/// lookup is a reasonable implementation.
abstract interface class BearerAuthenticator {
  FutureOr<String?> resolve(String token);
}

/// A [BearerAuthenticator] backed by a fixed token → VID map. For demos and
/// tests; §3 says a production deployment SHOULD bind tokens to identifiers
/// under a controlled trust framework instead.
final class StaticBearerAuth implements BearerAuthenticator {
  StaticBearerAuth(Map<String, String> tokens) : _tokens = Map.of(tokens);

  final Map<String, String> _tokens;

  @override
  String? resolve(String token) => _tokens[token];
}

/// Map a framework error code to its HTTP status, per the binding's §4 table.
///
/// The status is informative; the code in the body is authoritative. The
/// flat `422` bucket is deliberate: splitting `proofInvalid`, `identityMismatch`
/// and `wrongRecipient` across statuses hands an unauthenticated prober an
/// oracle that works without reading a body.
///
/// Extended codes (`<slug>:<local>`, §8.5) are application failures and map to
/// `422`. A standard-looking code this table does not know is a server that has
/// not kept up with the framework, and maps to `500`.
int statusForCode(String code) {
  if (code.contains(':')) return 422;
  final normalized = normalizeCode(code);
  if (normalized == StandardCode.malformedRequest.value) return 400;
  if (normalized == StandardCode.permissionDenied.value) return 403;
  if (normalized == StandardCode.idConflict.value) return 409;
  if (normalized == StandardCode.unavailable.value) return 503;
  if (normalized == StandardCode.internalError.value) return 500;
  const unprocessable = {
    'unsupportedType',
    'unsupportedVersion',
    'expired',
    'proofRequired',
    'proofInvalid',
    'identityMismatch',
    'wrongRecipient',
    'cancelled',
    'taskFailed',
  };
  return unprocessable.contains(normalized) ? 422 : 500;
}

final Random _random = Random.secure();

/// A fresh `urn:uuid:` identifier (UUID version 4).
String newUrnUuid() {
  final b = List<int>.generate(16, (_) => _random.nextInt(256));
  b[6] = (b[6] & 0x0f) | 0x40;
  b[8] = (b[8] & 0x3f) | 0x80;
  String hex(int from, int to) => [
        for (var i = from; i < to; i++) b[i].toRadixString(16).padLeft(2, '0'),
      ].join();
  return 'urn:uuid:${hex(0, 4)}-${hex(4, 6)}-${hex(6, 8)}-${hex(8, 10)}-'
      '${hex(10, 16)}';
}

/// The routing key for a Type URI: `#request` and no fragment route alike,
/// `#response` keeps its fragment so a response document never reaches a
/// request handler (§4.4.1 item 1).
String routingKey(String typeUri) =>
    typeUri.endsWith('#request') ? bareTypeUri(typeUri) : typeUri;
