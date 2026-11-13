package proof

import (
	"bytes"
	"crypto/rand"
	"encoding/hex"
	"strings"
	"testing"
	"time"
)

func TestBase58RoundTrip(t *testing.T) {
	cases := [][]byte{
		{},
		{0},
		{0, 0, 1},
		{0xff},
		bytes.Repeat([]byte{0xff}, maxBase58Bytes),
		bytes.Repeat([]byte{0}, maxBase58Bytes),
		append([]byte{0, 0}, bytes.Repeat([]byte{0x80}, 94)...),
	}
	for i := 0; i < 200; i++ {
		b := make([]byte, 1+i%maxBase58Bytes)
		if _, err := rand.Read(b); err != nil {
			t.Fatal(err)
		}
		if i%3 == 0 {
			b[0] = 0
		}
		cases = append(cases, b)
	}
	for _, in := range cases {
		enc := base58Encode(in)
		if len(enc) > maxBase58Len {
			t.Fatalf("encoding of %d bytes is %d characters, over maxBase58Len", len(in), len(enc))
		}
		out, err := base58Decode(enc)
		if err != nil {
			t.Fatalf("decode %q: %v", enc, err)
		}
		if !bytes.Equal(out, in) {
			t.Fatalf("round trip of %x gave %x", in, out)
		}
	}
}

func TestBase58KnownVectors(t *testing.T) {
	vectors := map[string]string{
		"":                     "",
		"61":                   "2g",
		"626262":               "a3gV",
		"636363":               "aPEr",
		"00000000000000000000": "1111111111",
		"516b6fcd0f":           "ABnLTmg",
		"bf4f89001e670274dd":   "3SEo3LWLoPntC",
		"572e4794":             "3EFU7m",
		"ecac89cad93923c02321": "EJDM8drfXA6uyA",
		"10c8511e":             "Rt5zm",
		"73696d706c792061206c6f6e6720737472696e67": "2cFupjhnEsSn59qHXstmK2ffpLv2",
	}
	for hexIn, want := range vectors {
		in := mustHex(t, hexIn)
		if got := base58Encode(in); got != want {
			t.Fatalf("encode %s = %q, want %q", hexIn, got, want)
		}
		out, err := base58Decode(want)
		if err != nil || !bytes.Equal(out, in) {
			t.Fatalf("decode %q = %x, %v; want %s", want, out, err, hexIn)
		}
	}
}

func TestBase58RefusesOverLongInput(t *testing.T) {
	if _, err := base58Decode(strings.Repeat("z", maxBase58Len+1)); err == nil {
		t.Fatal("decoded a value longer than maxBase58Len")
	}
	// maxBase58Len characters can still name a value one byte too large.
	if _, err := base58Decode(strings.Repeat("z", maxBase58Len)); err == nil {
		t.Fatal("decoded a value larger than maxBase58Bytes")
	}
	if _, err := base58Decode(strings.Repeat("1", maxBase58Bytes+1)); err == nil {
		t.Fatal("decoded more than maxBase58Bytes leading zeros")
	}
}

func TestBase58RejectsInvalidCharacter(t *testing.T) {
	for _, s := range []string{"0", "O", "I", "l", "abc+", "\x00"} {
		if _, err := base58Decode(s); err == nil {
			t.Fatalf("decoded %q", s)
		}
	}
}

// A megabyte-sized multibase value is refused without decoding it, so the cost
// of a verificationMethod or proofValue does not grow with its length.
func TestBase58LargeInputIsRefusedQuickly(t *testing.T) {
	huge := "z" + strings.Repeat("2", 1<<20)
	start := time.Now()
	for i := 0; i < 100; i++ {
		if _, err := fromMultibase(huge); err == nil {
			t.Fatal("decoded a 1 MiB multibase value")
		}
		if _, err := publicKeyFromDidKey("did:key:" + huge); err == nil {
			t.Fatal("accepted a 1 MiB did:key")
		}
	}
	if elapsed := time.Since(start); elapsed > time.Second {
		t.Fatalf("refusing 200 oversized values took %s", elapsed)
	}
}

func mustHex(t *testing.T, s string) []byte {
	t.Helper()
	b, err := hex.DecodeString(s)
	if err != nil {
		t.Fatal(err)
	}
	return b
}
