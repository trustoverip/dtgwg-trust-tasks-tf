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
//! * **Number formatting** is `serde_json`'s, which is shortest-round-trip
//!   like ECMAScript's `Number::toString` but spells some exponents
//!   differently (`1e100` vs JCS's `1e+100`). Again, the same `f64` always
//!   formats the same way, so the induced equality is unchanged.
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

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Value::Object(map) => {
            // Sorted explicitly rather than relying on `serde_json::Map`'s
            // backing store: the `preserve_order` feature swaps it for an
            // insertion-ordered map, and feature unification means another
            // crate in the graph can turn that on without this crate saying
            // so. A digest that silently became insertion-ordered would key
            // on the sender's member order.
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            out.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write_value(out, &map[key.as_str()]);
            }
            out.push('}');
        }
    }
}

/// RFC 8785 §3.2.2.2 string escaping: the two-character escapes where they
/// exist, `\u00xx` for the remaining control characters, and the literal
/// character otherwise.
fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Lowercase-hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
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

/// FIPS 180-4 SHA-256.
///
/// Hand-rolled rather than pulled in: this crate is deliberately
/// dependency-light, and the digest is used for content *identity* in the
/// consumer's own replay record, never as a signature or a MAC. The
/// implementation is checked against the FIPS 180-4 / NIST CAVP vectors in
/// this module's tests.
fn sha256(input: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut message = input.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
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

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
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
}
