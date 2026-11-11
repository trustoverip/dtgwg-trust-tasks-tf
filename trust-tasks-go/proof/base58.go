package proof

import (
	"errors"
	"fmt"
	"math/big"
)

// base58btc — the Bitcoin alphabet, as multibase 'z' and every did:key multikey
// use it. Rolled here rather than pulled in as a dependency, so this module
// stays free of third-party code (see go.mod).
const base58Alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

var base58Index = func() [256]int8 {
	var idx [256]int8
	for i := range idx {
		idx[i] = -1
	}
	for i := 0; i < len(base58Alphabet); i++ {
		idx[base58Alphabet[i]] = int8(i)
	}
	return idx
}()

// base58Encode returns the base58btc form of input. Leading zero bytes become
// leading '1's, as the encoding requires.
func base58Encode(input []byte) string {
	zeros := 0
	for zeros < len(input) && input[zeros] == 0 {
		zeros++
	}
	num := new(big.Int).SetBytes(input)
	radix := big.NewInt(58)
	mod := new(big.Int)
	var out []byte
	for num.Sign() > 0 {
		num.DivMod(num, radix, mod)
		out = append(out, base58Alphabet[mod.Int64()])
	}
	for i := 0; i < zeros; i++ {
		out = append(out, base58Alphabet[0])
	}
	// The digits were produced least-significant first; reverse them.
	for i, j := 0, len(out)-1; i < j; i, j = i+1, j-1 {
		out[i], out[j] = out[j], out[i]
	}
	return string(out)
}

// maxBase58Bytes bounds what base58Decode will produce. The largest value this
// package decodes is a P-384 signature (96 bytes); a did:key multikey is at most
// 51 (a 2-byte multicodec header and a 49-byte compressed P-384 point). 128
// leaves headroom without letting an input's size set the decoder's cost.
const maxBase58Bytes = 128

// maxBase58Len is the longest base58btc string that can encode maxBase58Bytes
// bytes: ceil(128 * log(256) / log(58)) = 175 characters.
const maxBase58Len = 175

// base58Decode reverses base58Encode. It rejects any character outside the
// alphabet, and any input longer than maxBase58Len characters or decoding to
// more than maxBase58Bytes bytes, before doing work proportional to its size.
func base58Decode(s string) ([]byte, error) {
	if len(s) > maxBase58Len {
		return nil, fmt.Errorf("base58: value is %d characters, longer than the %d this decoder accepts", len(s), maxBase58Len)
	}
	zeros := 0
	for zeros < len(s) && s[zeros] == base58Alphabet[0] {
		zeros++
	}
	// Big-endian accumulator in a fixed buffer: buf[len(buf)-used:] holds the
	// value decoded so far. Each character multiplies it by 58 and adds the
	// digit, touching only the bytes in use — bounded by the length cap above.
	var buf [maxBase58Len]byte
	used := 0
	for i := zeros; i < len(s); i++ {
		d := base58Index[s[i]]
		if d < 0 {
			return nil, errors.New("base58: invalid character")
		}
		carry := uint32(d)
		j := len(buf) - 1
		for k := 0; k < used || carry != 0; k++ {
			carry += 58 * uint32(buf[j])
			buf[j] = byte(carry)
			carry >>= 8
			j--
		}
		if n := len(buf) - 1 - j; n > used {
			used = n
		}
	}
	if zeros+used > maxBase58Bytes {
		return nil, fmt.Errorf("base58: value decodes to more than %d bytes", maxBase58Bytes)
	}
	out := make([]byte, zeros+used)
	copy(out[zeros:], buf[len(buf)-used:])
	return out, nil
}
