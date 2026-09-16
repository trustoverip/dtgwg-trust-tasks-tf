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
