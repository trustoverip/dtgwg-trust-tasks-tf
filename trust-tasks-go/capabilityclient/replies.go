package capabilityclient

import (
	"encoding/json"
	"strings"
)

// --- git-trust write replies (grant/revoke producers) -----------------------

// WriteOutcomeKind classifies a git-trust write reply.
type WriteOutcomeKind int

const (
	// WriteSuccess: the #response document acknowledged the write.
	WriteSuccess WriteOutcomeKind = iota
	// WriteIdempotentSuccess: rejected because the end state already holds
	// (already_granted / not_granted). Load-bearing for redelivery-safe
	// writers: the write is done, not failed.
	WriteIdempotentSuccess
	// WriteRejected: any other rejection.
	WriteRejected
)

// WriteOutcome is the classification of a git-trust write reply. Code and
// Message are set only for [WriteRejected]; Message is empty when the reply
// carried none.
type WriteOutcome struct {
	Kind    WriteOutcomeKind
	Code    string
	Message string
}

// ReplyPolicy is how much a caller is willing to infer from a non-conforming
// peer. The zero value infers nothing: an outcome is decided from the error
// code alone.
type ReplyPolicy struct {
	// AcceptLegacyFreeTextIdempotence — DEPRECATED, opt-in compatibility only —
	// also treats a taskFailed whose free-text message contains
	// "already_granted:" or "not_granted:" as [WriteIdempotentSuccess].
	//
	// SPEC §8.2 makes message non-normative free text, so deciding an outcome
	// from it hinges on wording the emitting service may reword, translate or
	// drop. Enable it only while a specific peer still emits the free-text form;
	// the correct fix is on the emitting side (send the extended codes SPEC §8.5
	// provides).
	AcceptLegacyFreeTextIdempotence bool
}

func errorCodeAndMessage(payload json.RawMessage) (code, message string) {
	var p struct {
		Code    string `json:"code"`
		Message string `json:"message"`
	}
	_ = json.Unmarshal(payload, &p)
	code = p.Code
	if code == "" {
		code = "unknown"
	}
	return code, p.Message
}

func isIdempotentCode(code string) bool {
	switch code {
	case GitTrustAlreadyGrantedCode, GitTrustNotGrantedCode,
		GitTrustAlreadyGrantedCodeCamel, GitTrustNotGrantedCodeCamel:
		return true
	default:
		return false
	}
}

// ClassifyGitTrustReply classifies the reply to a git-trust/grant or
// git-trust/revoke write, with the strict policy. See
// [ClassifyGitTrustReplyWithPolicy].
func ClassifyGitTrustReply(doc *Document, expectedThreadID string) (WriteOutcome, bool) {
	return ClassifyGitTrustReplyWithPolicy(doc, expectedThreadID, ReplyPolicy{})
}

// ClassifyGitTrustReplyWithPolicy classifies a git-trust write reply.
//
// expectedThreadID is [CorrelationThread] of the document you sent. A reply
// threaded to anything else yields ok=false — "not an answer to this request" —
// rather than an outcome: acting on an uncorrelated reply lets whichever
// document arrives next decide the fate of a write it has nothing to do with.
//
// Idempotent success is keyed on the SPEC §8.5 extended error code, never on the
// free-text message; policy.AcceptLegacyFreeTextIdempotence is the deprecated
// compatibility path.
func ClassifyGitTrustReplyWithPolicy(doc *Document, expectedThreadID string, policy ReplyPolicy) (WriteOutcome, bool) {
	// SPEC §4.9: correlation comes first.
	if !RepliesTo(doc, expectedThreadID) {
		return WriteOutcome{}, false
	}
	slug := slugOf(doc.Type)
	if slug == "trust-task-error" {
		code, message := errorCodeAndMessage(doc.Payload)
		if isIdempotentCode(code) {
			return WriteOutcome{Kind: WriteIdempotentSuccess}, true
		}
		// DEPRECATED: pre-extended-code peers signalled idempotence in the
		// free-text message under a bare taskFailed. SPEC §8.2 makes message
		// non-normative, so this is a string match on a field nobody promised to
		// keep stable — opt-in only.
		if policy.AcceptLegacyFreeTextIdempotence && code == "taskFailed" {
			if strings.Contains(message, "already_granted:") || strings.Contains(message, "not_granted:") {
				return WriteOutcome{Kind: WriteIdempotentSuccess}, true
			}
		}
		return WriteOutcome{Kind: WriteRejected, Code: code, Message: message}, true
	}
	if isResponseVariant(doc.Type) && (slug == "git-trust/grant" || slug == "git-trust/revoke") {
		return WriteOutcome{Kind: WriteSuccess}, true
	}
	return WriteOutcome{}, false
}

