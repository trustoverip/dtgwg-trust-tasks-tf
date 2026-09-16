package didcomm

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/ecdh"
	"crypto/hmac"
	"crypto/rand"
	"crypto/sha256"
	"crypto/sha512"
	"crypto/subtle"
	"encoding/base64"
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"sort"
	"strings"
)

// This file rolls the one DIDComm v2.1 authcrypt profile the binding uses:
// ECDH-1PU key agreement (draft-madden-jose-ecdh-1pu) with A256KW key wrapping
// and A256CBC-HS512 content encryption, over X25519 key-agreement keys. It is
// a JWE in general (flattened-per-recipient) JSON serialisation. anoncrypt
// (ECDH-ES, no authenticated sender) and any other alg/enc are rejected on
// unpack — binding §2 makes authcrypt a MUST.

const (
	algAuthcrypt = "ECDH-1PU+A256KW"
	encA256CBC   = "A256CBC-HS512"
	kekBits      = 256 // A256KW
	cekLen       = 64  // A256CBC-HS512: 32-byte MAC key ‖ 32-byte AES-256 key
)

var b64 = base64.RawURLEncoding

// epk is the ephemeral public key in a JWE protected header (an OKP JWK).
type epk struct {
	Kty string `json:"kty"`
	Crv string `json:"crv"`
	X   string `json:"x"`
}

// protectedHeader is the integrity-protected JWE header. For authcrypt the
// sender key id (skid) lives here, so it is authenticated by the AEAD.
type protectedHeader struct {
	Alg  string `json:"alg"`
	Enc  string `json:"enc"`
	Typ  string `json:"typ,omitempty"`
	Skid string `json:"skid"`
	Apu  string `json:"apu"`
	Apv  string `json:"apv"`
	Epk  epk    `json:"epk"`
}

type recipient struct {
	Header       recipientHeader `json:"header"`
	EncryptedKey string          `json:"encrypted_key"`
}

type recipientHeader struct {
	Kid string `json:"kid"`
}

// jweJSON is the flattened JWE JSON serialisation this binding emits and reads.
type jweJSON struct {
	Protected  string      `json:"protected"`
	Recipients []recipient `json:"recipients"`
	IV         string      `json:"iv"`
	Ciphertext string      `json:"ciphertext"`
	Tag        string      `json:"tag"`
}

// authcrypted is what unpacking a JWE yields: the plaintext and the
// authenticated sender / opening recipient key ids.
type authcrypted struct {
	plaintext    []byte
	senderKid    string
	recipientKid string
}

// packAuthcrypt seals plaintext to one recipient, authenticated as senderKid.
// senderPriv is the sender's X25519 private key, recipientPub the recipient's
// X25519 public key. The returned bytes are the JWE JSON.
func packAuthcrypt(plaintext []byte, senderPriv *ecdh.PrivateKey, senderKid string, recipientPub *ecdh.PublicKey, recipientKid string) ([]byte, error) {
	ephemeral, err := ecdh.X25519().GenerateKey(rand.Reader)
	if err != nil {
		return nil, err
	}

	// apu is the sender kid; apv is SHA-256 of the sorted, dot-joined recipient
	// kids (the DIDComm v2 convention — matches affinidi/askar/didcomm-python).
	apu := []byte(senderKid)
	apv := computeAPV([]string{recipientKid})
	header := protectedHeader{
		Alg:  algAuthcrypt,
		Enc:  encA256CBC,
		Typ:  mediaTypeEncrypted,
		Skid: senderKid,
		Apu:  b64.EncodeToString(apu),
		Apv:  b64.EncodeToString(apv),
		Epk:  epk{Kty: "OKP", Crv: "X25519", X: b64.EncodeToString(ephemeral.PublicKey().Bytes())},
	}
	protectedJSON, err := json.Marshal(header)
	if err != nil {
		return nil, err
	}
	protectedB64 := b64.EncodeToString(protectedJSON)

	// ECDH-1PU authcrypt is tag-in-KDF: encrypt the content FIRST, then derive
	// the key-wrapping KEK with the content-encryption tag as SuppPrivInfo, then
	// wrap the CEK. The AAD is the base64url protected header.
	cek := make([]byte, cekLen)
	if _, err := rand.Read(cek); err != nil {
		return nil, err
	}
	iv, ciphertext, tag, err := encryptA256CBCHS512(cek, plaintext, []byte(protectedB64))
	if err != nil {
		return nil, err
	}

	ze, err := ephemeral.ECDH(recipientPub)
	if err != nil {
		return nil, fmt.Errorf("didcomm: ephemeral ECDH: %w", err)
	}
	zs, err := senderPriv.ECDH(recipientPub)
	if err != nil {
		return nil, fmt.Errorf("didcomm: static ECDH: %w", err)
	}
	z := append(append([]byte{}, ze...), zs...)
	kek := concatKDF(z, []byte(algAuthcrypt), apu, apv, kekBits, tag)
	wrapped, err := aesKeyWrap(kek, cek)
	if err != nil {
		return nil, err
	}

	out := jweJSON{
		Protected:  protectedB64,
		Recipients: []recipient{{Header: recipientHeader{Kid: recipientKid}, EncryptedKey: b64.EncodeToString(wrapped)}},
		IV:         b64.EncodeToString(iv),
		Ciphertext: b64.EncodeToString(ciphertext),
		Tag:        b64.EncodeToString(tag),
	}
	return json.Marshal(out)
}

