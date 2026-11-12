package proof

import (
	"bytes"
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
	"unicode"
	"unicode/utf8"

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

	// The binding below reads the exact-case "issuer" and "proof". A Go consumer
	// decoding the same bytes with encoding/json would match "Issuer" or "PROOF"
	// to those members too, so a document carrying such a variant means one thing
	// here and another to the handler. Refuse it rather than vouch for either —
	// stricter than the Rust, TypeScript and Dart verifiers, which have no such
	// reader to disagree with.
	if name, variant := caseVariantOf(obj, envelopeMembers); variant != "" {
		return failf(MalformedProof, "document member %q differs from %q only by case", variant, name)
	}

	rawProof, present := obj["proof"]
	if !present || rawProof == nil {
		return failf(MalformedProof, "document carries no proof member")
	}
	proof, ok := rawProof.(map[string]any)
	if !ok {
		return failf(MalformedProof, "proof is not an object")
	}
	if name, variant := caseVariantOf(proof, proofMembers); variant != "" {
		return failf(MalformedProof, "proof member %q differs from %q only by case", variant, name)
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
	proofPurpose, err := requireString(proof, "proofPurpose")
	if err != nil {
		return err
	}
	if err := checkProofPurpose(proofPurpose); err != nil {
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
		// Every field in range, as for expires: time.Parse alone takes an
		// offset of +24:00 or +23:60.
		s, ok := created.(string)
		var t time.Time
		if ok {
			t, ok = parseDateTimeStamp(s)
		}
		if !ok {
			return failf(MalformedProof, "proof.created must be a dateTimeStamp with Z or an offset: %v", created)
		}
		if t.After(v.now().Add(v.clockSkew)) {
			return failf(MalformedProof, "proof.created is more than %s in the future", v.clockSkew)
		}
	}

	// expires, when present, is a dateTimeStamp that has not yet passed (VC
	// Data Integrity §2.1). No allowance applies: the signer chose the instant.
	if expires, hasExpires := proof["expires"]; hasExpires {
		s, _ := expires.(string)
		t, ok := parseDateTimeStamp(s)
		if !ok {
			return failf(MalformedProof, "proof.expires must be a dateTimeStamp with Z or an offset: %v", expires)
		}
		if v.now().After(t) {
			return failf(MalformedProof, "proof.expires %s has passed", s)
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

	if err := checkProofContext(obj, proof); err != nil {
		return err
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

// dateTimeStampParts captures each field of a dateTimeStamp for the range
// checks in parseDateTimeStamp.
var dateTimeStampParts = regexp.MustCompile(`^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(\.\d+)?(?:Z|([+-])(\d{2}):(\d{2}))$`)

// parseDateTimeStamp reads s as an XSD dateTimeStamp and refuses every field
// out of range: month 1-12, a day the month has (leap years included), hour
// 0-23, minute and second 0-59 (no leap second), and an offset of at most
// 14:00 with minutes 0-59. The Rust, TypeScript and Dart verifiers apply the
// same ranges, so no runtime accepts an instant another refuses — each
// language's own date parser rolls over or accepts a different subset.
func parseDateTimeStamp(s string) (time.Time, bool) {
	m := dateTimeStampParts.FindStringSubmatch(s)
	if m == nil {
		return time.Time{}, false
	}
	n := func(i int) int {
		v := 0
		for _, c := range m[i] {
			v = v*10 + int(c-'0')
		}
		return v
	}
	year, month, day, hour, minute, second := n(1), n(2), n(3), n(4), n(5), n(6)
	if month < 1 || month > 12 || day < 1 || day > daysIn(year, month) ||
		hour > 23 || minute > 59 || second > 59 {
		return time.Time{}, false
	}
	nanos := 0
	if frac := m[7]; frac != "" {
		digits := frac[1:]
		for i := 0; i < 9; i++ {
			nanos *= 10
			if i < len(digits) {
				nanos += int(digits[i] - '0')
			}
		}
	}
	t := time.Date(year, time.Month(month), day, hour, minute, second, nanos, time.UTC)
	if m[8] != "" {
		oh, om := n(9), n(10)
		if oh > 14 || om > 59 || (oh == 14 && om != 0) {
			return time.Time{}, false
		}
		offset := time.Duration(oh)*time.Hour + time.Duration(om)*time.Minute
		if m[8] == "+" {
			t = t.Add(-offset)
		} else {
			t = t.Add(offset)
		}
	}
	return t, true
}

func daysIn(year, month int) int {
	switch month {
	case 2:
		if year%4 == 0 && (year%100 != 0 || year%400 == 0) {
			return 29
		}
		return 28
	case 4, 6, 9, 11:
		return 30
	}
	return 31
}

// SigningPurposes are the proofPurpose values a proof may carry: the
// verification relationships a signature can be made under. A did:key's key is
// authorised for all four, as the did:key method defines. keyAgreement is
// deliberately absent — a key-agreement key never authorises a signature — and
// so is any value naming no relationship, as in the Rust, TypeScript and Dart
// verifiers.
var SigningPurposes = []string{"authentication", "assertionMethod", "capabilityInvocation", "capabilityDelegation"}

// checkProofPurpose refuses a proofPurpose that names no signing relationship.
func checkProofPurpose(purpose string) error {
	for _, p := range SigningPurposes {
		if purpose == p {
			return nil
		}
	}
	if purpose == "keyAgreement" {
		return failf(MalformedProof, "proofPurpose keyAgreement never authorises a signature")
	}
	return failf(MalformedProof, "proofPurpose names no signing verification relationship")
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

// checkProofContext refuses a proof whose @context disagrees with the
// document's. eddsa-jcs-2022 / ecdsa-jcs-2019 §3.3.2 step 4: "If
// proofOptions.@context exists: Check that the securedDocument.@context starts
// with all values contained in the proofOptions.@context in the same order.
// Otherwise, set verified to false". A proof without @context is hashed as
// received (step 4 does not apply).
//
// This verifier requires the two to be equal, not only the proof's to be a
// prefix. The cryptosuite verifies a longer document @context by hashing the
// proof's in its place (step 4.2) and returning that rewritten document as the
// verified one; a ProofVerifier returns no document, so the caller would go on
// to act on @context entries no signature covers. The Rust, TypeScript and
// Dart verifiers apply the same rule.
func checkProofContext(doc, proof map[string]any) error {
	proofContext, ok := proof["@context"]
	if !ok {
		return nil
	}
	docContext, ok := doc["@context"]
	if !ok {
		return failf(MalformedProof, "proof carries @context but the document has none")
	}
	a, err := canonicalize(proofContext)
	if err != nil {
		return failf(MalformedProof, "canonicalize proof @context: %v", err)
	}
	b, err := canonicalize(docContext)
	if err != nil {
		return failf(MalformedProof, "canonicalize document @context: %v", err)
	}
	if !bytes.Equal(a, b) {
		return failf(MalformedProof, "proof @context does not equal the document's @context")
	}
	return nil
}

// hashDataFor builds the signing input: the suite digest of the proof
// configuration (the proof minus proofValue, as received:
// @context included when it carries one) concatenated with the suite digest
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
		// crypto/ed25519 accepts a non-canonical or small-order key; refuse it
		// here too, whatever built the key (see checkEd25519PublicKey).
		if err := checkEd25519PublicKey(key.ed25519); err != nil {
			return failf(SignatureInvalid, "%v", err)
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

// envelopeMembers are the framework's top-level document members (SPEC §4).
var envelopeMembers = []string{
	"id", "threadId", "parentThreadId", "ceremony", "type", "issuer", "recipient",
	"issuedAt", "expiresAt", "payload", "@context", "proof",
}

// proofMembers are the Data Integrity proof members this verifier reads.
var proofMembers = []string{
	"type", "cryptosuite", "verificationMethod", "created", "expires", "proofPurpose", "proofValue",
}

// caseVariantOf returns a member of obj that is not one of names but folds to
// one — as encoding/json's case-insensitive matching would read it — and the
// name it folds to. '_' and '-' are significant, as they are to encoding/json.
func caseVariantOf(obj map[string]any, names []string) (name, variant string) {
	folded := make(map[string]string, len(names))
	exact := make(map[string]bool, len(names))
	for _, n := range names {
		folded[foldMemberName(n)] = n
		exact[n] = true
	}
	for member := range obj {
		if exact[member] {
			continue
		}
		if n, ok := folded[foldMemberName(member)]; ok {
			return n, member
		}
	}
	return "", ""
}

func foldMemberName(name string) string {
	var out strings.Builder
	for _, r := range name {
		if r < utf8.RuneSelf {
			out.WriteRune(unicode.ToUpper(r))
			continue
		}
		for {
			next := unicode.SimpleFold(r)
			if next <= r {
				r = next
				break
			}
			r = next
		}
		out.WriteRune(r)
	}
	return out.String()
}
