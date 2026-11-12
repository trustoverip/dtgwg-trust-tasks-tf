package tsp_test

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// linkedCoreDigestsReceived reports whether the core this module links takes
// the item-11 digest over ConsumeOptions.Received. Until the module's core
// requirement reaches a release that does, the consumer passes Received but
// the digest is still the typed view, and the binding test below has nothing
// to observe.
func linkedCoreDigestsReceived(t *testing.T) bool {
	t.Helper()
	type subject struct {
		Subject string `json:"subject"`
	}
	guard := tt.NewInMemoryReplayGuard(4)
	now := time.Date(2026, 6, 1, 12, 0, 30, 0, time.UTC)
	consume := func(body string) tt.ConsumeOutcome[subject] {
		var doc tt.Document[subject]
		if err := json.Unmarshal([]byte(body), &doc); err != nil {
			t.Fatal(err)
		}
		out, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[subject, subject]{
			Transport:     tt.UnauthenticatedTransport{},
			Spec:          tt.SpecPolicy{TypeURI: echoType},
			ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
			PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
			Checks:        tt.ConsequentialChecks(guard),
			Doc:           &doc,
			Received:      json.RawMessage(body),
			Now:           now,
			NewErrorID:    func() string { return "err-1" },
			Handler: func(context.Context, *tt.Document[subject], tt.ResolvedParties) (*tt.Document[subject], error) {
				return nil, nil
			},
		})
		if err != nil {
			t.Fatal(err)
		}
		return out
	}
	const head = `{"id":"probe","type":"` + echoType + `","issuedAt":"2026-06-01T12:00:00Z","payload":{"subject":"s"`
	consume(head + `}}`)
	return consume(head+`,"note":"x"}}`).Kind == tt.OutcomeRejected
}

// SPEC §7.2 item 11 compares the document as received: a payload member the
// handler's type does not model still makes a different document under the
// same id.
func TestConsumerIDConflictOnAMemberThePayloadTypeDoesNotModel(t *testing.T) {
	if !linkedCoreDigestsReceived(t) {
		t.Skip("the linked core release digests the typed view; this passes once the core requirement moves to a release that digests Received")
	}
	h := newHarness(t)
	h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	r := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", func(d map[string]any) {
		d["payload"].(map[string]any)["note"] = "added"
	})))
	if codeOf(t, r) != "idConflict" {
		t.Fatalf("code = %q", codeOf(t, r))
	}
	if len(*h.calls) != 1 {
		t.Fatalf("handler ran %d times, want 1", len(*h.calls))
	}
}
