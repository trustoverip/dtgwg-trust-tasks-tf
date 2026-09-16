// SPEC.md §7.2 conformance tests for the Go consumer pipeline.
//
// These deliberately mirror the test set in trust-tasks-rs/src/consume.rs and
// trust-tasks-ts/test/consume.test.ts. The three reference implementations must
// reach the same verdict on the same document; where a case exists there and not
// here, the languages can drift apart without anything noticing.
//
// External test package on purpose: it exercises the module exactly as a
// consumer imports it, so an identifier that is unexported by accident fails here
// rather than in somebody's build.
package trusttasks_test

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

const (
	me   = "did:web:maintainer.example"
	peer = "did:web:org.example"
)

func ptr[T any](v T) *T { return &v }

// requiredSpec stands in for a generated package's Spec.
var requiredSpec = tt.SpecPolicy{
	TypeURI:             "https://trusttasks.org/spec/acl/grant/0.1",
	IsProofRequired:     true,
	IsRecipientRequired: true,
}

// freshSpec declares issuedAtRequirement REQUIRED (SPEC §7.3 item 17).
var freshSpec = tt.SpecPolicy{
	TypeURI:             "https://trusttasks.org/spec/acl/grant/0.1",
	IsRecipientRequired: true,
	IsIssuedAtRequired:  true,
}

// relaxedSpec only RECOMMENDS a proof and does not require a recipient.
var relaxedSpec = tt.SpecPolicy{
	TypeURI: "https://trusttasks.org/spec/acl/list/0.1",
}

type payload struct {
	Role string `json:"role"`
}

type response struct {
	OK bool `json:"ok"`
}

var testProof = &tt.Proof{
	Type:               "DataIntegrityProof",
	Cryptosuite:        "eddsa-rdfc-2022",
	VerificationMethod: peer + "#key-1",
	Created:            "2026-01-01T00:00:00Z",
	ProofPurpose:       "assertionMethod",
	ProofValue:         "z3kg",
}

var testNow = time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)

func fixedClock() string { return "2026-01-01T00:00:00Z" }

func doc(mutate func(*tt.Document[payload])) *tt.Document[payload] {
	d := &tt.Document[payload]{
		ID:        "req-1",
		Type:      requiredSpec.TypeURI,
		Issuer:    ptr(peer),
		Recipient: ptr(me),
		Payload:   payload{Role: "admin"},
	}
	if mutate != nil {
		mutate(d)
	}
	return d
}

/* ── Verifier and validator doubles ──────────────────────────────────────── */

type verifierFunc func(ctx context.Context, doc json.RawMessage) error

func (f verifierFunc) Verify(ctx context.Context, doc json.RawMessage) error { return f(ctx, doc) }

var alwaysValid = verifierFunc(func(context.Context, json.RawMessage) error { return nil })
var alwaysInvalid = verifierFunc(func(context.Context, json.RawMessage) error {
	return errors.New("resolver could not reach did:web:org.example")
})

// requiredMembersValidator stands in for a real JSON Schema engine: it refuses
// any payload missing a member the schema's `required` names. Deliberately not a
// real engine — these tests assert that the pipeline consults the validator and
// routes its verdict, not that somebody else's validator is correct.
type requiredMembersValidator struct{}

func (requiredMembersValidator) Validate(schemaJSON string, payload json.RawMessage) error {
	var schema struct {
		Required []string `json:"required"`
	}
	if err := json.Unmarshal([]byte(schemaJSON), &schema); err != nil {
		return err
	}
	var members map[string]json.RawMessage
	if err := json.Unmarshal(payload, &members); err != nil {
		return err
	}
	var missing []string
	for _, name := range schema.Required {
		if _, present := members[name]; !present {
			missing = append(missing, "missing "+name)
		}
	}
	if len(missing) > 0 {
		return errors.New(strings.Join(missing, "; "))
	}
	return nil
}

type explodingValidator struct{}

func (explodingValidator) Validate(string, json.RawMessage) error {
	return errors.New("validator exploded")
}

const roleSchema = `{"$schema":"https://json-schema.org/draft/2020-12/schema",` +
	`"type":"object","required":["role"],"properties":{"role":{"type":"string"}}}`

/* ── Harness ─────────────────────────────────────────────────────────────── */

