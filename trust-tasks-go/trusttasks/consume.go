package trusttasks

// Inbound-document orchestration for SPEC.md §7.2 item 2, items 4–8, and the two
// stateful checks — the freshness bound of item 4 and the duplicate-execution
// record of item 11.
//
// Mirrors consume.rs in trust-tasks-rs and consume.ts in @openvtc/trust-tasks,
// deliberately closely: a Go consumer and a Rust or TypeScript one must reach the
// same verdict on the same document, or the reference implementations disagree
// about what conforms.
//
//	guard := trusttasks.NewInMemoryReplayGuard(0) // one guard per consumer: it IS the record
//
//	outcome, err := trusttasks.ConsumeInbound(ctx, trusttasks.ConsumeOptions[aclgrantv0_1.Payload, aclgrantv0_1.Response]{
//	    Transport:     transport,
//	    Spec:          aclgrantv0_1.Spec,
//	    ProofPolicy:   trusttasks.ProofPolicy{Kind: trusttasks.ProofVerify, Verifier: myVerifier},
//	    PayloadPolicy: trusttasks.PayloadPolicy{Kind: trusttasks.PayloadValidate, Validator: myValidator},
//	    Checks:        trusttasks.ConsequentialChecks(guard), // acl/grant is consequential
//	    Doc:           doc,
//	    MyVID:         "did:web:maintainer.example",
//	    Now:           time.Now(),
//	    NewErrorID:    uuid.NewString,
//	    Handler: func(ctx context.Context, doc *trusttasks.Document[aclgrantv0_1.Payload], parties trusttasks.ResolvedParties) (*trusttasks.Document[aclgrantv0_1.Response], error) {
//	        return trusttasks.RespondWith(doc, uuid.NewString(), buildResponse(parties), nil), nil
//	    },
//	})
//
//	switch outcome.Kind {
//	case trusttasks.OutcomeHandled:   emit(outcome.Response)
//	case trusttasks.OutcomeRejected:  emit(outcome.Error)
//	case trusttasks.OutcomeSuppressed: logSuppressed(outcome.Reason)
//	case trusttasks.OutcomeAccepted:  // fire-and-forget: nothing to emit
//	case trusttasks.OutcomeDuplicate: // §7.2 item 11: already executed, not an error
//	    if outcome.PriorResponse != nil {
//	        emit(outcome.PriorResponse)
//	    }
//	}
//
// Items 1 (framework schema) and 3 (unknown type) are not attempted here — they
// belong to the caller's parse and dispatch.
//
// Item 2 (payload schema) is. Go's encoding/json enforces less than it looks like
// it does: it ignores unknown members by default and does not enforce a single
// REQUIRED one, so a document whose payload is missing every required member
// unmarshals cleanly into a generated struct. That puts Go closer to TypeScript
// than to Rust, and is why [PayloadPolicy] is a required option rather than an
// optional one that defaults to skipping.

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"
)

// ConsumeChecks are the two stateful §7.2 checks: the freshness bound over
// issuedAt / expiresAt, and the duplicate-execution record of item 11.
//
// They travel together because the spec ties them together. §7.2 (*Bounding the
// record*) makes the acceptance window and the record's retention the same bound.
// Passing them as one option is what stops a deployment configuring a five-minute
// record and an unbounded acceptance window, which reads as a working replay
// defence and is not one.
//
// Like [PayloadPolicy], this is a required option. Whether the task a consumer
// implements is consequential (§2) is a decision only that consumer can make, and
// the failure mode of getting it wrong silently — an ACL grant applied twice by a
// mediator retry — is not one to discover after the fact.
type ConsumeChecks struct {
	// Freshness bounds the document in time (SPEC §4.2, §7.2 item 4).
	Freshness FreshnessPolicy
	// Replay applies (or knowingly disapplies) SPEC §7.2 item 11.
	Replay ReplayPolicy
}

// ConsequentialChecks is the posture for a consequential Trust Task (§2): item 11
// enforced against guard, and the bounded acceptance window that makes the record
// droppable.
//
// The right choice for any task whose execution grants access, moves value,
// discloses a secret, or otherwise cannot be undone by ignoring the next document.
func ConsequentialChecks(guard ReplayGuard) ConsumeChecks {
	return ConsumeChecks{
		Freshness: ConsequentialFreshness(),
		Replay:    ReplayPolicy{Kind: ReplayGuarded, Guard: guard},
	}
}

