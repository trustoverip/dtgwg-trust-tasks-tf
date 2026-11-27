package smoke

// An optional member whose schema admits null has three meanings, and the
// generated field keeps all three. acl/update 0.3 reads an absent `label` as
// "leave it unchanged", `"label": null` as "clear it" and a string as the new
// label; `expiresAt` the same way, where null makes the entry permanent. The
// field is a trusttasks.Nullable tagged omitzero, so absent writes nothing,
// null writes null and a value writes the value.
//
// vtc/registry/records/list 0.2's Record has two optional nullable booleans
// (`recognized`, `authorized`), where false is a value distinct from both null
// and absent.
//
// It lives here because this directory is hand-written and kept when
// scripts/build-go-bindings.mjs regenerates the tree.

import (
	"bytes"
	"encoding/json"
	"testing"

	aclupdatev0_3 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/acl/update/v0_3"
	vtcregistryrecordslistv0_2 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/vtc/registry/records/list/v0_2"
	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

func roundTrip[T any](t *testing.T, wire string) T {
	t.Helper()
	var v T
	if err := json.Unmarshal([]byte(wire), &v); err != nil {
		t.Fatalf("%s: %v", wire, err)
	}
	got, err := json.Marshal(v)
	if err != nil {
		t.Fatalf("%s: %v", wire, err)
	}
	if !bytes.Equal(got, []byte(wire)) {
		t.Errorf("round trip of %s gave %s", wire, got)
	}
	return v
}

func TestOptionalNullableMembersRoundTrip(t *testing.T) {
	for _, wire := range []string{
		`{"subject":"did:example:alice"}`,
		`{"subject":"did:example:alice","label":null}`,
		`{"subject":"did:example:alice","label":"ops"}`,
		`{"subject":"did:example:alice","expiresAt":null}`,
		`{"subject":"did:example:alice","label":null,"expiresAt":"2027-01-01T00:00:00Z"}`,
		`{"subject":"did:example:alice","stepUp":{"approver":"did:example:bob","require":null}}`,
	} {
		roundTrip[aclupdatev0_3.Payload](t, wire)
	}
	for _, wire := range []string{
		`{"entityId":"e","authorityId":"a","action":"x","resource":"r","recordType":"t"}`,
		`{"entityId":"e","authorityId":"a","action":"x","resource":"r","recordType":"t","recognized":null}`,
		`{"entityId":"e","authorityId":"a","action":"x","resource":"r","recordType":"t","recognized":false,"authorized":true}`,
	} {
		roundTrip[vtcregistryrecordslistv0_2.Record](t, wire)
	}
}

func TestOptionalNullableMemberDecodesEachState(t *testing.T) {
	absent := roundTrip[aclupdatev0_3.Payload](t, `{"subject":"did:example:alice"}`)
	if absent.Label.IsSet() {
		t.Error("an absent label decoded as set")
	}
	cleared := roundTrip[aclupdatev0_3.Payload](t, `{"subject":"did:example:alice","label":null}`)
	if !cleared.Label.IsNull() {
		t.Error(`"label": null did not decode as null`)
	}
	set := roundTrip[aclupdatev0_3.Payload](t, `{"subject":"did:example:alice","label":"ops"}`)
	if v, ok := set.Label.Get(); !ok || v != "ops" {
		t.Errorf("label decoded as %q, %v", v, ok)
	}
	r := roundTrip[vtcregistryrecordslistv0_2.Record](t,
		`{"entityId":"e","authorityId":"a","action":"x","resource":"r","recordType":"t","recognized":false}`)
	if v, ok := r.Recognized.Get(); !ok || v {
		t.Errorf("recognized decoded as %v, %v, want false, true", v, ok)
	}
	// Nullable[bool] compares with ==, so a decoded member can be checked
	// against the state it should be in directly.
	if r.Recognized != trusttasks.Value(false) || r.Authorized.IsSet() {
		t.Errorf("recognized %v, authorized set %v", r.Recognized, r.Authorized.IsSet())
	}
}

// A Go client can now send acl/update's "clear the label" as well as "leave
// it unchanged".
func TestOptionalNullableMemberEncodesEachState(t *testing.T) {
	for _, c := range []struct {
		p    aclupdatev0_3.Payload
		want string
	}{
		{aclupdatev0_3.Payload{Subject: "did:example:alice"}, `{"subject":"did:example:alice"}`},
		{aclupdatev0_3.Payload{Subject: "did:example:alice", Label: trusttasks.Null[string]()}, `{"subject":"did:example:alice","label":null}`},
		{aclupdatev0_3.Payload{Subject: "did:example:alice", Label: trusttasks.Value("ops")}, `{"subject":"did:example:alice","label":"ops"}`},
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

// A non-nullable optional member is unchanged: still a pointer. Its schema
// forbids null; the generated types never enforced that (PayloadSchemaJSON is
// for a caller's validator), and this change does not loosen anything.
func TestNonNullableOptionalMemberIsStillAPointer(t *testing.T) {
	var p aclupdatev0_3.Payload
	var _ *string = p.Reason
	var _ *aclupdatev0_3.PurposeScope = p.Purposes
}
