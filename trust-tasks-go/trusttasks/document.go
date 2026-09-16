// Package trusttasks is the Trust Tasks framework runtime for Go — the SPEC.md
// §7.2 consumer pipeline, the document envelope, and the transport seam.
//
// Hand-written, unlike everything under ../specs, which is generated from the
// spec registry by scripts/build-go-bindings.mjs. Kept in the same module so a
// consumer gets the types and the checks that make them meaningful from one
// `go get`, as trust-tasks-rs and @openvtc/trust-tasks do.
//
// The entry point is [ConsumeInbound].
//
// # Parity
//
// This package mirrors trust-tasks-rs and the _runtime directory of
// @openvtc/trust-tasks check for check. A Go consumer and a Rust or TypeScript
// one must reach the same verdict on the same document, or the reference
// implementations disagree about what conforms. Where a signature differs it is
// because Go's type system forced it, and the reason is written at the
// difference.
//
// # Dependencies
//
// None beyond the standard library, deliberately: the cryptosuite and the JSON
// Schema engine are the consumer's to choose, so [ProofVerifier] and
// [PayloadValidator] are interfaces this module does not implement.
package trusttasks

import (
	"encoding/json"
	"time"
)

// Proof is a W3C Data Integrity Proof object (SPEC.md §4.7). Opaque to the
// framework, which never inspects it beyond noticing that it is present.
type Proof struct {
	Type               string `json:"type"`
	Cryptosuite        string `json:"cryptosuite"`
	VerificationMethod string `json:"verificationMethod"`
	Created            string `json:"created"`
	ProofPurpose       string `json:"proofPurpose"`
	ProofValue         string `json:"proofValue"`

	// Extra carries any further members the cryptosuite defines, preserved on
	// round-trip so a proof this framework does not understand still verifies
	// against the bytes it arrived as.
	Extra map[string]json.RawMessage `json:"-"`
}

// MarshalJSON folds [Proof.Extra] back into the object.
func (p Proof) MarshalJSON() ([]byte, error) {
	type plain Proof
	return mergeExtra(plain(p), p.Extra)
}

// UnmarshalJSON collects members this struct does not name into [Proof.Extra].
func (p *Proof) UnmarshalJSON(data []byte) error {
	type plain Proof
	var base plain
	if err := json.Unmarshal(data, &base); err != nil {
		return err
	}
	extra, err := splitExtra(data, proofKnownMembers)
	if err != nil {
		return err
	}
	*p = Proof(base)
	p.Extra = extra
	return nil
}

var proofKnownMembers = map[string]struct{}{
	"type": {}, "cryptosuite": {}, "verificationMethod": {},
	"created": {}, "proofPurpose": {}, "proofValue": {},
}

// CeremonyPrev references a predecessor step (SPEC.md §4.11).
type CeremonyPrev struct {
	// ID is the predecessor document's id (§4.3), so a verifier can locate what
	// the digest is over.
	ID string `json:"id"`

	// DigestMultibase is a multibase-encoded multihash over the predecessor
	// document, salted per enactment — salted because many steps carry near-zero-
	// entropy payloads, and an unsalted digest over one is a confirmation oracle
	// for any party handed it, which a chain does by design.
	DigestMultibase string `json:"digestMultibase"`
}

// Ceremony records that a document is a step of a Trust Ceremony — a flow
// composed of several Trust Tasks (SPEC.md §4.11).
//
// Optional in every sense: no specification declares anything about ceremonies,
// a document without it is fully conforming, and a consumer that does not
// implement ceremonies processes the document unchanged. Ignoring it is always
// safe, because §4.11.4 forbids deriving authority from it — there is nothing a
// ceremony-aware consumer may do that an unaware one omits.
type Ceremony struct {
	// Enactment identifies one run of a ceremony. Globally unique and never
	// reused, on the same terms as the document id and unlike ThreadID —
	// evidence about a flow needs a stable anchor.
	Enactment string `json:"enactment"`

	// Step names this step within the ceremony. The step name, not the Type URI,
	// is the step's identity: one Type URI may serve several steps whose meaning
	// differs by context.
	Step string `json:"step"`

	// Definition is the ceremony definition this step is enacted under (§6.7).
	// Requires DefinitionDigest.
	Definition *string `json:"definition,omitempty"`

	// DefinitionDigest is a multibase-multihash over the JCS canonicalization of
	// the definition. It pins the definition by content rather than by name: a
	// URI alone would leave the flow's rules mutable by whoever controls it,
	// retroactively and for every enactment already performed.
	DefinitionDigest *string `json:"definitionDigest,omitempty"`

	// ParentEnactment is the enactment containing this one. One level,
	// navigation only — but the pointers chain, so nesting is unbounded.
	ParentEnactment *string `json:"parentEnactment,omitempty"`

	// Round distinguishes repetitions of the same step by the same party. Absent
	// means 1.
	Round *int64 `json:"round,omitempty"`

	// Terminal marks a step that ends the enactment; a set with none so marked
	// is a prefix, not a completed flow.
	Terminal *bool `json:"terminal,omitempty"`

	// Prev names the steps this one follows. A set, so concurrent branches are
	// expressible.
	Prev []CeremonyPrev `json:"prev,omitempty"`
}

