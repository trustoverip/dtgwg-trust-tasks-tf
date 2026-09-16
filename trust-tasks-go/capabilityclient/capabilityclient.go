// Package capabilityclient holds client-side wire helpers for the capability
// Trust Task families — governance/capability/* (enable / disable / list a
// community capability) and git-trust/* (grant / revoke commit-signing trust).
//
// It owns the documents, not a transport: it builds request documents, parses
// inbound envelope replies, and classifies them. A capability producer (a
// community service) and a management UI share this wire layer, so they cannot
// drift on the contract. It is the Go port of the Rust
// trust-tasks-capability-client crate.
//
// Signing is deliberately not here — attach a Data Integrity proof with
// trust-tasks-go/proof (over the document minus its proof member,
// eddsa-jcs-2022) — so this package stays free of any crypto dependency.
package capabilityclient

import (
	"crypto/rand"
	"encoding/json"
	"strings"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// Type URIs and extended error codes — the wire contract a capability producer
// and a management UI share.
const (
	// TrustTaskEnvelopeType is the trust-tasks-didcomm binding envelope type a
	// registry's DIDComm handler listens for.
	TrustTaskEnvelopeType = "https://trusttasks.org/binding/didcomm/0.1/envelope"

	// CapabilityListType and the toggle types are the governance/capability/*
	// type URIs.
	CapabilityListType    = "https://trusttasks.org/spec/governance/capability/list/0.1"
	CapabilityEnableType  = "https://trusttasks.org/spec/governance/capability/enable/0.1"
	CapabilityDisableType = "https://trusttasks.org/spec/governance/capability/disable/0.1"

	// GitTrustGrantType and GitTrustRevokeType are the git-trust/* type URIs.
	GitTrustGrantType  = "https://trusttasks.org/spec/git-trust/grant/0.1"
	GitTrustRevokeType = "https://trusttasks.org/spec/git-trust/revoke/0.1"

	// GitTrustAlreadyGrantedCode is the extended error code git-trust/grant
	// declares for "an active grant already exists" (SPEC §8.5) — the control
	// surface for idempotent success, never decided from the free-text message.
	GitTrustAlreadyGrantedCode = "git-trust/grant:already_granted"
	// GitTrustNotGrantedCode is the git-trust/revoke counterpart.
	GitTrustNotGrantedCode = "git-trust/revoke:not_granted"

	// The lowerCamelCase spellings the registry may normalise to (SPEC §4.10
	// rule 4). Accepting both means that normalisation is not a flag day.
	GitTrustAlreadyGrantedCodeCamel = "git-trust/grant:alreadyGranted"
	GitTrustNotGrantedCodeCamel     = "git-trust/revoke:notGranted"
)

// Document is a capability Trust Task: a document whose payload is arbitrary JSON.
type Document = tt.Document[json.RawMessage]

func ptr[T any](v T) *T { return &v }

// freshID mints a fresh document id — one per attempt, never reused; see
// [NewAttempt].
func freshID() string {
	var b [16]byte
	_, _ = rand.Read(b[:])
	b[6] = (b[6] & 0x0f) | 0x40
	b[8] = (b[8] & 0x3f) | 0x80
	const hexd = "0123456789abcdef"
	buf := make([]byte, 36)
	for i, p := range []int{0, 2, 4, 6, 9, 11, 14, 16, 19, 21, 24, 26, 28, 30, 32, 34} {
		buf[p] = hexd[b[i]>>4]
		buf[p+1] = hexd[b[i]&0x0f]
	}
	buf[8], buf[13], buf[18], buf[23] = '-', '-', '-', '-'
	return "urn:uuid:" + string(buf)
}

func nowRFC3339() string { return time.Now().UTC().Format(time.RFC3339) }

// BuildDocument builds a capability Trust Task addressed issuerDID → recipientDID.
//
// It mints a fresh id and stamps issuedAt on every call, so each built document
// is a new attempt in the sense of SPEC §8.4. To re-send one you have already
// built, see [NewAttempt] — the choice between resending the identical document
// (a §8.4 retry the consumer absorbs) and minting a new one is enforced by the
// consumer's §7.2 item-11 record. payload is marshalled to the document's
// payload; a value that does not marshal yields an error.
func BuildDocument(issuerDID, recipientDID, typeURI string, payload any) (*Document, error) {
	raw, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}
	return &Document{
		ID:        freshID(),
		Type:      typeURI,
		Issuer:    ptr(issuerDID),
		Recipient: ptr(recipientDID),
		IssuedAt:  ptr(nowRFC3339()),
		Payload:   raw,
	}, nil
}