type runOpts struct {
	spec                *tt.SpecPolicy
	proofPolicy         *tt.ProofPolicy
	payloadPolicy       *tt.PayloadPolicy
	checks              *tt.ConsumeChecks
	transport           tt.TransportHandler
	handlerShouldNotRun bool
	handlerReturnsNil   bool
	handlerRefuses      *tt.RejectReason
	now                 *time.Time
}

func run(t *testing.T, mutate func(*tt.Document[payload]), opts runOpts) tt.ConsumeOutcome[response] {
	t.Helper()

	spec := requiredSpec
	if opts.spec != nil {
		spec = *opts.spec
	}
	proofPolicy := tt.ProofPolicy{Kind: tt.ProofVerify, Verifier: alwaysValid}
	if opts.proofPolicy != nil {
		proofPolicy = *opts.proofPolicy
	}
	payloadPolicy := tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated}
	if opts.payloadPolicy != nil {
		payloadPolicy = *opts.payloadPolicy
	}
	checks := tt.NotConsequentialChecks()
	if opts.checks != nil {
		checks = *opts.checks
	}
	var transport tt.TransportHandler = tt.UnauthenticatedTransport{}
	if opts.transport != nil {
		transport = opts.transport
	}
	now := testNow
	if opts.now != nil {
		now = *opts.now
	}

	outcome, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
		Transport:     transport,
		Spec:          spec,
		ProofPolicy:   proofPolicy,
		PayloadPolicy: payloadPolicy,
		Checks:        checks,
		Doc:           doc(mutate),
		MyVID:         me,
		Now:           now,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(_ context.Context, accepted *tt.Document[payload], _ tt.ResolvedParties) (*tt.Document[response], error) {
			if opts.handlerShouldNotRun {
				t.Errorf("handler must not run")
			}
			if opts.handlerRefuses != nil {
				return nil, tt.Refuse(accepted, "err-1", *opts.handlerRefuses, fixedClock)
			}
			if opts.handlerReturnsNil {
				return nil, nil
			}
			return tt.RespondWith[payload, response](accepted, "resp-1", response{OK: true}, fixedClock), nil
		},
	})
	if err != nil {
		t.Fatalf("ConsumeInbound returned an error: %v", err)
	}
	return outcome
}

func rejectedCode(t *testing.T, outcome tt.ConsumeOutcome[response]) string {
	t.Helper()
	if outcome.Kind != tt.OutcomeRejected {
		t.Fatalf("expected %q, got %q", tt.OutcomeRejected, outcome.Kind)
	}
	return outcome.Error.Payload.Code
}

func wantCode(t *testing.T, outcome tt.ConsumeOutcome[response], code tt.StandardCode) {
	t.Helper()
	if got := rejectedCode(t, outcome); got != string(code) {
		t.Errorf("expected code %q, got %q", code, got)
	}
}

/* ── §7.2 pipeline ───────────────────────────────────────────────────────── */

func TestPipelineRunsHandlerWhenEveryCheckPasses(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) { d.Proof = testProof }, runOpts{})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
	if outcome.Response.ID != "resp-1" {
		t.Errorf("expected response id resp-1, got %q", outcome.Response.ID)
	}
	// §4.4.1 — the response carries the #response fragment...
	if outcome.Response.Type != requiredSpec.TypeURI+"#response" {
		t.Errorf("expected the #response variant, got %q", outcome.Response.Type)
	}
	// ...the parties swap...
	if outcome.Response.Issuer == nil || *outcome.Response.Issuer != me {
		t.Errorf("expected the response issued by this consumer")
	}
	if outcome.Response.Recipient == nil || *outcome.Response.Recipient != peer {
		t.Errorf("expected the response addressed to the original producer")
	}
	// ...and §4.9 continues the thread from the request's id.
	if outcome.Response.ThreadID == nil || *outcome.Response.ThreadID != "req-1" {
		t.Errorf("expected the thread to continue from the request id")
	}
}

func TestItem4RejectsAnExpiredDocument(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) {
		d.ExpiresAt = ptr("2025-01-01T00:00:00Z")
	}, runOpts{handlerShouldNotRun: true})
	wantCode(t, outcome, tt.CodeExpired)
}