// Document is a single Trust Task document, per SPEC.md §4.2.
//
// Optional members are pointers rather than zero values because every framework
// check in §7.2 turns on whether a member is *present*: a recipient of "" and an
// absent recipient lead to different verdicts, and collapsing them would make
// this runtime disagree with the Rust and TypeScript ones.
//
// Timestamps are strings, not time.Time, for the same parity reason. A malformed
// expiresAt must be reportable as malformedRequest by [ValidateBasic]; parsing
// it at unmarshal time would instead fail the decode, and the consumer would
// have no document to build an error response about.
type Document[P any] struct {
	// ID is globally unique to this instance (§4.3).
	ID string `json:"id"`

	// ThreadID correlates this document with others in the same exchange (§4.9).
	ThreadID *string `json:"threadId,omitempty"`

	// ParentThreadID is the threadId of the exchange containing this one, where
	// this exchange is conducted inside another (§4.9.2).
	//
	// A navigation aid. It records one level of containment and does not change
	// which exchange attests an event — §4.9.1 governs that, and holds whether or
	// not this member is present. Like ThreadID it carries no normative
	// validation semantics: a consumer MUST NOT reject a document on the basis of
	// parentThreadId alone.
	ParentThreadID *string `json:"parentThreadId,omitempty"`

	// Ceremony records that this document is a step of a Trust Ceremony (§4.11).
	//
	// Carries no authority: membership is an assertion by the issuer, not a
	// verified fact, so every authorization decision still rests on Issuer, Proof
	// and local policy (§4.11.4).
	Ceremony *Ceremony `json:"ceremony,omitempty"`

	// Type is the Type URI identifying specification and version (§4.4).
	Type string `json:"type"`

	// Issuer is the VID of the party responsible for the content (§4.8).
	Issuer *string `json:"issuer,omitempty"`

	// Recipient is the VID of the party expected to act on the document (§4.8).
	Recipient *string `json:"recipient,omitempty"`

	// IssuedAt records when the document was produced (§4.2), as RFC 3339.
	IssuedAt *string `json:"issuedAt,omitempty"`

	// ExpiresAt is the instant after which the document is no longer valid
	// (§4.2), as RFC 3339.
	ExpiresAt *string `json:"expiresAt,omitempty"`

	// Payload is the task-specific body, whose structure is defined by the
	// specification Type names.
	Payload P `json:"payload"`

	// Context is the optional JSON-LD context (§4.6). Held as raw JSON because
	// §4.6 permits a string, an array, or an object and the framework never
	// interprets it.
	Context json.RawMessage `json:"@context,omitempty"`

	// Proof is the optional Data Integrity proof binding the document to its
	// issuer (§4.7).
	Proof *Proof `json:"proof,omitempty"`

	// Extra holds top-level members this struct does not name. §7.2 says a
	// consumer SHOULD preserve but MUST NOT act upon them, and §7.1 asks a
	// forwarding producer to carry them through — so they survive a round trip
	// here rather than being dropped by the decoder.
	Extra map[string]json.RawMessage `json:"-"`
}

var documentKnownMembers = map[string]struct{}{
	"id": {}, "threadId": {}, "parentThreadId": {}, "ceremony": {}, "type": {},
	"issuer": {}, "recipient": {}, "issuedAt": {}, "expiresAt": {},
	"payload": {}, "@context": {}, "proof": {},
}

// MarshalJSON folds [Document.Extra] back into the top-level object, so an
// unrecognized member survives the round trip §7.1 asks for.
//
// A member in Extra that collides with one this struct names is dropped: the
// named member is authoritative, and emitting both would produce a document with
// a duplicate key.
func (d Document[P]) MarshalJSON() ([]byte, error) {
	type plain Document[P]
	return mergeExtra(plain(d), d.Extra)
}

