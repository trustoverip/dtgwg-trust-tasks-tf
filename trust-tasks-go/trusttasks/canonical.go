package trusttasks

// Deterministic document serialization and the SHA-256 digest built over it.
//
// Mirrors canonical.rs in trust-tasks-rs and canonical.ts in
// @openvtc/trust-tasks. SPEC §7.2 (*Keying and comparison for item 11*) defines
// two documents sharing an id as the same document when their serializations are
// identical under RFC 8785 canonicalization — the same identity §8.4 gives a
// retry. A consumer implementing item 11 has to retain a digest of what it
// accepted, not merely the id.
//
// # What this canonicalizes, and what it does not
//
// [CanonicalJSON] emits a serialization that is *equality-equivalent* to RFC
// 8785: two documents produce the same bytes here exactly when they produce the
// same bytes under JCS. It is deliberately not claimed to be byte-identical to a
// JCS implementation's output, and it must not be used where the bytes
// themselves are the interoperable artifact — notably the §4.9.3 task digest,
// which is published in citations and recomputed by other parties, and the
// eddsa-jcs-2022 proof input.
//
// Two deviations are known, and both are confined to the bytes rather than the
// equality relation they induce — the same two trust-tasks-rs documents:
//
//   - Member ordering is by Unicode scalar value (Go's byte-wise string
//     ordering, which for UTF-8 is scalar order), where JCS orders by UTF-16
//     code unit. The two disagree only for member names containing characters
//     above the BMP alongside names in U+E000..U+FFFF. Any such disagreement
//     reorders both documents being compared identically, so it cannot make two
//     JCS-identical documents differ here, nor two JCS-differing documents agree.
//   - Number formatting is encoding/json's, which is shortest-round-trip like
//     ECMAScript's Number::toString but spells some exponents differently. The
//     same float64 always formats the same way, so the induced equality is
//     unchanged.
//
// The item-11 digest is consumer-local: it goes into this consumer's own replay
// record and never onto the wire, into a citation, or into a proof. Only the
// equality relation is load-bearing, which is why this is the right size of tool.

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"sort"
)

// CanonicalJSON re-serializes a JSON document with object members recursively
// ordered and no insignificant whitespace.
//
// Input must be valid JSON. The result is stable for a given input value: two
// documents that differ only in member order or whitespace canonicalize to the
// same bytes.
func CanonicalJSON(document []byte) ([]byte, error) {
	var value any
	if err := json.Unmarshal(document, &value); err != nil {
		return nil, fmt.Errorf("trusttasks: canonicalize: %w", err)
	}
	var out bytes.Buffer
	if err := writeCanonical(&out, value); err != nil {
		return nil, err
	}
	return out.Bytes(), nil
}

func writeCanonical(out *bytes.Buffer, value any) error {
	switch v := value.(type) {
	case nil:
		out.WriteString("null")
	case bool:
		if v {
			out.WriteString("true")
		} else {
			out.WriteString("false")
		}
	case float64:
		// encoding/json's own float formatting, for the reason given in the
		// header: shortest-round-trip, and identical for identical values.
		encoded, err := json.Marshal(v)
		if err != nil {
			return fmt.Errorf("trusttasks: canonicalize number: %w", err)
		}
		out.Write(encoded)
	case string:
		writeCanonicalString(out, v)
	case []any:
		out.WriteByte('[')
		for i, item := range v {
			if i > 0 {
				out.WriteByte(',')
			}
			if err := writeCanonical(out, item); err != nil {
				return err
			}
		}
		out.WriteByte(']')
	case map[string]any:
		names := make([]string, 0, len(v))
		for name := range v {
			names = append(names, name)
		}
		sort.Strings(names)
		out.WriteByte('{')
		for i, name := range names {
			if i > 0 {
				out.WriteByte(',')
			}
			writeCanonicalString(out, name)
			out.WriteByte(':')
			if err := writeCanonical(out, v[name]); err != nil {
				return err
			}
		}
		out.WriteByte('}')
	default:
		return fmt.Errorf("trusttasks: canonicalize: unexpected JSON value %T", value)
	}
	return nil
}

// writeCanonicalString applies the RFC 8785 §3.2.2.2 escape set, and only that
// set.
//
// encoding/json cannot be used here: it escapes <, > and & to their six-character Unicode escapes
// for HTML safety, and escapes U+2028 / U+2029 for JavaScript safety. None of
// those are JCS escapes, and all of them would make this disagree with the Rust
// and TypeScript runtimes about the bytes a document canonicalizes to.
func writeCanonicalString(out *bytes.Buffer, s string) {
	out.WriteByte('"')
	for _, r := range s {
		switch r {
		case '"':
			out.WriteString(`\"`)
		case '\\':
			out.WriteString(`\\`)
		case '\b':
			out.WriteString(`\b`)
		case '\f':
			out.WriteString(`\f`)
		case '\n':
			out.WriteString(`\n`)
		case '\r':
			out.WriteString(`\r`)
		case '\t':
			out.WriteString(`\t`)
		default:
			if r < 0x20 {
				const hexDigits = "0123456789abcdef"
				out.WriteString(`\u00`)
				out.WriteByte(hexDigits[(r>>4)&0xf])
				out.WriteByte(hexDigits[r&0xf])
			} else {
				out.WriteRune(r)
			}
		}
	}
	out.WriteByte('"')
}

// SHA256Hex returns the lowercase hex SHA-256 of input.
func SHA256Hex(input []byte) string {
	sum := sha256.Sum256(input)
	return hex.EncodeToString(sum[:])
}
