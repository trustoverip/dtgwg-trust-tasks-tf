/// Deterministic document serialization and the SHA-256 digest built over it.
///
/// Mirrors `canonical.rs` in trust-tasks-rs, `_runtime/canonical.ts` in
/// @openvtc/trust-tasks and `trusttasks/canonical.go` in trust-tasks-go.
///
/// SPEC §7.2 (*Keying and comparison for item 11*) defines two documents sharing
/// an `id` as **the same document** when their serializations are identical
/// under RFC 8785 canonicalization — the same identity §8.4 gives a retry. A
/// consumer implementing item 11 has to retain a digest of what it accepted, not
/// merely the `id`.
///
/// ## What this canonicalizes, and what it does not
///
/// [canonicalJson] emits a serialization that is *equality-equivalent* to RFC
/// 8785: two documents produce the same bytes here exactly when they produce the
/// same bytes under JCS. It is deliberately not claimed to be byte-identical to
/// a JCS implementation's output, and it must not be used where the bytes
/// themselves are the interoperable artifact — notably the §4.9.3 task digest,
/// which is published in citations and recomputed by other parties, and the
/// `eddsa-jcs-2022` proof input.
///
/// Two deviations are known, and both are confined to the bytes rather than the
/// equality relation they induce — the same two the other runtimes document:
///
/// * Member ordering is by UTF-16 code unit, which is what Dart's [String]
///   comparison gives and what JCS specifies, so this side actually agrees with
///   JCS where the Rust and Go runtimes order by scalar value instead. The
///   disagreement is confined to names containing characters above the BMP
///   alongside names in `U+E000..U+FFFF`, and it reorders both documents being
///   compared identically, so it cannot make two JCS-identical documents differ
///   nor two JCS-differing documents agree.
/// * Number formatting is Dart's, which distinguishes `1` from `1.0` because a
///   decoded JSON `1` is an `int` and `1.0` is a `double`. The same value always
///   formats the same way, so the induced equality is unchanged.
///
/// The item-11 digest is **consumer-local**: it goes into this consumer's own
/// replay record and never onto the wire, into a citation, or into a proof. Only
/// the equality relation is load-bearing, which is why this dependency-free
/// serializer is the right size of tool for it.
///
/// SHA-256 is implemented here rather than taken from `package:crypto` so the
/// package keeps no dependencies at all — the same choice `canonical.rs` and
/// `canonical.ts` make, and pinned to the standard's published vectors in
/// `test/`.
library;

import 'dart:convert';
import 'dart:typed_data';

/// Serialize [value] with object members recursively ordered and no
/// insignificant whitespace.
///
/// [value] is a decoded JSON tree: maps, lists, strings, numbers, bools, null.
String canonicalJson(Object? value) {
  final out = StringBuffer();
  _writeCanonical(out, value);
  return out.toString();
}

void _writeCanonical(StringBuffer out, Object? value) {
  if (value == null) {
    out.write('null');
  } else if (value is bool) {
    out.write(value ? 'true' : 'false');
  } else if (value is num) {
    out.write(jsonEncode(value));
  } else if (value is String) {
    _writeCanonicalString(out, value);
  } else if (value is List) {
    out.write('[');
    for (var i = 0; i < value.length; i++) {
      if (i > 0) out.write(',');
      _writeCanonical(out, value[i]);
    }
    out.write(']');
  } else if (value is Map) {
    // RFC 8785 §3.2.3: sort by UTF-16 code unit, which is exactly what Dart's
    // String.compareTo does.
    final names = value.keys.map((k) => k as String).toList()..sort();
    out.write('{');
    for (var i = 0; i < names.length; i++) {
      if (i > 0) out.write(',');
      _writeCanonicalString(out, names[i]);
      out.write(':');
      _writeCanonical(out, value[names[i]]);
    }
    out.write('}');
  } else {
    throw ArgumentError.value(
      value,
      'value',
      'canonicalJson expects a decoded JSON tree',
    );
  }
}

const String _hexDigits = '0123456789abcdef';

/// Apply the RFC 8785 §3.2.2.2 escape set, and only that set.
void _writeCanonicalString(StringBuffer out, String s) {
  out.write('"');
  for (final unit in s.codeUnits) {
    switch (unit) {
      case 0x22:
        out.write(r'\"');
      case 0x5c:
        out.write(r'\\');
      case 0x08:
        out.write(r'\b');
      case 0x0c:
        out.write(r'\f');
      case 0x0a:
        out.write(r'\n');
      case 0x0d:
        out.write(r'\r');
      case 0x09:
        out.write(r'\t');
      default:
        if (unit < 0x20) {
          out.write(r'\u00');
          out.write(_hexDigits[(unit >> 4) & 0xf]);
          out.write(_hexDigits[unit & 0xf]);
        } else {
          out.writeCharCode(unit);
        }
    }
  }
  out.write('"');
}

