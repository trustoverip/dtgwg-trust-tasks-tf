package smoke

// Two properties of trusttasks.Nullable on real generated packages.
//
// vtc/join-requests/list 0.2's JoinRequest.Decision is a Nullable over a
// struct. encoding/json matches the struct's member names without regard to
// case, so the runtime's member-name check has to enter the Nullable: a
// decision carrying both "code" and "Code" means one thing to Rust, TS, Dart
// and a proof verifier (the exact-case "code") and must not decode in Go as
// the other.
//
// acl/grant 0.1's ACLEntry.AllowedKeys is a Nullable[[]string] where absent
// and null mean every key in scope and [] means none. All three must survive
// a round trip, and a nil slice set as a value must write [], never null.
//
// It lives here because this directory is hand-written and kept when
// scripts/build-go-bindings.mjs regenerates the tree.

import (
	"encoding/json"
	"strings"
	"testing"

	aclgrantv0_1 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/acl/grant/v0_1"
	vtcjoinrequestslistv0_2 "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs/vtc/join-requests/list/v0_2"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

const joinRequest = `"id":"jr-1","applicantDid":"did:example:alice","vp":{},` +
	`"submittedAt":"2026-01-01T00:00:00Z","status":"rejected"`

func joinRequestsResponse(decision string) string {
	return `{"id":"resp-1","type":"` + vtcjoinrequestslistv0_2.ResponseTypeURI + `",` +
		`"recipient":"did:example:alice","threadId":"req-1",` +
		`"payload":{"items":[{` + joinRequest + `,"decision":` + decision + `}]}}`
}

func TestNullableDecisionRefusesACaseVariantMember(t *testing.T) {
	for name, decision := range map[string]string{
		"beside the exact member":              `{"code":"approved","Code":"refused","decidedAt":"2026-01-01T00:00:00Z"}`,
		"alone":                                `{"CODE":"refused","decidedAt":"2026-01-01T00:00:00Z"}`,
		"a second member beside the exact one": `{"code":"refused","reason":"x","Reason":"y","decidedAt":"2026-01-01T00:00:00Z"}`,
	} {
		t.Run(name, func(t *testing.T) {
			payload := `{"items":[{` + joinRequest + `,"decision":` + decision + `}]}`
			if err := tt.CheckMemberNames[vtcjoinrequestslistv0_2.Response]([]byte(payload)); err == nil ||
				!strings.Contains(err.Error(), "case-sensitive") {
				t.Errorf("CheckMemberNames = %v, want a case-sensitive refusal", err)
			}
			var doc tt.Document[vtcjoinrequestslistv0_2.Response]
			err := json.Unmarshal([]byte(joinRequestsResponse(decision)), &doc)
			if err == nil || !strings.Contains(err.Error(), "case-sensitive") {
				t.Errorf("Document decode = %v, want a case-sensitive refusal", err)
			}
		})
	}
}

func TestNullableDecisionDecodesEachState(t *testing.T) {
	for _, c := range []struct {
		decision string
		null     bool
		code     string
	}{
		{`null`, true, ""},
		{`{"code":"refused","decidedAt":"2026-01-01T00:00:00Z"}`, false, "refused"},
	} {
		var doc tt.Document[vtcjoinrequestslistv0_2.Response]
		if err := json.Unmarshal([]byte(joinRequestsResponse(c.decision)), &doc); err != nil {
			t.Fatalf("%s: %v", c.decision, err)
		}
		d := doc.Payload.Items[0].Decision
		if d.IsNull() != c.null {
			t.Errorf("%s: IsNull %v", c.decision, d.IsNull())
		}
		if v, ok := d.Get(); ok != !c.null || v.Code != c.code {
			t.Errorf("%s: Get = %+v, %v", c.decision, v, ok)
		}
	}
}

// Absent and null both mean every key in scope; [] means none. Each keeps its
// spelling through a decode and an encode, and a nil slice set as a value
// writes [] — the producer that filtered every key away grants none.
func TestACLAllowedKeysRoundTrip(t *testing.T) {
	const entry = `"subject":"did:example:alice","role":"admin"`
	for _, c := range []struct {
		wire string
		set  bool
		null bool
		keys []string
	}{
		{`{` + entry + `}`, false, false, nil},
		{`{` + entry + `,"allowedKeys":null}`, true, true, nil},
		{`{` + entry + `,"allowedKeys":[]}`, true, false, []string{}},
		{`{` + entry + `,"allowedKeys":["k1"]}`, true, false, []string{"k1"}},
	} {
		e := roundTrip[aclgrantv0_1.ACLEntry](t, c.wire)
		if e.AllowedKeys.IsSet() != c.set || e.AllowedKeys.IsNull() != c.null {
			t.Errorf("%s: IsSet %v, IsNull %v", c.wire, e.AllowedKeys.IsSet(), e.AllowedKeys.IsNull())
		}
		keys, ok := e.AllowedKeys.Get()
		if ok != (c.set && !c.null) || len(keys) != len(c.keys) || (ok && keys == nil) {
			t.Errorf("%s: Get = %#v, %v", c.wire, keys, ok)
		}
	}

	var filtered []string
	got, err := json.Marshal(aclgrantv0_1.ACLEntry{Subject: "did:example:alice", Role: "admin",
		AllowedKeys: tt.Value(filtered)})
	if err != nil {
		t.Fatal(err)
	}
	if want := `{"subject":"did:example:alice","role":"admin","allowedKeys":[]}`; string(got) != want {
		t.Errorf("Value(nil) marshalled %s, want %s", got, want)
	}
}
