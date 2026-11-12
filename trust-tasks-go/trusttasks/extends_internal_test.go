package trusttasks

import "testing"

// #1112: when two patternProperties match one member with different
// subschemas, the first in sorted order is chosen, every time.
func TestMatchPatternIsDeterministic(t *testing.T) {
	patterns := map[string]any{
		"^x":  map[string]any{"title": "a"},
		"^x-": map[string]any{"title": "b"},
		"y$":  map[string]any{"title": "c"},
	}
	for i := 0; i < 200; i++ {
		sub, ok := matchPattern(patterns, "x-y")
		if !ok || sub.(map[string]any)["title"] != "a" {
			t.Fatalf("iteration %d: chose %v, want the ^x subschema", i, sub)
		}
	}
	if _, ok := matchPattern(patterns, "zzz"); ok {
		t.Error("a member no pattern matches must not match")
	}
	// An RE2-incompatible pattern is kept (treated as matching), cached or not.
	for i := 0; i < 2; i++ {
		if sub, ok := matchPattern(map[string]any{`(?<=a)b`: map[string]any{}}, "q"); !ok || sub != true {
			t.Errorf("uncompilable pattern: got %v %v, want true true", sub, ok)
		}
	}
}

// #1112: a payload schema is parsed once and the parse reused.
func TestParsedSchemaIsCached(t *testing.T) {
	const text = `{"type":"object","properties":{"a":{}}}`
	first, err := parsedSchema(text)
	if err != nil {
		t.Fatal(err)
	}
	second, err := parsedSchema(text)
	if err != nil {
		t.Fatal(err)
	}
	a, b := first.(map[string]any), second.(map[string]any)
	a["probe"] = true
	defer delete(a, "probe")
	if b["probe"] != true {
		t.Error("the second call re-parsed the schema instead of reusing the cached parse")
	}
	if _, err := parsedSchema(`{not json`); err == nil {
		t.Error("an unparsable schema must be an error")
	}
}
