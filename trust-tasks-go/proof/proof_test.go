package proof

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"os"
	"reflect"
	"testing"
	"time"
)

// A clock after the fixtures' created (2026-09) so proofs are not "in the
// future". It never runs behind the real clock either: the sign-then-verify
// tests stamp proof.created with time.Now(), and a fixed instant would put
// every fresh proof in the future once real time passed it (which it did on
// 2026-10-01).
func laterClock() func() time.Time {
	floor := time.Date(2026, 10, 1, 0, 0, 0, 0, time.UTC)
	return func() time.Time {
		if now := time.Now().UTC(); now.After(floor) {
			return now
		}
		return floor
	}
}

func seededSigner(t *testing.T, curve Curve, seed byte) *Signer {
	t.Helper()
	n := 32
	if curve == P384 {
		n = 48
	}
	key := bytes.Repeat([]byte{seed}, n)
	if curve != Ed25519 {
		key[0] = 1 // keep the p-curve scalar comfortably in range
	}
	s, err := SignerFromPrivateKey(curve, key)
	if err != nil {
		t.Fatalf("SignerFromPrivateKey(%s): %v", curve, err)
	}
	return s
}

func grant(t *testing.T, issuer, role string) json.RawMessage {
	t.Helper()
	doc := map[string]any{
		"id":        "urn:uuid:9b2c1e34-0000-4000-8000-000000000001",
		"type":      "https://trusttasks.org/spec/acl/grant/0.1",
		"issuer":    issuer,
		"recipient": "did:web:maintainer.example",
		"issuedAt":  "2026-01-01T00:00:00Z",
		"payload":   map[string]any{"entry": map[string]any{"subject": "did:web:alice.example", "role": role}},
	}
	b, err := json.Marshal(doc)
	if err != nil {
		t.Fatal(err)
	}
	return b
}

func kindOf(t *testing.T, err error) FailureKind {
	t.Helper()
	var pe *Error
	if !errors.As(err, &pe) {
		t.Fatalf("error %v is not a *proof.Error", err)
	}
	return pe.Kind
}

func loadFixture(t *testing.T) json.RawMessage {
	t.Helper()
	b, err := os.ReadFile("testdata/eddsa-jcs-2022.fixture.json")
	if err != nil {
		t.Fatal(err)
	}
	return b
}

func TestInteropVerifiesRustFixture(t *testing.T) {
	v := NewVerifier(WithClock(laterClock()))
	if err := v.Verify(context.Background(), loadFixture(t)); err != nil {
		t.Fatalf("verify Rust fixture: %v", err)
	}
}

func TestInteropRejectsTamperedRustFixture(t *testing.T) {
	var doc map[string]any
	if err := json.Unmarshal(loadFixture(t), &doc); err != nil {
		t.Fatal(err)
	}
	doc["payload"] = map[string]any{"entry": map[string]any{"role": "owner", "subject": "did:web:alice.example"}}
	b, _ := json.Marshal(doc)
	v := NewVerifier(WithClock(laterClock()))
	if got := kindOf(t, v.Verify(context.Background(), b)); got != SignatureInvalid {
		t.Errorf("tampered fixture: kind = %s, want signatureInvalid", got)
	}
}

// ecdsa-jcs-2019.fixture.json is a copy of trust-tasks-ts-proof's: `ts-*`
// signed by the TypeScript backend, `go-*` by this one, picked with a high S
// (Go's ecdsa.Sign does not normalise S, so every verifier must accept it).
func TestInteropVerifiesEcdsaFixtures(t *testing.T) {
	b, err := os.ReadFile("testdata/ecdsa-jcs-2019.fixture.json")
	if err != nil {
		t.Fatal(err)
	}
	var fixtures map[string]json.RawMessage
	if err := json.Unmarshal(b, &fixtures); err != nil {
		t.Fatal(err)
	}
	v := NewVerifier(WithClock(laterClock()))
	for name, doc := range fixtures {
		t.Run(name, func(t *testing.T) {
			if err := v.Verify(context.Background(), doc); err != nil {
				t.Errorf("verify %s fixture: %v", name, err)
			}
		})
	}
}

