package trusttasks

import (
	"bytes"
	"encoding/json"
	"fmt"
	"sort"
	"strconv"
	"strings"
)

// SPEC §4.5.2 — namespaces a producer marks critical with extCritical.
//
// An ext namespace is ignorable by default (§4.5.1 rule 5). A specification may
// let a producer list some of them in a sibling extCritical array, and a consumer
// that does not recognize a namespace listed there MUST NOT process the document
// and MUST reject it with unsupportedExtension (§4.5.2 rule 4). An entry naming
// a namespace absent from the sibling ext is non-conforming and is rejected with
// malformedRequest (rule 2).
//
// extCritical exists only at the object levels a specification opts in at, so
// the payload is walked alongside its schema:
//
//   - a level whose schema declares an extCritical property is checked;
//   - a level whose schema closes it (additionalProperties: false or
//     unevaluatedProperties: false) without declaring it is refused as
//     malformedRequest when the member is present — what a schema validator
//     reports, applied even under PayloadAcceptUnvalidated so the marking is
//     never dropped silently;
//   - a level the schema leaves open is data (a document a vault is asked to
//     sign, say), and a member named extCritical in it is not a marking.
//
// Every branch of allOf / anyOf / oneOf is followed, so a level is checked where
// any branch declares the member. With no schema, every level outside an ext
// namespace's own content is checked. The Rust, TypeScript and Dart runtimes
// implement the same walk; fixtures/consume/ext-critical.json pins the four
// together.

const (
	criticalMaxDepth        = 128
	criticalBaseSteps       = 100_000
	criticalStepsPerValue   = 64
	criticalMaxEchoedLength = 128
	extCriticalMember       = "extCritical"
)

type criticalRefusal struct{ reason RejectReason }

func criticalMalformed(message string) *criticalRefusal {
	return &criticalRefusal{RejectReason{
		Code:    CodeMalformedRequest,
		Message: message + " (SPEC §4.5.2)",
	}}
}

func criticalTooComplex() *criticalRefusal {
	return criticalMalformed("payload is too deeply nested or too complex to check its critical extensions")
}

// CheckCriticalExtensions applies SPEC §4.5.2 to payload, read under schema —
// the payload schema of the specification the document is consumed under
// ([SpecPolicy.PayloadSchema]), or "" to read every level. recognized is every
// ext namespace this consumer implements.
//
// It returns nil, or the refusal: unsupportedExtension for the first critical
// namespace not recognized, malformedRequest for a marking that breaks rules 1
// and 2, sits at a level that does not allow it, or for a payload that is not
// JSON. A payload with no member named extCritical anywhere returns nil after
// one pass.
func CheckCriticalExtensions(payload json.RawMessage, schema string, recognized []string) *RejectReason {
	if !bytes.Contains(payload, []byte(`"`+extCriticalMember+`"`)) && !bytes.Contains(payload, []byte(`\u`)) {
		return nil
	}
	dec := json.NewDecoder(bytes.NewReader(payload))
	dec.UseNumber()
	var value any
	if err := dec.Decode(&value); err != nil {
		return &criticalMalformed("payload is not JSON").reason
	}
	values, marked := countValuesIfMarked(value)
	if !marked {
		return nil
	}
	w := &criticalWalk{
		recognized: map[string]bool{},
		left:       criticalBaseSteps + criticalStepsPerValue*values,
	}
	for _, r := range recognized {
		w.recognized[r] = true
	}
	var refusal *criticalRefusal
	if strings.TrimSpace(schema) == "" {
		refusal = w.blind(value, 0)
	} else {
		var root any
		if err := json.Unmarshal([]byte(schema), &root); err != nil {
			// A schema that does not parse says nothing about where a marking
			// may sit; read every level, as with no schema.
			refusal = w.blind(value, 0)
		} else {
			w.root = root
			refusal = w.guided(value, []any{root}, 0)
		}
	}
	if refusal != nil {
		return &refusal.reason
	}
	return nil
}

