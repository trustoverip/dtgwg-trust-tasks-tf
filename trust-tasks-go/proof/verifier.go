package proof

import (
	"context"
	"crypto/ecdsa"
	"crypto/ed25519"
	"crypto/sha256"
	"crypto/sha512"
	"encoding/json"
	"fmt"
	"math/big"
	"regexp"
	"strings"
	"time"

	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// Verifier satisfies the core module's ProofVerifier seam.
var _ trusttasks.ProofVerifier = (*Verifier)(nil)

// FailureKind is why a proof did not verify — the same taxonomy the Rust and
// TypeScript proof libraries use, so a failure logged by any of them reads the
// same. Every kind reaches the wire as the single code proofInvalid; the kind
// and the detail on [Error] are for your logs only (SPEC §12.4 — a verifier's
// vocabulary names DIDs it resolved and what it found, an oracle for an
// unauthenticated sender).
type FailureKind string

const (
	// MalformedProof: the proof object is structurally wrong, names the wrong
	// type, or its created is not an acceptable dateTimeStamp.
	MalformedProof FailureKind = "malformedProof"
	// IssuerMismatch: the signature's key is not the document's in-band issuer,
	// or there is no issuer to bind it to. A valid signature proves only that
	// some key signed.
	IssuerMismatch FailureKind = "issuerMismatch"
	// UnsupportedCryptosuite: a suite this verifier is not configured for.
	UnsupportedCryptosuite FailureKind = "unsupportedCryptosuite"
	// SignatureInvalid: the signature does not verify over the document.
	SignatureInvalid FailureKind = "signatureInvalid"
)

// Error is the typed failure [Verifier.Verify] returns. Recover the kind with
// errors.As.
type Error struct {
	Kind   FailureKind
	Detail string
}

func (e *Error) Error() string { return fmt.Sprintf("proof %s: %s", e.Kind, e.Detail) }

func failf(kind FailureKind, format string, args ...any) *Error {
	return &Error{Kind: kind, Detail: fmt.Sprintf(format, args...)}
}

// DefaultClockSkew is how far into the verifier's future a proof's created may
// sit before it is refused.
const DefaultClockSkew = time.Minute

// Verifier is a ProofVerifier for W3C Data Integrity proofs, on the Go standard
// library. It verifies eddsa-jcs-2022 and ecdsa-jcs-2019 over the document minus
// its proof member, and binds the proof to the document's issuer: a valid
// signature proves only that some key signed, so that key's did:key must also be
// the issuer (SPEC §4.7, §7.2 item 7). did:key only — the key travels in the
// identifier, so there is no network and no resolver.
//
// The zero value is not usable; construct one with [NewVerifier].
type Verifier struct {
	suites    map[Cryptosuite]bool
	clockSkew time.Duration
	now       func() time.Time
}

// Option configures a [Verifier].
type Option func(*Verifier)

// WithCryptosuites restricts the accepted cryptosuites to the given subset.
// An empty or omitted list accepts every suite this package supports.
func WithCryptosuites(suites ...Cryptosuite) Option {
	return func(v *Verifier) {
		v.suites = map[Cryptosuite]bool{}
		for _, s := range suites {
			v.suites[s] = true
		}
	}
}

// WithClockSkew sets the allowance applied to a proof's created. Defaults to
// [DefaultClockSkew].
func WithClockSkew(d time.Duration) Option {
	return func(v *Verifier) { v.clockSkew = d }
}

// WithClock sets the clock, for testing. Defaults to time.Now.
func WithClock(now func() time.Time) Option {
	return func(v *Verifier) { v.now = now }
}

// NewVerifier builds a [Verifier]. With no options it accepts both supported
// cryptosuites, allows [DefaultClockSkew] of future dating, and reads time.Now.
func NewVerifier(opts ...Option) *Verifier {
	v := &Verifier{
		suites:    map[Cryptosuite]bool{EddsaJcs2022: true, EcdsaJcs2019: true},
		clockSkew: DefaultClockSkew,
		now:       time.Now,
	}
	for _, opt := range opts {
		opt(v)
	}
	return v
}

// dateTimeStamp matches an XSD dateTimeStamp: a date-time with an explicit zone
// designator (Z or ±hh:mm). An offset-less value is refused, not read as local
// time — as the Rust and TypeScript verifiers refuse it.
var dateTimeStamp = regexp.MustCompile(`^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$`)

// Verify implements trusttasks.ProofVerifier. It returns nil when doc's proof
// verifies against the in-band issuer, and a typed *[Error] otherwise. It never
// panics on malformed input.
func (v *Verifier) Verify(_ context.Context, doc json.RawMessage) error {
	tree, err := decode(doc)
	if err != nil {
		return failf(MalformedProof, "document is not JSON: %v", err)
	}
	obj, ok := tree.(map[string]any)
	if !ok {
		return failf(MalformedProof, "document is not a JSON object")
	}

	rawProof, present := obj["proof"]
	if !present || rawProof == nil {
		return failf(MalformedProof, "document carries no proof member")
	}
	proof, ok := rawProof.(map[string]any)
	if !ok {
		return failf(MalformedProof, "proof is not an object")
	}
	if s, _ := proof["type"].(string); s != "DataIntegrityProof" {
		return failf(MalformedProof, "proof type must be DataIntegrityProof")
	}
	cryptosuite, err := requireString(proof, "cryptosuite")
	if err != nil {
		return err
	}
	verificationMethod, err := requireString(proof, "verificationMethod")
	if err != nil {
		return err
	}
	if _, err := requireString(proof, "proofPurpose"); err != nil {
		return err
	}
	proofValue, err := requireString(proof, "proofValue")
	if err != nil {
		return err
	}

	suite := Cryptosuite(cryptosuite)
	if !v.suites[suite] {
		return failf(UnsupportedCryptosuite, "%s", cryptosuite)
	}

	// created, when present, is a dateTimeStamp within the skew allowance.
	if created, hasCreated := proof["created"]; hasCreated {
		s, ok := created.(string)
		if !ok || !dateTimeStamp.MatchString(s) {
			return failf(MalformedProof, "proof.created must be a dateTimeStamp with Z or an offset: %v", created)
		}
		t, perr := time.Parse(time.RFC3339, s)
		if perr != nil {
			return failf(MalformedProof, "proof.created does not parse: %v", perr)
		}
		if t.After(v.now().Add(v.clockSkew)) {
			return failf(MalformedProof, "proof.created is more than %s in the future", v.clockSkew)
		}
	}

	// Bind the proof to the in-band issuer (exact string, no normalisation —
	// SPEC §4.8).
	issuer, ok := obj["issuer"].(string)
	if !ok {
		return failf(IssuerMismatch, "document carries a proof but no in-band issuer to bind it to")
	}
	vmDid := verificationMethod
	if i := strings.IndexByte(vmDid, '#'); i >= 0 {
		vmDid = vmDid[:i]
	}
	if vmDid != issuer {
		return failf(IssuerMismatch, "verificationMethod is controlled by %s, not the document issuer %s", vmDid, issuer)
	}

	key, err := publicKeyFromDidKey(verificationMethod)
	if err != nil {
		return failf(SignatureInvalid, "verificationMethod is not a did:key this verifier supports: %v", err)
	}
	if err := suiteMatchesCurve(suite, key.curve); err != nil {
		return err
	}

	signature, err := fromMultibase(proofValue)
	if err != nil {
		return failf(SignatureInvalid, "proofValue is not base58btc multibase: %v", err)
	}

	hashData, err := hashDataFor(key.curve, proof, obj)
	if err != nil {
		return err
	}
	if err := verifySignature(key, hashData, signature); err != nil {
		return err
	}
	return nil
}

// suiteMatchesCurve rejects a suite/curve pairing the cryptosuites forbid.
func suiteMatchesCurve(suite Cryptosuite, curve Curve) error {
	switch suite {
	case EddsaJcs2022:
		if curve != Ed25519 {
			return failf(MalformedProof, "eddsa-jcs-2022 requires an Ed25519 key")
		}
	case EcdsaJcs2019:
		if curve == Ed25519 {
			return failf(MalformedProof, "ecdsa-jcs-2019 requires a P-256 or P-384 key")
		}
	}
	return nil
}

// hashDataFor builds the signing input: the suite digest of the proof
// configuration (the proof minus proofValue) concatenated with the suite digest
// of the document minus its proof, configuration first (VC Data Integrity §3).
func hashDataFor(curve Curve, proof, doc map[string]any) ([]byte, error) {
	config := make(map[string]any, len(proof))
	for k, val := range proof {
		if k != "proofValue" {
			config[k] = val
		}
	}
	body := make(map[string]any, len(doc))
	for k, val := range doc {
		if k != "proof" {
			body[k] = val
		}
	}
	configBytes, err := canonicalize(config)
	if err != nil {
		return nil, failf(MalformedProof, "canonicalize proof config: %v", err)
	}
	bodyBytes, err := canonicalize(body)
	if err != nil {
		return nil, failf(MalformedProof, "canonicalize document: %v", err)
	}
	cfg := digest(curve, configBytes)
	doc2 := digest(curve, bodyBytes)
	return append(cfg, doc2...), nil
}

// digest is the cryptosuite's hash: SHA-384 for P-384, SHA-256 otherwise.
func digest(curve Curve, data []byte) []byte {
	if curve == P384 {
		sum := sha512.Sum384(data)
		return sum[:]
	}
	sum := sha256.Sum256(data)
	return sum[:]
}

func verifySignature(key *publicKey, hashData, signature []byte) error {
	switch key.curve {
	case Ed25519:
		if len(signature) != ed25519.SignatureSize {
			return failf(SignatureInvalid, "ed25519 signature must be %d bytes", ed25519.SignatureSize)
		}
		if !ed25519.Verify(key.ed25519, hashData, signature) {
			return failf(SignatureInvalid, "signature does not verify")
		}
		return nil
	case P256, P384:
		size := (key.ecdsa.Curve.Params().BitSize + 7) / 8
		if len(signature) != 2*size {
			return failf(SignatureInvalid, "ecdsa signature must be %d bytes", 2*size)
		}
		r := new(big.Int).SetBytes(signature[:size])
		s := new(big.Int).SetBytes(signature[size:])
		// ecdsa-jcs-2019 signs the suite digest of hashData (the message hash
		// ECDSA expects), matching the Rust and TypeScript backends.
		if !ecdsa.Verify(key.ecdsa, digest(key.curve, hashData), r, s) {
			return failf(SignatureInvalid, "signature does not verify")
		}
		return nil
	default:
		return failf(SignatureInvalid, "unsupported curve %s", key.curve)
	}
}

func requireString(m map[string]any, member string) (string, error) {
	s, ok := m[member].(string)
	if !ok || s == "" {
		return "", failf(MalformedProof, "proof.%s must be a non-empty string", member)
	}
	return s, nil
}
