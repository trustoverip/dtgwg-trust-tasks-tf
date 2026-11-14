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

	// Extends is the lookup forward routing follows (SPEC §5.2 "Declared
	// extension"): a later version whose chain reaches the implemented one is
	// read as it. Nil means [tt.RegistryExtends], the generated registry index;
	// pass a lookup that also knows a third-party (§6.5) specification's
	// declarations to follow its chain.
	Extends tt.ExtendsLookup
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
// sender's key via resolve from the bare DID of the envelope's skid, and runs the
// document through §7.2 for the specification spec. The skid is authenticated
// by that key: a forged one derives a key-encryption key that fails to unwrap.
//
// decode reads the request payload from JSON. A document holding two members
// whose names differ only by case is refused malformedRequest before decode
// runs; decode itself sees exact names only as far as it checks them — build it
// on [tt.CheckMemberNames] to refuse a member encoding/json would match by case
// alone. handler returns the response document (build it with
// [tt.RespondWith]), nil for a fire-and-forget spec, or a [tt.Refusal] error.
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
// routes on the document's type before choosing the specification. It is
// [tt.ConsumeReceived] with this consumer's configuration, and binding §3.1's
// thread cross-check as its BindingCheck.
func Consume[P, R any](
	ctx context.Context,
	c *Consumer,
	unpacked *Unpacked,
	spec tt.SpecPolicy,
	decode func(json.RawMessage) (P, error),
	handler tt.Handler[P, R],
) (*Received[R], error) {
	proofPolicy := tt.ProofPolicy{Kind: tt.ProofRejectIfPresent}
	if c.ProofVerifier != nil {
		proofPolicy = tt.ProofPolicy{Kind: tt.ProofVerify, Verifier: c.ProofVerifier}
	}
	payloadPolicy := tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated}
	if c.PayloadValidator != nil {
		payloadPolicy = tt.PayloadPolicy{Kind: tt.PayloadValidate, Validator: c.PayloadValidator}
	}
	checks := tt.ConsumeChecks{Freshness: c.Freshness, Replay: tt.ReplayPolicy{Kind: tt.ReplayNotConsequential}}
	if c.ReplayProtection {
		checks.Replay = tt.ReplayPolicy{Kind: tt.ReplayGuarded, Guard: c.Guard}
	}

	// The core's inbound body (#1110): a body that does not parse is answered
	// malformedRequest to the authenticated sender (binding §4), addressed from
	// the routing members that did parse; the thread cross-check below runs
	// next; two members differing only by case are refused; a later version
	// whose extends chain (per c.Extends; nil is the registry's) reaches spec's
	// is read as spec's version (SPEC §5.2 "Declared extension"); and §7.2 runs
	// with the document as it arrived as Received, so the proof, the item-11
	// digest and SPEC §12.4's maxDocumentBytes are all over those bytes.
	result, err := tt.ConsumeReceived[P, R](ctx, unpacked.Document, unpacked.Transport, spec, decode, handler, tt.ReceiveOptions{
		ProofPolicy:   proofPolicy,
		PayloadPolicy: payloadPolicy,
		Checks:        checks,
		Extends:       c.Extends,
		MyVID:         unpacked.Transport.Local,
		Now:           c.now(),
		NewErrorID:    c.newID,
		BindingCheck:  threadCheck(unpacked),
	})
	if err != nil {
		// A handler error that is not a Refusal, or a misconfiguration. The core
		// has already released the claim if the handler failed.
		return nil, err
	}
	return &Received[R]{Outcome: result.Outcome, Transport: unpacked.Transport, Reply: result.Reply}, nil
}

// threadCheck is binding §3.1: where both a DIDComm thread header and its
// framework member are present they MUST be equal. A mismatch is
// malformedRequest (a structurally inconsistent document), never
// identityMismatch, and is routed to the authenticated sender.
func threadCheck(unpacked *Unpacked) func(*tt.Document[json.RawMessage]) *tt.RejectReason {
	return func(doc *tt.Document[json.RawMessage]) *tt.RejectReason {
		if h, m := unpacked.Thid, doc.ThreadID; threadMismatch(h, m) {
			return &tt.RejectReason{
				Code:    tt.CodeMalformedRequest,
				Message: fmt.Sprintf("DIDComm thid %q disagrees with the document threadId %q", h, *m),
			}
		}
		if h, m := unpacked.Pthid, doc.ParentThreadID; threadMismatch(h, m) {
			return &tt.RejectReason{
				Code:    tt.CodeMalformedRequest,
				Message: fmt.Sprintf("DIDComm pthid %q disagrees with the document parentThreadId %q", h, *m),
			}
		}
		return nil
	}
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