// UnmarshalJSON collects unrecognized top-level members into [Document.Extra].
func (d *Document[P]) UnmarshalJSON(data []byte) error {
	type plain Document[P]
	var base plain
	if err := json.Unmarshal(data, &base); err != nil {
		return err
	}
	extra, err := splitExtra(data, documentKnownMembers)
	if err != nil {
		return err
	}
	*d = Document[P](base)
	d.Extra = extra
	return nil
}

// mergeExtra marshals value, then folds any member of extra that value did not
// already emit into the resulting object.
func mergeExtra(value any, extra map[string]json.RawMessage) ([]byte, error) {
	encoded, err := json.Marshal(value)
	if err != nil {
		return nil, err
	}
	if len(extra) == 0 {
		return encoded, nil
	}
	var members map[string]json.RawMessage
	if err := json.Unmarshal(encoded, &members); err != nil {
		return nil, err
	}
	for name, raw := range extra {
		if _, taken := members[name]; !taken {
			members[name] = raw
		}
	}
	return json.Marshal(members)
}

// splitExtra returns the members of data that known does not name.
func splitExtra(data []byte, known map[string]struct{}) (map[string]json.RawMessage, error) {
	var members map[string]json.RawMessage
	if err := json.Unmarshal(data, &members); err != nil {
		return nil, err
	}
	var extra map[string]json.RawMessage
	for name, raw := range members {
		if _, isKnown := known[name]; isKnown {
			continue
		}
		if extra == nil {
			extra = make(map[string]json.RawMessage, 1)
		}
		extra[name] = raw
	}
	return extra, nil
}

// InResponseTo names the Trust Task document an [ErrorPayload] reports on (§8.2).
type InResponseTo struct {
	// TypeURI is the reported-on document's type, including any #request /
	// #response fragment — that fragment is what tells a consumer which variant's
	// semantics apply.
	TypeURI string `json:"typeUri"`

	// ID is the reported-on document's id. Globally unique and never reused
	// (§4.3), so it names one instance where threadId names an exchange.
	//
	// Omitted under identityMismatch: per §8.1 the response goes to the
	// transport-authenticated sender rather than the in-band issuer, and that
	// party did not necessarily compose the document.
	ID *string `json:"id,omitempty"`
}

// ErrorPayload is the payload of an error response (SPEC.md §8.2).
type ErrorPayload struct {
	Code string `json:"code"`

	// InResponseTo identifies the document this error reports on (§8.2).
	//
	// threadId correlates the exchange for a party that saw the request and
	// identifies nothing to anyone else, so without this a retained error names
	// neither the specification the failure occurred under nor the instance. The
	// builders below populate it.
	InResponseTo *InResponseTo `json:"inResponseTo,omitempty"`

	Message *string `json:"message,omitempty"`

	// Retryable is not a pointer: §8.2 makes it REQUIRED, and omitting it would
	// leave a producer unable to tell a final refusal from an invitation to
	// re-send.
	Retryable bool `json:"retryable"`

	RetryAfter *string                    `json:"retryAfter,omitempty"`
	Details    map[string]json.RawMessage `json:"details,omitempty"`
}

// ErrorResponse is a trust-task-error document.
type ErrorResponse = Document[ErrorPayload]

// SpecPolicy is the per-specification declaration a consumer needs to apply SPEC
// §7.2 items 5b, 7 and 8. Generated packages export it as Spec / ResponseSpec.
type SpecPolicy struct {
	TypeURI string

	// IsBearer records §4.8.3 — the specification opts out of the §4.8.2
	// audience-binding rule.
	IsBearer bool

	// IsProofRequired records §7.3 item 8 — proofRequirement is REQUIRED.
	IsProofRequired bool

	// IsRecipientRequired records §7.3 item 5 — the party filling recipient is
	// REQUIRED.
	IsRecipientRequired bool

	// IsIssuedAtRequired records §7.3 item 17 — issuedAtRequirement is REQUIRED,
	// raising §4.2's issuedAt SHOULD to a MUST for this specification's
	// documents. A specification defining a consequential Trust Task (§2) must
	// declare it, so that §7.2 item 11's duplicate-execution record always has a
	// window to sit in.
	//
	// This is the specification's requirement, published in the registry —
	// distinct from a consumer's own freshness posture, which is chosen at the
	// call site and applied to every document.
	IsIssuedAtRequired bool

	// PayloadSchema is this variant's payload schema as JSON text (§7.2 item 2),
	// carried so a consumer has something to validate against. Empty only for a
	// hand-written policy that omits it.
	PayloadSchema string
}

