package trusttasks

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

// The reserved-slug allowlist is a hand-maintained copy of the one the registry
// build enforces (#/properties/slug/anyOf[1]/enum in specs/spec.meta.schema.json).
// Read that one and fail if they disagree. Outside the repository — the
// published module — the meta-schema is absent and there is nothing to compare.
func TestReservedAllowlistMatchesTheMetaSchema(t *testing.T) {
	raw, err := os.ReadFile(filepath.Join("..", "..", "specs", "spec.meta.schema.json"))
	if err != nil {
		t.Skip("specs/spec.meta.schema.json not present")
	}
	var meta struct {
		Properties struct {
			Slug struct {
				AnyOf []struct {
					Enum []string `json:"enum"`
				} `json:"anyOf"`
			} `json:"slug"`
		} `json:"properties"`
	}
	if err := json.Unmarshal(raw, &meta); err != nil {
		t.Fatal(err)
	}
	if len(meta.Properties.Slug.AnyOf) < 2 || len(meta.Properties.Slug.AnyOf[1].Enum) == 0 {
		t.Fatal("meta-schema has no reserved-slug allowlist at #/properties/slug/anyOf[1]/enum")
	}
	for _, slug := range meta.Properties.Slug.AnyOf[1].Enum {
		if !isAllowedFrameworkSlug(slug) {
			t.Errorf("%s is allowlisted by spec.meta.schema.json but not by isAllowedFrameworkSlug", slug)
		}
		if !isReservedNamespace(slug) {
			t.Errorf("%s is allowlisted but not in the reserved namespace", slug)
		}
	}
}