func TestItem4ExpiryInstantIsInclusive(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) {
		d.IssuedAt = ptr("2025-12-31T23:00:00Z")
		d.ExpiresAt = ptr("2026-01-01T00:00:00Z") // exactly now
	}, runOpts{handlerShouldNotRun: true})
	wantCode(t, outcome, tt.CodeExpired)
}

func TestItem4RejectsAMalformedExpiresAt(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) {
		d.ExpiresAt = ptr("not-a-timestamp")
	}, runOpts{handlerShouldNotRun: true})
	wantCode(t, outcome, tt.CodeMalformedRequest)
}

func TestItem5aWrongRecipientRoutesToTheOriginalIssuer(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) {
		d.Recipient = ptr("did:web:someone-else.example")
	}, runOpts{handlerShouldNotRun: true})
	wantCode(t, outcome, tt.CodeWrongRecipient)
	if outcome.Error.Recipient == nil || *outcome.Error.Recipient != peer {
		t.Errorf("expected the error to be addressed to the original issuer")
	}
}

func TestItem5bRecipientRequiredButAbsent(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) {
		d.Recipient = nil
	}, runOpts{handlerShouldNotRun: true})
	wantCode(t, outcome, tt.CodeMalformedRequest)
}

func TestItem7ProofRequiredButAbsent(t *testing.T) {
	outcome := run(t, nil, runOpts{handlerShouldNotRun: true})
	wantCode(t, outcome, tt.CodeProofRequired)
}

func TestItem7RecommendedSpecAcceptsAProoflessDocument(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) { d.Recipient = nil }, runOpts{spec: &relaxedSpec})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
}

func TestItem17IssuedAtRequiredButAbsent(t *testing.T) {
	outcome := run(t, nil, runOpts{spec: &freshSpec, handlerShouldNotRun: true})
	wantCode(t, outcome, tt.CodeMalformedRequest)
	if got := *outcome.Error.Payload.Message; !strings.Contains(got, "§7.3 item 17") {
		t.Errorf("expected the §7.3 item 17 message, got %q", got)
	}
}

func TestItem17PassesOnceTheDocumentCarriesIssuedAt(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) {
		d.IssuedAt = ptr("2026-01-01T00:00:00Z")
	}, runOpts{spec: &freshSpec})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
}

func TestItem17SpecDeclaringNothingAcceptsADocumentWithoutIssuedAt(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) { d.Recipient = nil }, runOpts{spec: &relaxedSpec})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
}

func TestItem7FailingVerifierMapsToProofInvalid(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) { d.Proof = testProof }, runOpts{
		proofPolicy:         &tt.ProofPolicy{Kind: tt.ProofVerify, Verifier: alwaysInvalid},
		handlerShouldNotRun: true,
	})
	wantCode(t, outcome, tt.CodeProofInvalid)
}

func TestItem7RejectIfPresentDoesNotLeakConfiguration(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) { d.Proof = testProof }, runOpts{
		proofPolicy:         &tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
		handlerShouldNotRun: true,
	})
	wantCode(t, outcome, tt.CodeMalformedRequest)
	if got := *outcome.Error.Payload.Message; got != tt.ProofNotAcceptedByPolicy {
		t.Errorf("expected the constant wire message, got %q", got)
	}
}

func TestItem7AcceptUnverifiedPassesAProofBearingDocumentThrough(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) { d.Proof = testProof }, runOpts{
		proofPolicy: &tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
	})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
}

func TestItem8ProofWithNoRecipientOnANonBearerSpecIsMalformed(t *testing.T) {
	spec := relaxedSpec // no recipient requirement, so item 5b does not fire first
	outcome := run(t, func(d *tt.Document[payload]) {
		d.Recipient = nil
		d.Proof = testProof
	}, runOpts{spec: &spec, handlerShouldNotRun: true})
	wantCode(t, outcome, tt.CodeMalformedRequest)
	if got := *outcome.Error.Payload.Message; !strings.Contains(got, "audience binding") {
		t.Errorf("expected the §4.8.2 audience-binding message, got %q", got)
	}
}

func TestItem8BearerSpecIsExemptFromAudienceBinding(t *testing.T) {
	spec := relaxedSpec
	spec.IsBearer = true
	outcome := run(t, func(d *tt.Document[payload]) {
		d.Recipient = nil
		d.Proof = testProof
	}, runOpts{spec: &spec})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
}

