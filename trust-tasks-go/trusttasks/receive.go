package trusttasks

// The inbound body every message binding shares (#1110): a transport that has
// opened an envelope and authenticated its sender holds the document's bytes and
// who sent them, and from there the work is the same whatever the envelope was —
// read the bytes as a document, refuse what cannot be one, read a later version
// forward along its declared extension, decode the payload, run SPEC §7.2, and
// say what to send back. The TSP and DIDComm consumers each carried a copy of it,
// and the copies had already drifted apart. [ConsumeReceived] is the one copy; a
// binding keeps only the opening of its envelope and its own checks (DIDComm's
// §3.1 thread cross-check is a [ReceiveOptions.BindingCheck]).

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/json"
	"errors"
	"fmt"
	"time"
)

// ErrNoAuthenticatedSender is returned by [ConsumeReceived] when its transport
// authenticated no sender (DeriveParties().Issuer is nil).
//
// ConsumeReceived answers a body that does not parse as a document — and a
// document refused before the identity cross-check — by addressing the members
// it could read. That is safe only when the transport will deliver the answer
// to the party it authenticated as the sender, whatever the document claims; on
// a transport that authenticates nobody it would let any sender direct an error
// response at a third party. Such a transport uses [ConsumeInbound].
var ErrNoAuthenticatedSender = errors.New("trusttasks: ConsumeReceived needs a transport that authenticated the sender")

// Wire messages [ConsumeReceived] refuses with before the document reaches
// [ConsumeInbound]. Constants, like the others in this package, so a peer's
// test can match them and no detail of the decoder's error reaches the wire.
const (
	// MalformedDocumentWireMessage refuses a body that does not parse as a
	// Trust Task document.
	MalformedDocumentWireMessage = "body is not a well-formed Trust Task document"
	// CaseFoldedMembersWireMessage refuses a document holding two members whose
	// names differ only by case.
	CaseFoldedMembersWireMessage = "document has two members whose names differ only by case"
	// PayloadMismatchWireMessage refuses a payload the caller's decoder rejects.
	PayloadMismatchWireMessage = "payload does not match the specification"
)

// ReceiveOptions are the inputs to [ConsumeReceived] other than the document,
// its transport, the specification and the two functions. The policies are
// [ConsumeOptions]' own, and required for the same reason (see
// [ErrMissingOption]); a binding maps its consumer's configuration onto them.
type ReceiveOptions struct {
	// ProofPolicy is §7.2 item 7 clause B. The proof is verified over the
	// document as received, never over the decoded or forward-read copy.
	ProofPolicy ProofPolicy

	// PayloadPolicy performs §7.2 item 2. Required.
	PayloadPolicy PayloadPolicy

	// Checks performs §7.2 item 4 (freshness) and item 11 (duplicate execution).
	// Required — [ConsequentialChecks] or [NotConsequentialChecks].
	Checks ConsumeChecks

	// Extends is the lookup forward reading follows (SPEC §5.2 "Declared
	// extension"). Nil means [RegistryExtends].
	Extends ExtendsLookup

	// MyVID is this consumer's own VID, for the §7.2 item 5 recipient check.
	// Empty means the identity the transport opened the message as
	// (DeriveParties().Recipient).
	MyVID string

	// Now is the instant the checks are made at. Zero means time.Now().
	Now time.Time

	// NewErrorID mints error-response ids. Required. [NewURNUUID] will do.
	NewErrorID func() string

	// Clock overrides the response issuedAt clock, for deterministic tests. Nil
	// means [SystemClock].
	Clock Clock

	// BindingCheck is the binding's own check of the document — DIDComm's §3.1
	// thread cross-check, for one. It sees the envelope members as received
	// (the payload undecoded) and runs after the body has parsed and before
	// anything else, so a document it refuses never reaches the replay record.
	// A non-nil reason is answered to the authenticated sender, as every other
	// refusal is. Nil runs none.
	BindingCheck func(doc *Document[json.RawMessage]) *RejectReason
}

// ReceivedOutcome is what [ConsumeReceived] made of one received document.
type ReceivedOutcome[R any] struct {
	// Outcome is the §7.2 outcome, exactly as [ConsumeInbound] defines it.
	Outcome ConsumeOutcome[R]

	// Reply is the document to send back to the authenticated sender, as JSON,
	// or nil when nothing is to be sent: a fire-and-forget success, a suppressed
	// identityMismatch (§8.1), or a duplicate with no retained response (§7.1).
	Reply json.RawMessage
}