// NotConsequentialChecks is the posture for a task that is not consequential, or
// whose specification "explicitly declares repeated execution safe and intended"
// — the narrow disapplication item 11 permits.
//
// Keeps no record. Still applies the default freshness policy, because a
// future-dated document and one with an empty validity interval are malformed
// whatever the task does.
func NotConsequentialChecks() ConsumeChecks {
	return ConsumeChecks{
		Freshness: DefaultFreshness(),
		Replay:    ReplayPolicy{Kind: ReplayNotConsequential},
	}
}

// ProofVerifier verifies a document's Data Integrity proof (SPEC §4.7).
//
// The document arrives as the JSON [ConsumeInbound] holds it, not as a
// Document[P]: Go interfaces cannot carry a method that is itself generic over
// the payload, and a Data Integrity verifier works over the serialization
// regardless. Returning a nil error means the proof verifies against the in-band
// issuer.
type ProofVerifier interface {
	Verify(ctx context.Context, doc json.RawMessage) error
}

// ProofPolicyKind selects how [ConsumeInbound] treats a document's proof member
// (§7.2 item 7).
//
// The framework does not assume what integrity guarantees a consumer relies on.
// Some verify Data Integrity proofs in-band; some have transport-layer integrity
// (signed DIDComm, mTLS-bound HTTPS) and accept in-band proofs only
// opportunistically; some have none. Making the choice explicit at the call site
// is the point.
//
// Spec.IsProofRequired is consulted independently of the policy: a specification
// that requires a proof rejects a proofless document whichever policy is chosen.
type ProofPolicyKind string

const (
	// ProofVerify verifies the proof when present. Failures map to proofInvalid.
	// The safe default for any consumer that honours in-band proofs.
	ProofVerify ProofPolicyKind = "verify"

	// ProofRejectIfPresent rejects a document that carries an in-band proof, with
	// malformedRequest. For consumers with integrity from another layer that are
	// deliberately not verifying in-band proofs — silently dropping a
	// producer-supplied proof would mislead the producer about the guarantees of
	// the exchange.
	ProofRejectIfPresent ProofPolicyKind = "rejectIfPresent"

	// ProofAcceptUnverified accepts any document, proof or not, without
	// verifying. SECURITY: only where the transport already provides equivalent
	// end-to-end integrity. This is the explicit opt-out, and the name is
	// deliberately uncomfortable to type.
	ProofAcceptUnverified ProofPolicyKind = "acceptUnverified"
)

// ProofPolicy is a [ProofPolicyKind] plus the verifier it needs.
type ProofPolicy struct {
	Kind     ProofPolicyKind
	Verifier ProofVerifier
}

// PayloadValidator evaluates a payload against its schema (SPEC §7.2 item 2).
type PayloadValidator interface {
	// Validate checks payload against schemaJSON — the value from
	// [SpecPolicy.PayloadSchema], a JSON Schema 2020-12 document with all
	// cross-file $refs already inlined, so no resolver is needed. It is generated
	// from this repo at build time, so it is trusted input.
	//
	// Return nil to accept. The returned error's text lands in the
	// malformedRequest message, so a caller can see what failed.
	Validate(schemaJSON string, payload json.RawMessage) error
}

// PayloadPolicyKind selects how [ConsumeInbound] performs SPEC §7.2 item 2.
//
// This module bundles no JSON Schema implementation, for the same reason it
// bundles no cryptosuite: the engine, its draft support and its resource limits
// are the consumer's choice, and a zero-dependency module is worth keeping. So
// the schema ships with the generated package and the validator comes from you.
type PayloadPolicyKind string

const (
	// PayloadValidate validates against Spec.PayloadSchema. Failures map to
	// malformedRequest.
	PayloadValidate PayloadPolicyKind = "validate"

	// PayloadAcceptUnvalidated skips item 2 entirely. Appropriate only where
	// something upstream — an API gateway, a schema-validating transport — has
	// already performed it on the same bytes.
	//
	// This is a real choice, not a formality: encoding/json does not enforce
	// REQUIRED members, so with no validator a document whose payload is missing
	// every one of them reaches your handler indistinguishable from a conforming
	// one.
	PayloadAcceptUnvalidated PayloadPolicyKind = "acceptUnvalidated"
)

// PayloadPolicy is a [PayloadPolicyKind] plus the validator it needs.
type PayloadPolicy struct {
	Kind      PayloadPolicyKind
	Validator PayloadValidator
}

// OutcomeKind is the disposition [ConsumeInbound] reached.
type OutcomeKind string

