package proof

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"errors"
	"strings"
)

// Curve is the signing curve behind a cryptosuite.
type Curve string

const (
	// Ed25519 backs eddsa-jcs-2022.
	Ed25519 Curve = "ed25519"
	// P256 backs ecdsa-jcs-2019 (secp256r1).
	P256 Curve = "p256"
	// P384 backs ecdsa-jcs-2019 (secp384r1).
	P384 Curve = "p384"
)

// Cryptosuite is a W3C Data Integrity cryptosuite this package understands.
type Cryptosuite string

const (
	// EddsaJcs2022 is Ed25519 over JCS.
	EddsaJcs2022 Cryptosuite = "eddsa-jcs-2022"
	// EcdsaJcs2019 is P-256 or P-384 over JCS.
	EcdsaJcs2019 Cryptosuite = "ecdsa-jcs-2019"
)

// suiteForCurve names the cryptosuite a signing curve produces.
func suiteForCurve(c Curve) Cryptosuite {
	if c == Ed25519 {
		return EddsaJcs2022
	}
	return EcdsaJcs2019
}

// multicodec is a did:key multikey header — the leading bytes of the
// base58btc-decoded multibase value — and the curve it names.
type multicodec struct {
	curve  Curve
	header []byte
}

// The unsigned-varint multicodec prefixes: ed25519-pub 0xed, p256-pub 0x1200,
// p384-pub 0x1201.
var multicodecs = []multicodec{
	{Ed25519, []byte{0xed, 0x01}},
	{P256, []byte{0x80, 0x24}},
	{P384, []byte{0x81, 0x24}},
}

func headerForCurve(c Curve) []byte {
	for _, m := range multicodecs {
		if m.curve == c {
			return m.header
		}
	}
	return nil
}

// publicKey is a verification key recovered from a did:key.
type publicKey struct {
	curve   Curve
	ed25519 []byte           // set when curve == Ed25519 (32 raw bytes)
	ecdsa   *ecdsa.PublicKey // set for P256/P384
}

// curveParams maps the NIST curves to their elliptic.Curve.
func curveParams(c Curve) elliptic.Curve {
	switch c {
	case P256:
		return elliptic.P256()
	case P384:
		return elliptic.P384()
	default:
		return nil
	}
}

// publicKeyFromDidKey decodes the public key a did:key verification method
// names, locally — no network. Both did:key:<mb> and did:key:<mb>#<mb> carry
// the key in <mb>. Returns an error for a DID that is not a did:key, or one
// whose multikey names a curve this package does not verify.
func publicKeyFromDidKey(verificationMethod string) (*publicKey, error) {
	did := verificationMethod
	if i := strings.IndexByte(did, '#'); i >= 0 {
		did = did[:i]
	}
	const prefix = "did:key:"
	if !strings.HasPrefix(did, prefix) {
		return nil, errors.New("verificationMethod is not a did:key")
	}
	mb := did[len(prefix):]
	decoded, err := fromMultibase(mb)
	if err != nil {
		return nil, err
	}
	for _, m := range multicodecs {
		if len(decoded) > len(m.header) && hasPrefix(decoded, m.header) {
			raw := decoded[len(m.header):]
			return publicKeyFromRaw(m.curve, raw)
		}
	}
	return nil, errors.New("did:key multikey names an unsupported curve")
}

func publicKeyFromRaw(curve Curve, raw []byte) (*publicKey, error) {
	if curve == Ed25519 {
		if len(raw) != 32 {
			return nil, errors.New("ed25519 public key must be 32 bytes")
		}
		return &publicKey{curve: curve, ed25519: raw}, nil
	}
	params := curveParams(curve)
	x, y := elliptic.UnmarshalCompressed(params, raw)
	if x == nil {
		return nil, errors.New("did:key multikey is not a valid compressed point")
	}
	return &publicKey{curve: curve, ecdsa: &ecdsa.PublicKey{Curve: params, X: x, Y: y}}, nil
}

func hasPrefix(b, prefix []byte) bool {
	for i := range prefix {
		if b[i] != prefix[i] {
			return false
		}
	}
	return true
}

// didKeyFromEd25519 builds the did:key verification method for a raw Ed25519
// public key.
func didKeyFromEd25519(pub []byte) string {
	return didKeyFromMultikey(Ed25519, pub)
}

// didKeyFromEcdsa builds the did:key verification method for an ECDSA public
// key, using its compressed SEC1 encoding (the form the multikey stores).
func didKeyFromEcdsa(pub *ecdsa.PublicKey, curve Curve) string {
	compressed := elliptic.MarshalCompressed(curveParams(curve), pub.X, pub.Y)
	return didKeyFromMultikey(curve, compressed)
}

func didKeyFromMultikey(curve Curve, raw []byte) string {
	multikey := append(append([]byte{}, headerForCurve(curve)...), raw...)
	mb := toMultibase(multikey)
	return "did:key:" + mb + "#" + mb
}

// toMultibase returns the multibase base58btc form of raw bytes (leading 'z'),
// as a proofValue carries a signature.
func toMultibase(raw []byte) string {
	return "z" + base58Encode(raw)
}

// fromMultibase decodes a multibase base58btc "z..." value to its raw bytes.
func fromMultibase(value string) ([]byte, error) {
	if !strings.HasPrefix(value, "z") {
		return nil, errors.New("not base58btc multibase (no leading z)")
	}
	return base58Decode(value[1:])
}
