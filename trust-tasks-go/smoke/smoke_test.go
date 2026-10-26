// Smoke test across the seam between the generated packages and the
// hand-written runtime.
//
// The drift check proves the generator was re-run and the runtime tests prove
// the pipeline behaves, but neither imports a generated package. This does: it
// runs a real specification's Payload, Spec and PayloadSchemaJSON through
// ConsumeInbound exactly as a consumer would. It is the Go counterpart of
// `npm run smoke` in trust-tasks-ts, which exists because a package can typecheck
// and still not be importable.
package smoke

import (
	"context"
	"encoding/json"
	"strings"
	"testing"
	"time"

	aclgrantv0_1 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/acl/grant/v0_1"
	authauthenticatev0_1 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/auth/authenticate/v0_1"
	authauthenticatev0_2 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/auth/authenticate/v0_2"
	authauthenticatev0_3 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/auth/authenticate/v0_3"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

const (
	me   = "did:web:maintainer.example"
	peer = "did:web:org.example"
)

func ptr[T any](v T) *T { return &v }

func request() *tt.Document[aclgrantv0_1.Payload] {
	return &tt.Document[aclgrantv0_1.Payload]{
		ID:        "req-1",
		Type:      aclgrantv0_1.TypeURI,
		Issuer:    ptr(peer),
		Recipient: ptr(me),
		IssuedAt:  ptr("2026-01-01T00:00:00Z"),
		Proof: &tt.Proof{
			Type:               "DataIntegrityProof",
			Cryptosuite:        "eddsa-jcs-2022",
			VerificationMethod: peer + "#key-1",
			Created:            "2026-01-01T00:00:00Z",
			ProofPurpose:       "assertionMethod",
			ProofValue:         "z3kg",
		},
		Payload: aclgrantv0_1.Payload{
			Entry: aclgrantv0_1.ACLEntry{Subject: "did:web:alice.example", Role: "admin"},
		},
	}
}

type acceptingVerifier struct{}

func (acceptingVerifier) Verify(context.Context, json.RawMessage) error { return nil }

// TestGeneratedSpecDrivesTheRuntime runs a generated specification end to end.
func TestGeneratedSpecDrivesTheRuntime(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	now := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	clock := func() string { return "2026-01-01T00:00:00Z" }

	consume := func(doc *tt.Document[aclgrantv0_1.Payload]) tt.ConsumeOutcome[aclgrantv0_1.Response] {
		t.Helper()
		outcome, err := tt.ConsumeInbound(
			context.Background(),
			tt.ConsumeOptions[aclgrantv0_1.Payload, aclgrantv0_1.Response]{
				Transport:     tt.StaticTransport{Context: tt.TransportContext{Issuer: ptr(peer), Recipient: ptr(me)}},
				Spec:          aclgrantv0_1.Spec,
				ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofVerify, Verifier: acceptingVerifier{}},
				PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
				Checks:        tt.ConsequentialChecks(guard),
				Doc:           doc,
				MyVID:         me,
				Now:           now,
				NewErrorID:    func() string { return "err-1" },
				Clock:         clock,
				Handler: func(
					_ context.Context,
					accepted *tt.Document[aclgrantv0_1.Payload],
					_ tt.ResolvedParties,
				) (*tt.Document[aclgrantv0_1.Response], error) {
					return tt.RespondWith[aclgrantv0_1.Payload, aclgrantv0_1.Response](
						accepted, "resp-1",
						aclgrantv0_1.Response{Entry: accepted.Payload.Entry},
						clock,
					), nil
				},
			},
		)
		if err != nil {
			t.Fatalf("ConsumeInbound: %v", err)
		}
		return outcome
	}

	outcome := consume(request())
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
	if outcome.Response.Type != aclgrantv0_1.ResponseTypeURI {
		t.Errorf("expected %q, got %q", aclgrantv0_1.ResponseTypeURI, outcome.Response.Type)
	}

	// §7.2 item 11: the identical resend is absorbed, not executed again.
	if again := consume(request()); again.Kind != tt.OutcomeDuplicate {
		t.Errorf("expected the resend absorbed as a duplicate, got %q", again.Kind)
	}
}

// TestGeneratedSchemaIsSelfContained checks that the shipped schema needs no
// resolver — every cross-file $ref was inlined at generation time, which is what
// makes SPEC §7.2 item 2 performable from the package alone.
func TestGeneratedSchemaIsSelfContained(t *testing.T) {
	var schema map[string]any
	if err := json.Unmarshal([]byte(aclgrantv0_1.PayloadSchemaJSON), &schema); err != nil {
		t.Fatalf("PayloadSchemaJSON is not valid JSON: %v", err)
	}
	if _, present := schema["$defs"]; !present {
		t.Error("expected the inlined $defs to travel with the schema")
	}
	var external func(node any) bool
	external = func(node any) bool {
		switch v := node.(type) {
		case map[string]any:
			if ref, ok := v["$ref"].(string); ok && !strings.HasPrefix(ref, "#") {
				return true
			}
			for _, child := range v {
				if external(child) {
					return true
				}
			}
		case []any:
			for _, child := range v {
				if external(child) {
					return true
				}
			}
		}
		return false
	}
	if external(schema) {
		t.Error("the shipped schema still carries a cross-file $ref and cannot be resolved offline")
	}
	if aclgrantv0_1.Spec.PayloadSchema != aclgrantv0_1.PayloadSchemaJSON {
		t.Error("Spec.PayloadSchema does not carry the package's own schema")
	}
}

