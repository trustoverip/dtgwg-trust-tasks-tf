/// Document identifiers.
///
/// Hand-written. One generator for the whole Dart family: the HTTPS, DIDComm,
/// TSP and capability-client packages each carried their own copy of it before
/// it moved here (#1118).
library;

import 'dart:math';

final Random _random = Random.secure();

/// A fresh `urn:uuid:` identifier — a version-4 UUID from a cryptographically
/// secure source, for the `id` of a request, response or error document.
///
/// SPEC §4.1 makes `id` unique per document, and §7.2 item 11 keys the
/// duplicate-execution record on it, so a predictable `id` would let a party
/// that can guess the next one pre-empt it. Hence [Random.secure].
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