func TestInteropReproducesRustSignatureByteForByte(t *testing.T) {
	// Ed25519 is deterministic; the fixture was signed with seed [7; 32].
	signer, err := SignerFromPrivateKey(Ed25519, bytes.Repeat([]byte{7}, 32))
	if err != nil {
		t.Fatal(err)
	}
	var fixture map[string]any
	if err := json.Unmarshal(loadFixture(t), &fixture); err != nil {
		t.Fatal(err)
	}
	if signer.DID != fixture["issuer"] {
		t.Fatalf("signer DID %s != fixture issuer %v", signer.DID, fixture["issuer"])
	}
	rustProof := fixture["proof"].(map[string]any)
	created, err := time.Parse(time.RFC3339, rustProof["created"].(string))
	if err != nil {
		t.Fatal(err)
	}
	signed, err := signer.Sign(loadFixture(t), SignOptions{Created: created, ProofPurpose: rustProof["proofPurpose"].(string)})
	if err != nil {
		t.Fatal(err)
	}
	var out map[string]any
	if err := json.Unmarshal(signed, &out); err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(out["proof"], rustProof) {
		t.Errorf("reproduced proof does not match the Rust fixture\n got: %#v\nwant: %#v", out["proof"], rustProof)
	}
}

func TestSignThenVerifyEachCurve(t *testing.T) {
	v := NewVerifier(WithClock(laterClock()))
	for _, curve := range []Curve{Ed25519, P256, P384} {
		t.Run(string(curve), func(t *testing.T) {
			signer := seededSigner(t, curve, 1)
			signed, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{})
			if err != nil {
				t.Fatal(err)
			}
			if err := v.Verify(context.Background(), signed); err != nil {
				t.Fatalf("verify: %v", err)
			}
			var out map[string]any
			_ = json.Unmarshal(signed, &out)
			suite := out["proof"].(map[string]any)["cryptosuite"]
			want := "ecdsa-jcs-2019"
			if curve == Ed25519 {
				want = "eddsa-jcs-2022"
			}
			if suite != want {
				t.Errorf("cryptosuite = %v, want %s", suite, want)
			}
		})
	}
}

func TestTamperedPayloadNoLongerVerifies(t *testing.T) {
	v := NewVerifier(WithClock(laterClock()))
	for _, curve := range []Curve{Ed25519, P256, P384} {
		t.Run(string(curve), func(t *testing.T) {
			signer := seededSigner(t, curve, 1)
			signed, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{})
			if err != nil {
				t.Fatal(err)
			}
			var doc map[string]any
			_ = json.Unmarshal(signed, &doc)
			doc["payload"] = map[string]any{"entry": map[string]any{"subject": "did:web:alice.example", "role": "owner"}}
			b, _ := json.Marshal(doc)
			if got := kindOf(t, v.Verify(context.Background(), b)); got != SignatureInvalid {
				t.Errorf("kind = %s, want signatureInvalid", got)
			}
		})
	}
}

func TestIssuerBinding(t *testing.T) {
	v := NewVerifier(WithClock(laterClock()))

	t.Run("valid signature under a different issuer", func(t *testing.T) {
		attacker := seededSigner(t, Ed25519, 2)
		victim := seededSigner(t, Ed25519, 3)
		signed, err := attacker.Sign(grant(t, attacker.DID, "admin"), SignOptions{})
		if err != nil {
			t.Fatal(err)
		}
		var doc map[string]any
		_ = json.Unmarshal(signed, &doc)
		doc["issuer"] = victim.DID // claim to be the victim
		b, _ := json.Marshal(doc)
		if got := kindOf(t, v.Verify(context.Background(), b)); got != IssuerMismatch {
			t.Errorf("kind = %s, want issuerMismatch", got)
		}
	})

	t.Run("proof with no issuer", func(t *testing.T) {
		signer := seededSigner(t, Ed25519, 1)
		signed, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{})
		if err != nil {
			t.Fatal(err)
		}
		var doc map[string]any
		_ = json.Unmarshal(signed, &doc)
		delete(doc, "issuer")
		b, _ := json.Marshal(doc)
		if got := kindOf(t, v.Verify(context.Background(), b)); got != IssuerMismatch {
			t.Errorf("kind = %s, want issuerMismatch", got)
		}
	})
}

