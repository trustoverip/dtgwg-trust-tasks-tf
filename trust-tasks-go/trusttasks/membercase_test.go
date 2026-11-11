// Member names are case-sensitive in every runtime but Go's encoding/json, which
// matches them to struct fields without regard to case. These pin that a
// document whose member names differ from the declared ones only by case is
// refused at decode, so the decoded document is the one a proof verifier and a
// schema validator — which read the exact-case members — saw.
package trusttasks_test

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

const caseBase = `"id":"req-1","type":"https://trusttasks.org/spec/acl/grant/0.1",` +
	`"recipient":"did:web:maintainer.example","payload":{"role":"admin"}`

func TestDocumentDecodeRefusesCaseVariantMembers(t *testing.T) {
	for name, extra := range map[string]string{
		"beside the exact member": `"issuer":"did:web:a.example","Issuer":"did:web:b.example"`,
		"alone":                   `"Issuer":"did:web:b.example"`,
		"upper case":              `"issuer":"did:web:a.example","ISSUER":"did:web:b.example"`,
		"unicode fold (long s)":   `"issuer":"did:web:a.example","iſſuer":"did:web:b.example"`,
		"delimiter variant":       `"issuer":"did:web:a.example","issued_at":"2026-01-01T00:00:00Z"`,
		"in the payload":          `"issuer":"did:web:a.example","payload":{"role":"admin","Role":"owner"}`,
		"in the proof": `"issuer":"did:web:a.example","proof":{"type":"DataIntegrityProof",` +
			`"cryptosuite":"eddsa-jcs-2022","verificationMethod":"did:web:a.example#k",` +
			`"created":"2026-01-01T00:00:00Z","proofPurpose":"assertionMethod","proofValue":"z1",` +
			`"ProofValue":"z2"}`,
	} {
		t.Run(name, func(t *testing.T) {
			// A later duplicate "payload" member replaces the base one, as
			// encoding/json reads it.
			data := "{" + caseBase + "," + extra + "}"
			var doc tt.Document[payload]
			err := json.Unmarshal([]byte(data), &doc)
			if err == nil {
				t.Fatalf("decoded %s: issuer=%v payload=%+v", data, deref(doc.Issuer), doc.Payload)
			}
			if !strings.Contains(err.Error(), "case-sensitive") {
				t.Fatalf("unexpected error: %v", err)
			}
		})
	}
}

func TestDocumentDecodeKeepsExactAndUnrelatedMembers(t *testing.T) {
	data := "{" + caseBase + `,"issuer":"did:web:a.example","issuerNote":"kept","x-vendor":1`
	var doc tt.Document[payload]
	if err := json.Unmarshal([]byte(data+"}"), &doc); err != nil {
		t.Fatal(err)
	}
	if deref(doc.Issuer) != "did:web:a.example" || doc.Payload.Role != "admin" {
		t.Fatalf("decoded %+v", doc)
	}
	if _, ok := doc.Extra["issuerNote"]; !ok {
		t.Fatalf("an unrelated member was not preserved: %v", doc.Extra)
	}
}

type caseNested struct {
	Items []struct {
		Name string `json:"name"`
	} `json:"items"`
	ByKey map[string]struct {
		Value string `json:"value"`
	} `json:"byKey"`
	Free map[string]any `json:"free"`
}

func TestCheckMemberNamesFollowsNestedTypes(t *testing.T) {
	ok := `{"items":[{"name":"a"}],"byKey":{"Any":{"value":"v"}},"free":{"Name":"anything"}}`
	if err := tt.CheckMemberNames[caseNested]([]byte(ok)); err != nil {
		t.Fatalf("refused a clean value (map keys and free-form members are not names): %v", err)
	}
	for _, bad := range []string{
		`{"Items":[]}`,
		`{"items":[{"name":"a"},{"NAME":"b"}]}`,
		`{"byKey":{"k":{"Value":"v"}}}`,
	} {
		if err := tt.CheckMemberNames[caseNested]([]byte(bad)); err == nil {
			t.Errorf("accepted %s", bad)
		}
	}
}

// The verify-then-decode path: Received is the document the verifier saw. A
// Received carrying a case-variant issuer is not a document Doc can be a reading
// of, so it is refused rather than vouching for Doc.
func TestReceivedWithACaseVariantMemberIsNotVouchedFor(t *testing.T) {
	clean := "{" + caseBase + `,"issuer":"did:web:org.example","issuedAt":"2026-01-01T00:00:00Z"}`
	var doc tt.Document[payload]
	if err := json.Unmarshal([]byte(clean), &doc); err != nil {
		t.Fatal(err)
	}
	doc.Proof = testProof
	received := strings.TrimSuffix(clean, "}") + `,"Issuer":"did:web:someone-else.example"}`
	outcome, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          requiredSpec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofVerify, Verifier: alwaysValid},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.NotConsequentialChecks(),
		Doc:           &doc,
		Received:      json.RawMessage(received),
		MyVID:         me,
		Now:           testNow,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(context.Context, *tt.Document[payload], tt.ResolvedParties) (*tt.Document[response], error) {
			t.Error("handler must not run")
			return nil, nil
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	wantCode(t, outcome, tt.CodeProofInvalid)
}

func deref(s *string) string {
	if s == nil {
		return "<nil>"
	}
	return *s
}
