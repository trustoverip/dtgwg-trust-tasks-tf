package didcomm

import (
	"bytes"
	"crypto/ecdh"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"errors"
	"testing"
)

// RFC 3394 §4.6: wrap 128 bits of key data with a 256-bit KEK. Pins the key-wrap
// primitive to the standard's own test vector.
func TestAESKeyWrapRFC3394(t *testing.T) {
	kek, _ := hex.DecodeString("000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F")
	key, _ := hex.DecodeString("00112233445566778899AABBCCDDEEFF")
	want, _ := hex.DecodeString("64E8C3F9CE0F5BA263E9777905818A2A93C8191E7D6E8AE7")

	got, err := aesKeyWrap(kek, key)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, want) {
		t.Fatalf("wrap = %X, want %X", got, want)
	}
	back, err := aesKeyUnwrap(kek, got)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(back, key) {
		t.Fatalf("unwrap = %X, want %X", back, key)
	}
}

func TestAESKeyUnwrapRejectsTamper(t *testing.T) {
	kek := bytes.Repeat([]byte{0x2a}, 32)
	wrapped, err := aesKeyWrap(kek, bytes.Repeat([]byte{0x11}, 32))
	if err != nil {
		t.Fatal(err)
	}
	wrapped[len(wrapped)-1] ^= 0xff
	if _, err := aesKeyUnwrap(kek, wrapped); err == nil {
		t.Error("unwrap of a tampered ciphertext should fail the integrity check")
	}
}

func TestA256CBCHS512RoundTrip(t *testing.T) {
	cek := make([]byte, cekLen)
	_, _ = rand.Read(cek)
	aad := []byte("protected-header-b64")
	plaintext := []byte("the quick brown fox jumps over the lazy dog")

	iv, ct, tag, err := encryptA256CBCHS512(cek, plaintext, aad)
	if err != nil {
		t.Fatal(err)
	}
	got, err := decryptA256CBCHS512(cek, iv, ct, tag, aad)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, plaintext) {
		t.Fatalf("decrypt = %q, want %q", got, plaintext)
	}
	// A changed AAD must break the tag.
	if _, err := decryptA256CBCHS512(cek, iv, ct, tag, []byte("other-aad")); err == nil {
		t.Error("decryption under a different AAD should fail")
	}
}

func genX25519(t *testing.T) *ecdh.PrivateKey {
	t.Helper()
	k, err := ecdh.X25519().GenerateKey(rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	return k
}

func TestAuthcryptRoundTrip(t *testing.T) {
	sender := genX25519(t)
	recipient := genX25519(t)
	senderKid := "did:example:alice#key-agreement-1"
	recipientKid := "did:example:bob#key-agreement-1"
	plaintext := []byte(`{"hello":"world"}`)

	wire, err := packAuthcrypt(plaintext, sender, senderKid, recipient.PublicKey(), recipientKid)
	if err != nil {
		t.Fatal(err)
	}
	resolve := func(skid string) (*ecdh.PublicKey, error) {
		if skid != senderKid {
			return nil, errors.New("unknown sender")
		}
		return sender.PublicKey(), nil
	}
	got, err := unpackAuthcrypt(wire, recipient, recipientKid, resolve)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got.plaintext, plaintext) {
		t.Fatalf("plaintext = %q, want %q", got.plaintext, plaintext)
	}
	if got.senderKid != senderKid {
		t.Errorf("senderKid = %q, want %q", got.senderKid, senderKid)
	}
}

func TestAuthcryptRejectsWrongRecipient(t *testing.T) {
	sender := genX25519(t)
	recipient := genX25519(t)
	carol := genX25519(t)
	wire, err := packAuthcrypt([]byte("secret"), sender, "did:example:alice#k", recipient.PublicKey(), "did:example:bob#k")
	if err != nil {
		t.Fatal(err)
	}
	resolve := func(string) (*ecdh.PublicKey, error) { return sender.PublicKey(), nil }
	if _, err := unpackAuthcrypt(wire, carol, "did:example:bob#k", resolve); err == nil {
		t.Error("a message sealed to bob must not open for carol")
	}
}

func TestAuthcryptRejectsForgedSender(t *testing.T) {
	// An attacker seals a message but labels it with alice's skid. When the
	// recipient resolves alice's (real) key, the KEK will not match, so the
	// unwrap fails: the skid is authenticated by the ECDH-1PU static secret.
	attacker := genX25519(t)
	alice := genX25519(t)
	recipient := genX25519(t)
	wire, err := packAuthcrypt([]byte("i am alice"), attacker, "did:example:alice#k", recipient.PublicKey(), "did:example:bob#k")
	if err != nil {
		t.Fatal(err)
	}
	resolve := func(string) (*ecdh.PublicKey, error) { return alice.PublicKey(), nil }
	if _, err := unpackAuthcrypt(wire, recipient, "did:example:bob#k", resolve); err == nil {
		t.Error("a forged skid must fail authentication")
	}
}

func TestAuthcryptRejectsTamperedCiphertext(t *testing.T) {
	sender := genX25519(t)
	recipient := genX25519(t)
	wire, err := packAuthcrypt([]byte("secret"), sender, "did:example:alice#k", recipient.PublicKey(), "did:example:bob#k")
	if err != nil {
		t.Fatal(err)
	}
	var env jweJSON
	_ = json.Unmarshal(wire, &env)
	ct, _ := b64.DecodeString(env.Ciphertext)
	ct[0] ^= 0xff
	env.Ciphertext = b64.EncodeToString(ct)
	tampered, _ := json.Marshal(env)
	resolve := func(string) (*ecdh.PublicKey, error) { return sender.PublicKey(), nil }
	if _, err := unpackAuthcrypt(tampered, recipient, "did:example:bob#k", resolve); err == nil {
		t.Error("a tampered ciphertext must fail the content tag")
	}
}

func TestAuthcryptRejectsAnoncryptShape(t *testing.T) {
	// A protected header naming ECDH-ES (anoncrypt) has no authenticated sender.
	env := jweJSON{
		Protected:  b64.EncodeToString([]byte(`{"alg":"ECDH-ES+A256KW","enc":"A256CBC-HS512"}`)),
		Recipients: []recipient{{Header: recipientHeader{Kid: "did:example:bob#k"}, EncryptedKey: b64.EncodeToString(bytes.Repeat([]byte{0}, 40))}},
		IV:         b64.EncodeToString(bytes.Repeat([]byte{0}, 16)),
		Ciphertext: b64.EncodeToString([]byte("x")),
		Tag:        b64.EncodeToString(bytes.Repeat([]byte{0}, 32)),
	}
	wire, _ := json.Marshal(env)
	recipient := genX25519(t)
	resolve := func(string) (*ecdh.PublicKey, error) { return genX25519(t).PublicKey(), nil }
	_, err := unpackAuthcrypt(wire, recipient, "did:example:bob#k", resolve)
	var ee *EnvelopeError
	if !errors.As(err, &ee) || ee.Failure != FailNotAuthcrypt {
		t.Errorf("anoncrypt shape: err = %v, want FailNotAuthcrypt", err)
	}
}
