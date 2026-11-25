package smoke

// A required member whose schema admits null must keep its key when it is
// null. vta/contexts/update-did 1.2 requires `did` and reads `"did": null` as
// "leave the context with no DID of its own"; a document without `did` fails
// the schema's `required`. The generated field is a pointer (nil is null), and
// its tag must not carry omitempty, or a nil DID drops the key.
//
// It lives here because this directory is hand-written and kept when
// scripts/build-go-bindings.mjs regenerates the tree.

import (
	"bytes"
	"encoding/json"
	"testing"

	vtacontextsupdatedidv1_2 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/vta/contexts/update-did/v1_2"
)

func TestRequiredNullableMemberMarshalsNull(t *testing.T) {
	got, err := json.Marshal(vtacontextsupdatedidv1_2.Payload{ID: "ctx/payments"})
	if err != nil {
		t.Fatal(err)
	}
	if want := `{"id":"ctx/payments","did":null}`; string(got) != want {
		t.Errorf("marshalled %s, want %s", got, want)
	}
}

func TestRequiredNullableMemberRoundTrips(t *testing.T) {
	for _, wire := range []string{
		`{"id":"ctx/payments","did":null}`,
		`{"id":"ctx/payments","did":"did:web:payments.example"}`,
	} {
		var p vtacontextsupdatedidv1_2.Payload
		if err := json.Unmarshal([]byte(wire), &p); err != nil {
			t.Fatalf("%s: %v", wire, err)
		}
		got, err := json.Marshal(p)
		if err != nil {
			t.Fatalf("%s: %v", wire, err)
		}
		if !bytes.Equal(got, []byte(wire)) {
			t.Errorf("round trip of %s gave %s", wire, got)
		}
	}
}