// ConsumeReceived runs a document received over a transport that authenticated
// its sender through SPEC §7.2, from the bytes the transport delivered.
//
// received is the document exactly as it arrived (the opened envelope's body).
// transport reports the authenticated sender and the identity the message was
// opened as. decode reads the payload of spec's version from JSON — one built on
// [CheckMemberNames] refuses a member encoding/json would match by case alone.
// handler returns the response document (build it with [RespondWith]), nil for
// a fire-and-forget specification, or a [Refusal] error.
//
// In order:
//
//  1. A body that does not parse as a document is answered malformedRequest,
//     addressed from whichever routing members did parse — the sender is
//     authenticated, so it is told, not dropped.
//  2. opts.BindingCheck, if any.
//  3. A document holding two members whose names differ only by case, anywhere
//     in it, is refused malformedRequest: encoding/json would read them as one
//     member and every other reader as two.
//  4. [ReadForSpecWith] under opts.Extends: another specification is
//     unsupportedType, a version of this one whose extends chain does not reach
//     spec's unsupportedVersion; a later version whose chain does is read as
//     spec's, its payload pruned to the members spec's schema knows.
//  5. decode; an error is malformedRequest.
//  6. [ConsumeInbound], with received as [ConsumeOptions.Received]: the proof
//     is verified over those bytes, the §7.2 item 11 digest is over them, and a
//     document of spec's own version is held to spec's maxDocumentBytes
//     (SPEC §12.4).
//
// Every refusal before step 6, and every one [ConsumeInbound] makes, is made
// before the replay record is written, so a refused document never burns its id.
// The document keeps its received type, so a document read forward is answered
// as <received type>#response.
//
// The returned error is non-nil for a caller mistake ([ErrMissingOption],
// [ErrNoAuthenticatedSender]), for an error the handler returned that is not a
// [Refusal] — the core has then released the claim — and when the response
// cannot be marshalled, in which case the outcome is returned with it.
func ConsumeReceived[P, R any](
	ctx context.Context,
	received json.RawMessage,
	transport TransportHandler,
	spec SpecPolicy,
	decode func(json.RawMessage) (P, error),
	handler Handler[P, R],
	opts ReceiveOptions,
) (ReceivedOutcome[R], error) {
	var zero ReceivedOutcome[R]

	// The options ConsumeInbound would refuse, refused here as well: otherwise
	// a misconfigured consumer would answer malformed documents correctly and
	// discover the mistake on the first one that parses.
	switch {
	case transport == nil:
		return zero, fmt.Errorf("%w: transport", ErrMissingOption)
	case decode == nil:
		return zero, fmt.Errorf("%w: decode", ErrMissingOption)
	case handler == nil:
		return zero, fmt.Errorf("%w: handler", ErrMissingOption)
	case opts.NewErrorID == nil:
		return zero, fmt.Errorf("%w: NewErrorID", ErrMissingOption)
	case opts.PayloadPolicy.Kind == "":
		return zero, fmt.Errorf("%w: PayloadPolicy (SPEC §7.2 item 2) — see ConsumeOptions", ErrMissingOption)
	case opts.Checks.Replay.Kind == "":
		return zero, fmt.Errorf("%w: Checks (SPEC §7.2 items 4 and 11) — see ConsumeOptions", ErrMissingOption)
	}
	parties := transport.DeriveParties()
	if parties.Issuer == nil {
		return zero, ErrNoAuthenticatedSender
	}

	refused := func(doc *Document[json.RawMessage], reason RejectReason) ReceivedOutcome[R] {
		errorResponse := Reject(transport, doc, opts.NewErrorID(), reason, opts.Clock)
		if errorResponse == nil {
			return ReceivedOutcome[R]{Outcome: ConsumeOutcome[R]{Kind: OutcomeSuppressed, Reason: &reason}}
		}
		reply, _ := json.Marshal(errorResponse) // an ErrorResponse always marshals
		return ReceivedOutcome[R]{
			Outcome: ConsumeOutcome[R]{Kind: OutcomeRejected, Error: errorResponse},
			Reply:   reply,
		}
	}
	malformed := func(doc *Document[json.RawMessage], message string) ReceivedOutcome[R] {
		return refused(doc, RejectReason{Code: CodeMalformedRequest, Message: message})
	}

	// A raw view, for the refusals made before the typed decode. A member can
	// fail to parse (an issuedAt that is not a string, a threadId that is a
	// number) in a body that is otherwise a document; that body is answered from
	// the routing members that did parse.
	var raw Document[json.RawMessage]
	if err := json.Unmarshal(received, &raw); err != nil {
		return malformed(routingView(received), MalformedDocumentWireMessage), nil
	}

	if opts.BindingCheck != nil {
		if reason := opts.BindingCheck(&raw); reason != nil {
			return refused(&raw, *reason), nil
		}
	}

	if caseFoldedDuplicate(received) {
		return malformed(&raw, CaseFoldedMembersWireMessage), nil
	}

	reading, reason, err := ReadForSpecWith(received, spec, opts.Extends)
	if err != nil {
		return malformed(&raw, MalformedDocumentWireMessage), nil
	}
	if reason != nil {
		return refused(&raw, *reason), nil
	}
	var read Document[json.RawMessage]
	if err := json.Unmarshal(reading, &read); err != nil {
		return malformed(&raw, MalformedDocumentWireMessage), nil
	}
	payload, err := decode(read.Payload)
	if err != nil {
		return malformed(&raw, PayloadMismatchWireMessage), nil
	}

	myVID := opts.MyVID
	if myVID == "" && parties.Recipient != nil {
		myVID = *parties.Recipient
	}
	now := opts.Now
	if now.IsZero() {
		now = time.Now()
	}

	outcome, err := ConsumeInbound(ctx, ConsumeOptions[P, R]{
		Transport:     transport,
		Spec:          spec,
		ProofPolicy:   opts.ProofPolicy,
		PayloadPolicy: opts.PayloadPolicy,
		Checks:        opts.Checks,
		Doc:           withPayload(&raw, payload),
		Received:      received,
		Extends:       opts.Extends,
		MyVID:         myVID,
		Now:           now,
		NewErrorID:    opts.NewErrorID,
		Handler:       handler,
		Clock:         opts.Clock,
	})
	if err != nil {
		return zero, err
	}

	result := ReceivedOutcome[R]{Outcome: outcome}
	switch outcome.Kind {
	case OutcomeHandled:
		result.Reply, err = json.Marshal(outcome.Response)
		if err != nil {
			return result, fmt.Errorf("trusttasks: marshal response: %w", err)
		}
	case OutcomeRejected:
		result.Reply, _ = json.Marshal(outcome.Error)
	case OutcomeDuplicate:
		result.Reply = outcome.PriorResponse
	}
	return result, nil
}

