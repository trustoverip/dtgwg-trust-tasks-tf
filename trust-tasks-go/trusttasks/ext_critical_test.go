package trusttasks_test

import (
	"context"
	"encoding/json"
	"os"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// SPEC §4.5.2: a namespace marked critical is understood or refused. Runs the
// shared vectors in fixtures/consume/ext-critical.json, which the Rust,
// TypeScript and Dart runtimes run too, and drives the marking through
// ConsumeInbound.

func loadExtCriticalSchema(t *testing.T) string {
	t.Helper()
	raw, err := os.ReadFile("../../fixtures/consume/ext-critical.schema.json")
	if err != nil {
		t.Fatal(err)
	}
	return string(raw)
}

func TestExtCriticalSharedVectors(t *testing.T) {
	schema := loadExtCriticalSchema(t)
	raw, err := os.ReadFile("../../fixtures/consume/ext-critical.json")
	if err != nil {
		t.Fatal(err)
	}
	var vectors struct {
		Cases []struct {
			Name       string          `json:"name"`
			Payload    json.RawMessage `json:"payload"`
			Recognized []string        `json:"recognized"`
			Expect     string          `json:"expect"`
			Schemaless bool            `json:"schemaless"`
		} `json:"cases"`
	}
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatal(err)
	}
	if len(vectors.Cases) == 0 {
		t.Fatal("no vectors")
	}
	for _, c := range vectors.Cases {
		s := schema
		if c.Schemaless {
			s = ""
		}
		got := "ok"
		if reason := tt.CheckCriticalExtensions(c.Payload, s, c.Recognized); reason != nil {
			got = string(reason.Code)
		}
		if got != c.Expect {
			t.Errorf("%s: got %s, want %s", c.Name, got, c.Expect)
		}
	}
}

func TestExtCriticalRefusedBeforeTheHandlerAndTheRecord(t *testing.T) {
	const marked = "https://trusttasks.org/spec/conformance/marked/0.1"
	schema := loadExtCriticalSchema(t)
	now := time.Now().UTC().Truncate(time.Second)
	received, err := json.Marshal(map[string]any{
		"id":       "urn:uuid:00000000-0000-4000-8000-0000000000c1",
		"type":     marked,
		"issuedAt": now.Format(time.RFC3339),
		"payload": map[string]any{
			"subject":     "s",
			"ext":         map[string]any{"org.example.a": 1},
			"extCritical": []string{"org.example.a"},
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	guard := tt.NewInMemoryReplayGuard(0)
	runs := 0
	consume := func(recognized []string) tt.ConsumeOutcome[response] {
		t.Helper()
		// widget models only payload.subject: the typed decode drops the marking.
		var doc tt.Document[widget]
		if err := json.Unmarshal(received, &doc); err != nil {
			t.Fatal(err)
		}
		checks := tt.ConsequentialChecks(guard)
		checks.RecognizedExtensions = recognized
		outcome, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[widget, response]{
			Transport:     tt.UnauthenticatedTransport{},
			Spec:          tt.SpecPolicy{TypeURI: marked, PayloadSchema: schema},
			ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
			PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
			Checks:        checks,
			Doc:           &doc,
			Received:      received,
			MyVID:         "did:web:consumer.example",
			Now:           now,
			NewErrorID:    func() string { return "err-1" },
			Clock:         fixedClock,
			Handler: func(_ context.Context, d *tt.Document[widget], _ tt.ResolvedParties) (*tt.Document[response], error) {
				runs++
				return tt.RespondWith[widget, response](d, "resp-1", response{OK: true}, fixedClock), nil
			},
		})
		if err != nil {
			t.Fatal(err)
		}
		return outcome
	}

	refused := consume(nil)
	if refused.Kind != tt.OutcomeRejected || refused.Error.Payload.Code != string(tt.CodeUnsupportedExtension) {
		t.Fatalf("want unsupportedExtension, got %+v", refused)
	}
	if refused.Error.Payload.Retryable {
		t.Error("unsupportedExtension is not retryable")
	}
	if runs != 0 {
		t.Fatal("the handler must not run")
	}

	// The refusal wrote no record: the same document, consumed by a consumer
	// that implements the namespace, runs.
	if handled := consume([]string{"org.example.a"}); handled.Kind != tt.OutcomeHandled {
		t.Fatalf("want handled, got %+v", handled)
	}
	if runs != 1 {
		t.Fatalf("handler ran %d times", runs)
	}
}
