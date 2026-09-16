package proof

import (
	"crypto/ecdsa"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"time"
)

// Signer holds a private key and the did:key it publishes. Build one with
// [SignerFromPrivateKey].
type Signer struct {
	curve Curve
	// VerificationMethod is the did:key:...#... the proof names.
	VerificationMethod string
	// DID is the document issuer this signer produces — the did:key without a
	// fragment.
	DID  string
	sign func(hashData []byte) ([]byte, error)
}

// SignerFromPrivateKey builds a [Signer] from a raw private key: a 32-byte
// Ed25519 seed, or the big-endian scalar of a P-256 (32-byte) or P-384
// (48-byte) key.
func SignerFromPrivateKey(curve Curve, privateKey []byte) (*Signer, error) {
	switch curve {
	case Ed25519:
		if len(privateKey) != ed25519.SeedSize {
			return nil, fmt.Errorf("ed25519 private key must be a %d-byte seed", ed25519.SeedSize)
		}
		priv := ed25519.NewKeyFromSeed(privateKey)
		pub := priv.Public().(ed25519.PublicKey)
		vm := didKeyFromEd25519(pub)
		return &Signer{
			curve:              curve,
			VerificationMethod: vm,
			DID:                didOf(vm),
			sign: func(hashData []byte) ([]byte, error) {
				return ed25519.Sign(priv, hashData), nil
			},
		}, nil
	case P256, P384:
		params := curveParams(curve)
		d := new(big.Int).SetBytes(privateKey)
		x, y := params.ScalarBaseMult(privateKey)
		priv := &ecdsa.PrivateKey{PublicKey: ecdsa.PublicKey{Curve: params, X: x, Y: y}, D: d}
		vm := didKeyFromEcdsa(&priv.PublicKey, curve)
		byteLen := (params.Params().BitSize + 7) / 8
		return &Signer{
			curve:              curve,
			VerificationMethod: vm,
			DID:                didOf(vm),
			sign: func(hashData []byte) ([]byte, error) {
				// ecdsa-jcs-2019 signs the suite digest of hashData, matching
				// the verifier and the Rust/TypeScript backends.
				r, s, err := ecdsa.Sign(rand.Reader, priv, digest(curve, hashData))
				if err != nil {
					return nil, err
				}
				out := make([]byte, 2*byteLen)
				r.FillBytes(out[:byteLen])
				s.FillBytes(out[byteLen:])
				return out, nil
			},
		}, nil
	default:
		return nil, fmt.Errorf("unsupported curve %q", curve)
	}
}

// SignOptions tunes a signature.
type SignOptions struct {
	// ProofPurpose defaults to "assertionMethod".
	ProofPurpose string
	// Created is the proof's timestamp; the zero value means "now". It is
	// written in UTC, Z-terminated, with no sub-second field — chrono's form, so
	// the same key over the same document and Created reproduces the Rust
	// crate's proofValue byte for byte.
	Created time.Time
}

func didOf(verificationMethod string) string {
	for i := 0; i < len(verificationMethod); i++ {
		if verificationMethod[i] == '#' {
			return verificationMethod[:i]
		}
	}
	return verificationMethod
}

// Sign returns a copy of doc with a proof over everything else in it. It refuses
// — returning an error — to sign a document whose issuer is absent or is not the
// signer's did:key: such a document could never verify (SPEC §4.7). An existing
// proof is replaced.
func (s *Signer) Sign(doc json.RawMessage, opts SignOptions) (json.RawMessage, error) {
	tree, err := decode(doc)
	if err != nil {
		return nil, fmt.Errorf("proof: document is not JSON: %w", err)
	}
	obj, ok := tree.(map[string]any)
	if !ok {
		return nil, errors.New("proof: document is not a JSON object")
	}

	body := make(map[string]any, len(obj))
	for k, v := range obj {
		if k != "proof" {
			body[k] = v
		}
	}
	issuer, ok := body["issuer"].(string)
	if !ok {
		return nil, errors.New("proof: document carries no in-band issuer to bind the proof to")
	}
	if issuer != s.DID {
		return nil, fmt.Errorf("proof: document issuer %s is not the signer %s", issuer, s.DID)
	}

	created := opts.Created
	if created.IsZero() {
		created = time.Now()
	}
	purpose := opts.ProofPurpose
	if purpose == "" {
		purpose = "assertionMethod"
	}
	config := map[string]any{
		"type":               "DataIntegrityProof",
		"cryptosuite":        string(suiteForCurve(s.curve)),
		"created":            created.UTC().Format(time.RFC3339),
		"verificationMethod": s.VerificationMethod,
		"proofPurpose":       purpose,
	}

	configBytes, err := canonicalize(config)
	if err != nil {
		return nil, fmt.Errorf("proof: canonicalize config: %w", err)
	}
	bodyBytes, err := canonicalize(body)
	if err != nil {
		return nil, fmt.Errorf("proof: canonicalize document: %w", err)
	}
	hashData := append(digest(s.curve, configBytes), digest(s.curve, bodyBytes)...)

	signature, err := s.sign(hashData)
	if err != nil {
		return nil, fmt.Errorf("proof: sign: %w", err)
	}

	proof := make(map[string]any, len(config)+1)
	for k, v := range config {
		proof[k] = v
	}
	proof["proofValue"] = toMultibase(signature)
	body["proof"] = proof

	out, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("proof: marshal signed document: %w", err)
	}
	return out, nil
}