// NewAttempt returns a new attempt at the request previous carried: the same
// addressing, type and payload under a fresh id and issuedAt, and no proof.
//
// This is the counterpart of a SPEC §8.4 retry, and the two are not
// interchangeable. A retry is a bit-for-bit resend of previous (same id) that
// the consumer's §7.2 item-11 record absorbs. A new attempt is a different
// document — anything that changes the bytes makes it one, including a
// re-stamped issuedAt or a re-signed proof over identical content — and MUST
// carry a fresh id, or the consumer rejects it with idConflict.
//
// proof is cleared because it committed to the previous id and issuedAt; sign
// the returned document before sending it. Where previous opened its own
// exchange (no threadId), the new attempt opens a new one — wait on
// [CorrelationThread] of the returned document, not of previous.
func NewAttempt(previous *Document) *Document {
	next := *previous
	next.ID = freshID()
	next.IssuedAt = ptr(nowRFC3339())
	next.Proof = nil
	return &next
}

// BuildListDocument builds a governance/capability/list request (status "all").
func BuildListDocument(issuerDID, vtcDID string) *Document {
	doc, _ := BuildDocument(issuerDID, vtcDID, CapabilityListType, map[string]any{"status": "all"})
	return doc
}

// BuildToggleDocument builds a governance/capability/enable or /disable request.
// On enable, config.authority defaults to the community's own DID.
func BuildToggleDocument(issuerDID, vtcDID, slug, version string, enable bool) *Document {
	var doc *Document
	if enable {
		doc, _ = BuildDocument(issuerDID, vtcDID, CapabilityEnableType, map[string]any{
			"capability": slug,
			"version":    version,
			"config":     map[string]any{"authority": vtcDID},
		})
	} else {
		doc, _ = BuildDocument(issuerDID, vtcDID, CapabilityDisableType, map[string]any{"capability": slug})
	}
	return doc
}

// BuildGitTrustGrant builds a git-trust/grant: grant subjectDID commit-signing
// trust for resource (an org or org/repo slug).
func BuildGitTrustGrant(authorityDID, registryDID, subjectDID, resource string) *Document {
	doc, _ := BuildDocument(authorityDID, registryDID, GitTrustGrantType, map[string]any{
		"subject":  subjectDID,
		"resource": resource,
	})
	return doc
}

// BuildGitTrustRevoke builds a git-trust/revoke. An empty reason is omitted.
func BuildGitTrustRevoke(authorityDID, registryDID, subjectDID, resource, reason string) *Document {
	payload := map[string]any{"subject": subjectDID, "resource": resource}
	if reason != "" {
		payload["reason"] = reason
	}
	doc, _ := BuildDocument(authorityDID, registryDID, GitTrustRevokeType, payload)
	return doc
}

// --- correlation + envelope parsing -----------------------------------------

// slugOf returns a Type URI's slug (any #request/#response fragment removed), or
// "" when the URI is not a Trust Task Type URI.
func slugOf(typeURI string) string {
	slug, err := tt.SlugFromTypeURI(typeURI)
	if err != nil {
		return ""
	}
	return slug
}

// isResponseVariant reports whether a Type URI names the success-response variant.
func isResponseVariant(typeURI string) bool {
	return strings.HasSuffix(typeURI, "#response")
}

// CorrelationThread is the thread an exchange started by doc is correlated by:
// its own threadId, or its id where it opens the exchange (SPEC §4.9's
// fallback). Hold this from the moment you send a request; every reply-parsing
// function wants it as expectedThreadID.
func CorrelationThread(doc *Document) string {
	if doc.ThreadID != nil {
		return *doc.ThreadID
	}
	return doc.ID
}

// RepliesTo reports whether reply is threaded to expectedThreadID — SPEC §4.9
// correlation, and the precondition for acting on any reply. A reply with no
// threadId matches nothing.
func RepliesTo(reply *Document, expectedThreadID string) bool {
	return reply.ThreadID != nil && *reply.ThreadID == expectedThreadID
}

// ParseEnvelopeDocument parses a DIDComm envelope body into the threaded Trust
// Task document it carries. ok is false when the body is not a threaded Trust
// Task document. The returned threadID is a dispatch key, not a check —
// correlate before acting, with [ParseEnvelopeDocumentFor] or your own
// outstanding map.
func ParseEnvelopeDocument(body json.RawMessage) (doc *Document, threadID string, ok bool) {
	var d Document
	if err := json.Unmarshal(body, &d); err != nil {
		return nil, "", false
	}
	if d.ID == "" || d.Type == "" || d.ThreadID == nil {
		return nil, "", false
	}
	return &d, *d.ThreadID, true
}

// ParseEnvelopeDocumentFor parses a DIDComm envelope body into the document it
// carries, only if that document is threaded to expectedThreadID. ok is false
// for both "not a Trust Task document" and "belongs to another exchange".
func ParseEnvelopeDocumentFor(body json.RawMessage, expectedThreadID string) (*Document, bool) {
	doc, _, ok := ParseEnvelopeDocument(body)
	if !ok || !RepliesTo(doc, expectedThreadID) {
		return nil, false
	}
	return doc, true
}
