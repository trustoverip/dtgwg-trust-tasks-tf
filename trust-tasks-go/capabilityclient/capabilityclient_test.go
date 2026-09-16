package capabilityclient

import (
	"encoding/json"
	"strings"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

const errorType = "https://trusttasks.org/spec/trust-task-error/0.5"

func reply(t *testing.T, typeURI, threadID string, payload map[string]any) *Document {
	t.Helper()
	raw, err := json.Marshal(payload)
	if err != nil {
		t.Fatal(err)
	}
	return &Document{ID: "urn:uuid:reply", Type: typeURI, ThreadID: &threadID, Payload: raw}
}

func payloadField(t *testing.T, doc *Document, key string) any {
	t.Helper()
	var m map[string]any
	if err := json.Unmarshal(doc.Payload, &m); err != nil {
		t.Fatal(err)
	}
	return m[key]
}

func TestBuildersAreAddressedAndTyped(t *testing.T) {
	list := BuildListDocument("did:example:me", "did:example:vtc")
	if slugOf(list.Type) != "governance/capability/list" {
		t.Errorf("list slug = %q", slugOf(list.Type))
	}
	if list.Issuer == nil || *list.Issuer != "did:example:me" {
		t.Errorf("issuer = %v", list.Issuer)
	}
	if payloadField(t, list, "status") != "all" {
		t.Errorf("status = %v", payloadField(t, list, "status"))
	}

	enable := BuildToggleDocument("did:example:me", "did:example:vtc", "git-trust", "0.1", true)
	if slugOf(enable.Type) != "governance/capability/enable" {
		t.Errorf("enable slug = %q", slugOf(enable.Type))
	}
	config, _ := payloadField(t, enable, "config").(map[string]any)
	if config["authority"] != "did:example:vtc" {
		t.Errorf("authority = %v", config["authority"])
	}

	disable := BuildToggleDocument("did:example:me", "did:example:vtc", "git-trust", "0.1", false)
	if slugOf(disable.Type) != "governance/capability/disable" {
		t.Errorf("disable slug = %q", slugOf(disable.Type))
	}

	grant := BuildGitTrustGrant("did:a", "did:r", "did:s", "openvtc")
	if slugOf(grant.Type) != "git-trust/grant" || payloadField(t, grant, "subject") != "did:s" {
		t.Errorf("grant wrong: %s %v", slugOf(grant.Type), payloadField(t, grant, "subject"))
	}

	revoke := BuildGitTrustRevoke("did:a", "did:r", "did:s", "openvtc", "ended")
	if payloadField(t, revoke, "reason") != "ended" {
		t.Errorf("reason = %v", payloadField(t, revoke, "reason"))
	}
	revokeNoReason := BuildGitTrustRevoke("did:a", "did:r", "did:s", "openvtc", "")
	if payloadField(t, revokeNoReason, "reason") != nil {
		t.Error("empty reason should be omitted")
	}
}

func TestBuildMintsFreshIDEachCall(t *testing.T) {
	a := BuildListDocument("did:me", "did:vtc")
	b := BuildListDocument("did:me", "did:vtc")
	if a.ID == b.ID {
		t.Error("ids should differ per call")
	}
	if !strings.HasPrefix(a.ID, "urn:uuid:") || a.IssuedAt == nil {
		t.Errorf("id/issuedAt wrong: %s %v", a.ID, a.IssuedAt)
	}
}

func TestNewAttempt(t *testing.T) {
	previous := BuildGitTrustGrant("did:a", "did:r", "did:s", "openvtc")
	previous.Proof = &tt.Proof{Type: "DataIntegrityProof"}
	next := NewAttempt(previous)
	if next.ID == previous.ID {
		t.Error("new attempt must have a fresh id")
	}
	if next.Proof != nil {
		t.Error("new attempt must clear proof")
	}
	if string(next.Payload) != string(previous.Payload) || next.Type != previous.Type {
		t.Error("new attempt must keep type and payload")
	}
	// An explicit threadId is preserved.
	thread := "urn:thread:1"
	previous.ThreadID = &thread
	if got := NewAttempt(previous); got.ThreadID == nil || *got.ThreadID != thread {
		t.Errorf("threadId not preserved: %v", got.ThreadID)
	}
}

func TestSlugAndResponseVariant(t *testing.T) {
	if slugOf(GitTrustGrantType) != "git-trust/grant" || isResponseVariant(GitTrustGrantType) {
		t.Error("grant request wrong")
	}
	if !isResponseVariant(GitTrustGrantType + "#response") {
		t.Error("response variant not detected")
	}
	if isResponseVariant(GitTrustGrantType + "#request") {
		t.Error("#request is not a response variant")
	}
	if slugOf(errorType) != "trust-task-error" {
		t.Errorf("error slug = %q", slugOf(errorType))
	}
	if slugOf("not a uri") != "" {
		t.Error("a non-Trust-Task URI should yield an empty slug")
	}
}

func TestCorrelationAndRepliesTo(t *testing.T) {
	thread := "urn:thread:1"
	if CorrelationThread(&Document{ID: "urn:doc:1"}) != "urn:doc:1" {
		t.Error("no threadId → id")
	}
	if CorrelationThread(&Document{ID: "urn:doc:1", ThreadID: &thread}) != thread {
		t.Error("threadId wins")
	}
	if !RepliesTo(&Document{ThreadID: &thread}, "urn:thread:1") {
		t.Error("should reply to matching thread")
	}
	if RepliesTo(&Document{}, "urn:thread:1") {
		t.Error("no threadId matches nothing")
	}
}

func TestParseEnvelopeDocument(t *testing.T) {
	doc := reply(t, GitTrustGrantType, "urn:thread:1", map[string]any{})
	body, _ := json.Marshal(doc)
	got, thid, ok := ParseEnvelopeDocument(body)
	if !ok || thid != "urn:thread:1" || got.Type != GitTrustGrantType {
		t.Fatalf("parse failed: %v %q %v", ok, thid, got)
	}
	// No threadId → not a threaded document.
	if _, _, ok := ParseEnvelopeDocument([]byte(`{"id":"x","type":"y"}`)); ok {
		t.Error("a document with no threadId should not parse")
	}
	if _, ok := ParseEnvelopeDocumentFor(body, "urn:thread:1"); !ok {
		t.Error("should match its own thread")
	}
	if _, ok := ParseEnvelopeDocumentFor(body, "other"); ok {
		t.Error("should not match another thread")
	}
}

func TestClassifyGitTrustReply(t *testing.T) {
	success := reply(t, GitTrustGrantType+"#response", "urn:thread:1", map[string]any{})
	if o, ok := ClassifyGitTrustReply(success, "urn:thread:1"); !ok || o.Kind != WriteSuccess {
		t.Errorf("success: %v %v", ok, o.Kind)
	}

	for _, code := range []string{GitTrustAlreadyGrantedCode, GitTrustAlreadyGrantedCodeCamel, GitTrustNotGrantedCode} {
		doc := reply(t, errorType, "urn:thread:1", map[string]any{"code": code})
		if o, ok := ClassifyGitTrustReply(doc, "urn:thread:1"); !ok || o.Kind != WriteIdempotentSuccess {
			t.Errorf("idempotent %s: %v %v", code, ok, o.Kind)
		}
	}

	rej := reply(t, errorType, "urn:thread:1", map[string]any{"code": "notAuthorized", "message": "nope"})
	if o, ok := ClassifyGitTrustReply(rej, "urn:thread:1"); !ok || o.Kind != WriteRejected || o.Code != "notAuthorized" || o.Message != "nope" {
		t.Errorf("rejected: %v %+v", ok, o)
	}

	// Uncorrelated → not an answer.
	other := reply(t, GitTrustGrantType+"#response", "urn:thread:OTHER", map[string]any{})
	if _, ok := ClassifyGitTrustReply(other, "urn:thread:1"); ok {
		t.Error("uncorrelated reply must not be an answer")
	}
}

func TestClassifyGitTrustReplyLegacyFreeText(t *testing.T) {
	doc := reply(t, errorType, "urn:thread:1", map[string]any{"code": "taskFailed", "message": "already_granted: did:s on x"})
	if o, _ := ClassifyGitTrustReply(doc, "urn:thread:1"); o.Kind != WriteRejected {
		t.Error("strict policy must not read the free-text message")
	}
	o, ok := ClassifyGitTrustReplyWithPolicy(doc, "urn:thread:1", ReplyPolicy{AcceptLegacyFreeTextIdempotence: true})
	if !ok || o.Kind != WriteIdempotentSuccess {
		t.Errorf("legacy policy: %v %v", ok, o.Kind)
	}
}

func TestParseCapabilityReplyListing(t *testing.T) {
	doc := reply(t, CapabilityListType+"#response", "urn:thread:1", map[string]any{
		"capabilities": []any{
			map[string]any{
				"enabled":   true,
				"enabledAt": "2026-01-01T00:00:00Z",
				"delegate":  "did:del",
				"manifest":  map[string]any{"capability": "git-trust", "title": "Git Trust", "version": "0.1"},
			},
			map[string]any{"enabled": false, "manifest": map[string]any{"capability": "audit", "version": "1.0"}},
			map[string]any{"enabled": true}, // no manifest — dropped
		},
	})
	r, ok := ParseCapabilityReply(doc, "urn:thread:1")
	if !ok || r.Kind != ReplyListing || len(r.Entries) != 2 {
		t.Fatalf("listing: %v %v %d", ok, r.Kind, len(r.Entries))
	}
	first := r.Entries[0]
	if first.Slug != "git-trust" || first.Title != "Git Trust" || !first.Enabled || first.EnabledAt != "2026-01-01T00:00:00Z" {
		t.Errorf("first entry = %+v", first)
	}
}

func TestParseCapabilityReplyToggledAndError(t *testing.T) {
	toggled := reply(t, CapabilityEnableType+"#response", "urn:thread:1", map[string]any{"capability": "git-trust", "enabled": true})
	if r, ok := ParseCapabilityReply(toggled, "urn:thread:1"); !ok || r.Kind != ReplyToggled || r.Capability != "git-trust" || !r.Enabled {
		t.Errorf("toggled: %v %+v", ok, r)
	}
	err := reply(t, errorType, "urn:thread:1", map[string]any{"code": "notAuthorized"})
	if r, ok := ParseCapabilityReply(err, "urn:thread:1"); !ok || r.Kind != ReplyRejected || r.Code != "notAuthorized" {
		t.Errorf("error: %v %+v", ok, r)
	}
}

func TestParseCapabilityReplyGating(t *testing.T) {
	doc := reply(t, CapabilityListType+"#response", "urn:thread:1", map[string]any{"capabilities": []any{}})
	if _, ok := ParseCapabilityReply(doc, "other"); ok {
		t.Error("wrong thread must not be a reply")
	}
	request := reply(t, CapabilityListType, "urn:thread:1", map[string]any{})
	if _, ok := ParseCapabilityReply(request, "urn:thread:1"); ok {
		t.Error("a request (non-response) is not a reply")
	}
}

func TestParseEnvelopeReply(t *testing.T) {
	listing := reply(t, CapabilityListType+"#response", "urn:thread:9", map[string]any{"capabilities": []any{}})
	body, _ := json.Marshal(listing)
	r, ok := ParseEnvelopeReply(body, "urn:thread:9")
	if !ok || r.Kind != ReplyListing {
		t.Errorf("envelope reply: %v %v", ok, r.Kind)
	}
	if _, ok := ParseEnvelopeReply(body, "wrong"); ok {
		t.Error("wrong thread must not parse")
	}
}
