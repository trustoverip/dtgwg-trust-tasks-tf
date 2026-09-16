package didcomm

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// Consumer is the guarded inbound path for the DIDComm binding: open the
// authcrypt envelope, cross-check the thread headers (binding §3.1), then run
// SPEC §7.2 with the duplicate-execution record of item 11 and its freshness
// bound — both on by default.
//
// DIDComm is store-and-forward and "provides no replay protection" (binding §6):
// a mediator may re-deliver, so an ordinary re-delivery — no attacker — executes
// a consequential task twice unless the consumer keeps the record, which is keyed
// on the document id, never the DIDComm envelope (resealed with fresh material on
// every send).
//
// One consumer per process, not one per message — the guard is the record.
// Replicas share a store through Guard. Build one with [NewConsumer].
type Consumer struct {
	// Recipient is the identity messages are addressed to, and replies are
	// sealed from. Required.
	Recipient *Identity

	// Guard is the §7.2 item 11 record. The in-process default is wrong for a
	// replicated consumer.
	Guard tt.ReplayGuard

	// ReplayProtection keeps the record. True by default.
	ReplayProtection bool

	// Freshness is the acceptance window, and so also the record's retention.
	Freshness tt.FreshnessPolicy

	// ProofVerifier verifies in-band proofs. Nil refuses a proof-bearing
	// document with malformedRequest rather than ignoring the proof. DIDComm
	// authcrypt authenticates the sender end-to-end, so a proof may be omitted
	// over this binding (§5) unless the spec requires one.
	ProofVerifier tt.ProofVerifier

	// PayloadValidator validates payloads against their schema (§7.2 item 2).
	PayloadValidator tt.PayloadValidator

	// NewID mints response and error document ids. Defaults to [NewMessageID].
	NewID func() string

	// Now supplies the clock. Defaults to time.Now.
	Now func() time.Time
}

// NewConsumer returns a Consumer with the record on, an in-process guard and the
// consequential freshness policy — the safe defaults for a consumer that has not
// thought about item 11. Override the fields as needed.
func NewConsumer(recipient *Identity) *Consumer {
	return &Consumer{
		Recipient:        recipient,
		Guard:            tt.NewInMemoryReplayGuard(0),
		ReplayProtection: true,
		Freshness:        tt.ConsequentialFreshness(),
		NewID:            NewMessageID,
		Now:              time.Now,
	}
}

func (c *Consumer) newID() string {
	if c.NewID != nil {
		return c.NewID()
	}
	return NewMessageID()
}

func (c *Consumer) now() time.Time {
	if c.Now != nil {
		return c.Now()
	}
	return time.Now()
}

// Received is what [Receive] or [Consume] made of one DIDComm message.
type Received[R any] struct {
	// Outcome is the §7.2 outcome, exactly as [tt.ConsumeInbound] defines it.
	Outcome tt.ConsumeOutcome[R]
	// Transport is who sent it (authenticated) and who received it.
	Transport Transport
	// Reply is the document to seal back, as JSON, or nil when nothing is to be
	// sent. Seal it with [PackReply], addressed to the sender's key-agreement key.
	Reply json.RawMessage
}

// Receive opens wire addressed to the consumer's Recipient, resolving the
// sender's key from the envelope's authenticated skid via resolve, and runs the
// document through §7.2 for the specification spec.
//
// decode reads the request payload from JSON; handler returns the response
// document (build it with [tt.RespondWith]), nil for a fire-and-forget spec, or
// a [tt.Refusal] error.
//
// It returns an *[EnvelopeError] for a message that never reached the pipeline
// (binding §4): nothing can be sent back for those.
func Receive[P, R any](
	ctx context.Context,
	c *Consumer,
	wire []byte,
	resolve ResolveSender,
	spec tt.SpecPolicy,
	decode func(json.RawMessage) (P, error),
	handler tt.Handler[P, R],
) (*Received[R], error) {
	unpacked, err := UnpackTrustTask(wire, c.Recipient, resolve)
	if err != nil {
		return nil, err
	}
	return Consume[P, R](ctx, c, unpacked, spec, decode, handler)
}

