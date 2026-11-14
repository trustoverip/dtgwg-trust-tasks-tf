// Tests for ConsumeReceived, the inbound body the message bindings share
// (#1110), and NewURNUUID. The cases are the ones the TSP and DIDComm
// consumers' own tests pin, so moving a binding onto ConsumeReceived changes
// nothing a peer can observe.
package trusttasks_test

import (
	"context"
	"encoding/json"
	"errors"
	"regexp"
	"strings"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// countingGuard is an in-memory guard that counts claims, so a test can say a
// refusal never reached the record.
type countingGuard struct {
	inner  *tt.InMemoryReplayGuard
	claims int
}

func newCountingGuard() *countingGuard {
	return &countingGuard{inner: tt.NewInMemoryReplayGuard(0)}
}

func (g *countingGuard) Claim(ctx context.Context, id, digest string, retainUntil, now time.Time) (tt.ReplayVerdict, error) {
	g.claims++
	return g.inner.Claim(ctx, id, digest, retainUntil, now)
}

func (g *countingGuard) RecordResponse(ctx context.Context, id string, response json.RawMessage) error {
	return g.inner.RecordResponse(ctx, id, response)
}

func (g *countingGuard) Release(ctx context.Context, id, digest string) error {
	return g.inner.Release(ctx, id, digest)
}

var receivedTransport = tt.StaticTransport{Context: tt.TransportContext{Issuer: ptr(peer), Recipient: ptr(me)}}

var receivedSpec = tt.SpecPolicy{
	TypeURI:          "https://trusttasks.org/spec/acl/grant/0.1",
	MaxDocumentBytes: 600,
}

func receivedOptions(guard tt.ReplayGuard) tt.ReceiveOptions {
	return tt.ReceiveOptions{
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks: tt.ConsumeChecks{
			Freshness: tt.ConsequentialFreshness(),
			Replay:    tt.ReplayPolicy{Kind: tt.ReplayGuarded, Guard: guard},
		},
		Now:        testNow,
		NewErrorID: func() string { return "err-1" },
		Clock:      fixedClock,
	}
}

func decodePayload(raw json.RawMessage) (payload, error) {
	if err := tt.CheckMemberNames[payload](raw); err != nil {
		return payload{}, err
	}
	var p payload
	err := json.Unmarshal(raw, &p)
	return p, err
}

// countingHandler answers {"ok":true} and counts its calls.
func countingHandler(calls *int) tt.Handler[payload, response] {
	return func(_ context.Context, doc *tt.Document[payload], _ tt.ResolvedParties) (*tt.Document[response], error) {
		*calls++
		return tt.RespondWith(doc, "resp-1", response{OK: true}, fixedClock), nil
	}
}

func receivedBody(t *testing.T, mutate func(map[string]any)) json.RawMessage {
	t.Helper()
	body := map[string]any{
		"id":        "req-1",
		"type":      receivedSpec.TypeURI,
		"issuer":    peer,
		"recipient": me,
		"issuedAt":  "2026-01-01T00:00:00Z",
		"payload":   map[string]any{"role": "admin"},
	}
	if mutate != nil {
		mutate(body)
	}
	encoded, err := json.Marshal(body)
	if err != nil {
		t.Fatal(err)
	}
	return encoded
}

// replyOf decodes a reply and returns its error code ("" for a response).
func replyOf(t *testing.T, reply json.RawMessage) (typ, code, recipient string) {
	t.Helper()
	var doc struct {
		Type      string  `json:"type"`
		Recipient *string `json:"recipient"`
		Payload   struct {
			Code string `json:"code"`
		} `json:"payload"`
	}
	if err := json.Unmarshal(reply, &doc); err != nil {
		t.Fatalf("reply does not parse: %v (%s)", err, reply)
	}
	if doc.Recipient != nil {
		recipient = *doc.Recipient
	}
	return doc.Type, doc.Payload.Code, recipient
}

func TestConsumeReceivedHandlesAndReplies(t *testing.T) {
	guard := newCountingGuard()
	calls := 0
	out, err := tt.ConsumeReceived(context.Background(), receivedBody(t, nil), receivedTransport,
		receivedSpec, decodePayload, countingHandler(&calls), receivedOptions(guard))
	if err != nil {
		t.Fatal(err)
	}
	if out.Outcome.Kind != tt.OutcomeHandled || calls != 1 {
		t.Fatalf("outcome %q, %d handler calls", out.Outcome.Kind, calls)
	}
	typ, _, recipient := replyOf(t, out.Reply)
	if typ != receivedSpec.TypeURI+"#response" || recipient != peer {
		t.Fatalf("reply type %q to %q", typ, recipient)
	}

	// The same document again is a duplicate, answered with the retained response.
	again, err := tt.ConsumeReceived(context.Background(), receivedBody(t, nil), receivedTransport,
		receivedSpec, decodePayload, countingHandler(&calls), receivedOptions(guard))
	if err != nil {
		t.Fatal(err)
	}
	if again.Outcome.Kind != tt.OutcomeDuplicate || calls != 1 || string(again.Reply) != string(out.Reply) {
		t.Fatalf("resend: outcome %q, %d calls, reply %s", again.Outcome.Kind, calls, again.Reply)
	}
}

func TestConsumeReceivedFireAndForgetHasNoReply(t *testing.T) {
	out, err := tt.ConsumeReceived(context.Background(), receivedBody(t, nil), receivedTransport, receivedSpec,
		decodePayload,
		func(context.Context, *tt.Document[payload], tt.ResolvedParties) (*tt.Document[response], error) {
			return nil, nil
		},
		receivedOptions(newCountingGuard()))
	if err != nil {
		t.Fatal(err)
	}
	if out.Outcome.Kind != tt.OutcomeAccepted || out.Reply != nil {
		t.Fatalf("outcome %q, reply %s", out.Outcome.Kind, out.Reply)
	}
}

// Each of these is refused malformedRequest, addressed to the authenticated
// sender, without running the handler or touching the record.
func TestConsumeReceivedRefusesBeforeTheRecord(t *testing.T) {
	cases := []struct {
		name    string
		body    json.RawMessage
		check   func(*tt.Document[json.RawMessage]) *tt.RejectReason
		code    string
		message string
	}{
		{
			name: "a member that does not parse",
			body: receivedBody(t, func(m map[string]any) { m["issuedAt"] = 7 }),
			code: string(tt.CodeMalformedRequest), message: tt.MalformedDocumentWireMessage,
		},
		{
			name: "not a JSON object",
			body: json.RawMessage(`["not", "a", "document"]`),
			code: string(tt.CodeMalformedRequest), message: tt.MalformedDocumentWireMessage,
		},
		{
			name: "members differing only by case in the payload",
			body: json.RawMessage(`{"id":"req-1","type":"` + receivedSpec.TypeURI + `","issuer":"` + peer +
				`","recipient":"` + me + `","issuedAt":"2026-01-01T00:00:00Z","payload":{"role":"reader","Role":"admin"}}`),
			code: string(tt.CodeMalformedRequest), message: tt.CaseFoldedMembersWireMessage,
		},
		{
			name: "a payload the decoder rejects",
			body: receivedBody(t, func(m map[string]any) { m["payload"] = map[string]any{"role": 7} }),
			code: string(tt.CodeMalformedRequest), message: tt.PayloadMismatchWireMessage,
		},
		{
			name: "the binding's own check",
			body: receivedBody(t, nil),
			check: func(doc *tt.Document[json.RawMessage]) *tt.RejectReason {
				if doc.ID != "req-1" {
					t.Errorf("binding check saw id %q", doc.ID)
				}
				return &tt.RejectReason{Code: tt.CodeMalformedRequest, Message: "thread headers disagree"}
			},
			code: string(tt.CodeMalformedRequest), message: "thread headers disagree",
		},
		{
			name: "another specification",
			body: receivedBody(t, func(m map[string]any) { m["type"] = "https://trusttasks.org/spec/acl/revoke/0.1" }),
			code: string(tt.CodeUnsupportedType), message: tt.UnsupportedTypeWireMessage,
		},
		{
			name: "an unreached version",
			body: receivedBody(t, func(m map[string]any) { m["type"] = "https://trusttasks.org/spec/acl/grant/0.9" }),
			code: string(tt.CodeUnsupportedVersion), message: tt.UnsupportedVersionWireMessage,
		},
		{
			name: "over the specification's maxDocumentBytes",
			body: receivedBody(t, func(m map[string]any) { m["note"] = strings.Repeat("x", 700) }),
			code: string(tt.CodeMalformedRequest), message: tt.DocumentTooLargeWireMessage,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			guard := newCountingGuard()
			calls := 0
			opts := receivedOptions(guard)
			opts.BindingCheck = c.check
			out, err := tt.ConsumeReceived(context.Background(), c.body, receivedTransport, receivedSpec,
				decodePayload, countingHandler(&calls), opts)
			if err != nil {
				t.Fatal(err)
			}
			if out.Outcome.Kind != tt.OutcomeRejected {
				t.Fatalf("outcome %q", out.Outcome.Kind)
			}
			if calls != 0 || guard.claims != 0 {
				t.Fatalf("%d handler calls, %d claims", calls, guard.claims)
			}
			if got := string(out.Outcome.Error.Payload.Code); got != c.code {
				t.Fatalf("code %q, want %q", got, c.code)
			}
			if m := out.Outcome.Error.Payload.Message; m == nil || *m != c.message {
				t.Fatalf("message %v, want %q", m, c.message)
			}
			if out.Outcome.Error.Issuer == nil || *out.Outcome.Error.Issuer != me {
				t.Fatalf("error issued as %v, want the consumer", out.Outcome.Error.Issuer)
			}
			if _, code, _ := replyOf(t, out.Reply); code != c.code {
				t.Fatalf("reply code %q", code)
			}
		})
	}
}

