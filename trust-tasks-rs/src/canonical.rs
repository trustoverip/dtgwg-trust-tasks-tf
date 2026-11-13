//! Deterministic document serialization and the SHA-256 digest built over it.
//!
//! SPEC.md §7.2 (*Keying and comparison for item 11*) defines two documents
//! sharing an `id` as **the same document** when their serializations are
//! identical under [RFC 8785][] canonicalization — the same identity §8.4
//! gives a *retry*. A consumer implementing item 11 therefore has to retain a
//! digest of what it accepted, not merely the `id`.
//!
//! [RFC 8785]: https://www.rfc-editor.org/rfc/rfc8785
//!
//! # What this module canonicalizes, and what it does not
//!
//! [`canonical_json`] emits a JSON serialization that is *equality-equivalent*
//! to RFC 8785: two [`serde_json::Value`]s produce the same bytes here exactly
//! when they produce the same bytes under JCS. It is deliberately **not**
//! claimed to be byte-identical to a JCS implementation's output, and it must
//! not be used where the bytes themselves are the interoperable artifact —
//! notably the §4.9.3 *task digest*, which is published in citations and
//! recomputed by other parties, and the `eddsa-jcs-2022` proof input, which
//! `trust-tasks-proof` canonicalizes with a real RFC 8785 implementation.
//!
//! Two deviations are known and both are confined to the bytes, not the
//! equality relation they induce:
//!
//! * **Member ordering** is by Unicode scalar value (Rust's `str` ordering),
//!   where JCS orders by UTF-16 code unit. The two disagree only for member
//!   names containing characters above the BMP alongside names in
//!   `U+E000..=U+FFFF`. Any such disagreement reorders the output of *both*
//!   documents being compared identically, so it cannot make two
//!   JCS-identical documents differ here, nor two JCS-differing documents
//!   agree.
//! * **Number formatting.** Every number is read as the IEEE 754 double JCS
//!   reads it as, so `1`, `1.0` and `1e0` are one number, as they are to the
//!   TypeScript and Go runtimes. An integral value below `1e21` is written as
//!   an integer, as ECMAScript's `Number::toString` writes it; anything else
//!   is `serde_json`'s shortest-round-trip form, which spells some exponents
//!   differently (`1e100` vs JCS's `1e+100`). The same `f64` always formats
//!   the same way, so the induced equality is JCS's.
//!
//! The item-11 digest is **consumer-local**: it is written to the consumer's
//! own replay record and never appears on the wire, in a citation, or in a
//! proof. Only the equality relation is load-bearing, which is why this
//! dependency-free serializer is the right size of tool for it.
//!
//! # Why a digest of the canonical form rather than the received bytes
//!
//! A consumer that keyed on the octets it received would treat re-indented
//! JSON, a reordered `payload`, or a transport that re-serializes in transit
//! as a *different* document, and would answer a legitimate §8.4 retry with
//! `idConflict` — or, worse, execute it. Canonicalizing first is what makes
//! "the same document" a property of the document rather than of the pipe it
//! arrived through.

use std::fmt::Write as _;

use serde_json::Value;

/// Serialize `value` to the deterministic form described in the module
/// documentation: object members ordered, no insignificant whitespace,
/// strings escaped minimally per RFC 8785 §3.2.2.2.
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value);
    out
}

/// Lowercase-hex SHA-256 of [`canonical_json`]`(value)`, without building
/// that string: the canonical form is written straight into the hasher, so
/// the digest of a received document costs one walk of it rather than a
/// walk, a string of the whole document and a copy of that string (#1092).
/// The same digest, byte for byte.
pub(crate) fn canonical_sha256_hex(value: &Value) -> String {
    let mut sink = HashSink {
        hasher: Sha256::new(),
        buffer: Vec::with_capacity(HashSink::FLUSH_AT + 256),
    };
    write_value(&mut sink, value);
    sink.hasher.update(&sink.buffer);
    hex(&sink.hasher.finish())
}

/// Where the canonical form is written: a `String`, or the hasher.
trait Sink {
    fn put(&mut self, s: &str);
}

impl Sink for String {
    fn put(&mut self, s: &str) {
        self.push_str(s);
    }
}

/// The hasher, fed through a buffer: the canonical form arrives a token at a
/// time — often a single `,` or `:` — and handing each to the hasher on its
/// own costs more than the copy.
struct HashSink {
    hasher: Sha256,
    buffer: Vec<u8>,
}

impl HashSink {
    const FLUSH_AT: usize = 4096;
}

impl Sink for HashSink {
    fn put(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
        if self.buffer.len() >= Self::FLUSH_AT {
            self.hasher.update(&self.buffer);
            self.buffer.clear();
        }
    }
}

