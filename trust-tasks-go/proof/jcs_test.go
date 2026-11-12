package proof

import (
	"math"
	"testing"
)

func TestFormat8785Number(t *testing.T) {
	cases := []struct {
		in   float64
		want string
	}{
		{0, "0"},
		{math.Copysign(0, -1), "0"}, // -0 folds to 0
		{1, "1"},
		{-1, "-1"},
		{1.5, "1.5"},
		{100, "100"},
		{0.1, "0.1"},
		{-0.1, "-0.1"},
		{123456789, "123456789"},
		{1e20, "100000000000000000000"},
		{1e21, "1e+21"},
		{1e-6, "0.000001"},
		{1e-7, "1e-7"},
		{5e-324, "5e-324"}, // smallest positive subnormal double
	}
	for _, c := range cases {
		got, err := format8785Number(c.in)
		if err != nil {
			t.Fatalf("format8785Number(%v): %v", c.in, err)
		}
		if got != c.want {
			t.Errorf("format8785Number(%v) = %q, want %q", c.in, got, c.want)
		}
	}
}

func TestFormat8785NumberRejectsNonFinite(t *testing.T) {
	for _, f := range []float64{math.NaN(), math.Inf(1), math.Inf(-1)} {
		if _, err := format8785Number(f); err == nil {
			t.Errorf("format8785Number(%v) = nil error, want non-finite error", f)
		}
	}
}

// lessUTF16 must order by UTF-16 code unit, not by UTF-8 byte. The two disagree
// for a BMP character in U+E000..U+FFFF against an astral character, whose
// UTF-16 high surrogate (0xD800..0xDBFF) sorts below it while its UTF-8 scalar
// sorts above.
func TestLessUTF16(t *testing.T) {
	astral := "\U00010000" // surrogate 0xD800 in UTF-16, scalar 0x10000 in UTF-8
	bmp := ""             // 0xF000 in both
	if !lessUTF16(astral, bmp) {
		t.Errorf("lessUTF16(astral, bmp) = false, want true (UTF-16 orders the surrogate first)")
	}
	if lessUTF16(bmp, astral) {
		t.Errorf("lessUTF16(bmp, astral) = true, want false")
	}
	// Plain ASCII keeps its usual order.
	if !lessUTF16("a", "b") || lessUTF16("b", "a") {
		t.Errorf("ASCII ordering is wrong")
	}
}

func TestCanonicalizeSortsMembersAndEscapes(t *testing.T) {
	tree, err := decode([]byte(`{"b":1,"a":"x\ty","c":[3,2,1]}`))
	if err != nil {
		t.Fatal(err)
	}
	got, err := canonicalize(tree)
	if err != nil {
		t.Fatal(err)
	}
	want := `{"a":"x\ty","b":1,"c":[3,2,1]}`
	if string(got) != want {
		t.Errorf("canonicalize = %q, want %q", got, want)
	}
}

// decode accepts only I-JSON (RFC 7493), which RFC 8785 requires: the inputs
// below are ones encoding/json would accept by rewriting or dropping part of
// them, so two byte strings would share one canonical form.
func TestDecodeRefusesInputJCSCannotRepresentFaithfully(t *testing.T) {
	cases := map[string]string{
		"duplicate member":                   `{"a":1,"a":2}`,
		"duplicate after unescaping":         `{"a":1,"a":2}`,
		"duplicate in a nested object":       `{"x":[{"b":true,"c":null,"b":false}]}`,
		"lone high surrogate":                `{"a":"\ud800"}`,
		"lone high surrogate at end":         `"x\ud800"`,
		"high surrogate then non-surrogate":  `{"a":"\ud800A"}`,
		"high surrogate then high surrogate": `{"a":"\ud800\ud800"}`,
		"lone low surrogate":                 `{"a":"\udc00"}`,
		"lone surrogate in a member name":    `{"\udfff":1}`,
		"invalid UTF-8 in a string":          "{\"a\":\"\xff\"}",
		"UTF-8 encoded surrogate":            "{\"a\":\"\xed\xa0\x80\"}",
		"truncated UTF-8 sequence":           "{\"a\":\"\xe2\x82\"}",
		"trailing value":                     `{"a":1}{"b":2}`,
		"trailing garbage":                   `{"a":1} x`,
	}
	for name, in := range cases {
		t.Run(name, func(t *testing.T) {
			if v, err := decode([]byte(in)); err == nil {
				t.Errorf("decode(%q) = %#v, want an error", in, v)
			}
		})
	}
}

func TestDecodeAcceptsWellFormedInput(t *testing.T) {
	cases := []string{
		`{"a":{"a":1},"b":[{"a":1},{"a":2}]}`, // the same name in different objects
		`{"a":"😀"}`,                           // an escaped surrogate pair
		`{"a":"😀"}`,                           // upper-case hex
		`{"a":"\\ud800"}`,                     // an escaped backslash, not an escape
		"{\"a\":\"\xef\xbf\xbd\"}",            // a literal U+FFFD is valid text
		"{\"a\":\"\xf0\x9f\x98\x80\"}",        // a literal astral character
		"{\"a\":[1,2,{\"b\":\"]}\"}]}\n\t ",   // trailing whitespace only
	}
	for _, in := range cases {
		if _, err := decode([]byte(in)); err != nil {
			t.Errorf("decode(%q): %v", in, err)
		}
	}
}