// Consume runs an already-unpacked document through §7.2 — for a caller that
// routes on the document's type before choosing the specification.
func Consume[P, R any](
	ctx context.Context,
	c *Consumer,
	unpacked *Unpacked,
	spec tt.SpecPolicy,
	decode func(json.RawMessage) (P, error),
	handler tt.Handler[P, R],
) (*Received[R], error) {
	transport := unpacked.Transport
	now := c.now()

	refused := func(doc *tt.Document[json.RawMessage], reason tt.RejectReason) *Received[R] {
		errResp := tt.Reject(transport, doc, c.newID(), reason, nil)
		out := tt.ConsumeOutcome[R]{}
		var reply json.RawMessage
		if errResp == nil {
			out.Kind = tt.OutcomeSuppressed
			out.Reason = &reason
		} else {
			out.Kind = tt.OutcomeRejected
			out.Error = errResp
			reply, _ = json.Marshal(errResp)
		}
		return &Received[R]{Outcome: out, Transport: transport, Reply: reply}
	}

	var raw tt.Document[json.RawMessage]
	if err := json.Unmarshal(unpacked.Document, &raw); err != nil {
		return refused(&tt.Document[json.RawMessage]{}, tt.RejectReason{
			Code:    tt.CodeMalformedRequest,
			Message: "envelope body is not a Trust Task document",
		}), nil
	}

	// Binding §3.1 — where both a DIDComm thread header and its framework member
	// are present they MUST be equal. A mismatch is malformedRequest (a
	// structurally inconsistent document), never identityMismatch, and is routed
	// to the authenticated sender.
	if h, m := unpacked.Thid, raw.ThreadID; threadMismatch(h, m) {
		return refused(&raw, tt.RejectReason{
			Code:    tt.CodeMalformedRequest,
			Message: fmt.Sprintf("DIDComm thid %q disagrees with the document threadId %q", h, *m),
		}), nil
	}
	if h, m := unpacked.Pthid, raw.ParentThreadID; threadMismatch(h, m) {
		return refused(&raw, tt.RejectReason{
			Code:    tt.CodeMalformedRequest,
			Message: fmt.Sprintf("DIDComm pthid %q disagrees with the document parentThreadId %q", h, *m),
		}), nil
	}

	payload, err := decode(raw.Payload)
	if err != nil {
		return refused(&raw, tt.RejectReason{
			Code:    tt.CodeMalformedRequest,
			Message: "payload does not match the specification",
		}), nil
	}
	doc := documentWith(raw, payload)

	proofPolicy := tt.ProofPolicy{Kind: tt.ProofRejectIfPresent}
	if c.ProofVerifier != nil {
		proofPolicy = tt.ProofPolicy{
			Kind:     tt.ProofVerify,
			Verifier: asReceived{inner: c.ProofVerifier, json: unpacked.Document},
		}
	}
	payloadPolicy := tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated}
	if c.PayloadValidator != nil {
		payloadPolicy = tt.PayloadPolicy{Kind: tt.PayloadValidate, Validator: c.PayloadValidator}
	}
	checks := tt.ConsumeChecks{Freshness: c.Freshness, Replay: tt.ReplayPolicy{Kind: tt.ReplayNotConsequential}}
	if c.ReplayProtection {
		checks.Replay = tt.ReplayPolicy{Kind: tt.ReplayGuarded, Guard: c.Guard}
	}

	outcome, err := tt.ConsumeInbound(ctx, tt.ConsumeOptions[P, R]{
		Transport:     transport,
		Spec:          spec,
		ProofPolicy:   proofPolicy,
		PayloadPolicy: payloadPolicy,
		Checks:        checks,
		Doc:           doc,
		MyVID:         transport.Local,
		Now:           now,
		NewErrorID:    c.newID,
		Handler:       handler,
	})
	if err != nil {
		// A handler error that is not a Refusal, or a misconfiguration. Release
		// the claim so a re-delivery is not absorbed in silence, then surface it.
		if c.ReplayProtection {
			if digest, derr := tt.DocumentDigest(doc); derr == nil {
				_ = c.Guard.Release(ctx, doc.ID, digest)
			}
		}
		return nil, err
	}

	var reply json.RawMessage
	switch outcome.Kind {
	case tt.OutcomeHandled:
		reply, _ = json.Marshal(outcome.Response)
	case tt.OutcomeRejected:
		reply, _ = json.Marshal(outcome.Error)
	case tt.OutcomeDuplicate:
		reply = outcome.PriorResponse
	}
	return &Received[R]{Outcome: outcome, Transport: transport, Reply: reply}, nil
}

// PackReply seals a reply document back to the authenticated sender (binding §4,
// §6). recipient is the original sender's key-agreement key — resolve it the same
// way [Receive]'s ResolveSender did.
func PackReply(sender *Identity, recipient *PeerKey, reply json.RawMessage) ([]byte, error) {
	if len(reply) == 0 {
		return nil, errors.New("didcomm: nothing to reply with")
	}
	return PackTrustTask(reply, sender, recipient)
}

// threadMismatch is binding §3.1's both-present-and-differ test.
func threadMismatch(header string, inBand *string) bool {
	return header != "" && inBand != nil && header != *inBand
}

// documentWith rebuilds a typed document from the raw view and the decoded
// payload, preserving every other member.
func documentWith[P any](raw tt.Document[json.RawMessage], payload P) *tt.Document[P] {
	return &tt.Document[P]{
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

// asReceived verifies the document as it arrived rather than as the typed decode
// would re-encode it, which drops members the generated type does not model and
// so breaks a valid signature.
type asReceived struct {
	inner tt.ProofVerifier
	json  json.RawMessage
}

func (a asReceived) Verify(ctx context.Context, _ json.RawMessage) error {
	return a.inner.Verify(ctx, a.json)
}