// --- governance/capability replies (management UIs) -------------------------

// CapabilitySummary is one capability entry as rendered by a management UI. The
// optional string fields are empty when absent; Version defaults to "?".
type CapabilitySummary struct {
	Slug      string
	Title     string
	Version   string
	Enabled   bool
	EnabledAt string
	Delegate  string
	// Manifest is the full manifest, for a detail view.
	Manifest json.RawMessage
}

// CapabilityReplyKind classifies a governance/capability/* reply.
type CapabilityReplyKind int

const (
	// ReplyListing: a list response.
	ReplyListing CapabilityReplyKind = iota
	// ReplyToggled: an enable/disable acknowledgement.
	ReplyToggled
	// ReplyRejected: a trust-task-error document.
	ReplyRejected
)

// CapabilityReply is the classification of a governance/capability/* reply.
// Which fields are set depends on Kind.
type CapabilityReply struct {
	Kind CapabilityReplyKind
	// Entries is set for ReplyListing.
	Entries []CapabilitySummary
	// Capability and Enabled are set for ReplyToggled.
	Capability string
	Enabled    bool
	// Code and Message are set for ReplyRejected.
	Code    string
	Message string
}

func summaryOf(entry json.RawMessage) (CapabilitySummary, bool) {
	var e struct {
		Enabled   bool            `json:"enabled"`
		EnabledAt string          `json:"enabledAt"`
		Delegate  string          `json:"delegate"`
		Manifest  json.RawMessage `json:"manifest"`
	}
	if err := json.Unmarshal(entry, &e); err != nil || len(e.Manifest) == 0 {
		return CapabilitySummary{}, false
	}
	var m struct {
		Capability string `json:"capability"`
		Title      string `json:"title"`
		Version    string `json:"version"`
	}
	if err := json.Unmarshal(e.Manifest, &m); err != nil || m.Capability == "" {
		return CapabilitySummary{}, false
	}
	version := m.Version
	if version == "" {
		version = "?"
	}
	return CapabilitySummary{
		Slug:      m.Capability,
		Title:     m.Title,
		Version:   version,
		Enabled:   e.Enabled,
		EnabledAt: e.EnabledAt,
		Delegate:  e.Delegate,
		Manifest:  e.Manifest,
	}, true
}

// ParseCapabilityReply classifies a governance/capability/* reply document. ok
// is false when it is not part of this family, or is not threaded to
// expectedThreadID.
func ParseCapabilityReply(doc *Document, expectedThreadID string) (CapabilityReply, bool) {
	if !RepliesTo(doc, expectedThreadID) {
		return CapabilityReply{}, false
	}
	slug := slugOf(doc.Type)
	if slug == "trust-task-error" {
		code, message := errorCodeAndMessage(doc.Payload)
		return CapabilityReply{Kind: ReplyRejected, Code: code, Message: message}, true
	}
	if !isResponseVariant(doc.Type) {
		return CapabilityReply{}, false
	}
	switch slug {
	case "governance/capability/list":
		var p struct {
			Capabilities []json.RawMessage `json:"capabilities"`
		}
		_ = json.Unmarshal(doc.Payload, &p)
		entries := make([]CapabilitySummary, 0, len(p.Capabilities))
		for _, e := range p.Capabilities {
			if s, ok := summaryOf(e); ok {
				entries = append(entries, s)
			}
		}
		return CapabilityReply{Kind: ReplyListing, Entries: entries}, true
	case "governance/capability/enable", "governance/capability/disable":
		var p struct {
			Capability string `json:"capability"`
			Enabled    bool   `json:"enabled"`
		}
		_ = json.Unmarshal(doc.Payload, &p)
		return CapabilityReply{Kind: ReplyToggled, Capability: p.Capability, Enabled: p.Enabled}, true
	default:
		return CapabilityReply{}, false
	}
}

// ParseEnvelopeReply parses an inbound envelope body directly into a reply to
// the request threaded expectedThreadID — the entry point for a UI's inbound
// dispatch, which holds only the body value. ok is false when the body is not a
// governance/capability/* reply, or belongs to a different exchange.
func ParseEnvelopeReply(body json.RawMessage, expectedThreadID string) (CapabilityReply, bool) {
	doc, ok := ParseEnvelopeDocumentFor(body, expectedThreadID)
	if !ok {
		return CapabilityReply{}, false
	}
	return ParseCapabilityReply(doc, expectedThreadID)
}
