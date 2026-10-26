// Declared-extension routing (SPEC §5.2) for a third-party (§6.5) specification.
package trusttasks_test

import (
	"encoding/json"
	"strings"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// A third-party specification routes along its own declared chain, under its own
// authority and form, and never to a registry specification that shares its slug
// — in the path form and the URN form alike.
func TestExtendsChainUnderAThirdPartyAuthority(t *testing.T) {
	const (
		pathBase = "https://acme.example/tasks/spec/test/widget/"
		urnBase  = "urn:example:acme:spec:test:widget:"
	)
	declared := map[string]string{
		pathBase + "0.3": "0.2", pathBase + "0.2": "0.1",
		urnBase + "0.3": "0.2", urnBase + "0.2": "0.1",
		// The registry's widget declares 0.4 extends 0.1; acme's widget does not.
		widgetBase + "0.4": "0.1",
	}
	var asked []string
	lookup := func(typeURI string) string {
		asked = append(asked, typeURI)
		return declared[typeURI]
	}

	for _, base := range []string{pathBase, urnBase} {
		spec := tt.SpecPolicy{TypeURI: base + "0.1", PayloadSchema: widgetSchema}

		asked = nil
		reading, refusal, err := tt.ReadForSpecWith(json.RawMessage(widget03Typed(base+"0.3")), spec, lookup)
		if err != nil || refusal != nil {
			t.Fatalf("%s: chain refused: %v %+v", base, err, refusal)
		}
		if strings.Contains(string(reading), `"colour"`) || !strings.Contains(string(reading), base+"0.3") {
			t.Fatalf("%s: not read as 0.1 under its received type: %s", base, reading)
		}
		if len(asked) == 0 {
			t.Fatalf("%s: lookup never consulted", base)
		}
		for _, q := range asked {
			if !strings.HasPrefix(q, base) || strings.Contains(q, "#") {
				t.Fatalf("%s: chain looked up %q, outside its own authority and form", base, q)
			}
		}
		if !tt.ExtendsChainReaches(base+"0.3#response", base+"0.1#response", lookup) {
			t.Fatalf("%s: response chain not followed", base)
		}
		if !tt.ExtendsChainReaches(base+"0.3#request", base+"0.1", lookup) {
			t.Fatalf("%s: #request is not the request", base)
		}

		other := map[string]string{pathBase: urnBase, urnBase: pathBase}[base]
		cases := map[string]tt.StandardCode{
			// acme's 0.4 declares nothing; the registry's 0.4 declaration is not its.
			base + "0.4": tt.CodeUnsupportedVersion,
			// The same slug under the registry, or under acme in the other form,
			// is another specification.
			widgetBase + "0.1": tt.CodeUnsupportedType,
			widgetBase + "0.4": tt.CodeUnsupportedType,
			other + "0.1":      tt.CodeUnsupportedType,
		}
		for typeURI, code := range cases {
			received := json.RawMessage(`{"id":"x","type":"` + typeURI + `","payload":{"name":"w"}}`)
			reading, refusal, err := tt.ReadForSpecWith(received, spec, lookup)
			if err != nil || reading != nil || refusal == nil || refusal.Code != code {
				t.Fatalf("%s -> %s: got %s %+v %v, want %s", typeURI, spec.TypeURI, reading, refusal, err, code)
			}
		}
		if tt.ExtendsChainReaches(widgetBase+"0.4", base+"0.1", lookup) {
			t.Fatalf("%s: a registry document reached a third-party version", base)
		}
		if tt.ExtendsChainReaches(base+"0.4", widgetBase+"0.1", lookup) {
			t.Fatalf("%s: a third-party document reached a registry version", base)
		}
	}

	// The registry's own chain still works beside them.
	if !tt.ExtendsChainReaches(widgetBase+"0.4", widgetBase+"0.1", lookup) {
		t.Fatal("registry chain not followed")
	}
	// A lookup answering something that is not MAJOR.MINOR ends the chain.
	if tt.ExtendsChainReaches(pathBase+"0.3", pathBase+"0.1", func(string) string { return "zero" }) {
		t.Fatal("a garbage declaration was followed")
	}
}

// widget03Typed is widget03 under an arbitrary Type URI.
func widget03Typed(typeURI string) string {
	return strings.Replace(widget03("0.3"), `"type": "`+widgetBase+`0.3"`, `"type": "`+typeURI+`"`, 1)
}
