package trusttasks_test

import (
	"encoding/json"
	"testing"

	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// nullableProbe is how a generated payload type carries an optional member
// whose schema admits null: a Nullable tagged omitzero.
type nullableProbe struct {
	ID    string                        `json:"id"`
	Label trusttasks.Nullable[string]   `json:"label,omitzero"`
	Keys  trusttasks.Nullable[[]string] `json:"keys,omitzero"`
}

func TestNullableStates(t *testing.T) {
	var absent trusttasks.Nullable[string]
	if absent.IsSet() || absent.IsNull() {
		t.Errorf("zero value: IsSet %v, IsNull %v, want both false", absent.IsSet(), absent.IsNull())
	}
	if v, ok := absent.Get(); ok || v != "" {
		t.Errorf("zero value: Get = %q, %v", v, ok)
	}

	null := trusttasks.Null[string]()
	if !null.IsSet() || !null.IsNull() {
		t.Errorf("Null: IsSet %v, IsNull %v, want both true", null.IsSet(), null.IsNull())
	}
	if _, ok := null.Get(); ok {
		t.Error("Null: Get reported a value")
	}

	value := trusttasks.Value("ops")
	if !value.IsSet() || value.IsNull() {
		t.Errorf("Value: IsSet %v, IsNull %v, want true, false", value.IsSet(), value.IsNull())
	}
	if v, ok := value.Get(); !ok || v != "ops" {
		t.Errorf("Value: Get = %q, %v", v, ok)
	}

	// The zero value of T is a value like any other, not null.
	if v, ok := trusttasks.Value("").Get(); !ok || v != "" {
		t.Errorf(`Value(""): Get = %q, %v`, v, ok)
	}
}

// Each of the three states survives a decode and an encode: absent stays
// absent, null stays null, and a value stays that value, empty ones included.
func TestNullableRoundTrip(t *testing.T) {
	for _, wire := range []string{
		`{"id":"a"}`,
		`{"id":"a","label":null}`,
		`{"id":"a","label":"ops"}`,
		`{"id":"a","label":""}`,
		`{"id":"a","keys":null}`,
		`{"id":"a","keys":[]}`,
		`{"id":"a","keys":["k1"]}`,
		`{"id":"a","label":null,"keys":[]}`,
	} {
		var p nullableProbe
		if err := json.Unmarshal([]byte(wire), &p); err != nil {
			t.Fatalf("%s: %v", wire, err)
		}
		got, err := json.Marshal(p)
		if err != nil {
			t.Fatalf("%s: %v", wire, err)
		}
		if string(got) != wire {
			t.Errorf("round trip of %s gave %s", wire, got)
		}
	}
}

func TestNullableDecodedStates(t *testing.T) {
	var p nullableProbe
	if err := json.Unmarshal([]byte(`{"id":"a"}`), &p); err != nil {
		t.Fatal(err)
	}
	if p.Label.IsSet() {
		t.Error("absent label decoded as set")
	}
	if err := json.Unmarshal([]byte(`{"id":"a","label":null}`), &p); err != nil {
		t.Fatal(err)
	}
	if !p.Label.IsNull() {
		t.Error("null label did not decode as null")
	}
	p = nullableProbe{}
	if err := json.Unmarshal([]byte(`{"id":"a","label":"ops"}`), &p); err != nil {
		t.Fatal(err)
	}
	if v, ok := p.Label.Get(); !ok || v != "ops" {
		t.Errorf("label decoded as %q, %v", v, ok)
	}
	// A later member overwrites an earlier one, as encoding/json does for any
	// field, and does not leave both states behind.
	p = nullableProbe{}
	if err := json.Unmarshal([]byte(`{"id":"a","label":"ops","label":null}`), &p); err != nil {
		t.Fatal(err)
	}
	if !p.Label.IsNull() {
		t.Error("a null after a value did not leave the member null")
	}
}

func TestNullableEncodes(t *testing.T) {
	for _, c := range []struct {
		p    nullableProbe
		want string
	}{
		{nullableProbe{ID: "a"}, `{"id":"a"}`},
		{nullableProbe{ID: "a", Label: trusttasks.Null[string]()}, `{"id":"a","label":null}`},
		{nullableProbe{ID: "a", Label: trusttasks.Value("ops")}, `{"id":"a","label":"ops"}`},
		{nullableProbe{ID: "a", Keys: trusttasks.Value([]string{})}, `{"id":"a","keys":[]}`},
	} {
		got, err := json.Marshal(c.p)
		if err != nil {
			t.Fatal(err)
		}
		if string(got) != c.want {
			t.Errorf("marshalled %s, want %s", got, c.want)
		}
	}
}

// A value of the wrong type is refused, as it would be for a plain T.
func TestNullableRefusesTheWrongType(t *testing.T) {
	var p nullableProbe
	if err := json.Unmarshal([]byte(`{"id":"a","label":7}`), &p); err == nil {
		t.Error("a number decoded into Nullable[string]")
	}
}

// Without omitzero an absent Nullable has nothing it may write: writing null
// would turn "leave it unchanged" into "clear it". It refuses, and omitempty
// does not help, because encoding/json never treats a struct as empty.
func TestNullableAbsentRefusesToMarshal(t *testing.T) {
	type untagged struct {
		Label trusttasks.Nullable[string] `json:"label"`
	}
	type omitempty struct {
		Label trusttasks.Nullable[string] `json:"label,omitempty"`
	}
	if b, err := json.Marshal(untagged{}); err == nil {
		t.Errorf("an absent Nullable without omitzero marshalled as %s", b)
	}
	if b, err := json.Marshal(omitempty{}); err == nil {
		t.Errorf("an absent Nullable tagged only omitempty marshalled as %s", b)
	}
	if _, err := (trusttasks.Nullable[string]{}).MarshalJSON(); err == nil {
		t.Error("MarshalJSON of an absent Nullable did not fail")
	}
}

// IsZero is what omitzero consults: absent is zero, null and a value are not.
func TestNullableIsZero(t *testing.T) {
	var absent trusttasks.Nullable[int]
	if !absent.IsZero() || trusttasks.Null[int]().IsZero() || trusttasks.Value(0).IsZero() {
		t.Error("IsZero must be true exactly when the member is absent")
	}
}

// A struct with an omitzero Nullable member leaves an absent one out, writes
// null for Null and writes the value for Value — the zero value of T included,
// which is a value like any other.
func TestNullableOmitzeroField(t *testing.T) {
	type probe struct {
		Count trusttasks.Nullable[int] `json:"count,omitzero"`
	}
	for _, c := range []struct {
		name string
		p    probe
		want string
	}{
		{"absent", probe{}, `{}`},
		{"null", probe{Count: trusttasks.Null[int]()}, `{"count":null}`},
		{"value", probe{Count: trusttasks.Value(3)}, `{"count":3}`},
		{"zero value", probe{Count: trusttasks.Value(0)}, `{"count":0}`},
	} {
		got, err := json.Marshal(c.p)
		if err != nil {
			t.Fatalf("%s: %v", c.name, err)
		}
		if string(got) != c.want {
			t.Errorf("%s: marshalled %s, want %s", c.name, got, c.want)
		}
	}
}

// Nullable[T] is comparable when T is: == holds exactly for the same state
// and, for a value, == values. It can be a map key, and a struct of them is
// comparable too.
func TestNullableComparable(t *testing.T) {
	var absent trusttasks.Nullable[string]
	null := trusttasks.Null[string]()
	for _, c := range []struct {
		name string
		a, b trusttasks.Nullable[string]
		want bool
	}{
		{"absent == absent", absent, trusttasks.Nullable[string]{}, true},
		{"null == null", null, trusttasks.Null[string](), true},
		{"value == same value", trusttasks.Value("ops"), trusttasks.Value("ops"), true},
		{"value != other value", trusttasks.Value("ops"), trusttasks.Value("dev"), false},
		{"absent != null", absent, null, false},
		{"null != zero value", null, trusttasks.Value(""), false},
		{"absent != zero value", absent, trusttasks.Value(""), false},
	} {
		if got := c.a == c.b; got != c.want {
			t.Errorf("%s: == is %v", c.name, got)
		}
	}

	// A decoded null equals Null(), and a decoded value equals Value(v).
	var p nullableProbe
	if err := json.Unmarshal([]byte(`{"id":"a","label":null}`), &p); err != nil {
		t.Fatal(err)
	}
	if p.Label != null {
		t.Error("a decoded null is not == Null()")
	}
	if err := json.Unmarshal([]byte(`{"id":"a","label":"ops"}`), &p); err != nil {
		t.Fatal(err)
	}
	if p.Label != trusttasks.Value("ops") {
		t.Error(`a decoded "ops" is not == Value("ops")`)
	}

	seen := map[trusttasks.Nullable[int]]bool{trusttasks.Null[int](): true}
	if !seen[trusttasks.Null[int]()] || seen[trusttasks.Value(0)] {
		t.Error("Nullable is not usable as a map key")
	}
}

// Only Null writes null. A nil slice or map written as a value is empty, so a
// set member cannot widen into null: on acl/* allowedKeys null (like absent)
// means every key and [] means none, and Value(filtered) with nothing left in
// filtered must grant none.
func TestNullableNilSliceAndMapWriteEmpty(t *testing.T) {
	type probe struct {
		Keys trusttasks.Nullable[[]string]          `json:"keys,omitzero"`
		Meta trusttasks.Nullable[map[string]string] `json:"meta,omitzero"`
	}
	var filtered []string
	for _, c := range []struct {
		name string
		p    probe
		want string
	}{
		{"absent", probe{}, `{}`},
		{"null slice", probe{Keys: trusttasks.Null[[]string]()}, `{"keys":null}`},
		{"nil slice", probe{Keys: trusttasks.Value(filtered)}, `{"keys":[]}`},
		{"empty slice", probe{Keys: trusttasks.Value([]string{})}, `{"keys":[]}`},
		{"slice", probe{Keys: trusttasks.Value([]string{"k1"})}, `{"keys":["k1"]}`},
		{"null map", probe{Meta: trusttasks.Null[map[string]string]()}, `{"meta":null}`},
		{"nil map", probe{Meta: trusttasks.Value(map[string]string(nil))}, `{"meta":{}}`},
		{"empty map", probe{Meta: trusttasks.Value(map[string]string{})}, `{"meta":{}}`},
		{"map", probe{Meta: trusttasks.Value(map[string]string{"a": "b"})}, `{"meta":{"a":"b"}}`},
	} {
		got, err := json.Marshal(c.p)
		if err != nil {
			t.Fatalf("%s: %v", c.name, err)
		}
		if string(got) != c.want {
			t.Errorf("%s: marshalled %s, want %s", c.name, got, c.want)
		}
		// What was written reads back with the same meaning: null as null,
		// and a nil slice or map as a value, never as null.
		var back probe
		if err := json.Unmarshal(got, &back); err != nil {
			t.Fatalf("%s: %v", c.name, err)
		}
		if back.Keys.IsNull() != c.p.Keys.IsNull() || back.Keys.IsSet() != c.p.Keys.IsSet() ||
			back.Meta.IsNull() != c.p.Meta.IsNull() || back.Meta.IsSet() != c.p.Meta.IsSet() {
			t.Errorf("%s: %s read back in another state", c.name, got)
		}
	}
	if b, err := trusttasks.Value([]string(nil)).MarshalJSON(); err != nil || string(b) != "[]" {
		t.Errorf("Value(nil slice).MarshalJSON() = %s, %v", b, err)
	}
	if b, err := trusttasks.Value(map[string]int(nil)).MarshalJSON(); err != nil || string(b) != "{}" {
		t.Errorf("Value(nil map).MarshalJSON() = %s, %v", b, err)
	}
}

// The uses the type doc warns against, pinned so the warning stays true: each
// one loses a state without an error, or fails the whole encode.
func TestNullableUnsupportedUses(t *testing.T) {
	// Behind a pointer, null decodes as a nil pointer: the same as absent.
	var boxed struct {
		L *trusttasks.Nullable[string] `json:"l,omitempty"`
	}
	if err := json.Unmarshal([]byte(`{"l":null}`), &boxed); err != nil {
		t.Fatal(err)
	}
	if boxed.L != nil {
		t.Error("a *Nullable decoded null as non-nil; update the type doc")
	}
	// Nested, a null value and null write the same bytes.
	a, _ := json.Marshal(trusttasks.Value(trusttasks.Null[string]()))
	b, _ := json.Marshal(trusttasks.Null[trusttasks.Nullable[string]]())
	if string(a) != "null" || string(b) != "null" {
		t.Errorf("nested Nullable wrote %s and %s; update the type doc", a, b)
	}
	// As a map value, omitzero does not apply, so an absent entry fails.
	if _, err := json.Marshal(map[string]trusttasks.Nullable[string]{"k": {}}); err == nil {
		t.Error("a map holding an absent Nullable marshalled; update the type doc")
	}
}
