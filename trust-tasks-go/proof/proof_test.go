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
// future".
func laterClock() func() time.Time {
	return func() time.Time { return time.Date(2026, 10, 1, 0, 0, 0, 0, time.UTC) }
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
	signed, err := signer.Sign(loadFixture(t), SignOptions{Created: created})
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