// computeAPV is the DIDComm v2 PartyVInfo: SHA-256 of the recipient key ids,
// sorted and joined by ".". For this binding there is always one recipient.
func computeAPV(kids []string) []byte {
	sorted := append([]string(nil), kids...)
	sort.Strings(sorted)
	sum := sha256.Sum256([]byte(strings.Join(sorted, ".")))
	return sum[:]
}

// resolveSenderKey maps a sender key id (a DID URL) to its X25519 public key.
type resolveSenderKey func(senderKid string) (*ecdh.PublicKey, error)

// unpackAuthcrypt opens a JWE addressed to recipientKid with recipientPriv,
// resolving the sender's public key from the envelope's authenticated skid. A
// non-authcrypt envelope (anoncrypt, or any other alg/enc) is rejected: there
// is no authenticated sender. A wrong key, a forged skid, or tampering fails
// the key-unwrap or the content tag.
func unpackAuthcrypt(wire []byte, recipientPriv *ecdh.PrivateKey, recipientKid string, resolve resolveSenderKey) (*authcrypted, error) {
	var env jweJSON
	if err := json.Unmarshal(wire, &env); err != nil {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "not a JWE JSON envelope"}
	}
	if env.Protected == "" || env.Ciphertext == "" {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "envelope is not encrypted JWE JSON"}
	}
	protectedJSON, err := b64.DecodeString(env.Protected)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "protected header is not base64url"}
	}
	var header protectedHeader
	if err := json.Unmarshal(protectedJSON, &header); err != nil {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "protected header is not JSON"}
	}
	if header.Alg != algAuthcrypt || header.Enc != encA256CBC {
		// anoncrypt (ECDH-ES) or any other profile: no authenticated sender.
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: fmt.Sprintf("alg %q enc %q is not authcrypt", header.Alg, header.Enc)}
	}
	if header.Skid == "" {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "authcrypt envelope carries no skid"}
	}
	if header.Epk.Crv != "X25519" || header.Epk.Kty != "OKP" {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "epk is not an X25519 OKP key"}
	}
	epkBytes, err := b64.DecodeString(header.Epk.X)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "epk.x is not base64url"}
	}
	ephemeralPub, err := ecdh.X25519().NewPublicKey(epkBytes)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "epk is not a valid X25519 point"}
	}

	rec, ok := findRecipient(env.Recipients, recipientKid)
	if !ok {
		return nil, &EnvelopeError{Failure: FailNotForThisRecipient, Detail: fmt.Sprintf("no recipient entry for %s", recipientKid)}
	}
	wrapped, err := b64.DecodeString(rec.EncryptedKey)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailNotAuthcrypt, Detail: "encrypted_key is not base64url"}
	}

	senderPub, err := resolve(header.Skid)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailSenderUnresolved, Detail: fmt.Sprintf("cannot resolve sender %s: %v", header.Skid, err), Sender: header.Skid}
	}

	ze, err := recipientPriv.ECDH(ephemeralPub)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailDecrypt, Detail: "ephemeral ECDH failed", Sender: header.Skid}
	}
	zs, err := recipientPriv.ECDH(senderPub)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailDecrypt, Detail: "static ECDH failed", Sender: header.Skid}
	}
	z := append(append([]byte{}, ze...), zs...)

	// PartyUInfo/PartyVInfo are the raw (base64url-decoded) apu/apv the header
	// carries, so the KDF input matches the sender's regardless of how it chose
	// to fill them. The content-encryption tag is fed into the KDF as
	// SuppPrivInfo (ECDH-1PU tag-in-KDF), so it must be decoded before the KEK.
	apu, _ := b64.DecodeString(header.Apu)
	apv, _ := b64.DecodeString(header.Apv)
	tag, err := b64.DecodeString(env.Tag)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailDecrypt, Detail: "tag is not base64url", Sender: header.Skid}
	}
	kek := concatKDF(z, []byte(algAuthcrypt), apu, apv, kekBits, tag)

	cek, err := aesKeyUnwrap(kek, wrapped)
	if err != nil {
		// Wrong sender key (forged skid), wrong recipient, or tampering.
		return nil, &EnvelopeError{Failure: FailDecrypt, Detail: "key unwrap failed (sender authentication or integrity)", Sender: header.Skid}
	}

	iv, err := b64.DecodeString(env.IV)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailDecrypt, Detail: "iv is not base64url", Sender: header.Skid}
	}
	ciphertext, err := b64.DecodeString(env.Ciphertext)
	if err != nil {
		return nil, &EnvelopeError{Failure: FailDecrypt, Detail: "ciphertext is not base64url", Sender: header.Skid}
	}
	plaintext, err := decryptA256CBCHS512(cek, iv, ciphertext, tag, []byte(env.Protected))
	if err != nil {
		return nil, &EnvelopeError{Failure: FailDecrypt, Detail: "content decryption failed", Sender: header.Skid}
	}

	return &authcrypted{plaintext: plaintext, senderKid: header.Skid, recipientKid: rec.Header.Kid}, nil
}