const List<int> _k = <int>[
  0x428a2f98,
  0x71374491,
  0xb5c0fbcf,
  0xe9b5dba5,
  0x3956c25b,
  0x59f111f1,
  0x923f82a4,
  0xab1c5ed5,
  0xd807aa98,
  0x12835b01,
  0x243185be,
  0x550c7dc3,
  0x72be5d74,
  0x80deb1fe,
  0x9bdc06a7,
  0xc19bf174,
  0xe49b69c1,
  0xefbe4786,
  0x0fc19dc6,
  0x240ca1cc,
  0x2de92c6f,
  0x4a7484aa,
  0x5cb0a9dc,
  0x76f988da,
  0x983e5152,
  0xa831c66d,
  0xb00327c8,
  0xbf597fc7,
  0xc6e00bf3,
  0xd5a79147,
  0x06ca6351,
  0x14292967,
  0x27b70a85,
  0x2e1b2138,
  0x4d2c6dfc,
  0x53380d13,
  0x650a7354,
  0x766a0abb,
  0x81c2c92e,
  0x92722c85,
  0xa2bfe8a1,
  0xa81a664b,
  0xc24b8b70,
  0xc76c51a3,
  0xd192e819,
  0xd6990624,
  0xf40e3585,
  0x106aa070,
  0x19a4c116,
  0x1e376c08,
  0x2748774c,
  0x34b0bcb5,
  0x391c0cb3,
  0x4ed8aa4a,
  0x5b9cca4f,
  0x682e6ff3,
  0x748f82ee,
  0x78a5636f,
  0x84c87814,
  0x8cc70208,
  0x90befffa,
  0xa4506ceb,
  0xbef9a3f7,
  0xc67178f2,
];

/// A 32-bit rotate right. The mask keeps this correct on the web, where an
/// `int` is a JavaScript double and shifts are not otherwise 32-bit.
int _rotr(int x, int n) => ((x >> n) | (x << (32 - n))) & 0xffffffff;

/// FIPS 180-4 SHA-256 over [input], as lowercase hex.
String sha256Hex(List<int> input) {
  final bytes = Uint8List.fromList(input);
  final bitLength = bytes.length * 8;

  // Pad to a multiple of 64 bytes: 0x80, zeroes, then the 64-bit big-endian bit
  // length.
  final paddedLength = ((bytes.length + 9 + 63) ~/ 64) * 64;
  final padded = Uint8List(paddedLength)..setRange(0, bytes.length, bytes);
  padded[bytes.length] = 0x80;
  final view = ByteData.view(padded.buffer);
  view.setUint32(paddedLength - 8, bitLength ~/ 0x100000000, Endian.big);
  view.setUint32(paddedLength - 4, bitLength & 0xffffffff, Endian.big);

  final h = Uint32List.fromList(<int>[
    0x6a09e667,
    0xbb67ae85,
    0x3c6ef372,
    0xa54ff53a,
    0x510e527f,
    0x9b05688c,
    0x1f83d9ab,
    0x5be0cd19,
  ]);
  final w = Uint32List(64);

  for (var offset = 0; offset < paddedLength; offset += 64) {
    for (var i = 0; i < 16; i++) {
      w[i] = view.getUint32(offset + i * 4, Endian.big);
    }
    for (var i = 16; i < 64; i++) {
      final s0 = _rotr(w[i - 15], 7) ^ _rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
      final s1 = _rotr(w[i - 2], 17) ^ _rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
      w[i] = (w[i - 16] + s0 + w[i - 7] + s1) & 0xffffffff;
    }

    var a = h[0], b = h[1], c = h[2], d = h[3];
    var e = h[4], f = h[5], g = h[6], hh = h[7];

    for (var i = 0; i < 64; i++) {
      final s1 = _rotr(e, 6) ^ _rotr(e, 11) ^ _rotr(e, 25);
      final ch = (e & f) ^ ((~e & 0xffffffff) & g);
      final temp1 = (hh + s1 + ch + _k[i] + w[i]) & 0xffffffff;
      final s0 = _rotr(a, 2) ^ _rotr(a, 13) ^ _rotr(a, 22);
      final maj = (a & b) ^ (a & c) ^ (b & c);
      final temp2 = (s0 + maj) & 0xffffffff;

      hh = g;
      g = f;
      f = e;
      e = (d + temp1) & 0xffffffff;
      d = c;
      c = b;
      b = a;
      a = (temp1 + temp2) & 0xffffffff;
    }

    h[0] = (h[0] + a) & 0xffffffff;
    h[1] = (h[1] + b) & 0xffffffff;
    h[2] = (h[2] + c) & 0xffffffff;
    h[3] = (h[3] + d) & 0xffffffff;
    h[4] = (h[4] + e) & 0xffffffff;
    h[5] = (h[5] + f) & 0xffffffff;
    h[6] = (h[6] + g) & 0xffffffff;
    h[7] = (h[7] + hh) & 0xffffffff;
  }

  return h.map((word) => word.toRadixString(16).padLeft(8, '0')).join();
}

/// SHA-256 over the UTF-8 encoding of [input], as lowercase hex.
String sha256HexOfString(String input) => sha256Hex(utf8.encode(input));