fn write_value(out: &mut impl Sink, value: &Value) {
    match value {
        Value::Null => out.put("null"),
        Value::Bool(true) => out.put("true"),
        Value::Bool(false) => out.put("false"),
        Value::Number(n) => write_number(out, n),
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            out.put("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.put(",");
                }
                write_value(out, item);
            }
            out.put("]");
        }
        Value::Object(map) => {
            // Sorted explicitly rather than relying on `serde_json::Map`'s
            // backing store: the `preserve_order` feature swaps it for an
            // insertion-ordered map, and feature unification means another
            // crate in the graph can turn that on without this crate saying
            // so. A digest that silently became insertion-ordered would key
            // on the sender's member order.
            let mut members: Vec<(&String, &Value)> = map.iter().collect();
            members.sort_unstable_by(|a, b| a.0.cmp(b.0));
            out.put("{");
            for (i, (key, member)) in members.into_iter().enumerate() {
                if i > 0 {
                    out.put(",");
                }
                write_string(out, key);
                out.put(":");
                write_value(out, member);
            }
            out.put("}");
        }
    }
}

/// A number as the IEEE 754 double RFC 8785 §3.2.2.3 reads it as: integral
/// values below `1e21` as integers (`1.0` and `1e2` are `1` and `100`), and
/// everything else in `serde_json`'s shortest-round-trip form.
fn write_number(out: &mut impl Sink, n: &serde_json::Number) {
    let Some(f) = n.as_f64() else {
        // Not reachable without serde_json's `arbitrary_precision`; keep the
        // number's own spelling rather than guess.
        out.put(&n.to_string());
        return;
    };
    if f.is_finite() && f.fract() == 0.0 && f.abs() < 1e21 {
        // Exact: every integral f64 below 1e21 fits an i128. `-0.0` is `0`.
        out.put(&(f as i128).to_string());
    } else {
        match serde_json::Number::from_f64(f) {
            Some(n) => out.put(&n.to_string()),
            None => out.put(&n.to_string()),
        }
    }
}

/// RFC 8785 §3.2.2.2 string escaping: the two-character escapes where they
/// exist, `\u00xx` for the remaining control characters, and the literal
/// character otherwise.
///
/// Written a run at a time rather than a character at a time. Every byte that
/// needs escaping is ASCII and no byte of a multi-byte UTF-8 sequence is, so
/// splitting at those bytes always splits at a character boundary.
fn write_string(out: &mut impl Sink, s: &str) {
    out.put("\"");
    let mut run = 0;
    for (i, byte) in s.bytes().enumerate() {
        let escape = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            0x08 => "\\b",
            0x0c => "\\f",
            b'\n' => "\\n",
            b'\r' => "\\r",
            b'\t' => "\\t",
            0x00..=0x1f => "",
            _ => continue,
        };
        out.put(&s[run..i]);
        if escape.is_empty() {
            out.put(&format!("\\u{byte:04x}"));
        } else {
            out.put(escape);
        }
        run = i + 1;
    }
    out.put(&s[run..]);
    out.put("\"");
}

/// Lowercase-hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex(&hasher.finish())
}

