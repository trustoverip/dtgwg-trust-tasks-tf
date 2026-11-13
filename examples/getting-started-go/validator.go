package main

// A stand-in for a JSON Schema engine, so this example runs with no dependency
// beyond the Trust Tasks modules.
//
// It understands the subset of JSON Schema 2020-12 the generated payload
// schemas use for their structure — local $refs into $defs, type, required,
// properties, additionalProperties: false, const, enum, oneOf, items,
// minItems, minLength and maxLength — and ignores the rest (pattern, format,
// uniqueItems, …). That is enough to refuse a payload with a missing, misspelt
// or mistyped member, which encoding/json otherwise lets straight through.
//
// A real application puts a full Draft 2020-12 engine behind
// trusttasks.PayloadValidator instead. GETTING-STARTED.md §1, "What you bring
// yourself", says why the runtime bundles none.

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"math"
	"reflect"
	"strings"
	"unicode/utf8"
)

type structuralValidator struct{}

// Validate implements trusttasks.PayloadValidator.
func (structuralValidator) Validate(schemaJSON string, payload json.RawMessage) error {
	var schema map[string]any
	if err := json.Unmarshal([]byte(schemaJSON), &schema); err != nil {
		return fmt.Errorf("schema: %w", err)
	}
	dec := json.NewDecoder(bytes.NewReader(payload))
	dec.UseNumber()
	var value any
	if err := dec.Decode(&value); err != nil {
		return fmt.Errorf("payload: %w", err)
	}
	var problems []string
	check(schema, value, "payload", schema, &problems)
	if len(problems) > 0 {
		return errors.New(strings.Join(problems, "; "))
	}
	return nil
}

func typeMatches(typ any, value any) bool {
	switch typ {
	case "object":
		_, ok := value.(map[string]any)
		return ok
	case "array":
		_, ok := value.([]any)
		return ok
	case "string":
		_, ok := value.(string)
		return ok
	case "boolean":
		_, ok := value.(bool)
		return ok
	case "number":
		_, ok := value.(json.Number)
		return ok
	case "integer":
		n, ok := value.(json.Number)
		if !ok {
			return false
		}
		f, err := n.Float64()
		return err == nil && f == math.Trunc(f)
	default:
		return true
	}
}

// same compares a schema value (decoded without UseNumber) with a payload value.
func same(want, got any) bool {
	if n, ok := got.(json.Number); ok {
		f, err := n.Float64()
		return err == nil && reflect.DeepEqual(want, f)
	}
	return reflect.DeepEqual(want, got)
}

func check(schema map[string]any, value any, at string, root map[string]any, problems *[]string) {
	if ref, ok := schema["$ref"].(string); ok {
		defs, _ := root["$defs"].(map[string]any)
		target, ok := defs[strings.TrimPrefix(ref, "#/$defs/")].(map[string]any)
		if !ok || !strings.HasPrefix(ref, "#/$defs/") {
			*problems = append(*problems, fmt.Sprintf("%s: unsupported $ref %s", at, ref))
			return
		}
		check(target, value, at, root, problems)
	}
	if typ, ok := schema["type"]; ok && !typeMatches(typ, value) {
		*problems = append(*problems, fmt.Sprintf("%s: expected %v", at, typ))
		return
	}
	if want, ok := schema["const"]; ok && !same(want, value) {
		*problems = append(*problems, fmt.Sprintf("%s: expected %v", at, want))
	}
	if enum, ok := schema["enum"].([]any); ok {
		found := false
		for _, e := range enum {
			found = found || same(e, value)
		}
		if !found {
			*problems = append(*problems, fmt.Sprintf("%s: not one of %v", at, enum))
		}
	}
	if branches, ok := schema["oneOf"].([]any); ok {
		matching := 0
		for _, b := range branches {
			branch, _ := b.(map[string]any)
			var branchProblems []string
			check(branch, value, at, root, &branchProblems)
			if len(branchProblems) == 0 {
				matching++
			}
		}
		if matching != 1 {
			*problems = append(*problems, fmt.Sprintf("%s: matches %d of the oneOf branches, not exactly one", at, matching))
		}
	}
	switch v := value.(type) {
	case string:
		n := float64(utf8.RuneCountInString(v))
		if min, ok := schema["minLength"].(float64); ok && n < min {
			*problems = append(*problems, at+": too short")
		}
		if max, ok := schema["maxLength"].(float64); ok && n > max {
			*problems = append(*problems, at+": too long")
		}
	case []any:
		if min, ok := schema["minItems"].(float64); ok && float64(len(v)) < min {
			*problems = append(*problems, at+": too few items")
		}
		if items, ok := schema["items"].(map[string]any); ok {
			for i, item := range v {
				check(items, item, fmt.Sprintf("%s[%d]", at, i), root, problems)
			}
		}
	case map[string]any:
		properties, _ := schema["properties"].(map[string]any)
		required, _ := schema["required"].([]any)
		for _, r := range required {
			if name, _ := r.(string); name != "" {
				if _, present := v[name]; !present {
					*problems = append(*problems, fmt.Sprintf("%s: missing required member %s", at, name))
				}
			}
		}
		for name, member := range v {
			if sub, ok := properties[name].(map[string]any); ok {
				check(sub, member, at+"."+name, root, problems)
			} else if schema["additionalProperties"] == false {
				*problems = append(*problems, fmt.Sprintf("%s: unknown member %s", at, name))
			}
		}
	}
}
