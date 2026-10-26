package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"sort"
	"strconv"
)

// object is a JSON object that remembers its member order.
//
// encoding/json decodes an object into a map and forgets the order, and the
// order is the whole point here: a struct's fields are emitted in the order of
// its schema's `properties`. A bundle's schema is key-sorted (it is JCS), and
// the authored order is restored from the bundle's `propertyOrder` before any
// type is emitted — see applyPropertyOrder.
type object struct {
	keys []string
	vals map[string]any
}

func newObject() *object { return &object{vals: map[string]any{}} }

// get returns a member and whether it is present. A present JSON null is
// (nil, true), which is how the JavaScript generator's `!== undefined` and
// `??` tests are told apart.
func (o *object) get(key string) (any, bool) {
	if o == nil {
		return nil, false
	}
	v, ok := o.vals[key]
	return v, ok
}

// val returns a member, or nil when it is absent or null — `node.x ?? …`.
func (o *object) val(key string) any {
	v, _ := o.get(key)
	return v
}

func (o *object) set(key string, v any) {
	if _, ok := o.vals[key]; !ok {
		o.keys = append(o.keys, key)
	}
	o.vals[key] = v
}

func (o *object) del(key string) {
	if _, ok := o.vals[key]; !ok {
		return
	}
	delete(o.vals, key)
	for i, k := range o.keys {
		if k == key {
			o.keys = append(o.keys[:i:i], o.keys[i+1:]...)
			break
		}
	}
}

func (o *object) clone() *object {
	c := &object{keys: append([]string(nil), o.keys...), vals: make(map[string]any, len(o.vals))}
	for k, v := range o.vals {
		c.vals[k] = v
	}
	return c
}

// jsKeys is the order JavaScript's Object.keys / Object.entries would give:
// insertion order, except that every canonical array index ("0", "17", …)
// comes first, ascending. The generator this ports iterates objects that way,
// so a property literally named "1" sorts the same here as it does there.
func (o *object) jsKeys() []string {
	if o == nil {
		return nil
	}
	var ints []string
	var rest []string
	for _, k := range o.keys {
		if isArrayIndex(k) {
			ints = append(ints, k)
		} else {
			rest = append(rest, k)
		}
	}
	sort.SliceStable(ints, func(i, j int) bool {
		a, _ := strconv.ParseUint(ints[i], 10, 64)
		b, _ := strconv.ParseUint(ints[j], 10, 64)
		return a < b
	})
	return append(ints, rest...)
}

func isArrayIndex(k string) bool {
	if k == "" || (len(k) > 1 && k[0] == '0') {
		return false
	}
	n, err := strconv.ParseUint(k, 10, 64)
	return err == nil && n < 1<<32-1
}

// decodeOrdered parses JSON into *object, []any, string, json.Number, bool and
// nil, keeping every object's member order.
func decodeOrdered(data []byte) (any, error) {
	dec := json.NewDecoder(bytes.NewReader(data))
	dec.UseNumber()
	v, err := decodeValue(dec)
	if err != nil {
		return nil, err
	}
	if _, err := dec.Token(); err != io.EOF {
		return nil, fmt.Errorf("trailing data after the JSON value")
	}
	return v, nil
}

func decodeValue(dec *json.Decoder) (any, error) {
	tok, err := dec.Token()
	if err != nil {
		return nil, err
	}
	switch t := tok.(type) {
	case json.Delim:
		switch t {
		case '{':
			o := newObject()
			for dec.More() {
				kt, err := dec.Token()
				if err != nil {
					return nil, err
				}
				key, ok := kt.(string)
				if !ok {
					return nil, fmt.Errorf("object key is not a string")
				}
				if _, dup := o.vals[key]; dup {
					return nil, fmt.Errorf("duplicate object member %q", key)
				}
				v, err := decodeValue(dec)
				if err != nil {
					return nil, err
				}
				o.set(key, v)
			}
			if _, err := dec.Token(); err != nil {
				return nil, err
			}
			return o, nil
		case '[':
			arr := []any{}
			for dec.More() {
				v, err := decodeValue(dec)
				if err != nil {
					return nil, err
				}
				arr = append(arr, v)
			}
			if _, err := dec.Token(); err != nil {
				return nil, err
			}
			return arr, nil
		}
		return nil, fmt.Errorf("unexpected delimiter %v", t)
	default:
		return tok, nil
	}
}

// truthy is JavaScript truthiness over a decoded JSON value.
func truthy(v any) bool {
	switch t := v.(type) {
	case nil:
		return false
	case bool:
		return t
	case string:
		return t != ""
	case json.Number:
		f, err := t.Float64()
		return err != nil || f != 0
	default:
		return true
	}
}

// jsonEqual reports whether two decoded values are the same JSON, member order
// aside.
func jsonEqual(a, b any) bool {
	switch x := a.(type) {
	case *object:
		y, ok := b.(*object)
		if !ok || len(x.vals) != len(y.vals) {
			return false
		}
		for k, v := range x.vals {
			w, ok := y.vals[k]
			if !ok || !jsonEqual(v, w) {
				return false
			}
		}
		return true
	case []any:
		y, ok := b.([]any)
		if !ok || len(x) != len(y) {
			return false
		}
		for i := range x {
			if !jsonEqual(x[i], y[i]) {
				return false
			}
		}
		return true
	case json.Number:
		y, ok := b.(json.Number)
		if !ok {
			return false
		}
		if x == y {
			return true
		}
		fx, e1 := x.Float64()
		fy, e2 := y.Float64()
		return e1 == nil && e2 == nil && fx == fy
	default:
		return a == b
	}
}