func countValuesIfMarked(v any) (int, bool) {
	n := 0
	marked := false
	stack := []any{v}
	for len(stack) > 0 {
		top := stack[len(stack)-1]
		stack = stack[:len(stack)-1]
		n++
		switch t := top.(type) {
		case map[string]any:
			if _, ok := t[extCriticalMember]; ok {
				marked = true
			}
			for _, m := range t {
				stack = append(stack, m)
			}
		case []any:
			stack = append(stack, t...)
		}
	}
	return n, marked
}

type criticalWalk struct {
	recognized map[string]bool
	root       any
	left       int
}

func (w *criticalWalk) spend(depth int) *criticalRefusal {
	if depth > criticalMaxDepth || w.left <= 0 {
		return criticalTooComplex()
	}
	w.left--
	return nil
}

func sortedKeys(m map[string]any) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

func (w *criticalWalk) blind(v any, depth int) *criticalRefusal {
	if r := w.spend(depth); r != nil {
		return r
	}
	switch t := v.(type) {
	case map[string]any:
		if _, ok := t[extCriticalMember]; ok {
			if r := w.checkLevel(t); r != nil {
				return r
			}
		}
		for _, k := range sortedKeys(t) {
			if k == "ext" || k == extCriticalMember {
				continue
			}
			if r := w.blind(t[k], depth+1); r != nil {
				return r
			}
		}
	case []any:
		for _, e := range t {
			if r := w.blind(e, depth+1); r != nil {
				return r
			}
		}
	}
	return nil
}

func (w *criticalWalk) guided(v any, schemas []any, depth int) *criticalRefusal {
	if r := w.spend(depth); r != nil {
		return r
	}
	switch v.(type) {
	case map[string]any, []any:
	default:
		return nil
	}
	var applicable []map[string]any
	for _, s := range schemas {
		if r := w.expand(s, &applicable, depth); r != nil {
			return r
		}
	}
	if len(applicable) == 0 {
		return nil
	}
	switch t := v.(type) {
	case []any:
		for i, e := range t {
			if children := elementSchemas(applicable, i); len(children) > 0 {
				if r := w.guided(e, children, depth+1); r != nil {
					return r
				}
			}
		}
	case map[string]any:
		if _, ok := t[extCriticalMember]; ok {
			declared, closed := false, false
			for _, s := range applicable {
				if _, ok := schemaProperty(s, extCriticalMember); ok {
					declared = true
				}
				if schemaCloses(s) {
					closed = true
				}
			}
			if declared {
				if r := w.checkLevel(t); r != nil {
					return r
				}
			} else if closed {
				return criticalMalformed("`extCritical` is not allowed at this level of the payload")
			}
		}
		for _, k := range sortedKeys(t) {
			if k == extCriticalMember {
				continue
			}
			if children := memberSchemas(applicable, k); len(children) > 0 {
				if r := w.guided(t[k], children, depth+1); r != nil {
					return r
				}
			}
		}
	}
	return nil
}

func (w *criticalWalk) expand(schema any, out *[]map[string]any, depth int) *criticalRefusal {
	if r := w.spend(depth); r != nil {
		return r
	}
	obj, ok := schema.(map[string]any)
	if !ok {
		return nil
	}
	for _, seen := range *out {
		if sameMap(seen, obj) {
			return nil
		}
	}
	*out = append(*out, obj)
	if ref, ok := obj["$ref"].(string); ok && w.root != nil {
		if target, ok := resolveLocalRef(ref, w.root); ok {
			if r := w.expand(target, out, depth+1); r != nil {
				return r
			}
		}
	}
	for _, keyword := range []string{"allOf", "anyOf", "oneOf"} {
		if branches, ok := obj[keyword].([]any); ok {
			for _, b := range branches {
				if r := w.expand(b, out, depth+1); r != nil {
					return r
				}
			}
		}
	}
	return nil
}

// sameMap reports whether a and b are the same map value (not merely equal).
func sameMap(a, b map[string]any) bool {
	return fmt.Sprintf("%p", a) == fmt.Sprintf("%p", b)
}

