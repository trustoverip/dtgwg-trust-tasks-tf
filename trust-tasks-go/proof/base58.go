package proof

import (
	"errors"
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

// base58Decode reverses base58Encode. It rejects any character outside the
// alphabet.
func base58Decode(s string) ([]byte, error) {
	zeros := 0
	for zeros < len(s) && s[zeros] == base58Alphabet[0] {
		zeros++
	}
	num := new(big.Int)
	radix := big.NewInt(58)
	for i := 0; i < len(s); i++ {
		d := base58Index[s[i]]
		if d < 0 {
			return nil, errors.New("base58: invalid character")
		}
		num.Mul(num, radix)
		num.Add(num, big.NewInt(int64(d)))
	}
	decoded := num.Bytes()
	out := make([]byte, zeros+len(decoded))
	copy(out[zeros:], decoded)
	return out, nil
}