const (
	// OutcomeHandled means every check passed and the caller's handler produced a
	// response.
	OutcomeHandled OutcomeKind = "handled"

	// OutcomeRejected means a framework check failed, or the handler refused.
	// Either way the document is already addressed per §8.1 — emit it over the
	// transport.
	OutcomeRejected OutcomeKind = "rejected"

	// OutcomeSuppressed means §8.1 applied: the rejection was identityMismatch
	// and the transport authenticated no sender, so no response may be emitted —
	// one would be an oracle.
	//
	// Callers SHOULD log this: silent suppression is the spec rule, but invisible
	// suppression is an operational footgun.
	OutcomeSuppressed OutcomeKind = "suppressed"

	// OutcomeAccepted means every check passed and the handler completed without
	// producing a document — a fire-and-forget task.
	//
	// SPEC §4.4.1: a specification that defines no success response is one whose
	// consumers MUST NOT emit a #response-variant document. Emit nothing.
	OutcomeAccepted OutcomeKind = "accepted"

	// OutcomeDuplicate means SPEC §7.2 item 11 applied: a document with this id
	// and this content was already accepted for execution. The handler was not
	// called, and the consequential effect did not happen a second time. This is
	// the §8.4 retry being absorbed, which is what makes retrying safe.
	//
	// Not an error. §7.2 (*Disposition of a duplicate*): "In no case is a
	// duplicate reported as taskFailed; the task did not fail, it already
	// happened." Folding this into OutcomeRejected would report a failure that
	// did not occur.
	//
	// What to emit:
	//
	//   - PriorResponse non-nil — emit it. It is the response document the first
	//     execution produced (a success response, or a non-retryable error
	//     response; tell them apart by its type), which §7.2 says the consumer
	//     SHOULD return.
	//   - PriorResponse nil, InFlight false — the specification defines no success
	//     response (§4.4.1 fire-and-forget), or the guard retains none. Emit
	//     nothing: that silence is the correct disposition, not an error.
	//   - InFlight true — the first execution has not finished. §7.2: the consumer
	//     SHOULD "return or expose the existing execution state rather than begin
	//     another".
	OutcomeDuplicate OutcomeKind = "duplicate"
)

// ConsumeOutcome is the result of [ConsumeInbound]. Which members are populated
// depends on Kind — see [OutcomeKind].
type ConsumeOutcome[R any] struct {
	Kind OutcomeKind

	// Response is set for [OutcomeHandled].
	Response *Document[R]
	// Error is set for [OutcomeRejected].
	Error *ErrorResponse
	// Reason is set for [OutcomeSuppressed].
	Reason *RejectReason
	// PriorResponse and InFlight are set for [OutcomeDuplicate].
	PriorResponse json.RawMessage
	InFlight      bool
}

// Wire messages the pipeline emits. Constants so a store's hostname, a verifier's
// vocabulary or a consumer's clock never reaches the wire (SPEC §10.4), and so
// the Rust and TypeScript runtimes can be held equal to them.
const (
	// ProofNotAcceptedByPolicy is the message for the ProofRejectIfPresent path.
	ProofNotAcceptedByPolicy = "in-band proof not accepted by consumer policy (SPEC §7.2 item 7)"

	// ProofInvalidWireMessage is the message for proofInvalid.
	ProofInvalidWireMessage = "proof verification failed"

	// IDConflictWireMessage is the message for idConflict (SPEC §7.2 item 11, §8.3).
	IDConflictWireMessage = "a different document has already been accepted under this id " +
		"(SPEC §7.2 item 11)"

	// ReplayRecordUnavailable is the message for a replay-record outage.
	ReplayRecordUnavailable = "temporarily unavailable"
)

// Handler is the business handler, called only once every framework check has
// passed.
//
// Return a success response, or (nil, nil) for a specification that defines no
// success response (§4.4.1) — which yields [OutcomeAccepted]. Return a [Refusal]
// to refuse with a built error response. Any other error aborts the pipeline and
// is returned to the caller of [ConsumeInbound] unchanged.
type Handler[P, R any] func(
	ctx context.Context,
	doc *Document[P],
	parties ResolvedParties,
) (*Document[R], error)

// Refusal is the error a [Handler] returns to refuse with a specific error
// response.
//
// The response is passed through verbatim — the framework does not re-apply §8.1
// routing to it. A handler refusing for identity-style reasons MUST address the
// response itself (see [Reject]); [RejectWith] copies doc.Issuer into Recipient,
// which is safe for ordinary refusals but not for one that contests that identity.
type Refusal struct {
	Response *ErrorResponse
}

