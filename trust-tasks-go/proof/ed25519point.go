package proof

import (
	"errors"
	"math/big"
)

// crypto/ed25519 decodes a public key as filippo.io/edwards25519 does: it
// accepts a non-canonical encoding (a y coordinate of p or more) and a point of
// small order. A small-order key verifies a signature it never made — with the
// identity as the key and as R, and S = 0, ed25519.Verify returns true for any
// message. The Rust (verify_strict), TypeScript (RFC 8032 decoding) and Dart
// backends refuse such keys, so this one does too, before the key is used.
//
// The check is done on math/big rather than with a curve library, to keep this
// module free of third-party dependencies. It runs once per verification on 32
// bytes; its cost is a few modular exponentiations.

var (
	edP = func() *big.Int {
		p := new(big.Int).Lsh(big.NewInt(1), 255)
		return p.Sub(p, big.NewInt(19))
	}()
	// edD is -121665/121666 mod p.
	edD = func() *big.Int {
		num := new(big.Int).Neg(big.NewInt(121665))
		den := new(big.Int).ModInverse(big.NewInt(121666), edP)
		num.Mul(num, den)
		return num.Mod(num, edP)
	}()
	// edSqrtM1 is a square root of -1 mod p: 2^((p-1)/4).
	edSqrtM1 = func() *big.Int {
		e := new(big.Int).Sub(edP, big.NewInt(1))
		e.Rsh(e, 2)
		return new(big.Int).Exp(big.NewInt(2), e, edP)
	}()
)

type edPoint struct{ x, y *big.Int }

// checkEd25519PublicKey refuses an encoding RFC 8032 §5.1.3 does not decode
// (non-canonical y, no such point, or x = 0 with the sign bit set) and a point
// of small order (one of the eight whose order divides the cofactor 8).
func checkEd25519PublicKey(encoded []byte) error {
	p, err := decodeEd25519Point(encoded)
	if err != nil {
		return err
	}
	q := p
	for i := 0; i < 3; i++ {
		q = edAdd(q, q)
	}
	if q.x.Sign() == 0 && q.y.Cmp(big.NewInt(1)) == 0 {
		return errors.New("ed25519 public key has small order")
	}
	return nil
}

func decodeEd25519Point(encoded []byte) (edPoint, error) {
	if len(encoded) != 32 {
		return edPoint{}, errors.New("ed25519 public key must be 32 bytes")
	}
	le := make([]byte, 32)
	copy(le, encoded)
	sign := le[31] >> 7
	le[31] &= 0x7f
	be := make([]byte, 32)
	for i := range le {
		be[31-i] = le[i]
	}
	y := new(big.Int).SetBytes(be)
	if y.Cmp(edP) >= 0 {
		return edPoint{}, errors.New("ed25519 public key is not canonically encoded")
	}
	// x^2 = (y^2 - 1) / (d y^2 + 1)
	y2 := new(big.Int).Mul(y, y)
	y2.Mod(y2, edP)
	u := new(big.Int).Sub(y2, big.NewInt(1))
	u.Mod(u, edP)
	v := new(big.Int).Mul(edD, y2)
	v.Add(v, big.NewInt(1))
	v.Mod(v, edP)
	vInv := new(big.Int).ModInverse(v, edP)
	if vInv == nil {
		return edPoint{}, errors.New("ed25519 public key is not a curve point")
	}
	x2 := new(big.Int).Mul(u, vInv)
	x2.Mod(x2, edP)
	e := new(big.Int).Add(edP, big.NewInt(3))
	e.Rsh(e, 3)
	x := new(big.Int).Exp(x2, e, edP)
	if sq := new(big.Int).Mod(new(big.Int).Mul(x, x), edP); sq.Cmp(x2) != 0 {
		x.Mul(x, edSqrtM1)
		x.Mod(x, edP)
		if sq := new(big.Int).Mod(new(big.Int).Mul(x, x), edP); sq.Cmp(x2) != 0 {
			return edPoint{}, errors.New("ed25519 public key is not a curve point")
		}
	}
	if x.Sign() == 0 && sign == 1 {
		return edPoint{}, errors.New("ed25519 public key is not canonically encoded")
	}
	if byte(x.Bit(0)) != sign {
		x.Sub(edP, x)
	}
	return edPoint{x: x, y: y}, nil
}

// edAdd is the complete twisted Edwards addition (a = -1) in affine
// coordinates.
func edAdd(p, q edPoint) edPoint {
	x1y2 := new(big.Int).Mul(p.x, q.y)
	y1x2 := new(big.Int).Mul(p.y, q.x)
	y1y2 := new(big.Int).Mul(p.y, q.y)
	x1x2 := new(big.Int).Mul(p.x, q.x)
	t := new(big.Int).Mul(edD, x1x2)
	t.Mul(t, y1y2)
	t.Mod(t, edP)
	one := big.NewInt(1)
	xDen := new(big.Int).Add(one, t)
	yDen := new(big.Int).Sub(one, t)
	xDen.Mod(xDen, edP)
	yDen.Mod(yDen, edP)
	x := new(big.Int).Add(x1y2, y1x2)
	x.Mul(x, new(big.Int).ModInverse(xDen, edP))
	x.Mod(x, edP)
	y := new(big.Int).Add(y1y2, x1x2)
	y.Mul(y, new(big.Int).ModInverse(yDen, edP))
	y.Mod(y, edP)
	return edPoint{x: x, y: y}
}
