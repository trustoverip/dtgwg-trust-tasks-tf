// SPEC.md §8.3 and §8.5 error-code tests.
//
// Mirror trust-tasks-ts/test/codes.test.ts and the error.rs tests in
// trust-tasks-rs.
package trusttasks_test

import (
	"strings"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

const (
	grant         = "https://trusttasks.org/spec/acl/grant/0.1"
	grantResponse = "https://trusttasks.org/spec/acl/grant/0.1#response"
	didDelete     = "https://trusttasks.org/spec/did-management/did/delete/0.1"
	discovery     = "https://trusttasks.org/spec/trust-task-discovery/0.1"
)

/* ── §8.3 standard codes ─────────────────────────────────────────────────── */

func TestStandardCodesAcceptTheLowerCamelCaseSpellings(t *testing.T) {
	for _, code := range []string{"proofRequired", "identityMismatch", "expired"} {
		if _, ok := tt.IsStandardCode(code); !ok {
			t.Errorf("expected %q to be a standard code", code)
		}
	}
}

func TestStandardCodesAcceptTheFrozenSnakeCaseSpellingsAndNormalizeThem(t *testing.T) {
	// A 0.2 consumer must still read an error response from a 0.1 peer;
	// otherwise proof_required reads as an unrecognized extended code and falls
	// through to taskFailed (§8.5), losing the meaning.
	if _, ok := tt.IsStandardCode("proof_required"); !ok {
		t.Error("expected the frozen 0.1 spelling to be recognized")
	}
	if got := tt.NormalizeCode("proof_required"); got != "proofRequired" {
		t.Errorf("normalizeCode(proof_required) = %q", got)
	}
	if got := tt.NormalizeCode("identity_mismatch"); got != "identityMismatch" {
		t.Errorf("normalizeCode(identity_mismatch) = %q", got)
	}
	// `canceled` is the 0.1 spelling of `cancelled`, not a separate code.
	if got := tt.NormalizeCode("canceled"); got != "cancelled" {
		t.Errorf("normalizeCode(canceled) = %q", got)
	}
}

func TestNormalizeLeavesAnExtendedCodeAlone(t *testing.T) {
	const extended = "acl/grant:roleNotRecognized"
	if got := tt.NormalizeCode(extended); got != extended {
		t.Errorf("normalizeCode rewrote an extended code: %q", got)
	}
	if _, ok := tt.IsStandardCode(extended); ok {
		t.Error("an extended code must not read as a standard one")
	}
}

func TestStandardCodesListsEveryCodeAndCannotBeMutatedByACaller(t *testing.T) {
	first := tt.StandardCodes()
	if len(first) != 14 {
		t.Fatalf("expected the 14 §8.3 codes, got %d", len(first))
	}
	first[0] = "tampered"
	if tt.StandardCodes()[0] != tt.CodeMalformedRequest {
		t.Error("a caller mutated the framework's own code list")
	}
}

/* ── §8.5 extended codes ─────────────────────────────────────────────────── */

func mustExtended(t *testing.T, typeURI, local string) string {
	t.Helper()
	code, err := tt.ExtendedCode(typeURI, local)
	if err != nil {
		t.Fatalf("ExtendedCode(%q, %q): %v", typeURI, local, err)
	}
	return code
}

func TestExtendedCodeSourcesTheNamespaceFromTheTypeURI(t *testing.T) {
	if got := mustExtended(t, grant, "roleNotRecognized"); got != "acl/grant:roleNotRecognized" {
		t.Errorf("got %q", got)
	}
}

func TestExtendedCodeWorksForASingleSegmentSlug(t *testing.T) {
	want := "trust-task-discovery:filterUnsupported"
	if got := mustExtended(t, discovery, "filterUnsupported"); got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestExtendedCodeStripsTheResponseFragment(t *testing.T) {
	// An error raised while handling a response still belongs to the bare slug;
	// namespacing it acl/grant#response would name nothing.
	if got := mustExtended(t, grantResponse, "roleNotRecognized"); got != "acl/grant:roleNotRecognized" {
		t.Errorf("got %q", got)
	}
}

func TestExtendedCodeAcceptsBothCasingsOfTheLocalPart(t *testing.T) {
	if got := mustExtended(t, grant, "documentRevoked"); got != "acl/grant:documentRevoked" {
		t.Errorf("got %q", got)
	}
	if got := mustExtended(t, grant, "document_revoked"); got != "acl/grant:document_revoked" {
		t.Errorf("got %q", got)
	}
}

func TestExtendedCodeRejectsALeadingCapitalInTheLocalPart(t *testing.T) {
	// Only the first character must be lowercase; the resulting code would
	// otherwise fail to round-trip.
	_, err := tt.ExtendedCode(grant, "BadLocal")
	if err == nil || !strings.Contains(err.Error(), "must match") {
		t.Fatalf("expected a must-match error, got %v", err)
	}
}

/* ── §8.5 rule 2 — family namespaces ─────────────────────────────────────── */

func mustFamily(t *testing.T, typeURI, namespace, local string) string {
	t.Helper()
	code, err := tt.FamilyCode(typeURI, namespace, local)
	if err != nil {
		t.Fatalf("FamilyCode(%q, %q, %q): %v", typeURI, namespace, local, err)
	}
	return code
}

func TestFamilyCodeAcceptsEachPathPrefixOfTheSlug(t *testing.T) {
	if got := mustFamily(t, didDelete, "did-management", "unknown_domain"); got != "did-management:unknown_domain" {
		t.Errorf("got %q", got)
	}
	want := "did-management/did:unknown_domain"
	if got := mustFamily(t, didDelete, "did-management/did", "unknown_domain"); got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestFamilyCodeAcceptsTheFullSlug(t *testing.T) {
	// Making it a superset of ExtendedCode.
	want := "did-management/did/delete:notOwner"
	if got := mustFamily(t, didDelete, "did-management/did/delete", "notOwner"); got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestFamilyCodeRejectsNamespacesThatAreNotPrefixes(t *testing.T) {
	cases := []struct {
		name      string
		typeURI   string
		namespace string
	}{
		// A sibling shares a prefix but is not itself one — exactly the confusion
		// §8.5 forbids ("never that of a related or referenced specification").
		{"a sibling's slug", grant, "acl/revoke"},
		{"an unrelated namespace", grant, "vault"},
		// `ac` is a string prefix of `acl/grant` but names nothing.
		{"a partial segment", grant, "ac"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			_, err := tt.FamilyCode(c.typeURI, c.namespace, "somethingElse")
			if err == nil || !strings.Contains(err.Error(), "path prefix") {
				t.Fatalf("expected a path-prefix error, got %v", err)
			}
		})
	}
}

func TestFamilyCodeStripsTheResponseFragmentBeforeCheckingThePrefix(t *testing.T) {
	if got := mustFamily(t, grantResponse, "acl", "permissionDenied"); got != "acl:permissionDenied" {
		t.Errorf("got %q", got)
	}
}

/* ── SlugFromTypeURI ─────────────────────────────────────────────────────── */

func TestSlugFromTypeURIDropsTheVersionSegmentAndAnyFragment(t *testing.T) {
	cases := []struct{ typeURI, want string }{
		{grant, "acl/grant"},
		{grantResponse, "acl/grant"},
		{didDelete, "did-management/did/delete"},
	}
	for _, c := range cases {
		got, err := tt.SlugFromTypeURI(c.typeURI)
		if err != nil {
			t.Fatalf("SlugFromTypeURI(%q): %v", c.typeURI, err)
		}
		if got != c.want {
			t.Errorf("SlugFromTypeURI(%q) = %q, want %q", c.typeURI, got, c.want)
		}
	}
}

func TestSlugFromTypeURIRefusesAURIOutsideTheTrustTasksNamespace(t *testing.T) {
	_, err := tt.SlugFromTypeURI("https://example.com/spec/acl/grant/0.1")
	if err == nil || !strings.Contains(err.Error(), "Type URI") {
		t.Fatalf("expected a Type URI error, got %v", err)
	}
}