func (r *Refusal) Error() string {
	if r.Response == nil {
		return "trusttasks: handler refused"
	}
	return "trusttasks: handler refused with " + r.Response.Payload.Code
}

// Refuse builds a handler-side refusal addressed to the original producer.
//
// Convenience over [RejectWith] plus [ToErrorPayload] for the common case where a
// handler refuses for a business reason. Not safe for refusals that contest the
// in-band identity — see [Refusal].
func Refuse[P any](request *Document[P], id string, reason RejectReason, clock Clock) *Refusal {
	return &Refusal{Response: RejectWith(request, id, ToErrorPayload(reason), clock)}
}

// ConsumeOptions are the inputs to [ConsumeInbound].
type ConsumeOptions[P, R any] struct {
	Transport TransportHandler

	// Spec is the generated package's Spec (request) or ResponseSpec (response).
	Spec SpecPolicy

	ProofPolicy ProofPolicy

	// PayloadPolicy performs §7.2 item 2. Required — see [PayloadPolicyKind].
	PayloadPolicy PayloadPolicy

	// Checks performs §7.2 item 4 (freshness) and item 11 (duplicate execution).
	// Required — use [ConsequentialChecks] or [NotConsequentialChecks].
	Checks ConsumeChecks

	Doc *Document[P]

	// MyVID is this consumer's own VID, for the §7.2 item 5 recipient check.
	MyVID string

	Now time.Time

	// NewErrorID is invoked at most once, only when a rejection needs an
	// error-response id.
	NewErrorID func() string

	Handler Handler[P, R]

	// Clock overrides the response issuedAt clock, for deterministic tests. Nil
	// means [SystemClock].
	Clock Clock
}

// ErrMissingOption reports an option [ConsumeInbound] requires and did not get.
//
// Go zero values make every struct field optional at the language level, and the
// two options that most need a deliberate answer — whether to validate the
// payload, and whether to keep a duplicate-execution record — both have a zero
// value that reads as "skip it". Defaulting to skipping would silently reproduce
// exactly the defects those options exist to remove, so an unset one is an error
// rather than a default.
var ErrMissingOption = errors.New("trusttasks: required ConsumeOptions field is unset")

