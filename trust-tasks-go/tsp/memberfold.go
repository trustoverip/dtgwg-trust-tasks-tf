package tsp

import (
	"bytes"
	"encoding/json"
	"strings"
	"unicode"
	"unicode/utf8"
)

// caseFoldedDuplicate reports a JSON object anywhere in document — envelope or
// payload — holding two members whose names differ only by case, and one of
// those names.
//
// encoding/json reads such a pair into one Go field (the later member wins),
// while a proof verifier, a JSON Schema validator and the Rust, TypeScript and
// Dart runtimes read the exact-case member. A signed {"role":"reader",
// "Role":"admin"} would therefore verify as "reader" and reach a Go handler as
// "admin". [Consume] refuses such a document malformedRequest before decoding
// it.
//
// The core module's Document decoding and trusttasks.CheckMemberNames refuse
// the wider set — any member that matches a declared field only when case is
// ignored — from the core release that introduced them; this module requires an
// earlier core, so it carries this self-contained check, which needs no type
// information.
func caseFoldedDuplicate(document []byte) (string, bool) {
	decoder := json.NewDecoder(bytes.NewReader(document))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		return "", false
	}
	return foldedDuplicateIn(value, 0)
}

func foldedDuplicateIn(value any, depth int) (string, bool) {
	if depth > 512 {
		return "", false
	}
	switch v := value.(type) {
	case map[string]any:
		seen := make(map[string]struct{}, len(v))
		for name, member := range v {
			folded := foldMemberName(name)
			if _, dup := seen[folded]; dup {
				return name, true
			}
			seen[folded] = struct{}{}
			if found, ok := foldedDuplicateIn(member, depth+1); ok {
				return found, true
			}
		}
	case []any:
		for _, item := range v {
			if found, ok := foldedDuplicateIn(item, depth+1); ok {
				return found, true
			}
		}
	}
	return "", false
}

// foldMemberName folds a name as encoding/json compares names: ASCII case and
// Unicode simple case folding.
func foldMemberName(name string) string {
	var out strings.Builder
	for _, r := range name {
		if r < utf8.RuneSelf {
			out.WriteRune(unicode.ToUpper(r))
			continue
		}
		for {
			next := unicode.SimpleFold(r)
			if next <= r {
				r = next
				break
			}
			r = next
		}
		out.WriteRune(r)
	}
	return out.String()
}