// A body that does not parse as a whole is still answered from the routing
// members that did.
func TestConsumeReceivedAddressesAnUnparsableBodyFromItsRoutingMembers(t *testing.T) {
	body := receivedBody(t, func(m map[string]any) {
		m["threadId"] = "thread-1"
		m["expiresAt"] = false
	})
	out, err := tt.ConsumeReceived(context.Background(), body, receivedTransport, receivedSpec,
		decodePayload, countingHandler(new(int)), receivedOptions(newCountingGuard()))
	if err != nil {
		t.Fatal(err)
	}
	e := out.Outcome.Error
	if e == nil || e.Recipient == nil || *e.Recipient != peer || e.ThreadID == nil || *e.ThreadID != "thread-1" {
		t.Fatalf("error response %+v", e)
	}
}

// The size bound is the received document's, so a document within it under the
// same id is handled normally afterwards: the refusal burnt nothing.
func TestConsumeReceivedOversizedRefusalBurnsNoID(t *testing.T) {
	guard := newCountingGuard()
	calls := 0
	big := receivedBody(t, func(m map[string]any) { m["note"] = strings.Repeat("x", 700) })
	if out, _ := tt.ConsumeReceived(context.Background(), big, receivedTransport, receivedSpec,
		decodePayload, countingHandler(&calls), receivedOptions(guard)); out.Outcome.Kind != tt.OutcomeRejected {
		t.Fatalf("oversized: outcome %q", out.Outcome.Kind)
	}
	out, err := tt.ConsumeReceived(context.Background(), receivedBody(t, nil), receivedTransport, receivedSpec,
		decodePayload, countingHandler(&calls), receivedOptions(guard))
	if err != nil || out.Outcome.Kind != tt.OutcomeHandled || calls != 1 {
		t.Fatalf("within the bound: outcome %q, err %v, %d calls", out.Outcome.Kind, err, calls)
	}
}

