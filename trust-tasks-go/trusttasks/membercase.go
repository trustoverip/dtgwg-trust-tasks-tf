package trusttasks

import (
	"bytes"
	"encoding"
	"encoding/json"
	"fmt"
	"reflect"
	"strings"
	"sync"
	"unicode"
	"unicode/utf8"
)

// CheckMemberNames reports an error when data, a JSON value to be decoded into a
// T, holds an object member whose name is not one T declares but which
// encoding/json would match to one anyway.
//
// encoding/json matches object member names to struct fields without regard to
// case, so `{"issuer":"a","Issuer":"b"}` decodes with Issuer "b" — while a proof
// verifier, a JSON Schema validator, and the Rust, TypeScript and Dart runtimes
// all read the exact-case "issuer" and see "a". A document must mean one thing to
// every reader, so a member that differs from a declared one only by case (as
// encoding/json compares names: ASCII case and Unicode simple case folding) is
// refused here rather than silently read as the declared one.
//
// This is stricter than the other runtimes, which keep such a member as an
// unrecognised one: a Go consumer refuses `{"issuer":"a","Issuer":"b"}`, where
// Rust, TypeScript and Dart accept it with issuer "a". Refusing is the only
// reading in which Go agrees with them about issuer. Names differing in any
// other way ("thread_id", "issuerNote") are ordinary unrecognised members and
// are kept, as everywhere else.
//
// The check follows T into nested structs, pointers, slices, arrays and map
// values. A type that decodes itself ([json.Unmarshaler] or
// [encoding.TextUnmarshaler]) is that type's responsibility, and is not entered.
// [Document] and [Proof] decode through this check already; call it yourself
// when you decode a payload outside a [Document] — as a binding's payload
// decoder does.
func CheckMemberNames[T any](data []byte) error {
	return checkMemberNames(data, reflect.TypeOf((*T)(nil)).Elem())
}

func checkMemberNames(data []byte, t reflect.Type) error {
	decoder := json.NewDecoder(bytes.NewReader(data))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		// Not this check's to report: the decode that follows says it in its own
		// terms.
		return nil
	}
	return walkMemberNames(value, t, "")
}

var (
	jsonUnmarshalerType = reflect.TypeOf((*json.Unmarshaler)(nil)).Elem()
	textUnmarshalerType = reflect.TypeOf((*encoding.TextUnmarshaler)(nil)).Elem()
)

// maxMemberNameDepth bounds the walk; encoding/json itself refuses nesting
// deeper than 10000, and no Trust Task schema comes near this.
const maxMemberNameDepth = 512

func walkMemberNames(value any, t reflect.Type, path string) error {
	return walkMemberNamesAt(value, t, path, 0)
}

func walkMemberNamesAt(value any, t reflect.Type, path string, depth int) error {
	if depth > maxMemberNameDepth {
		return nil
	}
	for t.Kind() == reflect.Pointer {
		t = t.Elem()
	}
	if reflect.PointerTo(t).Implements(jsonUnmarshalerType) ||
		reflect.PointerTo(t).Implements(textUnmarshalerType) ||
		t.Implements(jsonUnmarshalerType) {
		return nil
	}
	switch t.Kind() {
	case reflect.Struct:
		members, ok := value.(map[string]any)
		if !ok {
			return nil
		}
		fields := fieldsOf(t)
		for name, member := range members {
			if fieldType, declared := fields.exact[name]; declared {
				if err := walkMemberNamesAt(member, fieldType, path+"/"+name, depth+1); err != nil {
					return err
				}
				continue
			}
			if declared, folds := fields.folded[foldMemberName(name)]; folds {
				return fmt.Errorf("trusttasks: member %q at %q is not %q: member names are case-sensitive",
					name, path+"/", declared)
			}
		}
	case reflect.Slice, reflect.Array:
		items, ok := value.([]any)
		if !ok {
			return nil
		}
		for i, item := range items {
			if err := walkMemberNamesAt(item, t.Elem(), fmt.Sprintf("%s/%d", path, i), depth+1); err != nil {
				return err
			}
		}
	case reflect.Map:
		members, ok := value.(map[string]any)
		if !ok {
			return nil
		}
		for name, member := range members {
			if err := walkMemberNamesAt(member, t.Elem(), path+"/"+name, depth+1); err != nil {
				return err
			}
		}
	}
	return nil
}

// structFields is what encoding/json matches a struct's members against: each
// field's wire name, and the same names folded.
type structFields struct {
	exact  map[string]reflect.Type
	folded map[string]string
}

var structFieldsCache sync.Map // reflect.Type → *structFields

func fieldsOf(t reflect.Type) *structFields {
	if cached, ok := structFieldsCache.Load(t); ok {
		return cached.(*structFields)
	}
	fields := &structFields{exact: map[string]reflect.Type{}, folded: map[string]string{}}
	collectFields(t, fields, 0)
	for name := range fields.exact {
		fields.folded[foldMemberName(name)] = name
	}
	cached, _ := structFieldsCache.LoadOrStore(t, fields)
	return cached.(*structFields)
}

// collectFields follows encoding/json's naming: the tag name, or the Go field
// name when the tag gives none; `json:"-"` excluded; the fields of an untagged
// embedded struct promoted, breadth first, so a shallower field wins a name
// over a deeper one as it does in encoding/json.
func collectFields(t reflect.Type, into *structFields, _ int) {
	level := []reflect.Type{t}
	for depth := 0; depth <= 16 && len(level) > 0; depth++ {
		var next []reflect.Type
		named := map[string]reflect.Type{}
		for _, st := range level {
			for i := 0; i < st.NumField(); i++ {
				field := st.Field(i)
				tag := field.Tag.Get("json")
				if tag == "-" {
					continue
				}
				name, _, _ := strings.Cut(tag, ",")
				if field.Anonymous && name == "" {
					embedded := field.Type
					if embedded.Kind() == reflect.Pointer {
						embedded = embedded.Elem()
					}
					if embedded.Kind() == reflect.Struct {
						next = append(next, embedded)
						continue
					}
				}
				if !field.IsExported() {
					continue
				}
				if name == "" {
					name = field.Name
				}
				if _, taken := named[name]; !taken {
					named[name] = field.Type
				}
			}
		}
		for name, fieldType := range named {
			if _, shadowed := into.exact[name]; !shadowed {
				into.exact[name] = fieldType
			}
		}
		level = next
	}
}

// foldMemberName folds a member name as encoding/json's (v1) matcher does:
// ASCII case and Unicode simple case folding, so "ſ" and "s", "K" and "k" fold
// together. '_' and '-' are significant, as they are in v1 (and in v1 run on the
// v2 implementation, whose v1 defaults set MatchCaseSensitiveDelimiter).
func foldMemberName(name string) string {
	var out strings.Builder
	for _, r := range name {
		if r < utf8.RuneSelf {
			out.WriteRune(unicode.ToUpper(r))
			continue
		}
		out.WriteRune(foldRune(r))
	}
	return out.String()
}

// foldRune returns the smallest rune in r's simple fold set — what
// encoding/json compares.
func foldRune(r rune) rune {
	for {
		next := unicode.SimpleFold(r)
		if next <= r {
			return next
		}
		r = next
	}
}