/* ── §4.8.1 party resolution ─────────────────────────────────────────────── */

func TestPartyResolutionFillsAbsentMembersFromTheTransport(t *testing.T) {
	var seen tt.ResolvedParties
	_, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
		Transport:     tt.StaticTransport{Context: tt.TransportContext{Issuer: ptr(peer), Recipient: ptr(me)}},
		Spec:          relaxedSpec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.NotConsequentialChecks(),
		Doc:           doc(func(d *tt.Document[payload]) { d.Issuer = nil; d.Recipient = nil }),
		MyVID:         me,
		Now:           testNow,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(_ context.Context, accepted *tt.Document[payload], parties tt.ResolvedParties) (*tt.Document[response], error) {
			seen = parties
			return tt.RespondWith[payload, response](accepted, "resp-1", response{OK: true}, fixedClock), nil
		},
	})
	if err != nil {
		t.Fatalf("ConsumeInbound: %v", err)
	}
	if seen.Issuer == nil || *seen.Issuer != peer {
		t.Errorf("expected the transport-derived issuer to fill in")
	}
	if seen.Recipient == nil || *seen.Recipient != me {
		t.Errorf("expected the transport-derived recipient to fill in")
	}
}

func TestItem6InBandAndTransportDisagreementIsAnIdentityMismatch(t *testing.T) {
	outcome := run(t, nil, runOpts{
		transport: tt.StaticTransport{Context: tt.TransportContext{
			Issuer: ptr("did:web:impostor.example"), Recipient: ptr(me),
		}},
		handlerShouldNotRun: true,
	})
	wantCode(t, outcome, tt.CodeIdentityMismatch)
	// §8.1: addressed to the transport-authenticated sender, not the in-band issuer.
	if outcome.Error.Recipient == nil || *outcome.Error.Recipient != "did:web:impostor.example" {
		t.Errorf("expected the error routed to the transport-authenticated sender")
	}
	// §8.1 / §10.4: the wire message names neither value.
	message := *outcome.Error.Payload.Message
	if strings.Contains(message, "impostor") || strings.Contains(message, peer) {
		t.Errorf("identityMismatch message leaked an identity: %q", message)
	}
}

func TestIdentityMismatchWithNoTransportSenderIsSuppressed(t *testing.T) {
	outcome := run(t, nil, runOpts{
		// The transport authenticates a recipient but no sender, so the mismatch
		// fires on `recipient` and §8.1 leaves nobody safe to answer.
		transport:           tt.StaticTransport{Context: tt.TransportContext{Recipient: ptr("did:web:other.example")}},
		handlerShouldNotRun: true,
	})
	if outcome.Kind != tt.OutcomeSuppressed {
		t.Fatalf("expected suppressed, got %q", outcome.Kind)
	}
	if outcome.Reason.Code != tt.CodeIdentityMismatch {
		t.Errorf("expected identityMismatch, got %q", outcome.Reason.Code)
	}
}

/* ── §8.2 inResponseTo ───────────────────────────────────────────────────── */

func TestInResponseToNamesTheDocumentTheErrorReportsOn(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) { d.Recipient = nil }, runOpts{handlerShouldNotRun: true})
	about := outcome.Error.Payload.InResponseTo
	if about == nil {
		t.Fatal("expected inResponseTo to be populated")
	}
	if about.TypeURI != requiredSpec.TypeURI {
		t.Errorf("expected the reported-on type URI, got %q", about.TypeURI)
	}
	if about.ID == nil || *about.ID != "req-1" {
		t.Errorf("expected the reported-on id")
	}
}

func TestInResponseToWithholdsTheIDUnderIdentityMismatch(t *testing.T) {
	outcome := run(t, nil, runOpts{
		transport: tt.StaticTransport{Context: tt.TransportContext{
			Issuer: ptr("did:web:impostor.example"), Recipient: ptr(me),
		}},
		handlerShouldNotRun: true,
	})
	about := outcome.Error.Payload.InResponseTo
	if about == nil {
		t.Fatal("expected inResponseTo to be populated")
	}
	if about.ID != nil {
		t.Errorf("expected the originating id to be withheld, got %q", *about.ID)
	}
}

