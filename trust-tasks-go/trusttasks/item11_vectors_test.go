package trusttasks_test

import (
	"context"
	"encoding/json"
	"os"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// The shared item-11 vectors (fixtures/consume at the repository root), which
// the Rust, TypeScript and Dart runtimes run too: the duplicate-execution record
// compares the document as received, so a member the consumer's payload type
// does not model still tells two documents under one id apart.

type item11Vectors struct {
	Spec     string `json:"spec"`
	Consumer string `json:"consumer"`
	Now      string `json:"now"`
	First    string `json:"first"`
	Cases    []struct {
		Name   string `json:"name"`
		First  string `json:"first"`
		Second string `json:"second"`
		Expect string `json:"expect"`
	} `json:"cases"`
}

// widget models only payload.subject, as the vectors require.
type widget struct {
	Subject string `json:"subject"`
}

func loadItem11Vectors(t *testing.T) item11Vectors {
	t.Helper()
	raw, err := os.ReadFile("../../fixtures/consume/item11-digest.json")
	if err != nil {
		t.Fatal(err)
	}
	var v item11Vectors
	if err := json.Unmarshal(raw, &v); err != nil {
		t.Fatal(err)
	}
	if len(v.Cases) == 0 {
		t.Fatal("no vectors")
	}
	return v
}

func consumeVector(
	t *testing.T,
	v item11Vectors,
	guard tt.ReplayGuard,
	received string,
	withReceived bool,
) tt.ConsumeOutcome[response] {
	t.Helper()
	var doc tt.Document[widget]
	if err := json.Unmarshal([]byte(received), &doc); err != nil {
		t.Fatal(err)
	}
	now, err := time.Parse(time.RFC3339, v.Now)
	if err != nil {
		t.Fatal(err)
	}
	opts := tt.ConsumeOptions[widget, response]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          tt.SpecPolicy{TypeURI: v.Spec},
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.ConsequentialChecks(guard),
		Doc:           &doc,
		MyVID:         v.Consumer,
		Now:           now,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(_ context.Context, d *tt.Document[widget], _ tt.ResolvedParties) (*tt.Document[response], error) {
			return tt.RespondWith[widget, response](d, "resp-1", response{OK: true}, fixedClock), nil
		},
	}
	if withReceived {
		opts.Received = json.RawMessage(received)
	}
	outcome, err := tt.ConsumeInbound(context.Background(), opts)
	if err != nil {
		t.Fatal(err)
	}
	return outcome
}

func verdictOf(outcome tt.ConsumeOutcome[response]) string {
	switch {
	case outcome.Kind == tt.OutcomeDuplicate:
		return "duplicate"
	case outcome.Kind == tt.OutcomeRejected && outcome.Error.Payload.Code == string(tt.CodeIDConflict):
		return "idConflict"
	default:
		return string(outcome.Kind)
	}
}

func TestItem11SharedVectorsCompareTheDocumentAsReceived(t *testing.T) {
	v := loadItem11Vectors(t)
	for _, c := range v.Cases {
		t.Run(c.Name, func(t *testing.T) {
			guard := tt.NewInMemoryReplayGuard(16)
			firstBody := v.First
			if c.First != "" {
				firstBody = c.First
			}
			if first := consumeVector(t, v, guard, firstBody, true); first.Kind != tt.OutcomeHandled {
				t.Fatalf("first: %s %+v", first.Kind, first.Error)
			}
			if got := verdictOf(consumeVector(t, v, guard, c.Second, true)); got != c.Expect {
				t.Fatalf("second: got %s, want %s", got, c.Expect)
			}
		})
	}
}

// Without the received bytes the digest is of the typed re-encoding, which
// cannot see a payload member the type does not model. Pinned so the
// documented limit stays the actual one.
func TestItem11WithoutReceivedDigestsTheTypedView(t *testing.T) {
	v := loadItem11Vectors(t)
	for _, c := range v.Cases {
		if c.Name != "extra payload member the consumer's type does not model" {
			continue
		}
		guard := tt.NewInMemoryReplayGuard(16)
		if first := consumeVector(t, v, guard, v.First, false); first.Kind != tt.OutcomeHandled {
			t.Fatalf("first: %s %+v", first.Kind, first.Error)
		}
		if got := verdictOf(consumeVector(t, v, guard, c.Second, false)); got != "duplicate" {
			t.Fatalf("got %s", got)
		}
		return
	}
	t.Fatal("vector not found")
}

// A Received that is not a reading of Doc vouches for nothing about Doc, so the
// digest falls back to Doc's own re-encoding rather than keying the record on
// another document.
func TestItem11IgnoresAReceivedThatIsNotTheDocumentBeingConsumed(t *testing.T) {
	v := loadItem11Vectors(t)
	now, err := time.Parse(time.RFC3339, v.Now)
	if err != nil {
		t.Fatal(err)
	}
	guard := tt.NewInMemoryReplayGuard(16)
	consume := func(received string) tt.ConsumeOutcome[response] {
		var doc tt.Document[widget]
		if err := json.Unmarshal([]byte(v.First), &doc); err != nil {
			t.Fatal(err)
		}
		out, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[widget, response]{
			Transport:     tt.UnauthenticatedTransport{},
			Spec:          tt.SpecPolicy{TypeURI: v.Spec},
			ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
			PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
			Checks:        tt.ConsequentialChecks(guard),
			Doc:           &doc,
			Received:      json.RawMessage(received),
			MyVID:         v.Consumer,
			Now:           now,
			NewErrorID:    func() string { return "err-1" },
			Clock:         fixedClock,
			Handler: func(_ context.Context, d *tt.Document[widget], _ tt.ResolvedParties) (*tt.Document[response], error) {
				return tt.RespondWith[widget, response](d, "resp-1", response{OK: true}, fixedClock), nil
			},
		})
		if err != nil {
			t.Fatal(err)
		}
		return out
	}
	// Another document entirely: a different subject, so not a reading.
	other := `{"id":"elsewhere","type":"` + v.Spec + `","payload":{"subject":"did:web:bob.example"}}`
	if first := consume(other); first.Kind != tt.OutcomeHandled {
		t.Fatalf("first: %s %+v", first.Kind, first.Error)
	}
	if got := verdictOf(consume(v.First)); got != "duplicate" {
		t.Fatalf("got %s: the record was keyed on the unrelated Received", got)
	}
}
