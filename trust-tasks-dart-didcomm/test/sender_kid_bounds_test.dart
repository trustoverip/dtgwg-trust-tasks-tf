import 'dart:convert';

import 'package:ssi/ssi.dart';
import 'package:test/test.dart';
import 'package:trust_tasks_didcomm/trust_tasks_didcomm.dart';

import 'support/fixtures.dart';

/// [wire] with its protected header's `skid` replaced by [skid]. The header is
/// no longer the one the content was encrypted under, so the envelope could
/// never open; what matters is how soon it is refused.
Map<String, dynamic> withSkid(Map<String, dynamic> wire, String skid) {
  final copy = jsonDecode(jsonEncode(wire)) as Map<String, dynamic>;
  final header = jsonDecode(
    utf8.decode(
        base64Url.decode(base64Url.normalize(copy['protected'] as String))),
  ) as Map<String, dynamic>;
  header['skid'] = skid;
  copy['protected'] =
      base64Url.encode(utf8.encode(jsonEncode(header))).replaceAll('=', '');
  return copy;
}

void main() {
  late DidKeyManager alice;
  late DidKeyManager bob;
  late Map<String, dynamic> wire;
  setUpAll(() async {
    alice = await party(KeyType.p256);
    bob = await party(KeyType.p256);
    wire = await packTrustTask(
      echoDoc(),
      sender: alice,
      recipient: await bob.getDidDocument(),
    );
  });

  test('an envelope within the bounds still opens', () async {
    final unpacked = await unpackTrustTask(wire, recipient: bob);
    expect(unpacked.transport.peer, await didOf(alice));
  });

  for (final (name, skid) in [
    ('an over-long did:key', 'did:key:zDn${'2' * 176}#key-1'),
    ('a sender kid over 4096 characters', 'did:web:${'a' * 4096}#key-1'),
  ]) {
    test('$name is refused before resolution', () async {
      await expectLater(
        unpackTrustTask(withSkid(wire, skid), recipient: bob),
        throwsA(isA<EnvelopeException>()
            .having((e) => e.failure, 'failure', EnvelopeFailure.undecryptable)
            .having((e) => e.detail, 'detail', startsWith('sender '))),
      );
    });
  }

  // A megabyte-sized sender key id is refused without resolving it, so what an
  // unauthenticated envelope costs does not grow with its skid's length.
  test('a 1 MiB did:key sender kid is refused quickly', () async {
    // `zDn` is the P-256 multikey prefix, so ssi would decode the rest.
    final mb = 'zDn${'2' * (1 << 20)}';
    final huge = withSkid(wire, 'did:key:$mb#$mb');
    final sw = Stopwatch()..start();
    for (var i = 0; i < 20; i++) {
      await expectLater(
        unpackTrustTask(huge, recipient: bob),
        throwsA(isA<EnvelopeException>()),
      );
    }
    expect(sw.elapsed, lessThan(const Duration(seconds: 2)),
        reason: 'refusing 20 oversized envelopes took ${sw.elapsed}');
  });
}