fn hex(digest: &[u8; 32]) -> String {
    let mut hex = String::with_capacity(64);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// FIPS 180-4 SHA-256, fed incrementally.
///
/// Hand-rolled rather than pulled in: this crate is deliberately
/// dependency-light, and the digest is used for content *identity* in the
/// consumer's own replay record, never as a signature or a MAC. The
/// implementation is checked against the FIPS 180-4 / NIST CAVP vectors in
/// this module's tests.
///
/// Incremental, so that the canonical form can be hashed as it is written:
/// whole blocks are compressed straight from the input, and only a partial
/// block is held back.
struct Sha256 {
    state: [u32; 8],
    pending: [u8; 64],
    pending_len: usize,
    total_len: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            pending: [0; 64],
            pending_len: 0,
            total_len: 0,
        }
    }

    fn update(&mut self, mut input: &[u8]) {
        self.total_len = self.total_len.wrapping_add(input.len() as u64);
        if self.pending_len > 0 {
            let take = (64 - self.pending_len).min(input.len());
            self.pending[self.pending_len..self.pending_len + take].copy_from_slice(&input[..take]);
            self.pending_len += take;
            input = &input[take..];
            if self.pending_len < 64 {
                return;
            }
            let block = self.pending;
            compress(&mut self.state, &block);
            self.pending_len = 0;
        }
        // `as_chunks` rather than `chunks_exact`: the sizes are compile-time
        // constants, so the compiler can see the block and word shapes.
        let (blocks, rest) = input.as_chunks::<64>();
        for block in blocks {
            compress(&mut self.state, block);
        }
        self.pending[..rest.len()].copy_from_slice(rest);
        self.pending_len = rest.len();
    }

    fn finish(mut self) -> [u8; 32] {
        // 0x80, zeros up to 56 mod 64, then the message length in bits as a
        // 64-bit big-endian integer.
        let bit_len = self.total_len.wrapping_mul(8);
        let zeros = (64 + 55 - self.pending_len) % 64;
        let mut padding = [0u8; 1 + 63 + 8];
        padding[0] = 0x80;
        padding[1 + zeros..1 + zeros + 8].copy_from_slice(&bit_len.to_be_bytes());
        self.update(&padding[..1 + zeros + 8]);
        debug_assert_eq!(self.pending_len, 0, "padded to a block multiple");

        let mut out = [0u8; 32];
        for (i, word) in self.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

/// One 64-byte block of the FIPS 180-4 compression function.
fn compress(h: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
        w[i] = u32::from_be_bytes(*word);
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }

    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;
    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = hh
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    for (slot, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
        *slot = slot.wrapping_add(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// FIPS 180-4 / NIST CAVP published vectors. A hand-rolled hash is only
    /// worth having if it is pinned to the standard's own answers.
    #[test]
    fn sha256_matches_published_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        // Crosses many block boundaries and exercises the length padding.
        assert_eq!(
            sha256_hex(&[b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn member_order_does_not_change_the_canonical_form() {
        let a = json!({"b": 1, "a": 2, "c": {"z": 1, "y": 2}});
        let b = json!({"c": {"y": 2, "z": 1}, "a": 2, "b": 1});
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(canonical_json(&a), r#"{"a":2,"b":1,"c":{"y":2,"z":1}}"#);
    }

    #[test]
    fn array_order_does_change_the_canonical_form() {
        assert_ne!(
            canonical_json(&json!([1, 2])),
            canonical_json(&json!([2, 1]))
        );
    }

    #[test]
    fn strings_use_the_rfc_8785_escape_set() {
        let v = json!({"k": "a\"b\\c\nd\te\u{1}f\u{7f}g€"});
        // Control characters escaped; DEL and non-ASCII emitted literally.
        assert_eq!(
            canonical_json(&v),
            "{\"k\":\"a\\\"b\\\\c\\nd\\te\\u0001f\u{7f}g€\"}"
        );
    }

    #[test]
    fn whitespace_in_the_received_bytes_is_not_part_of_the_identity() {
        let a: Value = serde_json::from_str(r#"{ "a" :  1 , "b": [ 2 ] }"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"b":[2],"a":1}"#).unwrap();
        assert_eq!(canonical_json(&a), canonical_json(&b));
    }

    /// RFC 8785 §3.2.2.3 reads every number as an IEEE 754 double, so these
    /// spellings are one number — as they are to the TypeScript and Go
    /// runtimes, whose parsers produce a double for each.
    #[test]
    fn numbers_canonicalise_as_the_double_they_denote() {
        let canon = |s: &str| canonical_json(&serde_json::from_str::<Value>(s).unwrap());
        for (spellings, expected) in [
            (&["1", "1.0", "1e0", "10e-1"][..], "1"),
            (&["100", "100.0", "1e2", "1E+2"][..], "100"),
            (&["-0", "-0.0", "0"][..], "0"),
            (&["-7", "-7.0"][..], "-7"),
            (&["1.5", "15e-1"][..], "1.5"),
        ] {
            for s in spellings {
                assert_eq!(canon(s), expected, "{s}");
            }
        }
        assert_ne!(canon("1"), canon("1.5"));
        // Large integral values keep integer spelling below 1e21 and agree
        // with their exponent forms.
        assert_eq!(canon("1e20"), "100000000000000000000");
        assert_eq!(canon("1e20"), canon("100000000000000000000"));
        assert_eq!(canon("1e21"), canon("1000000000000000000000"));
    }

    /// The hasher fed in pieces agrees with it fed at once, for every split of
    /// every length around the block and padding boundaries.
    #[test]
    fn sha256_fed_in_pieces_matches_sha256_fed_at_once() {
        let data: Vec<u8> = (0..200u32).map(|i| (i * 31 + 7) as u8).collect();
        for len in 0..data.len() {
            let whole = sha256_hex(&data[..len]);
            for split in 0..=len {
                let mut hasher = Sha256::new();
                hasher.update(&data[..split]);
                hasher.update(&data[split..len]);
                assert_eq!(hex(&hasher.finish()), whole, "len {len}, split {split}");
            }
            let mut bytewise = Sha256::new();
            for byte in &data[..len] {
                bytewise.update(&[*byte]);
            }
            assert_eq!(
                hex(&bytewise.finish()),
                whole,
                "len {len}, a byte at a time"
            );
        }
    }

    /// Streaming the canonical form into the hasher hashes exactly the bytes
    /// `canonical_json` writes.
    #[test]
    fn the_streamed_digest_is_the_digest_of_the_canonical_string() {
        for v in [
            json!(null),
            json!("plain"),
            json!({"k": "a\"b\\c\nd\te\u{1}f\u{1f}\u{7f}g€😀\u{8}\u{c}\r"}),
            json!({"b": [1, 2.5, -0.0, 1e300, {"z": true, "a": false}], "a": {}}),
            json!({ "long": "x".repeat(1000), "list": (0..100).collect::<Vec<_>>() }),
            // Past the sink's flush point, in pieces and in one string.
            json!({ "list": (0..3000).collect::<Vec<_>>(), "long": "y".repeat(9000) }),
        ] {
            assert_eq!(
                canonical_sha256_hex(&v),
                sha256_hex(canonical_json(&v).as_bytes()),
                "{v}"
            );
        }
    }
}