func TestMalformedProofs(t *testing.T) {
	v := NewVerifier(WithClock(laterClock()))
	signer := seededSigner(t, Ed25519, 1)
	base, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{})
	if err != nil {
		t.Fatal(err)
	}
	mutations := []func(p map[string]any){
		func(p map[string]any) { p["type"] = "Ed25519Signature2020" },
		func(p map[string]any) { delete(p, "cryptosuite") },
		func(p map[string]any) { delete(p, "verificationMethod") },
		func(p map[string]any) { delete(p, "proofPurpose") },
		func(p map[string]any) { delete(p, "proofValue") },
		func(p map[string]any) { p["created"] = "2026-01-01T00:00:00" }, // no zone
	}
	for i, mutate := range mutations {
		var doc map[string]any
		_ = json.Unmarshal(base, &doc)
		mutate(doc["proof"].(map[string]any))
		b, _ := json.Marshal(doc)
		if got := kindOf(t, v.Verify(context.Background(), b)); got != MalformedProof {
			t.Errorf("mutation %d: kind = %s, want malformedProof", i, got)
		}
	}
}

func TestUnsupportedCryptosuite(t *testing.T) {
	edOnly := NewVerifier(WithCryptosuites(EddsaJcs2022), WithClock(laterClock()))
	signer := seededSigner(t, P256, 1)
	signed, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{})
	if err != nil {
		t.Fatal(err)
	}
	if got := kindOf(t, edOnly.Verify(context.Background(), signed)); got != UnsupportedCryptosuite {
		t.Errorf("kind = %s, want unsupportedCryptosuite", got)
	}
}

func TestCreatedIsUTCZNoZeroFraction(t *testing.T) {
	signer := seededSigner(t, Ed25519, 1)
	signed, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{Created: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)})
	if err != nil {
		t.Fatal(err)
	}
	var doc map[string]any
	_ = json.Unmarshal(signed, &doc)
	if got := doc["proof"].(map[string]any)["created"]; got != "2026-01-01T00:00:00Z" {
		t.Errorf("created = %v, want 2026-01-01T00:00:00Z", got)
	}
}

func TestFutureCreatedRefused(t *testing.T) {
	clock := func() time.Time { return time.Date(2026, 6, 1, 12, 0, 0, 0, time.UTC) }
	strict := NewVerifier(WithClock(clock))
	signer := seededSigner(t, Ed25519, 1)
	signed, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{Created: clock().Add(5 * time.Minute)})
	if err != nil {
		t.Fatal(err)
	}
	if got := kindOf(t, strict.Verify(context.Background(), signed)); got != MalformedProof {
		t.Errorf("kind = %s, want malformedProof", got)
	}
}

func TestSigningRefusesDocumentThatCouldNeverVerify(t *testing.T) {
	signer := seededSigner(t, Ed25519, 1)
	if _, err := signer.Sign(grant(t, "did:web:someone-else.example", "admin"), SignOptions{}); err == nil {
		t.Error("signing a document whose issuer is not the signer should fail")
	}
	noIssuer := map[string]any{"id": "urn:uuid:x", "type": "t", "payload": map[string]any{}}
	b, _ := json.Marshal(noIssuer)
	if _, err := signer.Sign(b, SignOptions{}); err == nil {
		t.Error("signing a document with no issuer should fail")
	}
}

