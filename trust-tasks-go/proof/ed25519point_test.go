package proof

import (
	"bytes"
	"crypto/ed25519"
	"encoding/hex"
	"testing"
)

// The eight points of small order, canonically encoded.
var smallOrderEncodings = []string{
	"0100000000000000000000000000000000000000000000000000000000000000", // identity
	"ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f", // order 2
	"0000000000000000000000000000000000000000000000000000000000000000", // order 4
	"0000000000000000000000000000000000000000000000000000000000000080", // order 4 (y = 0, sign set)
	"26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05", // order 8
	"26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc85", // order 8
	"c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a", // order 8
	"c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac03fa", // order 8
}

var nonCanonicalEncodings = []string{
	"eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f", // y = p + 1
	"edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f", // y = p
	"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", // y = 2^255 - 1
	"0100000000000000000000000000000000000000000000000000000000000080", // identity (x = 0), sign set
	"ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", // order 2 (x = 0), sign set
}

func TestEd25519KeyCheckRefusesSmallOrderAndNonCanonical(t *testing.T) {
	for _, h := range append(append([]string{}, smallOrderEncodings...), nonCanonicalEncodings...) {
		b, _ := hex.DecodeString(h)
		if err := checkEd25519PublicKey(b); err == nil {
			t.Errorf("%s: accepted", h)
		}
	}
}

func TestEd25519KeyCheckAcceptsRealKeys(t *testing.T) {
	for seed := byte(0); seed < 64; seed++ {
		pub := ed25519.NewKeyFromSeed(bytes.Repeat([]byte{seed}, 32)).Public().(ed25519.PublicKey)
		if err := checkEd25519PublicKey(pub); err != nil {
			t.Errorf("seed %d: %v", seed, err)
		}
	}
	// The base point.
	b, _ := hex.DecodeString("5866666666666666666666666666666666666666666666666666666666666666")
	if err := checkEd25519PublicKey(b); err != nil {
		t.Errorf("base point: %v", err)
	}
}

// With the identity as key and R, and S = 0, crypto/ed25519 verifies any
// message. The verifier must not.
func TestIdentityKeyDoesNotVerifyEverything(t *testing.T) {
	identity, _ := hex.DecodeString(smallOrderEncodings[0])
	if !ed25519.Verify(identity, []byte("anything"), append(append([]byte{}, identity...), make([]byte, 32)...)) {
		t.Skip("crypto/ed25519 now refuses the identity key itself")
	}
	if _, err := publicKeyFromRaw(Ed25519, identity); err == nil {
		t.Error("a did:key naming the identity point was accepted")
	}
	key := &publicKey{curve: Ed25519, ed25519: identity}
	sig := append(append([]byte{}, identity...), make([]byte, 32)...)
	if got := kindOf(t, verifySignature(key, []byte("anything"), sig)); got != SignatureInvalid {
		t.Errorf("kind = %s", got)
	}
}
