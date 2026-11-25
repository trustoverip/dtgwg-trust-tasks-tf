package trusttasks

import (
	"bytes"
	"encoding/json"
	"errors"
	"reflect"
)

// Nullable is an optional member whose schema admits null. It has three
// states, which a pointer cannot keep apart:
//
//   - absent: the zero value. The member is not in the document.
//   - null: [Null]. The member is present as null.
//   - a value: [Value]. The member is present with that value.
//
// An update that clears a field is why: an absent `label` leaves the label
// unchanged, `"label": null` clears it, and a string replaces it.
//
//	type update struct {
//		Subject string                      `json:"subject"`
//		Label   trusttasks.Nullable[string] `json:"label,omitzero"`
//	}
//
//	u := update{Subject: s, Label: trusttasks.Null[string]()} // clear the label
//	u := update{Subject: s, Label: trusttasks.Value("ops")}   // set it
//	u := update{Subject: s}                                   // leave it
//
//	if label, ok := u.Label.Get(); ok { ... } else if u.Label.IsNull() { ... }
//
// Tag a Nullable member `omitzero` (Go 1.24 and later): encoding/json then
// asks [Nullable.IsZero] and leaves an absent member out. `omitempty` has no
// effect on a struct, so an absent member tagged only `omitempty` reaches
// [Nullable.MarshalJSON], which refuses it rather than write null.
//
// Build one with [Value] or [Null] and read it with the methods; the fields
// are unexported. When T is comparable, so is Nullable[T]: two are == exactly
// when they are in the same state and, for a value, hold == values.
//
// Only [Null] writes null. With a slice or map T, Value(nil) writes [] or {}
// (unlike encoding/json, which writes a nil slice or map as null), so a set
// member has one spelling per meaning and a nil slice cannot turn into null.
// That matters where null and empty mean opposite things: on acl/* `allowedKeys`,
// null (like absent) means every key and [] means none, so
// Value(filtered) with nothing left in filtered writes [] and grants no key.
// A decoded [] or {} is a Value holding an empty, non-nil slice or map.
//
// Use a Nullable only as a struct field tagged `omitzero`. Each of these
// fails silently:
//   - behind a pointer (*Nullable[T]): encoding/json decodes null as a nil
//     pointer, so null and absent are one state again;
//   - nested (Nullable[Nullable[T]]): Value(Null()) and Null() both write null;
//   - as a map value (map[string]Nullable[T]): omitzero does not apply to map
//     values, so an absent entry makes the whole encode fail.
type Nullable[T any] struct {
	set   bool // present, as null or as a value
	null  bool // present as null; value is then the zero T
	value T
}

// Value returns a Nullable that is present with v.
func Value[T any](v T) Nullable[T] {
	return Nullable[T]{set: true, value: v}
}

// Null returns a Nullable that is present as null.
func Null[T any]() Nullable[T] {
	return Nullable[T]{set: true, null: true}
}

// IsSet reports whether the member is present, as null or as a value.
func (n Nullable[T]) IsSet() bool {
	return n.set
}

// IsNull reports whether the member is present as null.
func (n Nullable[T]) IsNull() bool {
	return n.set && n.null
}

// Get returns the member's value and true, or the zero value of T and false
// when the member is absent or null.
func (n Nullable[T]) Get() (T, bool) {
	if !n.set || n.null {
		var zero T
		return zero, false
	}
	return n.value, true
}

// IsZero reports whether the member is absent. It is what `omitzero`
// consults.
func (n Nullable[T]) IsZero() bool {
	return !n.set
}

// errAbsentNullable is MarshalJSON's answer for an absent member.
var errAbsentNullable = errors.New("trusttasks: an absent Nullable cannot be written; tag its member omitzero so that it is left out")

// MarshalJSON writes null or the value, and a nil slice or map value as [] or
// {}. An absent Nullable is an error: it
// has nothing it may write, and writing null would turn "leave it unchanged"
// into "clear it". A member tagged `omitzero` is left out before this is
// called.
func (n Nullable[T]) MarshalJSON() ([]byte, error) {
	switch {
	case !n.set:
		return nil, errAbsentNullable
	case n.null:
		return []byte("null"), nil
	}
	// Only Null writes null: a nil slice or map written as a value is empty.
	switch v := reflect.ValueOf(&n.value).Elem(); v.Kind() {
	case reflect.Slice:
		if v.IsNil() {
			return []byte("[]"), nil
		}
	case reflect.Map:
		if v.IsNil() {
			return []byte("{}"), nil
		}
	}
	return json.Marshal(n.value)
}

// UnmarshalJSON reads null as null and anything else as a value of T,
// refusing what T refuses. encoding/json calls it only for a member that is
// present, so an absent member stays the zero value.
func (n *Nullable[T]) UnmarshalJSON(b []byte) error {
	if bytes.Equal(bytes.TrimSpace(b), []byte("null")) {
		*n = Null[T]()
		return nil
	}
	var v T
	if err := json.Unmarshal(b, &v); err != nil {
		return err
	}
	*n = Value(v)
	return nil
}

// nullableElem is T, for [CheckMemberNames]: the walk enters a Nullable
// member as the T it decodes into, so member names inside it are checked as
// they are for a bare T or *T.
func (Nullable[T]) nullableElem() reflect.Type {
	return reflect.TypeFor[T]()
}
