/**
 * Deterministic document serialization and the SHA-256 digest over it.
 *
 * Mirrors `canonical.rs` in trust-tasks-rs. SPEC §7.2 (*Keying and comparison
 * for item 11*) defines two documents sharing an `id` as **the same document**
 * when their serializations are identical under RFC 8785 canonicalization —
 * the same identity §8.4 gives a retry. A consumer implementing item 11 has to
 * retain a digest of what it accepted, not merely the `id`.
 *
 * `canonicalJson` here is closer to literal RFC 8785 than the Rust side is,
 * because JavaScript's own primitives are what JCS was specified against:
 * `Array.prototype.sort()` orders by UTF-16 code unit, and `JSON.stringify`
 * formats numbers with `Number::toString` and escapes strings with exactly the
 * §3.2.2.2 escape set. The only thing this adds is recursive member ordering.
 *
 * The item-11 digest is **consumer-local**: it goes into this consumer's own
 * replay record and never onto the wire, into a citation, or into a proof. It
 * is *not* the §4.9.3 task digest, which is computed over `document ∖ proof`
 * and is a published, multibase-encoded multihash. See `replay.ts`.
 *
 * Hand-written, and dependency-free on purpose: this package ships no runtime
 * dependencies, and `node:crypto` would not resolve in a browser or a worker.
 */

/**
 * Serialize `value` with object members recursively ordered and no
 * insignificant whitespace.
 *
 * `undefined` members are omitted, as `JSON.stringify` omits them — which is
 * what makes this agree with the document as it would be sent.
 */
export function canonicalJson(value: unknown): string {
  if (value === null || typeof value !== "object") return JSON.stringify(value) ?? "null";
  if (Array.isArray(value)) return `[${value.map((v) => canonicalJson(v)).join(",")}]`;

  const entries = Object.entries(value as Record<string, unknown>)
    .filter(([, v]) => v !== undefined)
    // RFC 8785 §3.2.3: sort by UTF-16 code unit, which is what the default
    // comparator does after String() — and these are already strings.
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));

  return `{${entries.map(([k, v]) => `${JSON.stringify(k)}:${canonicalJson(v)}`).join(",")}}`;
}

const K = new Uint32Array([
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
]);

const rotr = (x: number, n: number): number => (x >>> n) | (x << (32 - n));

/**
 * UTF-8 encode `s`, without `TextEncoder`.
 *
 * `TextEncoder` is a host global, not an ES2022 one: this package's `lib` is
 * `["ES2022"]` and adding `DOM` to reach one name would pull the whole browser
 * surface into a package that must also typecheck for Node and for workers.
 * Fifteen lines is the cheaper price.
 *
 * Unpaired surrogates — which `JSON.parse` will produce from a `\uD800` escape
 * — encode as U+FFFD, matching `TextEncoder`. Two documents differing only in
 * an unpaired surrogate would therefore collide; they are not valid JSON text
 * to begin with, and the collision absorbs a duplicate rather than executing
 * one, so it fails safe.
 */
function utf8(s: string): Uint8Array {
  const out: number[] = [];
  for (let i = 0; i < s.length; i++) {
    let cp = s.charCodeAt(i);
    if (cp >= 0xd800 && cp <= 0xdbff && i + 1 < s.length) {
      const low = s.charCodeAt(i + 1);
      if (low >= 0xdc00 && low <= 0xdfff) {
        cp = 0x10000 + ((cp - 0xd800) << 10) + (low - 0xdc00);
        i++;
      }
    }
    if (cp >= 0xd800 && cp <= 0xdfff) cp = 0xfffd; // unpaired surrogate

    if (cp < 0x80) out.push(cp);
    else if (cp < 0x800) out.push(0xc0 | (cp >> 6), 0x80 | (cp & 0x3f));
    else if (cp < 0x10000)
      out.push(0xe0 | (cp >> 12), 0x80 | ((cp >> 6) & 0x3f), 0x80 | (cp & 0x3f));
    else
      out.push(
        0xf0 | (cp >> 18),
        0x80 | ((cp >> 12) & 0x3f),
        0x80 | ((cp >> 6) & 0x3f),
        0x80 | (cp & 0x3f),
      );
  }
  return Uint8Array.from(out);
}

/**
 * FIPS 180-4 SHA-256 over the UTF-8 encoding of `input`, as lowercase hex.
 *
 * Synchronous by necessity: `crypto.subtle.digest` is a Promise, and the
 * consume pipeline needs the digest before it decides whether to await
 * anything. Pinned to the standard's published vectors in `test/`.
 */
export function sha256Hex(input: string): string {
  const bytes = utf8(input);
  const bitLen = bytes.length * 8;

  // Pad to a multiple of 64 bytes: 0x80, zeroes, then the 64-bit big-endian
  // bit length.
  const padded = new Uint8Array(Math.ceil((bytes.length + 9) / 64) * 64);
  padded.set(bytes);
  padded[bytes.length] = 0x80;
  const view = new DataView(padded.buffer);
  // Lengths above 2^53 bits are unreachable from a JS string; the high word is
  // written anyway so the padding is well-formed.
  view.setUint32(padded.length - 8, Math.floor(bitLen / 0x100000000), false);
  view.setUint32(padded.length - 4, bitLen >>> 0, false);

  const h = new Uint32Array([
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
  ]);
  const w = new Uint32Array(64);

  for (let offset = 0; offset < padded.length; offset += 64) {
    for (let i = 0; i < 16; i++) w[i] = view.getUint32(offset + i * 4, false);
    for (let i = 16; i < 64; i++) {
      const s0 = rotr(w[i - 15]!, 7) ^ rotr(w[i - 15]!, 18) ^ (w[i - 15]! >>> 3);
      const s1 = rotr(w[i - 2]!, 17) ^ rotr(w[i - 2]!, 19) ^ (w[i - 2]! >>> 10);
      w[i] = (w[i - 16]! + s0 + w[i - 7]! + s1) >>> 0;
    }

    let [a, b, c, d, e, f, g, hh] = [h[0]!, h[1]!, h[2]!, h[3]!, h[4]!, h[5]!, h[6]!, h[7]!];
    for (let i = 0; i < 64; i++) {
      const s1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
      const ch = (e & f) ^ (~e & g);
      const temp1 = (hh + s1 + ch + K[i]! + w[i]!) >>> 0;
      const s0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
      const maj = (a & b) ^ (a & c) ^ (b & c);
      const temp2 = (s0 + maj) >>> 0;

      hh = g;
      g = f;
      f = e;
      e = (d + temp1) >>> 0;
      d = c;
      c = b;
      b = a;
      a = (temp1 + temp2) >>> 0;
    }

    const next = [a, b, c, d, e, f, g, hh];
    for (let i = 0; i < 8; i++) h[i] = (h[i]! + next[i]!) >>> 0;
  }

  return Array.from(h, (word) => word.toString(16).padStart(8, "0")).join("");
}
