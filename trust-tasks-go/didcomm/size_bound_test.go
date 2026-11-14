package didcomm

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// SPEC §12.4: a document larger than its type's maxDocumentBytes is refused
// malformedRequest, answered to the transport-authenticated sender, before the
// handler and before the item-11 record, so the id stays free for a document
// within the bound. The consumer hands ConsumeInbound the exact received bytes.
func TestConsumerRefusesADocumentOverItsSizeBound(t *testing.T) {
	h := newHarness(t)
	guard := tt.NewInMemoryReplayGuard(0)
	h.consumer.Guard = guard
	bounded := tt.SpecPolicy{TypeURI: echoType, MaxDocumentBytes: 512}
	addressed := func(m map[string]any) {
		m["issuer"] = h.alice.DID
		m["recipient"] = h.bob.DID
	}
	receive := func(text string) *Received[echo] {
		t.Helper()
		handler := func(_ context.Context, doc *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[echo], error) {
			*h.calls = append(*h.calls, doc.Payload.Text)
			return tt.RespondWith[echo, echo](doc, NewMessageID(), echo{Text: upper(doc.Payload.Text)}, nil), nil
		}
		r, err := Receive(context.Background(), h.consumer, h.seal(t, echoDoc(defaultID, text, addressed)), h.resolve, bounded, decodeEcho, handler)
		if err != nil {
			t.Fatalf("receive: %v", err)
		}
		return r
	}

	r := receive(strings.Repeat("x", 1024))
	if code := codeOf(t, r); code != "malformedRequest" {
		t.Fatalf("code = %q, want malformedRequest", code)
	}
	var reply struct {
		Recipient string `json:"recipient"`
		Payload   struct {
			Message string `json:"message"`
		} `json:"payload"`
	}
	if err := json.Unmarshal(r.Reply, &reply); err != nil {
		t.Fatal(err)
	}
	if reply.Recipient != h.alice.DID {
		t.Fatalf("reply recipient = %q, want the authenticated sender", reply.Recipient)
	}
	if reply.Payload.Message != tt.DocumentTooLargeWireMessage {
		t.Fatalf("message = %q", reply.Payload.Message)
	}
	if len(*h.calls) != 0 {
		t.Fatal("handler ran on an over-bound document")
	}
	if guard.Size() != 0 {
		t.Fatalf("replay record holds %d ids, want 0", guard.Size())
	}

	// The same id, within the bound, is handled: the refusal burned nothing.
	if r := receive("hello"); r.Outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("outcome = %s, reply=%s", r.Outcome.Kind, r.Reply)
	}
	if len(*h.calls) != 1 {
		t.Fatalf("handler ran %d times, want 1", len(*h.calls))
	}
}