// ConsumeInbound runs SPEC §7.2 items 4–8 against the document, then either calls
// the handler or builds the routed error response per §8.1.
//
// The returned error is non-nil only for a caller mistake ([ErrMissingOption]), a
// document that cannot be marshalled for its item-11 digest, or an error the
// handler returned that is not a [Refusal]. Every framework rejection is a
// [ConsumeOutcome], not an error.
func ConsumeInbound[P, R any](
	ctx context.Context,
	opts ConsumeOptions[P, R],
) (ConsumeOutcome[R], error) {
	var zero ConsumeOutcome[R]

	if opts.PayloadPolicy.Kind == "" {
		return zero, fmt.Errorf(
			"%w: PayloadPolicy (SPEC §7.2 item 2). Set Kind: PayloadValidate with a Validator to "+
				"check the payload against Spec.PayloadSchema, or Kind: PayloadAcceptUnvalidated to "+
				"state that you are deliberately not checking it",
			ErrMissingOption,
		)
	}
	if opts.Checks.Replay.Kind == "" {
		return zero, fmt.Errorf(
			"%w: Checks (SPEC §7.2 items 4 and 11). Pass ConsequentialChecks(guard) for a task "+
				"whose execution grants access, moves value or is otherwise irreversible, or "+
				"NotConsequentialChecks() to state that repeated execution of this task is safe "+
				"and intended",
			ErrMissingOption,
		)
	}
	if opts.Doc == nil {
		return zero, fmt.Errorf("%w: Doc", ErrMissingOption)
	}
	if opts.Transport == nil {
		return zero, fmt.Errorf("%w: Transport", ErrMissingOption)
	}
	if opts.NewErrorID == nil {
		return zero, fmt.Errorf("%w: NewErrorID", ErrMissingOption)
	}
	if opts.Handler == nil {
		return zero, fmt.Errorf("%w: Handler", ErrMissingOption)
	}

	doc := opts.Doc
	route := func(reason *RejectReason) ConsumeOutcome[R] {
		errorResponse := Reject(opts.Transport, doc, opts.NewErrorID(), *reason, opts.Clock)
		if errorResponse == nil {
			return ConsumeOutcome[R]{Kind: OutcomeSuppressed, Reason: reason}
		}
		return ConsumeOutcome[R]{Kind: OutcomeRejected, Error: errorResponse}
	}

	// §7.2 item 2 — payload schema. Runs first, in the spec's own order, and
	// before anything that reasons about what the payload means: a payload that is
	// not the shape the specification declares should be refused as malformed
	// rather than interpreted.
	if opts.PayloadPolicy.Kind == PayloadValidate && opts.Spec.PayloadSchema != "" {
		if opts.PayloadPolicy.Validator == nil {
			return zero, fmt.Errorf("%w: PayloadPolicy.Validator under PayloadValidate", ErrMissingOption)
		}
		payload, err := json.Marshal(doc.Payload)
		if err != nil {
			return zero, fmt.Errorf("trusttasks: marshal payload for §7.2 item 2: %w", err)
		}
		if err := opts.PayloadPolicy.Validator.Validate(opts.Spec.PayloadSchema, payload); err != nil {
			message := "payload does not conform to its schema (SPEC §7.2 item 2)"
			if detail := strings.TrimSpace(err.Error()); detail != "" {
				message += ": " + detail
			}
			return route(&RejectReason{Code: CodeMalformedRequest, Message: message}), nil
		}
	}

	// §7.2 items 4 + 5a — expiry and wrong-recipient.
	if reason := ValidateBasic(doc, opts.Now, opts.MyVID); reason != nil {
		return route(reason), nil
	}

	// §7.2 item 4, the other half — the freshness bound over issuedAt.
	// ValidateBasic honours expiresAt, which is optional and which a producer sets
	// for its own reasons; on its own it leaves a document stamped years ago, or
	// years hence, indefinitely acceptable. It is also what bounds the replay
	// record below: §7.2 makes the acceptance window and the record's retention
	// one bound.
	if reason := ValidateFreshness(doc, opts.Now, opts.Checks.Freshness); reason != nil {
		return route(reason), nil
	}

	// §7.2 item 6 — in-band vs transport-derived identity cross-check.
	parties, mismatch := ResolveParties(opts.Transport, doc)
	if mismatch != nil {
		reason := IdentityMismatchReason(mismatch)
		return route(&reason), nil
	}

	// §7.2 item 7 clause B — the consumer's chosen proof policy.
	if doc.Proof != nil {
		switch opts.ProofPolicy.Kind {
		case ProofVerify:
			if opts.ProofPolicy.Verifier == nil {
				return zero, fmt.Errorf("%w: ProofPolicy.Verifier under ProofVerify", ErrMissingOption)
			}
			encoded, err := json.Marshal(doc)
			if err != nil {
				return zero, fmt.Errorf("trusttasks: marshal document for proof verification: %w", err)
			}
			if err := opts.ProofPolicy.Verifier.Verify(ctx, encoded); err != nil {
				// A constant, never the verifier's own error text. SPEC §10.4
				// extends the §8.1 identity rule to every code, and a verifier's
				// vocabulary names DIDs it tried to resolve, whether a resolver
				// answered, and what a fetched DID document contained — a
				// resolver-reachability oracle for a sender who is, by
				// construction, unauthenticated. Log the detail; do not send it.
				return route(&RejectReason{
					Code:    CodeProofInvalid,
					Message: ProofInvalidWireMessage,
				}), nil
			}
		case ProofRejectIfPresent:
			return route(&RejectReason{
				Code:    CodeMalformedRequest,
				Message: ProofNotAcceptedByPolicy,
			}), nil
		case ProofAcceptUnverified:
			// Deliberately nothing.
		default:
			return zero, fmt.Errorf("%w: ProofPolicy.Kind", ErrMissingOption)
		}
	}

	// §7.2 items 5b + 7 clause A + 8 — the policy-driven checks, in one place so
	// this pipeline and any binding-specific one cannot diverge on the check set.
	if reason := EnforceSpecPolicy(doc, opts.Spec); reason != nil {
		return route(reason), nil
	}

	// §7.2 item 11 — the duplicate-execution record. Deliberately last: claiming
	// the id marks the document as accepted for execution, and a document some
	// earlier check refuses was never accepted. Claiming first would burn the id
	// on every malformed or unauthorised arrival, so a corrected resend under the
	// same id would come back idConflict forever — and an attacker could pre-burn
	// an id it had merely observed.
	var claimed *claim
	if opts.Checks.Replay.Kind == ReplayGuarded {
		guard := opts.Checks.Replay.Guard
		if guard == nil {
			return zero, fmt.Errorf("%w: Checks.Replay.Guard under ReplayGuarded", ErrMissingOption)
		}
		digest, err := DocumentDigest(doc)
		if err != nil {
			return zero, err
		}

		// §7.2 (*Bounding the record*): "A consumer that can establish neither an
		// expiresAt nor an age for a document has no window in which to place it,
		// and MUST NOT execute a consequential Trust Task on it." A guard asked to
		// retain a record forever is not a guard, so refuse rather than pretend.
		retainUntil, bounded := RecordExpiry(doc, opts.Checks.Freshness, opts.Now)
		if !bounded {
			return route(&RejectReason{Code: CodeExpired, Message: StaleWireMessage}), nil
		}

		verdict, err := guard.Claim(ctx, doc.ID, digest, retainUntil, opts.Now)
		if err != nil {
			// Fail closed. A consumer that cannot consult its record has not
			// satisfied item 11, and executing anyway is exactly the double
			// execution the rule forbids. unavailable is retryable, which is the
			// honest answer: the producer's bit-for-bit resend will be absorbed
			// correctly once the store is back. The error detail — a hostname, a
			// connection string — stays out of the message, per §10.4.
			return route(&RejectReason{
				Code:      CodeUnavailable,
				Message:   ReplayRecordUnavailable,
				Retryable: true,
			}), nil
		}

		switch verdict.Kind {
		case VerdictDuplicate:
			return ConsumeOutcome[R]{
				Kind:          OutcomeDuplicate,
				PriorResponse: verdict.PriorResponse,
				InFlight:      verdict.InFlight,
			}, nil
		case VerdictConflict:
			return route(&RejectReason{
				Code:    CodeIDConflict,
				Message: IDConflictWireMessage,
			}), nil
		case VerdictFresh:
			claimed = &claim{guard: guard, digest: digest}
		}
	}

	response, err := opts.Handler(ctx, doc, parties)
	if err != nil {
		var refusal *Refusal
		if !errors.As(err, &refusal) {
			return zero, err
		}
		// §8.4: a retryable refusal has just invited the producer to re-send this
		// document bit-for-bit. Holding the claim would answer that invited retry
		// with the cached failure forever. A non-retryable refusal is final, so the
		// record stands and a replay is answered with the same determination.
		if refusal.Response != nil && refusal.Response.Payload.Retryable {
			claimed.release(ctx, doc.ID)
		} else {
			claimed.settle(ctx, doc.ID, refusal.Response)
		}
		return ConsumeOutcome[R]{Kind: OutcomeRejected, Error: refusal.Response}, nil
	}

	if response == nil {
		// Fire-and-forget: nothing to cache, but the claim stands — the effect
		// happened, and item 11 is about the effect, not about the response.
		claimed.settle(ctx, doc.ID, nil)
		return ConsumeOutcome[R]{Kind: OutcomeAccepted}, nil
	}

	claimed.settle(ctx, doc.ID, response)
	return ConsumeOutcome[R]{Kind: OutcomeHandled, Response: response}, nil
}

