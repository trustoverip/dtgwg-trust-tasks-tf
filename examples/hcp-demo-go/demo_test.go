// The demo's own tests: hermetic (no network, no files outside the examples),
// deterministic (fixed keys, ids and clock), and run with `go test ./...`.
// ../hcp-demo-ts/SCENARIO.md "Tests every port has" lists them; every port has
// the same ones.
package main

import (
	"encoding/json"
	"maps"
	"os"
	"strings"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"

	query "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/entries/query/v0_1"
	grantrequest "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/grants/request/v0_1"
	revoked "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/grants/revoked/v0_1"
)

// golden is the transcript every language's port prints, owned by the
// TypeScript demo.
const golden = "../hcp-demo-ts/transcript.golden.txt"

// token is the grant's token, as the provider minted it.
const token = "hcp_at_demo_0001"

const grantRequestType = grantrequest.TypeURI + "#request"

// afterDemo is a world after the whole demo has run.
func afterDemo() (*World, Result) {
	w := NewWorld()
	return w, RunDemo(w)
}

// afterGrant is a world after steps 1 and 2: the grant is issued and active.
func afterGrant() (w *World, threadID, grantID string) {
	w = NewWorld()
	StepDiscover(w)
	threadID, grantID = StepGrantRequest(w)
	return w, threadID, grantID
}

type replyDoc struct {
	Type    string          `json:"type"`
	Payload json.RawMessage `json:"payload"`
}

func decodeReply(t *testing.T, reply json.RawMessage) replyDoc {
	t.Helper()
	var doc replyDoc
	if err := json.Unmarshal(reply, &doc); err != nil {
		t.Fatal(err)
	}
	return doc
}

func code(t *testing.T, reply json.RawMessage) string {
	t.Helper()
	var p tt.ErrorPayload
	if err := json.Unmarshal(decodeReply(t, reply).Payload, &p); err != nil {
		t.Fatal(err)
	}
	return p.Code
}

// asMap is a payload as a JSON object, to drop or reorder its members.
func asMap(t *testing.T, payload any) map[string]any {
	t.Helper()
	raw, err := json.Marshal(payload)
	if err != nil {
		t.Fatal(err)
	}
	var m map[string]any
	if err := json.Unmarshal(raw, &m); err != nil {
		t.Fatal(err)
	}
	return m
}

func Test01TranscriptMatchesTheSharedGolden(t *testing.T) {
	want, err := os.ReadFile(golden)
	if err != nil {
		t.Fatal(err)
	}
	if got := RunDemo(nil).Transcript; got != string(want) {
		t.Errorf("transcript differs from %s at byte %d", golden, firstDifference(got, string(want)))
	}
}

func firstDifference(a, b string) int {
	for i := 0; i < len(a) && i < len(b); i++ {
		if a[i] != b[i] {
			return i
		}
	}
	return min(len(a), len(b))
}

func Test02QueryAfterRevokeFailsWithInvalidToken(t *testing.T) { // HCP 6.2.2
	r := RunDemo(nil).PostRevokeError
	if r.Type != tt.TrustTaskErrorTypeURI || r.Payload.Code != "hcp:invalidToken" {
		t.Errorf("got %s %q, want %s hcp:invalidToken", r.Type, r.Payload.Code, tt.TrustTaskErrorTypeURI)
	}
}

func Test03DocumentAlteredAfterSigningIsProofInvalid(t *testing.T) {
	w, _ := afterDemo()
	C, P := w.ContextClient, w.Provider
	signed := Request(w, C, P, query.TypeURI+"#request", QueryPayload(ManagementToken), "")
	var doc map[string]any
	if err := json.Unmarshal(signed, &doc); err != nil {
		t.Fatal(err)
	}
	doc["payload"].(map[string]any)["query"] = "Tell me everything."
	tampered, err := json.Marshal(doc)
	if err != nil {
		t.Fatal(err)
	}
	if got := code(t, w.Exchange(C, P, tampered)); got != "proofInvalid" {
		t.Errorf("got %q, want proofInvalid", got)
	}
}

func Test04SchemaInvalidDocumentIsMalformedRequest(t *testing.T) {
	w, _ := afterDemo()
	C, P := w.ContextClient, w.Provider
	noPurpose := asMap(t, QueryPayload(ManagementToken))
	delete(noPurpose, "purpose")
	if got := code(t, w.Exchange(C, P, Request(w, C, P, query.TypeURI+"#request", noPurpose, ""))); got != "malformedRequest" {
		t.Errorf("got %q, want malformedRequest", got)
	}
}