// The item-11 digest is over the bytes as received: two documents under one id
// differing only in a member the payload type does not model are an idConflict.
func TestConsumeReceivedDigestsTheReceivedBytes(t *testing.T) {
	guard := newCountingGuard()
	calls := 0
	first := receivedBody(t, func(m map[string]any) { m["payload"] = map[string]any{"role": "admin", "scope": "a"} })
	second := receivedBody(t, func(m map[string]any) { m["payload"] = map[string]any{"role": "admin", "scope": "b"} })
	if out, err := tt.ConsumeReceived(context.Background(), first, receivedTransport, receivedSpec,
		decodePayload, countingHandler(&calls), receivedOptions(guard)); err != nil || out.Outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("first: %q %v", out.Outcome.Kind, err)
	}
	out, err := tt.ConsumeReceived(context.Background(), second, receivedTransport, receivedSpec,
		decodePayload, countingHandler(&calls), receivedOptions(guard))
	if err != nil {
		t.Fatal(err)
	}
	if out.Outcome.Kind != tt.OutcomeRejected || out.Outcome.Error.Payload.Code != string(tt.CodeIDConflict) || calls != 1 {
		t.Fatalf("second: outcome %q, error %+v, %d calls", out.Outcome.Kind, out.Outcome.Error, calls)
	}
}

