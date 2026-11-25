package trusttasks

import (
	"bytes"
	"encoding/json"
	"errors"
)

// Nullable is an optional member whose schema admits null. It has three
// states, which a pointer cannot keep apart:
//
//   - absent: the zero value. The member is not in the document.
//   - null: [Null]. The member is present as null.
//   - a value: [Value]. The member is present with that value.
//
// acl/update 0.3 is why: an absent `label` leaves the label unchanged,
// `"label": null` clears it, and a string replaces it.
//
//	p := aclupdatev0_3.Payload{Subject: s, Label: trusttasks.Null[string]()} // clear the label
//	p := aclupdatev0_3.Payload{Subject: s, Label: trusttasks.Value("ops")}   // set it
//	p := aclupdatev0_3.Payload{Subject: s}                                   // leave it
//
//	if label, ok := p.Label.Get(); ok { ... } else if p.Label.IsNull() { ... }
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
// With a slice or map T, Value(nil) writes null, as encoding/json writes a nil
// slice; Value([]T{}) writes [].
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

// MarshalJSON writes null or the value. An absent Nullable is an error: it
// has nothing it may write, and writing null would turn "leave it unchanged"
// into "clear it". A member tagged `omitzero` is left out before this is
// called.
func (n Nullable[T]) MarshalJSON() ([]byte, error) {
	switch {
	case !n.set:
		return nil, errAbsentNullable
	case n.null:
		return []byte("null"), nil
	default:
		return json.Marshal(n.value)
	}
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