// checkLevel applies rules 1 and 2, then rule 4, at one object level.
func (w *criticalWalk) checkLevel(level map[string]any) *criticalRefusal {
	entries, ok := level[extCriticalMember].([]any)
	if !ok {
		return criticalMalformed("`extCritical` must be an array")
	}
	if len(entries) == 0 {
		return criticalMalformed("`extCritical` must name at least one namespace")
	}
	ext, ok := level["ext"].(map[string]any)
	if !ok {
		return criticalMalformed("`extCritical` is present where there is no sibling `ext` object")
	}
	seen := map[string]bool{}
	names := make([]string, 0, len(entries))
	for _, e := range entries {
		name, ok := e.(string)
		if !ok {
			return criticalMalformed("every `extCritical` entry must be a string")
		}
		if !isExtNamespace(name) {
			return criticalMalformed("every `extCritical` entry must be a reverse-DNS namespace")
		}
		if seen[name] {
			return criticalMalformed("`extCritical` entries must be unique")
		}
		seen[name] = true
		if _, ok := ext[name]; !ok {
			return criticalMalformed("every `extCritical` entry must name a namespace present in the sibling `ext`")
		}
		names = append(names, name)
	}
	for _, name := range names {
		if !w.recognized[name] {
			echoed := name
			if len(echoed) > criticalMaxEchoedLength {
				echoed = echoed[:criticalMaxEchoedLength]
			}
			return &criticalRefusal{RejectReason{
				Code:    CodeUnsupportedExtension,
				Message: "unsupported critical extension: " + echoed,
			}}
		}
	}
	return nil
}

func schemaProperty(schema map[string]any, name string) (any, bool) {
	props, ok := schema["properties"].(map[string]any)
	if !ok {
		return nil, false
	}
	p, ok := props[name]
	return p, ok
}

func schemaCloses(schema map[string]any) bool {
	return schema["additionalProperties"] == false || schema["unevaluatedProperties"] == false
}

// memberSchemas returns the schemas that apply to member name. patternProperties
// are taken whole: matching the patterns would only narrow the places a marking
// may be read.
func memberSchemas(applicable []map[string]any, name string) []any {
	var out []any
	for _, s := range applicable {
		if p, ok := schemaProperty(s, name); ok {
			out = append(out, p)
			continue
		}
		if patterns, ok := s["patternProperties"].(map[string]any); ok {
			for _, k := range sortedKeys(patterns) {
				out = append(out, patterns[k])
			}
		}
		if additional, ok := s["additionalProperties"].(map[string]any); ok {
			out = append(out, additional)
		}
	}
	return out
}

func elementSchemas(applicable []map[string]any, index int) []any {
	var out []any
	for _, s := range applicable {
		if prefix, ok := s["prefixItems"].([]any); ok && index < len(prefix) {
			out = append(out, prefix[index])
			continue
		}
		if items, ok := s["items"].(map[string]any); ok {
			out = append(out, items)
		}
	}
	return out
}

func resolveLocalRef(ref string, root any) (any, bool) {
	if !strings.HasPrefix(ref, "#") {
		return nil, false
	}
	pointer := ref[1:]
	if pointer == "" {
		return root, true
	}
	if !strings.HasPrefix(pointer, "/") {
		return nil, false
	}
	node := root
	for _, raw := range strings.Split(pointer[1:], "/") {
		token := strings.ReplaceAll(strings.ReplaceAll(raw, "~1", "/"), "~0", "~")
		switch t := node.(type) {
		case map[string]any:
			next, ok := t[token]
			if !ok {
				return nil, false
			}
			node = next
		case []any:
			i, err := strconv.Atoi(token)
			if err != nil || i < 0 || i >= len(t) {
				return nil, false
			}
			node = t[i]
		default:
			return nil, false
		}
	}
	return node, true
}

// isExtNamespace reports whether name matches §4.5.1 rule 2:
// ^[a-z][a-z0-9-]*(\.[a-z0-9-]+)+$.
func isExtNamespace(name string) bool {
	labels := strings.Split(name, ".")
	if len(labels) < 2 {
		return false
	}
	for i, label := range labels {
		if label == "" {
			return false
		}
		for j := 0; j < len(label); j++ {
			c := label[j]
			if i == 0 && j == 0 {
				if c < 'a' || c > 'z' {
					return false
				}
				continue
			}
			if !(c >= 'a' && c <= 'z' || c >= '0' && c <= '9' || c == '-') {
				return false
			}
		}
	}
	return true
}
