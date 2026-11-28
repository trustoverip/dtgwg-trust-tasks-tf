package trusttasks_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// The shared withheld-response vectors (fixtures/consume/withheld-response.json),
// which the Rust, TypeScript and Dart runtimes run too: a handler may keep its
// response out of the item-11 record (Delivery.WithholdResponse), and a duplicate
// of that document then comes back to it as a re-delivery (Delivery.Redelivery)
// instead of being answered from the record.

type withheldVectors struct {
	Spec      string            `json:"spec"`
	Consumer  string            `json:"consumer"`
	Now       string            `json:"now"`
	Documents map[string]string `json:"documents"`
	Cases     []struct {
		Name       string `json:"name"`
		Withhold   bool   `json:"withhold"`
		Deliveries []struct {
			Document        string  `json:"document"`
			Handler         *string `json:"handler"`
			Outcome         string  `json:"outcome"`
			PriorResponseOf *int    `json:"priorResponseOf"`
		} `json:"deliveries"`
	} `json:"cases"`
}

type deliveryAnswer struct {
	Delivery string `json:"delivery"`
}

func loadWithheldVectors(t *testing.T) withheldVectors {
	t.Helper()
	raw, err := os.ReadFile("../../fixtures/consume/withheld-response.json")
	if err != nil {
		t.Fatal(err)
	}
	var v withheldVectors
	if err := json.Unmarshal(raw, &v); err != nil {
		t.Fatal(err)
	}
	if len(v.Cases) == 0 {
		t.Fatal("no vectors")
	}
	return v
}

// consumeWithheld runs one delivery; seen is the signal the handler was called
// with, "" when it was not called.
func consumeWithheld(
	t *testing.T,
	v withheldVectors,
	guard tt.ReplayGuard,
	body string,
	index int,
	withhold bool,
) (outcome tt.ConsumeOutcome[deliveryAnswer], seen string) {
	t.Helper()
	var doc tt.Document[widget]
	if err := json.Unmarshal([]byte(body), &doc); err != nil {
		t.Fatal(err)
	}
	now, err := time.Parse(time.RFC3339Nano, v.Now)
	if err != nil {
		t.Fatal(err)
	}
	outcome, err = tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[widget, deliveryAnswer]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          tt.SpecPolicy{TypeURI: v.Spec},
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.ConsequentialChecks(guard),
		Doc:           &doc,
		Received:      json.RawMessage(body),
		MyVID:         v.Consumer,
		Now:           now,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(ctx context.Context, d *tt.Document[widget], _ tt.ResolvedParties) (*tt.Document[deliveryAnswer], error) {
			delivery := tt.DeliveryFrom(ctx)
			if delivery == nil {
				t.Fatal("ConsumeInbound sets the delivery")
			}
			seen = "first"
			if delivery.Redelivery() {
				seen = "redelivery"
			}
			if withhold {
				delivery.WithholdResponse()
			}
			return tt.RespondWith[widget, deliveryAnswer](d, fmt.Sprintf("resp-%d", index), deliveryAnswer{seen}, fixedClock), nil
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	return outcome, seen
}

func TestWithheldResponseSharedVectors(t *testing.T) {
	v := loadWithheldVectors(t)
	for _, c := range v.Cases {
		t.Run(c.Name, func(t *testing.T) {
			guard := tt.NewInMemoryReplayGuard(16)
			var responses []string
			for index, d := range c.Deliveries {
				outcome, seen := consumeWithheld(t, v, guard, v.Documents[d.Document], index, c.Withhold)
				got := string(outcome.Kind)
				if outcome.Kind == tt.OutcomeRejected {
					got = outcome.Error.Payload.Code
				}
				if got != d.Outcome {
					t.Fatalf("#%d: got %s, want %s", index, got, d.Outcome)
				}
				want := ""
				if d.Handler != nil {
					want = *d.Handler
				}
				if seen != want {
					t.Fatalf("#%d: handler saw %q, want %q", index, seen, want)
				}
				switch outcome.Kind {
				case tt.OutcomeHandled:
					if outcome.Response.Payload.Delivery != want {
						t.Fatalf("#%d: answered %q", index, outcome.Response.Payload.Delivery)
					}
					responses = append(responses, outcome.Response.ID)
				case tt.OutcomeDuplicate:
					if outcome.InFlight || d.PriorResponseOf == nil {
						t.Fatalf("#%d: %+v", index, outcome)
					}
					var prior struct {
						ID string `json:"id"`
					}
					if err := json.Unmarshal(outcome.PriorResponse, &prior); err != nil {
						t.Fatal(err)
					}
					if prior.ID != responses[*d.PriorResponseOf] {
						t.Fatalf("#%d: replays %q, want %q", index, prior.ID, responses[*d.PriorResponseOf])
					}
					responses = append(responses, "")
				default:
					responses = append(responses, "")
				}
			}
		})
	}
}

// claimOnly hides InMemoryReplayGuard's RecordWithheld, as a guard written
// before WithholdingReplayGuard existed would.
type claimOnly struct{ inner *tt.InMemoryReplayGuard }

func (g claimOnly) Claim(ctx context.Context, id, digest string, retainUntil, now time.Time) (tt.ReplayVerdict, error) {
	return g.inner.Claim(ctx, id, digest, retainUntil, now)
}

func (g claimOnly) RecordResponse(ctx context.Context, id string, response json.RawMessage) error {
	return g.inner.RecordResponse(ctx, id, response)
}

func (g claimOnly) Release(ctx context.Context, id, digest string) error {
	return g.inner.Release(ctx, id, digest)
}

func TestWithheldResponseOnAGuardWithoutRecordWithheldIsAbsorbedInSilence(t *testing.T) {
	v := loadWithheldVectors(t)
	guard := claimOnly{tt.NewInMemoryReplayGuard(16)}
	body := v.Documents["original"]
	if first, _ := consumeWithheld(t, v, guard, body, 0, true); first.Kind != tt.OutcomeHandled {
		t.Fatalf("first: %s", first.Kind)
	}
	again, seen := consumeWithheld(t, v, guard, body, 1, true)
	if again.Kind != tt.OutcomeDuplicate || again.PriorResponse != nil || again.InFlight || seen != "" {
		t.Fatalf("got %+v, handler saw %q", again, seen)
	}
}

func TestDeliveryFromAContextNotFromTheConsumerIsNil(t *testing.T) {
	d := tt.DeliveryFrom(context.Background())
	if d != nil || d.Redelivery() {
		t.Fatal("no delivery outside ConsumeInbound")
	}
	d.WithholdResponse() // a no-op on nil
}