// The proof is verified over the bytes as received — members the payload type
// does not model included — never over a re-encoding.
func TestConsumeReceivedVerifiesTheProofOverTheReceivedBytes(t *testing.T) {
	body := receivedBody(t, func(m map[string]any) {
		m["payload"] = map[string]any{"role": "admin", "unmodelled": true}
		m["proof"] = testProof
	})
	verifier := &recordingVerifier{}
	opts := receivedOptions(newCountingGuard())
	opts.ProofPolicy = tt.ProofPolicy{Kind: tt.ProofVerify, Verifier: verifier}
	out, err := tt.ConsumeReceived(context.Background(), body, receivedTransport, receivedSpec,
		decodePayload, countingHandler(new(int)), opts)
	if err != nil || out.Outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("outcome %q, err %v, error %+v", out.Outcome.Kind, err, out.Outcome.Error)
	}
	if string(verifier.got) != string(body) {
		t.Fatalf("verified %s, want the received bytes", verifier.got)
	}
}

type recordingVerifier struct{ got json.RawMessage }

func (v *recordingVerifier) Verify(_ context.Context, document json.RawMessage) error {
	v.got = append(json.RawMessage(nil), document...)
	return nil
}

// A later version whose extends chain reaches the implemented one is read as
// it, the payload pruned, and answered under its own type.
func TestConsumeReceivedReadsForwardAlongExtends(t *testing.T) {
	body := json.RawMessage(`{"id":"w-1","type":"` + widgetBase + `0.3","issuer":"` + peer + `","recipient":"` + me +
		`","issuedAt":"2026-01-01T00:00:00Z","payload":{"name":"a","added":"in 0.3"}}`)
	opts := receivedOptions(newCountingGuard())
	opts.Extends = widgetChain
	var seen widgetPayload
	out, err := tt.ConsumeReceived(context.Background(), body, receivedTransport, widgetSpec,
		func(raw json.RawMessage) (widgetPayload, error) {
			if strings.Contains(string(raw), "added") {
				t.Errorf("decode saw the unpruned payload %s", raw)
			}
			var p widgetPayload
			err := json.Unmarshal(raw, &p)
			return p, err
		},
		func(_ context.Context, doc *tt.Document[widgetPayload], _ tt.ResolvedParties) (*tt.Document[response], error) {
			seen = doc.Payload
			return tt.RespondWith(doc, "resp-1", response{OK: true}, fixedClock), nil
		}, opts)
	if err != nil || out.Outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("outcome %q, err %v, error %+v", out.Outcome.Kind, err, out.Outcome.Error)
	}
	if seen.Name != "a" {
		t.Fatalf("handler saw %+v", seen)
	}
	if typ, _, _ := replyOf(t, out.Reply); typ != widgetBase+"0.3#response" {
		t.Fatalf("reply type %q", typ)
	}

	// Without the chain, the same document is an unsupportedVersion.
	opts = receivedOptions(newCountingGuard())
	opts.Extends = func(string) string { return "" }
	out, err = tt.ConsumeReceived(context.Background(), body, receivedTransport, widgetSpec,
		func(json.RawMessage) (widgetPayload, error) { return widgetPayload{}, nil },
		func(context.Context, *tt.Document[widgetPayload], tt.ResolvedParties) (*tt.Document[response], error) {
			t.Error("handler ran")
			return nil, nil
		}, opts)
	if err != nil || out.Outcome.Kind != tt.OutcomeRejected || out.Outcome.Error.Payload.Code != string(tt.CodeUnsupportedVersion) {
		t.Fatalf("no chain: outcome %q, err %v", out.Outcome.Kind, err)
	}
}