func Test05AfterTheNoticeTheClientHoldsNoTokenAndDeduplicates(t *testing.T) {
	w, r := afterDemo()
	P, C := w.Provider, w.ContextClient
	if _, ok := w.Wallet[r.GrantID]; ok {
		t.Error("the wallet still holds the grant")
	}
	want := map[string]string{r.Notice.Payload.NoticeID: r.GrantID}
	if !maps.Equal(w.ProcessedNotices, want) {
		t.Fatalf("processed notices %v, want %v", w.ProcessedNotices, want)
	}
	// A later delivery attempt: a new document, the same threadId and payload (noticeId included).
	ack := decodeReply(t, w.Exchange(P, C, Request(w, P, C, revoked.TypeURI+"#request", r.Notice.Payload, *r.Notice.ThreadID)))
	if ack.Type != revoked.ResponseTypeURI || string(ack.Payload) != "{}" {
		t.Errorf("got %s %s, want %s {}", ack.Type, ack.Payload, revoked.ResponseTypeURI)
	}
	if len(w.ProcessedNotices) != 1 {
		t.Errorf("processed notices %v, want one entry", w.ProcessedNotices)
	}
}

func Test06TheNoticeAloneMakesTheClientDropTheGrant(t *testing.T) {
	w, _, grantID := afterGrant()
	StepRevoke(w, grantID)
	if _, ok := w.Wallet[grantID]; !ok {
		t.Fatal("the wallet lost the grant before the notice")
	}
	StepNotice(w, grantID)
	if _, ok := w.Wallet[grantID]; ok {
		t.Error("the wallet still holds the grant after the notice")
	}
}

func Test07TheGrantsTokenFromAnotherVIDIsPermissionDenied(t *testing.T) {
	w, _, _ := afterGrant()
	M, P := w.Management, w.Provider
	reply := w.Exchange(M, P, Request(w, M, P, query.TypeURI+"#request", QueryPayload(token), ""))
	if typ := decodeReply(t, reply).Type; typ != tt.TrustTaskErrorTypeURI {
		t.Fatalf("got %s, want %s", typ, tt.TrustTaskErrorTypeURI)
	}
	if got := code(t, reply); got != "permissionDenied" {
		t.Errorf("got %q, want permissionDenied", got)
	}
}

func Test08AGrantRequestFromAnUnregisteredIssuerIsPermissionDenied(t *testing.T) {
	w, _ := afterDemo()
	M, P := w.Management, w.Provider
	if got := code(t, w.Exchange(M, P, Request(w, M, P, grantRequestType, Ask(), ""))); got != "permissionDenied" {
		t.Errorf("got %q, want permissionDenied", got)
	}
	if len(w.held) != 1 {
		t.Errorf("%d holds, want 1", len(w.held))
	}
}

func Test09AResubmissionThatDiffersIsMalformedRequest(t *testing.T) {
	w, threadID, _ := afterGrant()
	C, P := w.ContextClient, w.Provider
	ask := Ask()
	shorter := int64(3600)
	ask.ExpiresIn = &shorter
	if got := code(t, w.Exchange(C, P, Request(w, C, P, grantRequestType, ask, threadID))); got != "malformedRequest" {
		t.Errorf("got %q, want malformedRequest", got)
	}
}

func Test10WithinTheHourAResubmissionGetsTheSameGrant(t *testing.T) {
	w, threadID, grantID := afterGrant()
	C, P := w.ContextClient, w.Provider
	// The same payload with its members in another order: the provider compares JCS forms.
	m := asMap(t, Ask())
	var reordered strings.Builder
	reordered.WriteString("{")
	for i, k := range []string{"subjectHint", "expiresIn", "access"} {
		v, err := json.Marshal(m[k])
		if err != nil {
			t.Fatal(err)
		}
		if i > 0 {
			reordered.WriteString(",")
		}
		reordered.WriteString(`"` + k + `":` + string(v))
	}
	reordered.WriteString("}")
	// The signer writes members in JCS order, so the reordered payload goes in
	// after signing; the proof is over the JCS form and still verifies.
	var doc map[string]json.RawMessage
	if err := json.Unmarshal(Request(w, C, P, grantRequestType, Ask(), threadID), &doc); err != nil {
		t.Fatal(err)
	}
	doc["payload"] = json.RawMessage(reordered.String())
	sent, err := json.Marshal(doc)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(sent), `"payload":{"subjectHint":`) {
		t.Fatal("the payload sent is not reordered")
	}
	reply := decodeReply(t, w.Exchange(C, P, sent))
	if reply.Type != grantrequest.ResponseTypeURI {
		t.Fatalf("got %s, want %s", reply.Type, grantrequest.ResponseTypeURI)
	}
	var r grantrequest.Response
	if err := json.Unmarshal(reply.Payload, &r); err != nil {
		t.Fatal(err)
	}
	if r.Grant.GrantID != grantID {
		t.Errorf("grantId %s, want %s", r.Grant.GrantID, grantID)
	}
	if len(w.grants) != 1 {
		t.Errorf("%d grants, want 1", len(w.grants))
	}
}