func TestErrorResponsesCarryTheDeclaredErrorTypeURI(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) { d.Recipient = nil }, runOpts{handlerShouldNotRun: true})
	if outcome.Error.Type != tt.TrustTaskErrorTypeURI {
		t.Errorf("expected %q, got %q", tt.TrustTaskErrorTypeURI, outcome.Error.Type)
	}
	if !tt.IsErrorResponse(outcome.Error) {
		t.Errorf("IsErrorResponse did not recognize the error response")
	}
}

/* ── §4.9.2 parentThreadId ───────────────────────────────────────────────── */

func TestParentThreadIDCarriesOntoTheSuccessResponse(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) {
		d.ParentThreadID = ptr("parent-1")
		d.Proof = testProof
	}, runOpts{})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
	if outcome.Response.ParentThreadID == nil || *outcome.Response.ParentThreadID != "parent-1" {
		t.Errorf("expected the parent thread to carry through")
	}
}

func TestParentThreadIDCarriesOntoAnErrorResponse(t *testing.T) {
	outcome := run(t, func(d *tt.Document[payload]) {
		d.ParentThreadID = ptr("parent-1")
		d.Recipient = nil
	}, runOpts{handlerShouldNotRun: true})
	if outcome.Error.ParentThreadID == nil || *outcome.Error.ParentThreadID != "parent-1" {
		t.Errorf("expected the parent thread to carry through")
	}
}

func TestParentThreadIDIsOmittedEntirelyWhenAbsent(t *testing.T) {
	response := tt.RespondWith[payload, response](doc(nil), "resp-1", response{OK: true}, fixedClock)
	encoded, err := json.Marshal(response)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	if strings.Contains(string(encoded), "parentThreadId") {
		t.Errorf("expected no parentThreadId member at all, got %s", encoded)
	}
}

/* ── §7.2 item 2 — payload schema validation ─────────────────────────────── */

func TestItem2RejectsAPayloadMissingARequiredMember(t *testing.T) {
	spec := requiredSpec
	spec.IsProofRequired = false
	spec.PayloadSchema = roleSchema
	outcome := run(t, func(d *tt.Document[payload]) { d.Payload = payload{} }, runOpts{
		spec:          &spec,
		payloadPolicy: &tt.PayloadPolicy{Kind: tt.PayloadValidate, Validator: requiredMembersValidator{}},
		// The struct always marshals `role`, so drive the miss from the schema
		// side instead: a schema requiring a member the payload type has no
		// member for.
	})
	// `role` is always present in this Go struct, so the document conforms.
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled for a conforming payload, got %q", outcome.Kind)
	}

	spec.PayloadSchema = `{"type":"object","required":["role","scope"]}`
	outcome = run(t, nil, runOpts{
		spec:                &spec,
		payloadPolicy:       &tt.PayloadPolicy{Kind: tt.PayloadValidate, Validator: requiredMembersValidator{}},
		handlerShouldNotRun: true,
	})
	wantCode(t, outcome, tt.CodeMalformedRequest)
	if got := *outcome.Error.Payload.Message; !strings.Contains(got, "missing scope") {
		t.Errorf("expected the validator's detail in the message, got %q", got)
	}
}

func TestItem2AValidatorThatFailsIsAFailureNotACrash(t *testing.T) {
	spec := requiredSpec
	spec.IsProofRequired = false
	spec.PayloadSchema = roleSchema
	outcome := run(t, nil, runOpts{
		spec:                &spec,
		payloadPolicy:       &tt.PayloadPolicy{Kind: tt.PayloadValidate, Validator: explodingValidator{}},
		handlerShouldNotRun: true,
	})
	wantCode(t, outcome, tt.CodeMalformedRequest)
}

func TestItem2AcceptUnvalidatedReallyDoesSkipTheCheck(t *testing.T) {
	spec := requiredSpec
	spec.IsProofRequired = false
	spec.PayloadSchema = `{"type":"object","required":["nothing-has-this"]}`
	outcome := run(t, nil, runOpts{
		spec:          &spec,
		payloadPolicy: &tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
	})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
}

func TestItem2IsANoOpWhenTheSpecCarriesNoSchema(t *testing.T) {
	spec := requiredSpec
	spec.IsProofRequired = false
	outcome := run(t, nil, runOpts{
		spec:          &spec,
		payloadPolicy: &tt.PayloadPolicy{Kind: tt.PayloadValidate, Validator: explodingValidator{}},
	})
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", outcome.Kind)
	}
}