// RejectReason records why a consumer rejected a document, and the §8.3 code it
// maps to.
type RejectReason struct {
	Code       StandardCode
	Message    string
	Retryable  bool
	RetryAfter *string
	Details    map[string]json.RawMessage
}

// TrustTaskErrorTypeURI is the Type URI a consumer emits error responses under.
//
// The single source of truth for the emitted trust-task-error version on this
// side. Build error responses with [RejectWith] / [RejectWithRecipient], or read
// the URI from here; do not spell it out again.
//
// The Rust counterpart is trust_task_error_type_uri() in trust-tasks-rs and the
// TypeScript one is TRUST_TASK_ERROR_TYPE_URI in @openvtc/trust-tasks, both kept
// equal to this value. scripts/check-bindings-conformance.mjs enforces it.
//
// 0.5 because this runtime populates the inResponseTo member of §8.2 and can
// emit idConflict (§8.3), which is absent from 0.3's code enum and does not match
// its extended-code pattern — a document carrying it would not validate as 0.3.
// Per §5.2 forward-minor compatibility a 0.3 consumer SHOULD accept it.
const TrustTaskErrorTypeURI = "https://trusttasks.org/spec/trust-task-error/0.5"

// ValidateBasic applies SPEC §7.2 items 4 and 5a — expiry and wrong-recipient.
//
// This is not the full §7.2 check. Items 1–3 (framework schema, payload schema,
// unknown type) belong to the caller's parse and dispatch; items 5b, 7 and 8 need
// the specification's policy and live in [EnforceSpecPolicy]. [ConsumeInbound]
// bundles 4–8.
//
// Returns nil when the document passes.
func ValidateBasic[P any](doc *Document[P], now time.Time, myVID string) *RejectReason {
	if doc.ExpiresAt != nil {
		expiresAt, err := parseTimestamp(*doc.ExpiresAt)
		if err != nil {
			return &RejectReason{
				Code:    CodeMalformedRequest,
				Message: "expiresAt is not a valid RFC 3339 timestamp",
			}
		}
		// §4.2 / §7.2 item 4: inclusive bound — now >= expiresAt is expired.
		if !expiresAt.After(now) {
			return &RejectReason{
				Code:    CodeExpired,
				Message: "document expired at " + *doc.ExpiresAt,
			}
		}
	}

	if doc.Recipient != nil && *doc.Recipient != myVID {
		return &RejectReason{
			Code:    CodeWrongRecipient,
			Message: "in-band recipient does not identify this consumer",
		}
	}

	return nil
}

// EnforceAudienceBinding applies SPEC §7.2 item 8 — a proof-bearing document on a
// non-bearer specification must carry an in-band recipient, so the proof binds
// the audience as well as the content (§4.8.2).
func EnforceAudienceBinding[P any](doc *Document[P], spec SpecPolicy) *RejectReason {
	if doc.Proof != nil && doc.Recipient == nil && !spec.IsBearer {
		return &RejectReason{
			Code: CodeMalformedRequest,
			Message: "proof present with no in-band recipient on a non-bearer specification " +
				"(SPEC §4.8.2 audience binding)",
		}
	}
	return nil
}

// EnforceSpecPolicy applies the policy-driven subset of SPEC §7.2 — items 5b, 7
// clause A, and 8, plus §7.3 item 17's issuedAt requirement.
//
// Single source of truth for the flag-driven checks, so a binding-specific
// pipeline and [ConsumeInbound] cannot diverge on the check set. Ordering matches
// trust-tasks-rs enforce_spec_policy: recipient, then proof, then issuedAt (§7.3
// item 17), then audience binding.
func EnforceSpecPolicy[P any](doc *Document[P], spec SpecPolicy) *RejectReason {
	if doc.Recipient == nil && spec.IsRecipientRequired {
		return &RejectReason{
			Code: CodeMalformedRequest,
			Message: "specification declares recipient REQUIRED but the document carries no " +
				"in-band recipient",
		}
	}
	if doc.Proof == nil && spec.IsProofRequired {
		return &RejectReason{
			Code:    CodeProofRequired,
			Message: "specification declares proof REQUIRED but the document carries none",
		}
	}
	if doc.IssuedAt == nil && spec.IsIssuedAtRequired {
		return &RejectReason{
			Code:    CodeMalformedRequest,
			Message: "issuedAt is required by this Trust Task specification (SPEC §7.3 item 17)",
		}
	}
	return EnforceAudienceBinding(doc, spec)
}

// Clock returns the timestamp a built response is stamped with. Injectable so
// tests are deterministic.
type Clock func() string

