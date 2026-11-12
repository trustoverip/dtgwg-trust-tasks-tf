import 'dart:typed_data';

import 'package:ssi/ssi.dart' as ssi;
import 'package:test/test.dart';
import 'package:trust_tasks_proof/src/ed25519_point.dart';

List<int> hex(String h) => [
      for (var i = 0; i < h.length; i += 2)
        int.parse(h.substring(i, i + 2), radix: 16),
    ];

void main() {
  test('refuses the small-order points and non-canonical encodings', () {
    for (final h in [
      '0100000000000000000000000000000000000000000000000000000000000000',
      'ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f',
      '0000000000000000000000000000000000000000000000000000000000000000',
      '26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05',
      '26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc85',
      'c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a',
      'c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac03fa',
      'eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f',
      'edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f',
      'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff',
      '0000000000000000000000000000000000000000000000000000000000000080',
    ]) {
      expect(ed25519PublicKeyError(hex(h)), isNotNull, reason: h);
    }
  });

  test('accepts real keys and the base point', () {
    expect(
      ed25519PublicKeyError(hex(
          '5866666666666666666666666666666666666666666666666666666666666666')),
      isNull,
    );
    for (var seed = 0; seed < 32; seed++) {
      final pair = ssi.Ed25519KeyPair.fromSeed(
          Uint8List.fromList(List.filled(32, seed)));
      expect(ed25519PublicKeyError(pair.publicKey.bytes), isNull,
          reason: 'seed $seed');
    }
  });
}
