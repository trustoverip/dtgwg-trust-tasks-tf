// An Ed25519 public-key check, on BigInt so this package takes no new
// dependency.
//
// The Ed25519 verifier ssi 3.9 uses decodes a public key leniently: it accepts
// a non-canonical encoding (a y coordinate of p or more) and a point of small
// order. A small-order key verifies a signature it never made — with the
// identity as the key and as R, and S = 0, the equation holds for any message.
// ssi 4 refuses both, as the Go, Rust and TypeScript backends do; this check
// makes the result the same whichever ssi resolves.

final _p = (BigInt.one << 255) - BigInt.from(19);
final _d = (BigInt.from(-121665) * BigInt.from(121666).modInverse(_p)) % _p;
final _sqrtM1 = BigInt.two.modPow((_p - BigInt.one) >> 2, _p);

/// Why [encoded] is not an Ed25519 public key a signature may be checked
/// against — not a canonical RFC 8032 §5.1.3 encoding of a curve point, or a
/// point of small order — or `null` when it is.
String? ed25519PublicKeyError(List<int> encoded) {
  if (encoded.length != 32) return 'ed25519 public key must be 32 bytes';
  final sign = encoded[31] >> 7;
  var y = BigInt.zero;
  for (var i = 31; i >= 0; i--) {
    y = (y << 8) | BigInt.from(i == 31 ? encoded[i] & 0x7f : encoded[i]);
  }
  if (y >= _p) return 'ed25519 public key is not canonically encoded';
  final y2 = (y * y) % _p;
  final u = (y2 - BigInt.one) % _p;
  final v = (_d * y2 + BigInt.one) % _p;
  if (v == BigInt.zero) return 'ed25519 public key is not a curve point';
  final x2 = (u * v.modInverse(_p)) % _p;
  var x = x2.modPow((_p + BigInt.from(3)) >> 3, _p);
  if ((x * x) % _p != x2) {
    x = (x * _sqrtM1) % _p;
    if ((x * x) % _p != x2) return 'ed25519 public key is not a curve point';
  }
  if (x == BigInt.zero && sign == 1) {
    return 'ed25519 public key is not canonically encoded';
  }
  if ((x & BigInt.one).toInt() != sign) x = _p - x;

  var q = (x, y);
  for (var i = 0; i < 3; i++) {
    q = _add(q, q);
  }
  if (q.$1 == BigInt.zero && q.$2 == BigInt.one) {
    return 'ed25519 public key has small order';
  }
  return null;
}

/// Complete twisted Edwards addition (a = -1), affine coordinates.
(BigInt, BigInt) _add((BigInt, BigInt) p, (BigInt, BigInt) q) {
  final (x1, y1) = p;
  final (x2, y2) = q;
  final t = (_d * x1 * x2 * y1 * y2) % _p;
  final x = ((x1 * y2 + y1 * x2) * ((BigInt.one + t) % _p).modInverse(_p)) % _p;
  final y = ((y1 * y2 + x1 * x2) * ((BigInt.one - t) % _p).modInverse(_p)) % _p;
  return (x, y);
}
