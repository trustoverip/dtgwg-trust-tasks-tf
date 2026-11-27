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
	// Names differing other than by case — a delimiter included — are ordinary
	// unrecognised members, kept as the other runtimes keep them.
	data := "{" + caseBase + `,"issuer":"did:web:a.example","issuerNote":"kept","x-vendor":1,"thread_id":"t"`
	var doc tt.Document[payload]
	if err := json.Unmarshal([]byte(data+"}"), &doc); err != nil {
		t.Fatal(err)
	}
	if deref(doc.Issuer) != "did:web:a.example" || doc.Payload.Role != "admin" {
		t.Fatalf("decoded %+v", doc)
	}
	for _, name := range []string{"issuerNote", "x-vendor", "thread_id"} {
		if _, ok := doc.Extra[name]; !ok {
			t.Fatalf("unrelated member %q was not preserved: %v", name, doc.Extra)
		}
	}
	if doc.ThreadID != nil {
		t.Fatalf("thread_id was read as threadId")
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

type caseInner struct {
	Name string `json:"name"`
}

type caseOuter struct {
	caseInner
	Name int `json:"name"`
}

// A shallower field wins a name over a promoted one, as in encoding/json, so the
// walk follows the shallower field's type.
func TestCheckMemberNamesFollowsTheShallowestField(t *testing.T) {
	if err := tt.CheckMemberNames[caseOuter]([]byte(`{"name":1}`)); err != nil {
		t.Fatal(err)
	}
	if err := tt.CheckMemberNames[caseOuter]([]byte(`{"Name":1}`)); err == nil {
		t.Fatal("accepted a case variant of a promoted name")
	}
}

type caseStep struct {
	X string `json:"x"`
}

// caseNullable carries an optional nullable object, slice of objects and map
// of objects as generated payload types do: Nullable members tagged omitzero.
// Ptr is the same object behind a pointer, the spelling Nullable replaced.
type caseNullable struct {
	Ptr   *caseStep                            `json:"ptr,omitempty"`
	Step  tt.Nullable[caseStep]                `json:"step,omitzero"`
	Steps tt.Nullable[[]caseStep]              `json:"steps,omitzero"`
	ByKey tt.Nullable[map[string]caseStep]     `json:"byKey,omitzero"`
	Deep  tt.Nullable[[]tt.Nullable[caseStep]] `json:"deep,omitzero"`
	Boxed *tt.Nullable[caseStep]               `json:"boxed,omitzero"`
	Inner tt.Nullable[map[string][]*caseStep]  `json:"inner,omitzero"`
}

// Nullable decodes itself, but through a plain encoding/json decode of T that
// matches names without regard to case, so the check must enter it: a member
// that differs from a declared one only by case is refused inside a Nullable
// exactly as it is behind a pointer.
func TestCheckMemberNamesEntersNullable(t *testing.T) {
	for name, data := range map[string]string{
		"pointer (the baseline)": `{"ptr":{"x":"a","X":"b"}}`,
		"struct":                 `{"step":{"x":"a","X":"b"}}`,
		"slice of structs":       `{"steps":[{"x":"a"},{"x":"a","X":"b"}]}`,
		"map of structs":         `{"byKey":{"k":{"x":"a","X":"b"}}}`,
		"case variant alone":     `{"step":{"X":"b"}}`,
		"nested in a slice":      `{"deep":[null,{"x":"a","X":"b"}]}`,
		"behind a pointer":       `{"boxed":{"x":"a","X":"b"}}`,
		"map of slices":          `{"inner":{"k":[null,{"x":"a","X":"b"}]}}`,
	} {
		t.Run(name, func(t *testing.T) {
			err := tt.CheckMemberNames[caseNullable]([]byte(data))
			if err == nil || !strings.Contains(err.Error(), "case-sensitive") {
				t.Fatalf("CheckMemberNames(%s) = %v, want a case-sensitive refusal", data, err)
			}
		})
	}
}

// Null, absent and exact-case members inside a Nullable pass, and a Nullable
// over a type with no struct inside it has nothing to check.
func TestCheckMemberNamesAcceptsNullableMembers(t *testing.T) {
	for _, data := range []string{
		`{}`,
		`{"step":null,"steps":null,"byKey":null,"deep":null,"boxed":null,"inner":null}`,
		`{"step":{"x":"a"},"steps":[{"x":"a"}],"byKey":{"K":{"x":"a"}},"deep":[null,{"x":"a"}]}`,
		`{"step":{"x":"a","y":"unrelated"}}`,
	} {
		if err := tt.CheckMemberNames[caseNullable]([]byte(data)); err != nil {
			t.Errorf("CheckMemberNames(%s) = %v", data, err)
		}
	}
	if err := tt.CheckMemberNames[tt.Nullable[map[string]any]]([]byte(`{"x":1,"X":2}`)); err != nil {
		t.Errorf("a free-form map inside a Nullable was checked: %v", err)
	}
}
