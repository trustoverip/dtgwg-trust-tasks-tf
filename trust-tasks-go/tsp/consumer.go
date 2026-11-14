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

	// Extends is the lookup forward routing follows (SPEC §5.2 "Declared
	// extension"): a later version whose chain reaches the implemented one is
	// read as it. Nil means [tt.RegistryExtends], the generated registry index;
	// pass a lookup that also knows a third-party (§6.5) specification's
	// declarations to follow its chain.
	Extends tt.ExtendsLookup
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
// the request payload from JSON. A document holding two members whose names
// differ only by case is refused malformedRequest before decode runs; decode
// itself sees exact names only as far as it checks them — build it on
// [tt.CheckMemberNames] to refuse a member encoding/json would match by case
// alone. handler returns the response document (build it with
// [tt.RespondWith]) or nil for a fire-and-forget spec, or a [tt.Refusal]
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
// routes on the document's type before choosing the specification. It is
// [tt.ConsumeReceived] with this consumer's configuration.
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
	// the routing members that did parse; two members differing only by case are
	// refused; a later version whose extends chain (per c.Extends; nil is the
	// registry's) reaches spec's is read as spec's version (SPEC §5.2 "Declared
	// extension"); and §7.2 runs with the document as it arrived as Received, so
	// the proof, the item-11 digest and SPEC §12.4's maxDocumentBytes are all
	// over those bytes.
	result, err := tt.ConsumeReceived[P, R](ctx, unpacked.Document, unpacked.Transport, spec, decode, handler, tt.ReceiveOptions{
		ProofPolicy:   proofPolicy,
		PayloadPolicy: payloadPolicy,
		Checks:        checks,
		Extends:       c.Extends,
		MyVID:         unpacked.Transport.Local,
		Now:           c.now(),
		NewErrorID:    c.newID,
	})
	if err != nil {
		// A handler error that is not a Refusal, or a misconfiguration. The core
		// has already released the claim if the handler failed.
		return nil, err
	}
	return &Received[R]{Outcome: result.Outcome, Transport: unpacked.Transport, Reply: result.Reply}, nil
}

// PackReply seals a reply document back to the authenticated sender (§6).
func PackReply(sender *tsp.PrivateIdentity, receiver *tsp.Identity, reply json.RawMessage) ([]byte, error) {
	if len(reply) == 0 {
		return nil, errors.New("tsp: nothing to reply with")
	}
	return PackTrustTask(reply, sender, receiver)
}
