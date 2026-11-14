package didcomm

import (
	"bytes"
	"encoding/json"
	"strings"
	"unicode"
	"unicode/utf8"
)

// caseFoldedDuplicate reports a JSON object anywhere in document holding two
// members whose names differ only by case, and one of those names.
//
// encoding/json reads such a pair into one Go field (the later member wins),
// while every other reader reads the exact-case member. [checkJOSEStructure]
// refuses a JWE envelope or protected header holding such a pair. The Trust
// Task document inside is checked by the core (trusttasks.ConsumeReceived), which
// carries its own copy of this check; this one covers the JOSE layer the core
// never sees.
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