// NewURNUUID returns a fresh urn:uuid: identifier (a version 4 UUID), for
// document ids.
//
// It panics when crypto/rand fails. A document id needs entropy: carrying on
// with an all-zero buffer would mint the same id every time, which a peer's
// §7.2 item 11 record then reads as an idConflict — every document after the
// first refused, quietly. crypto/rand does not fail on a supported platform,
// and from Go 1.24 its Read never returns an error at all.
func NewURNUUID() string {
	var b [16]byte
	if _, err := rand.Read(b[:]); err != nil {
		panic("trusttasks: crypto/rand failed: " + err.Error())
	}
	b[6] = (b[6] & 0x0f) | 0x40 // version 4
	b[8] = (b[8] & 0x3f) | 0x80 // RFC 9562 variant
	return fmt.Sprintf("urn:uuid:%x-%x-%x-%x-%x", b[0:4], b[4:6], b[6:8], b[8:10], b[10:16])
}

// routingView reads the members an error response is addressed from — id,
// type, issuer, recipient and the thread members — out of a body that did not
// parse as a whole, keeping each only when it is a string.
func routingView(body json.RawMessage) *Document[json.RawMessage] {
	var members map[string]json.RawMessage
	_ = json.Unmarshal(body, &members) // not an object: every member stays nil
	str := func(key string) *string {
		var v *string // a JSON null leaves it nil
		if json.Unmarshal(members[key], &v) != nil {
			return nil
		}
		return v
	}
	doc := &Document[json.RawMessage]{
		Issuer:         str("issuer"),
		Recipient:      str("recipient"),
		ThreadID:       str("threadId"),
		ParentThreadID: str("parentThreadId"),
	}
	if id := str("id"); id != nil {
		doc.ID = *id
	}
	if typ := str("type"); typ != nil {
		doc.Type = *typ
	}
	return doc
}

// withPayload is raw with its payload replaced by the decoded one, every other
// member kept.
func withPayload[P any](raw *Document[json.RawMessage], payload P) *Document[P] {
	return &Document[P]{
		ID:             raw.ID,
		ThreadID:       raw.ThreadID,
		ParentThreadID: raw.ParentThreadID,
		Ceremony:       raw.Ceremony,
		Type:           raw.Type,
		Issuer:         raw.Issuer,
		Recipient:      raw.Recipient,
		IssuedAt:       raw.IssuedAt,
		ExpiresAt:      raw.ExpiresAt,
		Payload:        payload,
		Context:        raw.Context,
		Proof:          raw.Proof,
		Extra:          raw.Extra,
	}
}

// caseFoldedDuplicate reports whether a JSON object anywhere in document —
// envelope or payload — holds two members whose names differ only by case.
//
// encoding/json reads such a pair into one Go field (the later member wins),
// while a proof verifier, a JSON Schema validator and the Rust, TypeScript and
// Dart runtimes read the exact-case member: a signed {"role":"reader",
// "Role":"admin"} would verify as "reader" and reach a Go handler as "admin".
// [CheckMemberNames] refuses the wider, type-informed set for the members a
// type declares; this needs no type, so it also covers a payload whose decoder
// does not call it.
func caseFoldedDuplicate(document []byte) bool {
	decoder := json.NewDecoder(bytes.NewReader(document))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		return false
	}
	return foldedDuplicateIn(value, 0)
}

func foldedDuplicateIn(value any, depth int) bool {
	if depth > maxMemberNameDepth {
		return false
	}
	switch v := value.(type) {
	case map[string]any:
		seen := make(map[string]struct{}, len(v))
		for name, member := range v {
			folded := foldMemberName(name)
			if _, dup := seen[folded]; dup {
				return true
			}
			seen[folded] = struct{}{}
			if foldedDuplicateIn(member, depth+1) {
				return true
			}
		}
	case []any:
		for _, item := range v {
			if foldedDuplicateIn(item, depth+1) {
				return true
			}
		}
	}
	return false
}
