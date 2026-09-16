// Package proof verifies and produces W3C Data Integrity proofs for Trust Task
// documents, on the Go standard library — the eddsa-jcs-2022 and
// ecdsa-jcs-2019 cryptosuites, did:key only, no network and no third-party
// dependency.
//
// [Verifier] implements the core module's ProofVerifier seam
// (trusttasks.ProofVerifier), and [Signer.Sign] produces what it verifies:
// a proof signed here verifies with the stock verifier by construction, and is
// byte-compatible with the Rust and TypeScript proof libraries — a test
// reproduces the shared eddsa-jcs-2022 fixture's proofValue exactly.
package proof

import (
	"encoding/json"
	"errors"
	"fmt"
	"math"
	"sort"
	"strconv"
	"strings"
	"unicode/utf16"
)

// canonicalize returns the RFC 8785 (JSON Canonicalization Scheme) form of
// value — the byte string eddsa-jcs-2022 and ecdsa-jcs-2019 hash.
//
// This is a real JCS, deliberately NOT the core module's CanonicalJSON: that
// one documents itself as only equality-equivalent to JCS (byte-wise member
// ordering, encoding/json number formatting) and warns it must not be used for
// the proof input. Here the bytes themselves are the interoperable artifact, so
// members are ordered by UTF-16 code unit and numbers use the ECMAScript
// Number-to-String form, matching the Rust and TypeScript runtimes.
//
// value must have been decoded with [decode] so numbers arrive as json.Number.
func canonicalize(value any) ([]byte, error) {
	var b strings.Builder
	if err := writeCanonical(&b, value); err != nil {
		return nil, err
	}
	return []byte(b.String()), nil
}

func writeCanonical(b *strings.Builder, value any) error {
	switch v := value.(type) {
	case nil:
		b.WriteString("null")
	case bool:
		if v {
			b.WriteString("true")
		} else {
			b.WriteString("false")
		}
	case json.Number:
		f, err := v.Float64()
		if err != nil {
			return fmt.Errorf("jcs: %q is not a JSON number: %w", v.String(), err)
		}
		s, err := format8785Number(f)
		if err != nil {
			return err
		}
		b.WriteString(s)
	case float64:
		// Only reached if a caller decoded without json.Number; handled for
		// completeness so the canonicaliser is total.
		s, err := format8785Number(v)
		if err != nil {
			return err
		}
		b.WriteString(s)
	case string:
		writeCanonicalString(b, v)
	case []any:
		b.WriteByte('[')
		for i, item := range v {
			if i > 0 {
				b.WriteByte(',')
			}
			if err := writeCanonical(b, item); err != nil {
				return err
			}
		}
		b.WriteByte(']')
	case map[string]any:
		names := make([]string, 0, len(v))
		for name := range v {
			names = append(names, name)
		}
		sort.Slice(names, func(i, j int) bool { return lessUTF16(names[i], names[j]) })
		b.WriteByte('{')
		for i, name := range names {
			if i > 0 {
				b.WriteByte(',')
			}
			writeCanonicalString(b, name)
			b.WriteByte(':')
			if err := writeCanonical(b, v[name]); err != nil {
				return err
			}
		}
		b.WriteByte('}')
	default:
		return fmt.Errorf("jcs: cannot canonicalize %T", value)
	}
	return nil
}

// lessUTF16 orders two strings by their UTF-16 code units, as RFC 8785 §3.2.3
// requires (and as JavaScript's default Array.prototype.sort does). For strings
// that are entirely within the Basic Multilingual Plane this is identical to a
// byte-wise (UTF-8 scalar) comparison; the two disagree only where an
// astral-plane character (encoded as a surrogate pair, 0xD800..0xDFFF) sorts
// against a BMP character in U+E000..U+FFFF.
func lessUTF16(a, b string) bool {
	ua := utf16.Encode([]rune(a))
	ub := utf16.Encode([]rune(b))
	n := len(ua)
	if len(ub) < n {
		n = len(ub)
	}
	for i := 0; i < n; i++ {
		if ua[i] != ub[i] {
			return ua[i] < ub[i]
		}
	}
	return len(ua) < len(ub)
}

// writeCanonicalString applies the RFC 8785 §3.2.2.2 escape set, and only that
// set — encoding/json cannot be used, as it escapes <, >, &, U+2028 and U+2029,
// none of which are JCS escapes.
func writeCanonicalString(b *strings.Builder, s string) {
	b.WriteByte('"')
	for _, r := range s {
		switch r {
		case '"':
			b.WriteString(`\"`)
		case '\\':
			b.WriteString(`\\`)
		case '\b':
			b.WriteString(`\b`)
		case '\f':
			b.WriteString(`\f`)
		case '\n':
			b.WriteString(`\n`)
		case '\r':
			b.WriteString(`\r`)
		case '\t':
			b.WriteString(`\t`)
		default:
			if r < 0x20 {
				const hexDigits = "0123456789abcdef"
				b.WriteString(`\u00`)
				b.WriteByte(hexDigits[(r>>4)&0xf])
				b.WriteByte(hexDigits[r&0xf])
			} else {
				b.WriteRune(r)
			}
		}
	}
	b.WriteByte('"')
}

// format8785Number returns the RFC 8785 §3.2.2.3 serialization of a JSON number
// — the ECMAScript Number.prototype.toString form.
//
// Go's Ryu-based strconv.FormatFloat already produces the shortest
// round-tripping representation; ES6 differs only in (1) choosing fixed over
// exponential notation across 1e-6 .. <1e21, and (2) not zero-padding the
// exponent ("1e+9", not Go's "1e+09"). This is the well-known reference
// conversion (cyberphone/json-canonicalization). Trust Task documents carry no
// number outside the range this handles cleanly, but handling all finite
// doubles keeps the canonicaliser honest.
func format8785Number(f float64) (string, error) {
	if math.IsNaN(f) || math.IsInf(f, 0) {
		return "", errors.New("jcs: non-finite number")
	}
	if f == 0 { // folds -0 into 0, as JCS mandates
		return "0", nil
	}
	sign := ""
	if f < 0 {
		f = -f
		sign = "-"
	}
	// ES6 uses fixed notation for the magnitude window below, exponential
	// outside it.
	format := byte('e')
	if f < 1e21 && f >= 1e-6 {
		format = 'f'
	}
	s := strconv.FormatFloat(f, format, -1, 64)
	if e := strings.IndexByte(s, 'e'); e >= 0 {
		// Go writes "1e+09" / "1e-09"; ES6 writes "1e+9" / "1e-9".
		exp, err := strconv.Atoi(s[e+2:])
		if err != nil {
			return "", fmt.Errorf("jcs: exponent: %w", err)
		}
		s = s[:e+2] + strconv.Itoa(exp)
	}
	return sign + s, nil
}

// decode parses JSON into the generic tree canonicalize walks, keeping numbers
// as json.Number so their exact lexical value survives to the canonicaliser
// rather than being flattened through float64 twice.
func decode(data []byte) (any, error) {
	dec := json.NewDecoder(strings.NewReader(string(data)))
	dec.UseNumber()
	var v any
	if err := dec.Decode(&v); err != nil {
		return nil, err
	}
	return v, nil
}
