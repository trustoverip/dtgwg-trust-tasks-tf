// SPEC.md §6.1 Type URI parsing tests.
//
// Vectors mirror the tests in trust-tasks-rs/src/type_uri.rs.
package trusttasks_test

import (
	"errors"
	"strings"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// Every SPEC §6.1 example parses, names its slug and version, and prints back as
// the same string — with each variant fragment.
func TestEverySpec61FormRoundTrips(t *testing.T) {
	cases := []struct {
		uri, authority, slug string
		urn                  bool
		major, minor         uint32
	}{
		{"https://trusttasks.org/spec/acl/grant/0.1", "https://trusttasks.org", "acl/grant", false, 0, 1},
		{"https://trusttasks.org/spec/trust-task-error/0.5", "https://trusttasks.org", "trust-task-error", false, 0, 5},
		{"https://example.com/trust-tasks/spec/acl/grant/0.1", "https://example.com/trust-tasks", "acl/grant", false, 0, 1},
		{"did:example:123456789abcdefghi/spec/acl/grant/0.1", "did:example:123456789abcdefghi", "acl/grant", false, 0, 1},
		{"did:web:tasks.example.com:x/spec/pay/0.1", "did:web:tasks.example.com:x", "pay", false, 0, 1},
		{"did:web:acme.example/spec/payments/settle/2.0", "did:web:acme.example", "payments/settle", false, 2, 0},
		{"urn:example:tasks:spec:acl:grant:0.1", "urn:example:tasks", "acl/grant", true, 0, 1},
		{"urn:example:tasks:sub:spec:pay:3.10", "urn:example:tasks:sub", "pay", true, 3, 10},
		{"tag:example.com,2026:tasks/spec/acl/grant/0.1", "tag:example.com,2026:tasks", "acl/grant", false, 0, 1},
	}
	variants := map[string]tt.TypeURIVariant{
		"":          tt.VariantNone,
		"#request":  tt.VariantRequest,
		"#response": tt.VariantResponse,
	}
	for _, c := range cases {
		for fragment, variant := range variants {
			s := c.uri + fragment
			u, err := tt.ParseTypeURI(s)
			if err != nil {
				t.Fatalf("%s: %v", s, err)
			}
			if u.Authority() != c.authority || u.IsURN() != c.urn || u.Slug() != c.slug ||
				u.Major() != c.major || u.Minor() != c.minor || u.Variant() != variant {
				t.Errorf("%s: parsed as %#v", s, u)
			}
			if u.String() != s {
				t.Errorf("%s: printed back as %s", s, u)
			}
			if u.Bare().String() != c.uri {
				t.Errorf("%s: bare is %s", s, u.Bare())
			}
			if u.IsResponse() != (variant == tt.VariantResponse) {
				t.Errorf("%s: IsResponse = %v", s, u.IsResponse())
			}
			if u.IsRegistry() != strings.HasPrefix(s, "https://trusttasks.org/spec/") {
				t.Errorf("%s: IsRegistry = %v", s, u.IsRegistry())
			}
		}
	}

	urn, _ := tt.ParseTypeURI("urn:example:tasks:spec:acl:grant:0.1")
	if got := urn.WithVariant(tt.VariantResponse).String(); got != "urn:example:tasks:spec:acl:grant:0.1#response" {
		t.Errorf("WithVariant: %s", got)
	}
	if got := urn.WithVersion(0, 2).String(); got != "urn:example:tasks:spec:acl:grant:0.2" {
		t.Errorf("WithVersion: %s", got)
	}
	if urn.Version() != "0.1" {
		t.Errorf("Version: %s", urn.Version())
	}
}

// A Type URI is a name compared by its whole string: the same slug and version
// under two authorities, or in two forms, are different specifications.
func TestTheAuthorityIsPartOfIdentity(t *testing.T) {
	var uris []tt.TypeURI
	for _, s := range []string{
		"https://trusttasks.org/spec/vault/get/0.3",
		"https://tasks.example.com/spec/vault/get/0.3",
		"urn:example:tasks:spec:vault:get:0.3",
		"did:web:tasks.example.com/spec/vault/get/0.3",
	} {
		u, err := tt.ParseTypeURI(s)
		if err != nil {
			t.Fatal(err)
		}
		uris = append(uris, u)
	}
	for i := range uris {
		for j := range uris {
			if i == j {
				continue
			}
			if uris[i].Slug() != uris[j].Slug() {
				t.Fatalf("slugs differ: %s, %s", uris[i], uris[j])
			}
			if uris[i] == uris[j] || uris[i].SameSpecification(uris[j]) {
				t.Errorf("%s and %s compare as the same specification", uris[i], uris[j])
			}
		}
		if !uris[i].SameSpecification(uris[i].WithVersion(9, 9).WithVariant(tt.VariantResponse)) {
			t.Errorf("%s: another version is not the same specification", uris[i])
		}
	}
}

func TestMalformedTypeURIsAreRefused(t *testing.T) {
	cases := []struct {
		uri  string
		kind tt.TypeURIErrorKind
	}{
		// http is non-conformant in every form (§6.1).
		{"http://trusttasks.org/spec/acl/grant/0.1", tt.TypeURIErrScheme},
		{"HTTP://example.com/spec/acl/grant/0.1", tt.TypeURIErrScheme},
		{"http://example.com/x/spec/acl/grant/0.1#response", tt.TypeURIErrScheme},
		// No scheme, or a malformed one.
		{"trusttasks.org/spec/acl/grant/0.1", tt.TypeURIErrScheme},
		{"1https://example.com/spec/a/0.1", tt.TypeURIErrScheme},
		{"", tt.TypeURIErrScheme},
		// Shape.
		{"https:/spec/a/0.1", tt.TypeURIErrShape},
		{"https:///spec/a/0.1", tt.TypeURIErrShape},
		{"https:example.com/spec/a/0.1", tt.TypeURIErrShape},
		{"did:/spec/a/0.1", tt.TypeURIErrShape},
		{"did:example:123", tt.TypeURIErrShape},
		{"https://trusttasks.org/acl/grant/0.1", tt.TypeURIErrShape},
		// A urn: must use the URN form, with a NID and a prefix, and no '/'.
		{"urn:example:tasks:acl:grant:0.1", tt.TypeURIErrShape},
		{"urn:example:spec:a:0.1", tt.TypeURIErrShape},
		{"urn::tasks:spec:a:0.1", tt.TypeURIErrShape},
		{"urn:example:tasks:spec:acl/grant:0.1", tt.TypeURIErrShape},
		{"urn:example:tasks/spec/acl/grant/0.1", tt.TypeURIErrShape},
		// Missing version: the slug has nothing after it.
		{"https://trusttasks.org/spec/acl", tt.TypeURIErrShape},
		{"urn:example:tasks:spec:acl", tt.TypeURIErrShape},
		// Empty slug.
		{"https://trusttasks.org/spec//0.1", tt.TypeURIErrSlug},
		{"urn:example:tasks:spec::0.1", tt.TypeURIErrSlug},
		// Slug grammar, in both forms.
		{"https://trusttasks.org/spec/Acl/1.0", tt.TypeURIErrSlug},
		{"https://trusttasks.org/spec/acl--bad/1.0", tt.TypeURIErrSlug},
		{"https://trusttasks.org/spec/-leading/1.0", tt.TypeURIErrSlug},
		{"https://trusttasks.org/spec/trailing-/1.0", tt.TypeURIErrSlug},
		{"https://trusttasks.org/spec/1numeric-start/1.0", tt.TypeURIErrSlug},
		{"https://trusttasks.org/spec/with_underscore/1.0", tt.TypeURIErrSlug},
		{"https://trusttasks.org/spec/acl//grant/1.0", tt.TypeURIErrSlug},
		{"urn:example:tasks:spec:Acl:0.1", tt.TypeURIErrSlug},
		// The §6.1 reservation, under any authority.
		{"https://example.com/spec/trust-task-custom/0.1", tt.TypeURIErrReservedSlug},
		{"https://trusttasks.org/spec/trust-ceremony-evil/0.1", tt.TypeURIErrReservedSlug},
		{"urn:example:tasks:spec:trust-task-custom:0.1", tt.TypeURIErrReservedSlug},
		// Garbage versions, in both forms.
		{"https://trusttasks.org/spec/x/01.0", tt.TypeURIErrVersion},
		{"https://trusttasks.org/spec/x/1.01", tt.TypeURIErrVersion},
		{"https://trusttasks.org/spec/x/1", tt.TypeURIErrVersion},
		{"https://trusttasks.org/spec/x/1.0.0", tt.TypeURIErrVersion},
		{"https://trusttasks.org/spec/x/a.b", tt.TypeURIErrVersion},
		{"https://trusttasks.org/spec/x/+1.0", tt.TypeURIErrVersion},
		{"https://trusttasks.org/spec/x/1.", tt.TypeURIErrVersion},
		{"https://trusttasks.org/spec/x/4294967296.0", tt.TypeURIErrVersion},
		{"https://trusttasks.org/spec/acl/grant/", tt.TypeURIErrVersion},
		{"urn:example:tasks:spec:acl:01.0", tt.TypeURIErrVersion},
		// Fragments.
		{"https://trusttasks.org/spec/acl/grant/0.1#cancel", tt.TypeURIErrFragment},
		{"https://trusttasks.org/spec/acl/grant/0.1#", tt.TypeURIErrFragment},
	}
	for _, c := range cases {
		_, err := tt.ParseTypeURI(c.uri)
		var parseErr *tt.TypeURIError
		if !errors.As(err, &parseErr) {
			t.Errorf("%q: expected a *TypeURIError, got %v", c.uri, err)
			continue
		}
		if parseErr.Kind != c.kind {
			t.Errorf("%q: kind %d, want %d (%v)", c.uri, parseErr.Kind, c.kind, err)
		}
		if !strings.Contains(err.Error(), "Type URI") {
			t.Errorf("%q: message %q does not say Type URI", c.uri, err)
		}
	}
}

func TestEveryPublishedFrameworkSlugParses(t *testing.T) {
	for _, slug := range []string{
		"trust-task-error", "trust-task-ok", "trust-task-next-step",
		"trust-task-discovery", "trust-task-control", "trust-ceremony-receipt",
	} {
		if _, err := tt.ParseTypeURI("https://trusttasks.org/spec/" + slug + "/0.1"); err != nil {
			t.Errorf("%s: %v", slug, err)
		}
	}
}

// §8.5: an extended code is <slug>:<local> whatever the Type URI's form — the
// authority is not part of the code, as in Rust's TrustTaskCode::Extended.
func TestExtendedAndFamilyCodesForEveryForm(t *testing.T) {
	for _, typeURI := range []string{
		"https://trusttasks.org/spec/payments/settle/0.1",
		"https://acme.example/tasks/spec/payments/settle/0.1#response",
		"did:web:acme.example/spec/payments/settle/0.1",
		"urn:example:acme:spec:payments:settle:0.1#request",
	} {
		if got := mustExtended(t, typeURI, "insufficientFunds"); got != "payments/settle:insufficientFunds" {
			t.Errorf("ExtendedCode(%q) = %q", typeURI, got)
		}
		if got := mustFamily(t, typeURI, "payments", "ledgerClosed"); got != "payments:ledgerClosed" {
			t.Errorf("FamilyCode(%q) = %q", typeURI, got)
		}
		if _, err := tt.FamilyCode(typeURI, "payments/refund", "x"); err == nil {
			t.Errorf("FamilyCode(%q) accepted a sibling's slug", typeURI)
		}
	}
	if _, err := tt.ExtendedCode("http://acme.example/spec/payments/settle/0.1", "x"); err == nil {
		t.Error("ExtendedCode accepted an http Type URI")
	}
}