// An in-band issuer contradicting the authenticated sender is identityMismatch,
// answered to the authenticated sender rather than the claimed one.
func TestConsumeReceivedAnswersAnIdentityMismatchToTheAuthenticatedSender(t *testing.T) {
	body := receivedBody(t, func(m map[string]any) { m["issuer"] = "did:web:someone-else.example" })
	out, err := tt.ConsumeReceived(context.Background(), body, receivedTransport, receivedSpec,
		decodePayload, countingHandler(new(int)), receivedOptions(newCountingGuard()))
	if err != nil {
		t.Fatal(err)
	}
	if out.Outcome.Kind != tt.OutcomeRejected || out.Outcome.Error.Payload.Code != string(tt.CodeIdentityMismatch) {
		t.Fatalf("outcome %q", out.Outcome.Kind)
	}
	if _, _, recipient := replyOf(t, out.Reply); recipient != peer {
		t.Fatalf("answered to %q", recipient)
	}
}

func TestConsumeReceivedRefusesAMisconfiguration(t *testing.T) {
	body := receivedBody(t, nil)
	handler := countingHandler(new(int))

	unset := receivedOptions(newCountingGuard())
	unset.Checks = tt.ConsumeChecks{}
	if _, err := tt.ConsumeReceived(context.Background(), body, receivedTransport, receivedSpec,
		decodePayload, handler, unset); !errors.Is(err, tt.ErrMissingOption) {
		t.Fatalf("unset Checks: %v", err)
	}
	// Even for a body that would be refused before ConsumeInbound runs.
	unset = receivedOptions(newCountingGuard())
	unset.PayloadPolicy = tt.PayloadPolicy{}
	if _, err := tt.ConsumeReceived(context.Background(), json.RawMessage(`[]`), receivedTransport, receivedSpec,
		decodePayload, handler, unset); !errors.Is(err, tt.ErrMissingOption) {
		t.Fatalf("unset PayloadPolicy: %v", err)
	}
	if _, err := tt.ConsumeReceived(context.Background(), body, tt.UnauthenticatedTransport{}, receivedSpec,
		decodePayload, handler, receivedOptions(newCountingGuard())); !errors.Is(err, tt.ErrNoAuthenticatedSender) {
		t.Fatalf("unauthenticated transport: %v", err)
	}
}

func TestNewURNUUID(t *testing.T) {
	form := regexp.MustCompile(`^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`)
	seen := map[string]bool{}
	for i := 0; i < 1000; i++ {
		id := tt.NewURNUUID()
		if !form.MatchString(id) {
			t.Fatalf("%q is not a version 4 urn:uuid", id)
		}
		if seen[id] {
			t.Fatalf("%q minted twice", id)
		}
		seen[id] = true
	}
}
