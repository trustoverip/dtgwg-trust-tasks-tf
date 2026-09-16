package didcomm

import (
	"crypto/aes"
	"crypto/subtle"
	"errors"
)

// AES Key Wrap (RFC 3394) — the A256KW half of ECDH-1PU+A256KW. The derived
// key-encryption key wraps the randomly generated content-encryption key.
// Rolled here because the standard library has no key-wrap primitive.

// defaultIV is the RFC 3394 §2.2.3.1 default initial value.
var defaultIV = []byte{0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6}

// aesKeyWrap wraps plaintext (a multiple of 8 bytes, at least 16) under kek.
func aesKeyWrap(kek, plaintext []byte) ([]byte, error) {
	if len(plaintext)%8 != 0 || len(plaintext) < 16 {
		return nil, errors.New("didcomm: key wrap input must be a multiple of 8 bytes and at least 16")
	}
	block, err := aes.NewCipher(kek)
	if err != nil {
		return nil, err
	}
	n := len(plaintext) / 8
	a := make([]byte, 8)
	copy(a, defaultIV)
	r := make([]byte, len(plaintext))
	copy(r, plaintext)

	var buf [16]byte
	for j := 0; j < 6; j++ {
		for i := 0; i < n; i++ {
			copy(buf[:8], a)
			copy(buf[8:], r[i*8:i*8+8])
			block.Encrypt(buf[:], buf[:])
			copy(a, buf[:8])
			t := uint64(n*j + i + 1)
			a[0] ^= byte(t >> 56)
			a[1] ^= byte(t >> 48)
			a[2] ^= byte(t >> 40)
			a[3] ^= byte(t >> 32)
			a[4] ^= byte(t >> 24)
			a[5] ^= byte(t >> 16)
			a[6] ^= byte(t >> 8)
			a[7] ^= byte(t)
			copy(r[i*8:i*8+8], buf[8:])
		}
	}
	out := make([]byte, 8+len(r))
	copy(out[:8], a)
	copy(out[8:], r)
	return out, nil
}

// aesKeyUnwrap reverses aesKeyWrap and checks the integrity value in constant
// time. A tampered or wrong-key ciphertext fails here.
func aesKeyUnwrap(kek, ciphertext []byte) ([]byte, error) {
	if len(ciphertext)%8 != 0 || len(ciphertext) < 24 {
		return nil, errors.New("didcomm: key unwrap input must be a multiple of 8 bytes and at least 24")
	}
	block, err := aes.NewCipher(kek)
	if err != nil {
		return nil, err
	}
	n := len(ciphertext)/8 - 1
	a := make([]byte, 8)
	copy(a, ciphertext[:8])
	r := make([]byte, len(ciphertext)-8)
	copy(r, ciphertext[8:])

	var buf [16]byte
	for j := 5; j >= 0; j-- {
		for i := n - 1; i >= 0; i-- {
			t := uint64(n*j + i + 1)
			a[0] ^= byte(t >> 56)
			a[1] ^= byte(t >> 48)
			a[2] ^= byte(t >> 40)
			a[3] ^= byte(t >> 32)
			a[4] ^= byte(t >> 24)
			a[5] ^= byte(t >> 16)
			a[6] ^= byte(t >> 8)
			a[7] ^= byte(t)
			copy(buf[:8], a)
			copy(buf[8:], r[i*8:i*8+8])
			block.Decrypt(buf[:], buf[:])
			copy(a, buf[:8])
			copy(r[i*8:i*8+8], buf[8:])
		}
	}
	if subtle.ConstantTimeCompare(a, defaultIV) != 1 {
		return nil, errors.New("didcomm: key unwrap integrity check failed")
	}
	return r, nil
}
