package tsp

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	tsp "github.com/affinidi/affinidi-tsp-go"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// Consumer is the guarded inbound path for the TSP binding: open the sealed
// message, then run SPEC §7.2 with the duplicate-execution record of item 11 and
// the freshness bound it needs — both on by default.
//
// §7 records that TSP data messages "do not inherently prevent replay", and
// under §5's routed and nested carriage an intermediary may re-send the sealed
// inner message. So an ordinary re-forward — no attacker involved — executes a
// consequential task twice unless the consumer keeps the record. The record is
// keyed on the document id, never the TSP envelope, which is resealed with fresh
// material on every send (§7.1).
//
// One consumer per process, not one per message. The guard is the record; a
// consumer built per message remembers nothing. Replicas share a store through
// Guard. Build one with [NewConsumer].
type Consumer struct {
	// Recipient is the identity messages are addressed to, and replies are
	// sealed from. Required.
	Recipient *tsp.PrivateIdentity

	// Guard is the §7.2 item 11 record. The in-process default is wrong for a
	// replicated consumer: each replica would execute the same document once.
	Guard tt.ReplayGuard

	// ReplayProtection keeps the record. True by default; turn off only where
	// every task handled declares repeated execution safe.
	ReplayProtection bool

	// Freshness is the acceptance window, and so also the record's retention.
	Freshness tt.FreshnessPolicy

	// ProofVerifier verifies in-band proofs. Nil refuses a proof-bearing
	// document with malformedRequest rather than ignoring the proof. TSP
	// authenticates the sender end-to-end, so a proof may be omitted over this
	// binding in direct and nested modes (§5.3) — unless the spec requires one.
	ProofVerifier tt.ProofVerifier

	// PayloadValidator validates payloads against their schema (§7.2 item 2).
	PayloadValidator tt.PayloadValidator

	// NewID mints response and error document ids. Defaults to [NewURNUUID].
	NewID func() string

	// Now supplies the clock. Defaults to time.Now.
	Now func() time.Time
}

// NewConsumer returns a Consumer with the record on, an in-process guard and the
// consequential freshness policy — the safe defaults for a consumer that has not
// thought about item 11. Override the fields on the returned value as needed.
func NewConsumer(recipient *tsp.PrivateIdentity) *Consumer {
	return &Consumer{
		Recipient:        recipient,
		Guard:            tt.NewInMemoryReplayGuard(0),
		ReplayProtection: true,
		Freshness:        tt.ConsequentialFreshness(),
		NewID:            NewURNUUID,
		Now:              time.Now,
	}
}

func (c *Consumer) newID() string {
	if c.NewID != nil {
		return c.NewID()
	}
	return NewURNUUID()
}

func (c *Consumer) now() time.Time {
	if c.Now != nil {
		return c.Now()
	}
	return time.Now()
}

// Received is what [Receive] or [Consume] made of one TSP message.
type Received[R any] struct {
	// Outcome is the §7.2 outcome, exactly as [tt.ConsumeInbound] defines it.
	Outcome tt.ConsumeOutcome[R]
	// Transport is who sent it (authenticated) and who received it.
	Transport Transport
	// Reply is the document to seal back, as JSON, or nil when nothing is to be
	// sent: a fire-and-forget success, a suppressed identityMismatch (§8.1), or a
	// duplicate with no retained response (§7.1). Seal it with [PackReply].
	Reply json.RawMessage
}

// Receive opens wire under the resolved sender identity and runs it through
// §7.2 for the specification spec.
//
// The Go TSP library resolves no VIDs, so the caller supplies the sender
// identity — look it up from [AdvertisedSender] in your own store. decode reads
// the request payload from JSON. handler returns the response document (build it
// with [tt.RespondWith]) or nil for a fire-and-forget spec, or a [tt.Refusal]
// error.
//
// It returns an *EnvelopeError for a message that never reached the pipeline
// (§4): nothing can be sent back for those.
func Receive[P, R any](
	ctx context.Context,
	c *Consumer,
	wire []byte,
	sender *tsp.Identity,
	spec tt.SpecPolicy,
	decode func(json.RawMessage) (P, error),
	handler tt.Handler[P, R],
) (*Received[R], error) {
	unpacked, err := UnpackTrustTask(wire, c.Recipient, sender)
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

	// A raw view for the paths that answer before the typed decode. UnpackTrustTask
	// guarantees string id and type, so this always parses.
	var raw tt.Document[json.RawMessage]
	if err := json.Unmarshal(unpacked.Document, &raw); err != nil {
		return &Received[R]{
			Outcome:   tt.ConsumeOutcome[R]{Kind: tt.OutcomeRejected},
			Transport: transport,
		}, err
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
		// the claim so a re-forward is not absorbed in silence, then surface it.
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

// PackReply seals a reply document back to the authenticated sender (§6).
func PackReply(sender *tsp.PrivateIdentity, receiver *tsp.Identity, reply json.RawMessage) ([]byte, error) {
	if len(reply) == 0 {
		return nil, errors.New("tsp: nothing to reply with")
	}
	return PackTrustTask(reply, sender, receiver)
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