// TestOptionalArraysDistinguishAbsentFromEmpty pins the representation the
// acl/grant schema demands of `allowedKeys`: "PRESENT-BUT-EMPTY means authorized
// on NO keys — the opposite of absent". A plain []string with omitempty cannot
// say that, which is why optional arrays are generated as pointers.
func TestOptionalArraysDistinguishAbsentFromEmpty(t *testing.T) {
	encode := func(entry aclgrantv0_1.ACLEntry) string {
		t.Helper()
		encoded, err := json.Marshal(entry)
		if err != nil {
			t.Fatalf("marshal: %v", err)
		}
		return string(encoded)
	}

	absent := encode(aclgrantv0_1.ACLEntry{Subject: "did:web:alice.example", Role: "admin"})
	if strings.Contains(absent, "allowedKeys") {
		t.Errorf("an absent member reached the wire: %s", absent)
	}

	empty := encode(aclgrantv0_1.ACLEntry{
		Subject:     "did:web:alice.example",
		Role:        "admin",
		AllowedKeys: &[]string{},
	})
	if !strings.Contains(empty, `"allowedKeys":[]`) {
		t.Errorf("present-but-empty collapsed into absent: %s", empty)
	}
}

// TestExtendingVersionIsReadForTheImplementedOne drives ReadForSpec against a
// real generated package — auth/authenticate/0.1's schema and Payload — with a
// document of 0.3, which adds `principal`, `delegationEvidence` and `sessionKey`.
//
// No specification declares `extends` yet, so the chain 0.3 -> 0.2 -> 0.1 is
// supplied by a stub lookup; RegistryExtends is the same function type over the
// generated index, and the generated constants say what each version declares.
func TestExtendingVersionIsReadForTheImplementedOne(t *testing.T) {
	if authauthenticatev0_3.Extends != tt.RegistryExtends(authauthenticatev0_3.TypeURI) {
		t.Fatalf("generated constant %q disagrees with the generated index %q",
			authauthenticatev0_3.Extends, tt.RegistryExtends(authauthenticatev0_3.TypeURI))
	}
	chain := map[string]string{
		authauthenticatev0_3.TypeURI: "0.2",
		authauthenticatev0_2.TypeURI: "0.1",
	}
	lookup := func(typeURI string) string { return chain[typeURI] }

	received := json.RawMessage(`{
	  "id": "req-1",
	  "type": "` + authauthenticatev0_3.TypeURI + `",
	  "issuer": "` + peer + `",
	  "recipient": "` + me + `",
	  "payload": {
	    "challenge": "0123456789abcdef0123",
	    "sessionId": "s-1",
	    "scope": ["read"],
	    "principal": "` + peer + `",
	    "sessionKey": "z6MkexampleSessionKey"
	  }
	}`)
	reading, refusal, err := tt.ReadForSpecWith(received, authauthenticatev0_1.Spec, lookup)
	if err != nil || refusal != nil {
		t.Fatalf("refused: %v %+v", err, refusal)
	}
	var doc struct {
		Type    string                     `json:"type"`
		Payload map[string]json.RawMessage `json:"payload"`
	}
	if err := json.Unmarshal(reading, &doc); err != nil {
		t.Fatal(err)
	}
	if doc.Type != authauthenticatev0_3.TypeURI {
		t.Errorf("type rewritten to %q", doc.Type)
	}
	for name := range doc.Payload {
		switch name {
		case "challenge", "sessionId", "scope", "ext":
		default:
			t.Errorf("0.1's schema does not know %q, and it survived the prune", name)
		}
	}
	var typed tt.Document[authauthenticatev0_1.Payload]
	if err := json.Unmarshal(reading, &typed); err != nil || typed.Payload.SessionID != "s-1" {
		t.Fatalf("typed decode: %v %+v", err, typed.Payload)
	}

	// Through the generated index: refused while nothing declares the chain,
	// read exactly as above once 0.3 and 0.2 do.
	viaIndex, refusal, _ := tt.ReadForSpec(received, authauthenticatev0_1.Spec)
	if authauthenticatev0_3.Extends == "" {
		if refusal == nil || refusal.Code != tt.CodeUnsupportedVersion {
			t.Fatalf("undeclared extension accepted: %+v", refusal)
		}
	} else if string(viaIndex) != string(reading) {
		t.Fatalf("the generated index reads differently from the declared chain: %s", viaIndex)
	}
}