// TestSharedProofVectors runs the registry's shared proof vectors
// (fixtures/proof at the repository root), which every library's verifier
// runs, so a document one implementation signs is checked by all four. It
// includes a document whose timestamps a typed parser would re-spell (SPEC
// §4.7, "What a consumer verifies"): it must verify, and its re-spelled twin
// must not.
func TestSharedProofVectors(t *testing.T) {
	const dir = "../../fixtures/proof/"
	b, err := os.ReadFile(dir + "manifest.json")
	if err != nil {
		t.Fatal(err)
	}
	var manifest struct {
		Entries []struct {
			File        string `json:"file"`
			Cryptosuite string `json:"cryptosuite"`
			Expect      string `json:"expect"`
		} `json:"entries"`
	}
	if err := json.Unmarshal(b, &manifest); err != nil {
		t.Fatal(err)
	}
	// Every suite this verifier implements; an entry for any other suite fails
	// the test rather than being skipped silently.
	supported := map[string]bool{"eddsa-jcs-2022": true, "ecdsa-jcs-2019": true}
	v := NewVerifier(WithClock(laterClock()))
	for _, e := range manifest.Entries {
		t.Run(e.File, func(t *testing.T) {
			if !supported[e.Cryptosuite] {
				t.Fatalf("no support for %s; list it as unsupported here if that is intended", e.Cryptosuite)
			}
			doc, err := os.ReadFile(dir + e.File)
			if err != nil {
				t.Fatal(err)
			}
			err = v.Verify(context.Background(), doc)
			switch e.Expect {
			case "valid":
				if err != nil {
					t.Errorf("expected valid, got %v", err)
				}
			case "invalid":
				if err == nil {
					t.Error("expected invalid, but it verified")
				}
			default:
				t.Fatalf("unknown expect %q", e.Expect)
			}
		})
	}
}

// A signed document carrying a member that differs from a framework member only
// by case is refused: encoding/json reads "Issuer" as issuer, so a Go consumer
// would decode a different issuer from the one the proof is bound to.
func TestCaseVariantMembersAreRefused(t *testing.T) {
	v := NewVerifier(WithClock(laterClock()))
	signer := seededSigner(t, Ed25519, 2)
	other := seededSigner(t, Ed25519, 3)
	for name, add := range map[string]func(doc map[string]any){
		"envelope": func(doc map[string]any) { doc["Issuer"] = other.DID },
		"envelope, folded rune": func(doc map[string]any) {
			doc["iſſuer"] = other.DID
		},
	} {
		t.Run(name, func(t *testing.T) {
			var doc map[string]any
			_ = json.Unmarshal(grant(t, signer.DID, "admin"), &doc)
			add(doc)
			unsigned, _ := json.Marshal(doc)
			signed, err := signer.Sign(unsigned, SignOptions{})
			if err != nil {
				t.Fatal(err)
			}
			if got := kindOf(t, v.Verify(context.Background(), signed)); got != MalformedProof {
				t.Errorf("kind = %s, want malformedProof", got)
			}
		})
	}

	t.Run("proof", func(t *testing.T) {
		signed, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{})
		if err != nil {
			t.Fatal(err)
		}
		var doc map[string]any
		_ = json.Unmarshal(signed, &doc)
		doc["proof"].(map[string]any)["VerificationMethod"] = other.DID + "#k"
		b, _ := json.Marshal(doc)
		if got := kindOf(t, v.Verify(context.Background(), b)); got != MalformedProof {
			t.Errorf("kind = %s, want malformedProof", got)
		}
	})

	t.Run("an unrelated member still verifies", func(t *testing.T) {
		var doc map[string]any
		_ = json.Unmarshal(grant(t, signer.DID, "admin"), &doc)
		doc["issuerNote"] = "kept"
		unsigned, _ := json.Marshal(doc)
		signed, err := signer.Sign(unsigned, SignOptions{})
		if err != nil {
			t.Fatal(err)
		}
		if err := v.Verify(context.Background(), signed); err != nil {
			t.Fatalf("verify: %v", err)
		}
	})
}