func findRecipient(recipients []recipient, kid string) (recipient, bool) {
	for _, r := range recipients {
		if r.Header.Kid == kid {
			return r, true
		}
	}
	// A single unlabelled recipient is taken to be ours.
	if len(recipients) == 1 && recipients[0].Header.Kid == "" {
		return recipients[0], true
	}
	return recipient{}, false
}

// encryptA256CBCHS512 is AEAD_AES_256_CBC_HMAC_SHA_512 (RFC 7518 §5.2.5).
func encryptA256CBCHS512(cek, plaintext, aad []byte) (iv, ciphertext, tag []byte, err error) {
	if len(cek) != cekLen {
		return nil, nil, nil, errors.New("didcomm: CEK must be 64 bytes for A256CBC-HS512")
	}
	macKey := cek[:32]
	encKey := cek[32:]
	block, err := aes.NewCipher(encKey)
	if err != nil {
		return nil, nil, nil, err
	}
	iv = make([]byte, aes.BlockSize)
	if _, err := rand.Read(iv); err != nil {
		return nil, nil, nil, err
	}
	padded := pkcs7Pad(plaintext, aes.BlockSize)
	ciphertext = make([]byte, len(padded))
	cipher.NewCBCEncrypter(block, iv).CryptBlocks(ciphertext, padded)
	tag = authTag(macKey, aad, iv, ciphertext)
	return iv, ciphertext, tag, nil
}

func decryptA256CBCHS512(cek, iv, ciphertext, tag, aad []byte) ([]byte, error) {
	if len(cek) != cekLen {
		return nil, errors.New("didcomm: CEK must be 64 bytes")
	}
	macKey := cek[:32]
	encKey := cek[32:]
	expected := authTag(macKey, aad, iv, ciphertext)
	if subtle.ConstantTimeCompare(expected, tag) != 1 {
		return nil, errors.New("didcomm: authentication tag mismatch")
	}
	block, err := aes.NewCipher(encKey)
	if err != nil {
		return nil, err
	}
	if len(ciphertext) == 0 || len(ciphertext)%aes.BlockSize != 0 || len(iv) != aes.BlockSize {
		return nil, errors.New("didcomm: malformed ciphertext")
	}
	padded := make([]byte, len(ciphertext))
	cipher.NewCBCDecrypter(block, iv).CryptBlocks(padded, ciphertext)
	return pkcs7Unpad(padded, aes.BlockSize)
}

// authTag is T = HMAC-SHA-512(MAC_KEY, AAD ‖ IV ‖ ciphertext ‖ AL)[:32], where
// AL is the AAD length in bits as a 64-bit big-endian integer.
func authTag(macKey, aad, iv, ciphertext []byte) []byte {
	var al [8]byte
	binary.BigEndian.PutUint64(al[:], uint64(len(aad))*8)
	h := hmac.New(sha512.New, macKey)
	h.Write(aad)
	h.Write(iv)
	h.Write(ciphertext)
	h.Write(al[:])
	return h.Sum(nil)[:32]
}

func pkcs7Pad(b []byte, blockSize int) []byte {
	n := blockSize - len(b)%blockSize
	pad := make([]byte, n)
	for i := range pad {
		pad[i] = byte(n)
	}
	return append(b, pad...)
}

func pkcs7Unpad(b []byte, blockSize int) ([]byte, error) {
	if len(b) == 0 || len(b)%blockSize != 0 {
		return nil, errors.New("didcomm: invalid padded length")
	}
	n := int(b[len(b)-1])
	if n == 0 || n > blockSize || n > len(b) {
		return nil, errors.New("didcomm: invalid PKCS#7 padding")
	}
	var bad byte
	for _, c := range b[len(b)-n:] {
		bad |= c ^ byte(n)
	}
	if bad != 0 {
		return nil, errors.New("didcomm: invalid PKCS#7 padding")
	}
	return b[:len(b)-n], nil
}