func Test11AResubmissionAfterTheHourIsPermissionDenied(t *testing.T) {
	w, threadID, _ := afterGrant()
	C, P := w.ContextClient, w.Provider
	w.Clock.Advance(AfterApprovalWindow + 1)
	if got := code(t, w.Exchange(C, P, Request(w, C, P, grantRequestType, Ask(), threadID))); got != "permissionDenied" {
		t.Errorf("got %q, want permissionDenied", got)
	}
}

func Test12AResubmissionOnceRevokedIsPermissionDenied(t *testing.T) {
	w, r := afterDemo()
	C, P := w.ContextClient, w.Provider
	if got := code(t, w.Exchange(C, P, Request(w, C, P, grantRequestType, Ask(), r.ThreadID))); got != "permissionDenied" {
		t.Errorf("got %q, want permissionDenied", got)
	}
	if len(w.grants) != 1 {
		t.Errorf("%d grants, want 1", len(w.grants))
	}
}

func Test13AReplyIsUsedOnlyFromThePartyAskedOnTheThreadAndSigned(t *testing.T) {
	w := NewWorld()
	P, C := w.Provider, w.ContextClient
	build := func(code string, signed bool) json.RawMessage {
		doc := map[string]any{
			"id": "urn:uuid:00000000-0000-4000-8000-0000000000f0", "type": tt.TrustTaskErrorTypeURI,
			"issuer": P.VID, "recipient": C.VID, "issuedAt": "2026-11-25T10:00:01Z", "threadId": "t",
			"payload": map[string]any{"code": code},
		}
		if signed {
			doc["proof"] = map[string]any{"type": "DataIntegrityProof"} // presence only: the proof itself is verified separately
		}
		raw, err := json.Marshal(doc)
		if err != nil {
			t.Fatal(err)
		}
		return raw
	}
	if err := CheckReply(build("proofInvalid", false), P, "t"); err != nil { // a library-built error, unsigned
		t.Errorf("unsigned proofInvalid refused: %v", err)
	}
	if err := CheckReply(build("permissionDenied", true), P, "t"); err != nil {
		t.Errorf("signed permissionDenied refused: %v", err)
	}
	refused := []struct {
		name   string
		reply  json.RawMessage
		from   *Party
		thread string
		want   string
	}{
		{"unsigned permissionDenied", build("permissionDenied", false), P, "t", "no proof"},
		{"issuer not the party asked", build("proofInvalid", false), C, "t", "issuer"},
		{"another threadId", build("proofInvalid", false), P, "other", "threadId"},
	}
	for _, c := range refused {
		if err := CheckReply(c.reply, c.from, c.thread); err == nil || !strings.Contains(err.Error(), c.want) {
			t.Errorf("%s: got %v, want an error naming %q", c.name, err, c.want)
		}
	}
}

func Test14AnotherRegisteredClientOnAHeldThreadIsPermissionDenied(t *testing.T) {
	w := NewWorld()
	C, M, P := w.ContextClient, w.Management, w.Provider
	// A second registered client, so the check on the hold's issuer is reached
	// (rule 1 passes).
	w.registeredClients[M.VID] = registration{name: "Alex's wallet", operator: grantrequest.Named{ID: "wallet.example", Name: "Wallet Example Ltd"}}
	first := Request(w, C, P, grantRequestType, Ask(), "")
	if held := replyOf[json.RawMessage](w.Exchange(C, P, first)); held.Type != "https://trusttasks.org/spec/trust-task-next-step/0.1" {
		t.Fatalf("got %q, want a next step", held.Type)
	}
	threadID := *replyOf[json.RawMessage](first).ThreadID
	w.PersonApproves(threadID)
	if got := code(t, w.Exchange(M, P, Request(w, M, P, grantRequestType, Ask(), threadID))); got != "permissionDenied" {
		t.Errorf("got %q, want permissionDenied", got)
	}
	h := w.held[threadID]
	if h == nil || h.issuer != C.VID || h.state != approved {
		t.Errorf("hold %+v, want the Context Client's, approved", h)
	}
	if len(w.grants) != 0 {
		t.Errorf("%d grants, want 0", len(w.grants))
	}
}