// signWithConfig signs doc over exactly the proof configuration given, so a
// test can produce a correctly signed proof carrying members Sign never writes
// (expires) or values it refuses (a non-signing proofPurpose).
func signWithConfig(t *testing.T, signer *Signer, doc json.RawMessage, config map[string]any) json.RawMessage {
	t.Helper()
	var body map[string]any
	if err := json.Unmarshal(doc, &body); err != nil {
		t.Fatal(err)
	}
	config["type"] = "DataIntegrityProof"
	config["cryptosuite"] = string(suiteForCurve(signer.curve))
	config["verificationMethod"] = signer.VerificationMethod
	hashData, err := hashDataFor(signer.curve, config, body)
	if err != nil {
		t.Fatal(err)
	}
	signature, err := signer.sign(hashData)
	if err != nil {
		t.Fatal(err)
	}
	config["proofValue"] = toMultibase(signature)
	body["proof"] = config
	out, err := json.Marshal(body)
	if err != nil {
		t.Fatal(err)
	}
	return out
}

func TestProofPurposeMustNameASigningRelationship(t *testing.T) {
	v := NewVerifier(WithClock(laterClock()))
	signer := seededSigner(t, Ed25519, 1)
	for _, purpose := range SigningPurposes {
		t.Run(purpose, func(t *testing.T) {
			signed := signWithConfig(t, signer, grant(t, signer.DID, "admin"), map[string]any{
				"created": "2026-01-01T00:00:00Z", "proofPurpose": purpose,
			})
			if err := v.Verify(context.Background(), signed); err != nil {
				t.Errorf("verify: %v", err)
			}
		})
	}
	for _, purpose := range []string{"keyAgreement", "assertion", "AUTHENTICATION", "authentication "} {
		t.Run(purpose, func(t *testing.T) {
			signed := signWithConfig(t, signer, grant(t, signer.DID, "admin"), map[string]any{
				"created": "2026-01-01T00:00:00Z", "proofPurpose": purpose,
			})
			if got := kindOf(t, v.Verify(context.Background(), signed)); got != MalformedProof {
				t.Errorf("kind = %s, want malformedProof", got)
			}
		})
	}
}

func TestSignDefaultsToAuthenticationAndRefusesNonSigningPurposes(t *testing.T) {
	signer := seededSigner(t, Ed25519, 1)
	signed, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{})
	if err != nil {
		t.Fatal(err)
	}
	var doc map[string]any
	_ = json.Unmarshal(signed, &doc)
	if got := doc["proof"].(map[string]any)["proofPurpose"]; got != "authentication" {
		t.Errorf("default proofPurpose = %v, want authentication", got)
	}
	for _, purpose := range []string{"keyAgreement", "assertion"} {
		if _, err := signer.Sign(grant(t, signer.DID, "admin"), SignOptions{ProofPurpose: purpose}); err == nil {
			t.Errorf("signing with proofPurpose %q should fail", purpose)
		}
	}
}

func TestExpires(t *testing.T) {
	now := time.Date(2026, 6, 1, 12, 0, 0, 0, time.UTC)
	v := NewVerifier(WithClock(func() time.Time { return now }))
	signer := seededSigner(t, Ed25519, 1)
	sign := func(expires any) json.RawMessage {
		return signWithConfig(t, signer, grant(t, signer.DID, "admin"), map[string]any{
			"created": "2026-06-01T11:00:00Z", "proofPurpose": "authentication", "expires": expires,
		})
	}

	if err := v.Verify(context.Background(), sign("2026-06-01T12:00:01Z")); err != nil {
		t.Errorf("an unexpired proof: %v", err)
	}
	if err := v.Verify(context.Background(), sign("2026-06-01T12:00:00Z")); err != nil {
		t.Errorf("a proof at its expires instant: %v", err)
	}
	for name, expires := range map[string]any{
		"passed":           "2026-06-01T11:59:59Z",
		"passed, offset":   "2026-06-01T13:59:59+02:00",
		"no zone":          "2027-01-01T00:00:00",
		"not a string":     float64(1767225600),
		"not a date":       "tomorrow",
		"date only":        "2027-01-01",
		"impossible month": "2027-13-01T00:00:00Z",
		"empty":            "",
	} {
		t.Run(name, func(t *testing.T) {
			if got := kindOf(t, v.Verify(context.Background(), sign(expires))); got != MalformedProof {
				t.Errorf("kind = %s, want malformedProof", got)
			}
		})
	}
}