/* ── Required options ────────────────────────────────────────────────────── */

func TestPayloadPolicyIsRequiredRatherThanDefaultingToSkip(t *testing.T) {
	_, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
		Transport:  tt.UnauthenticatedTransport{},
		Spec:       relaxedSpec,
		Checks:     tt.NotConsequentialChecks(),
		Doc:        doc(nil),
		MyVID:      me,
		Now:        testNow,
		NewErrorID: func() string { return "err-1" },
		Handler: func(context.Context, *tt.Document[payload], tt.ResolvedParties) (*tt.Document[response], error) {
			return nil, nil
		},
	})
	if !errors.Is(err, tt.ErrMissingOption) {
		t.Fatalf("expected ErrMissingOption, got %v", err)
	}
}

func TestChecksIsRequiredRatherThanDefaultingToNoRecord(t *testing.T) {
	_, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          relaxedSpec,
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Doc:           doc(nil),
		MyVID:         me,
		Now:           testNow,
		NewErrorID:    func() string { return "err-1" },
		Handler: func(context.Context, *tt.Document[payload], tt.ResolvedParties) (*tt.Document[response], error) {
			return nil, nil
		},
	})
	if !errors.Is(err, tt.ErrMissingOption) {
		t.Fatalf("expected ErrMissingOption, got %v", err)
	}
}

/* ── Handler outcomes ────────────────────────────────────────────────────── */

func TestHandlerReturningNothingYieldsAccepted(t *testing.T) {
	spec := relaxedSpec
	outcome := run(t, func(d *tt.Document[payload]) { d.Recipient = nil }, runOpts{
		spec:              &spec,
		handlerReturnsNil: true,
	})
	if outcome.Kind != tt.OutcomeAccepted {
		t.Fatalf("expected accepted, got %q", outcome.Kind)
	}
}

func TestHandlerRefusalPassesThroughVerbatim(t *testing.T) {
	spec := relaxedSpec
	outcome := run(t, func(d *tt.Document[payload]) { d.Recipient = nil }, runOpts{
		spec: &spec,
		handlerRefuses: &tt.RejectReason{
			Code:    tt.CodePermissionDenied,
			Message: "not on the approver list",
		},
	})
	wantCode(t, outcome, tt.CodePermissionDenied)
	if outcome.Error.Recipient == nil || *outcome.Error.Recipient != peer {
		t.Errorf("expected the refusal addressed to the original producer")
	}
}

func TestHandlerErrorThatIsNotARefusalPropagates(t *testing.T) {
	sentinel := errors.New("the database is on fire")
	_, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          relaxedSpec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.NotConsequentialChecks(),
		Doc:           doc(func(d *tt.Document[payload]) { d.Recipient = nil }),
		MyVID:         me,
		Now:           testNow,
		NewErrorID:    func() string { return "err-1" },
		Handler: func(context.Context, *tt.Document[payload], tt.ResolvedParties) (*tt.Document[response], error) {
			return nil, sentinel
		},
	})
	if !errors.Is(err, sentinel) {
		t.Fatalf("expected the handler's error to propagate, got %v", err)
	}
}

/* ── §7.1 / §7.2 — unrecognized top-level members survive ────────────────── */

func TestUnrecognizedTopLevelMembersSurviveARoundTrip(t *testing.T) {
	const wire = `{"id":"req-1","type":"https://trusttasks.org/spec/acl/grant/0.1",` +
		`"payload":{"role":"admin"},"x-vendor-hint":{"keep":"me"}}`

	var parsed tt.Document[payload]
	if err := json.Unmarshal([]byte(wire), &parsed); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if _, kept := parsed.Extra["x-vendor-hint"]; !kept {
		t.Fatalf("expected the unrecognized member in Extra, got %v", parsed.Extra)
	}
	encoded, err := json.Marshal(parsed)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var members map[string]json.RawMessage
	if err := json.Unmarshal(encoded, &members); err != nil {
		t.Fatalf("re-unmarshal: %v", err)
	}
	if string(members["x-vendor-hint"]) != `{"keep":"me"}` {
		t.Errorf("expected the member preserved verbatim, got %s", members["x-vendor-hint"])
	}
}