// SystemClock stamps responses with the current time in RFC 3339, to the second.
func SystemClock() string {
	return time.Now().UTC().Format(time.RFC3339)
}

func (c Clock) or() Clock {
	if c == nil {
		return SystemClock
	}
	return c
}

// RejectWithRecipient builds the error response for request, addressed to an
// explicit recipient.
//
// Prefer [RejectWith] for ordinary refusals. This form exists for
// identityMismatch, where §8.1 forbids addressing the contested in-band issuer —
// see [Reject], which applies that rule.
//
// A nil recipient produces a response with no recipient member.
func RejectWithRecipient[P any](
	request *Document[P],
	id string,
	payload ErrorPayload,
	recipient *string,
	clock Clock,
) *ErrorResponse {
	// §8.2 — name the document this error reports on, so it means something to a
	// party that did not see the request. Filled here rather than left to the
	// caller because the builder is the only place that reliably has the
	// originating document in hand; a caller-supplied value is kept.
	if payload.InResponseTo == nil {
		about := &InResponseTo{TypeURI: request.Type}
		// §8.1/§8.2 — under identityMismatch the response is addressed to the
		// transport-authenticated sender, not the in-band issuer. That party did
		// not necessarily compose the document, so its id is not echoed back.
		if NormalizeCode(payload.Code) != string(CodeIdentityMismatch) {
			about.ID = strPtr(request.ID)
		}
		payload.InResponseTo = about
	}

	// §4.9: continue the thread, falling back to the request's own id.
	threadID := request.ThreadID
	if threadID == nil {
		threadID = strPtr(request.ID)
	}

	issuedAt := clock.or()()
	response := &ErrorResponse{
		ID:        id,
		ThreadID:  threadID,
		Type:      TrustTaskErrorTypeURI,
		Issuer:    request.Recipient,
		Recipient: recipient,
		IssuedAt:  &issuedAt,
		Payload:   payload,
	}
	// §4.9.2 — the whole exchange shares one parent, so the error response stays
	// inside the same enclosing exchange.
	if request.ParentThreadID != nil {
		response.ParentThreadID = request.ParentThreadID
	}
	return response
}

// RejectWith builds the error response for request, addressed to its original
// producer.
//
// Not safe under a rejection that contests the in-band identity: it copies
// request.Issuer into Recipient, which under identityMismatch is the contested
// value §8.1 says MUST NOT be addressed. Use [Reject] for those.
func RejectWith[P any](
	request *Document[P],
	id string,
	payload ErrorPayload,
	clock Clock,
) *ErrorResponse {
	return RejectWithRecipient(request, id, payload, request.Issuer, clock)
}

// ToErrorPayload turns a [RejectReason] into the §8.2 payload it maps to.
func ToErrorPayload(reason RejectReason) ErrorPayload {
	payload := ErrorPayload{
		Code:      string(reason.Code),
		Message:   strPtr(reason.Message),
		Retryable: reason.Retryable,
	}
	if reason.RetryAfter != nil {
		payload.RetryAfter = reason.RetryAfter
	}
	if reason.Details != nil {
		payload.Details = reason.Details
	}
	return payload
}

// RespondWith builds the success-response document for request, per SPEC §4.4.1 —
// the request's Type URI with the #response fragment, the parties swapped, and
// the thread continued.
func RespondWith[P, R any](
	request *Document[P],
	id string,
	payload R,
	clock Clock,
) *Document[R] {
	threadID := request.ThreadID
	if threadID == nil {
		threadID = strPtr(request.ID)
	}
	issuedAt := clock.or()()
	response := &Document[R]{
		ID:        id,
		ThreadID:  threadID,
		Type:      bareTypeURI(request.Type) + "#response",
		Issuer:    request.Recipient,
		Recipient: request.Issuer,
		IssuedAt:  &issuedAt,
		Payload:   payload,
	}
	// §4.9.2 — the whole exchange shares one parent.
	if request.ParentThreadID != nil {
		response.ParentThreadID = request.ParentThreadID
	}
	return response
}

// bareTypeURI strips any fragment from a Type URI.
func bareTypeURI(typeURI string) string {
	for i := 0; i < len(typeURI); i++ {
		if typeURI[i] == '#' {
			return typeURI[:i]
		}
	}
	return typeURI
}

// parseTimestamp reads an RFC 3339 timestamp, accepting the fractional-second
// form every producer in this ecosystem emits.
func parseTimestamp(s string) (time.Time, error) {
	return time.Parse(time.RFC3339, s)
}

func strPtr(s string) *string { return &s }