// claim is the record [ConsumeInbound] holds between a successful Claim and the
// handler returning. Nil when no guard is in use, and every method tolerates that.
type claim struct {
	guard  ReplayGuard
	digest string
}

// settle records the response for a completed execution, best-effort.
//
// The effect has already happened, so a guard that cannot cache the response
// cannot un-happen it. The record of the claim is what item 11 needs and it is
// already written; all that is lost is the ability to hand the same response back,
// and the duplicate is still absorbed. Guards should log their own failures —
// there is nowhere to report this from here.
func (c *claim) settle(ctx context.Context, id string, response any) {
	if c == nil {
		return
	}
	var encoded json.RawMessage
	if response != nil {
		if raw, err := json.Marshal(response); err == nil {
			encoded = raw
		}
	}
	_ = c.guard.RecordResponse(ctx, id, encoded)
}

// release drops the claim, best-effort. See the §8.4 note at the call site.
func (c *claim) release(ctx context.Context, id string) {
	if c == nil {
		return
	}
	_ = c.guard.Release(ctx, id, c.digest)
}

// IsErrorResponse reports whether a document is an error response rather than a
// success response.
//
// Keys off the trust-task-error Type URI rather than payload shape: §8 makes the
// type the discriminator, and a success payload could coincidentally carry a code
// member.
func IsErrorResponse[P any](doc *Document[P]) bool {
	return strings.Contains(bareTypeURI(doc.Type), "/trust-task-error/")
}
