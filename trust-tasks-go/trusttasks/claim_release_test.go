package trusttasks_test

// SPEC §7.2 item 11: a claim whose handler never returned is released, and the
// acceptance window ends where the record's retention does. Uses the helpers in
// replay_test.go.

import (
	"context"
	"errors"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

func TestItem11ReleasesTheClaimWhenTheHandlerPanics(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	executions := 0

	func() {
		defer func() {
			if recovered := recover(); recovered != "handler failed part-way" {
				t.Fatalf("expected the handler's panic to propagate unchanged, got %v", recovered)
			}
		}()
		_, _ = tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
			Transport:     tt.UnauthenticatedTransport{},
			Spec:          relaxedSpec,
			ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
			PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
			Checks:        tt.ConsequentialChecks(guard),
			Doc:           consequentialDoc(nil),
			MyVID:         me,
			Now:           testNow,
			NewErrorID:    func() string { return "err-1" },
			Clock:         fixedClock,
			Handler: func(context.Context, *tt.Document[payload], tt.ResolvedParties) (*tt.Document[response], error) {
				panic("handler failed part-way")
			},
		})
	}()
	if guard.Size() != 0 {
		t.Fatalf("the panicking handler's claim was left in flight (%d records)", guard.Size())
	}

	// The handler never finished, so a resend is a fresh attempt, not an
	// in-flight duplicate held until the record expires.
	if outcome := consequential(t, guard, consequentialDoc(nil), &executions, testNow); outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected the resend to be handled, got %q (inFlight=%v)", outcome.Kind, outcome.InFlight)
	}
	// The successful execution's record stands.
	if outcome := consequential(t, guard, consequentialDoc(nil), &executions, testNow); outcome.Kind != tt.OutcomeDuplicate || outcome.InFlight {
		t.Fatalf("expected a settled duplicate, got %q (inFlight=%v)", outcome.Kind, outcome.InFlight)
	}
	if executions != 1 {
		t.Errorf("expected one execution, got %d", executions)
	}
}

// contextCheckingGuard refuses any Release made on a context that has already
// ended, as a guard backed by a network client would.
type contextCheckingGuard struct {
	*tt.InMemoryReplayGuard
}

func (g contextCheckingGuard) Release(ctx context.Context, id, digest string) error {
	if err := ctx.Err(); err != nil {
		return err
	}
	return g.InMemoryReplayGuard.Release(ctx, id, digest)
}

func TestItem11ReleasesTheClaimWhenTheHandlerFailsBecauseItsContextWasCancelled(t *testing.T) {
	guard := contextCheckingGuard{tt.NewInMemoryReplayGuard(0)}
	executions := 0

	ctx, cancel := context.WithCancel(context.Background())
	_, err := tt.ConsumeInbound(ctx, tt.ConsumeOptions[payload, response]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          relaxedSpec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.ConsequentialChecks(guard),
		Doc:           consequentialDoc(nil),
		MyVID:         me,
		Now:           testNow,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(ctx context.Context, _ *tt.Document[payload], _ tt.ResolvedParties) (*tt.Document[response], error) {
			// The client goes away while the handler runs.
			cancel()
			<-ctx.Done()
			return nil, ctx.Err()
		},
	})
	if !errors.Is(err, context.Canceled) {
		t.Fatalf("expected the handler's cancellation to propagate, got %v", err)
	}

	if outcome := consequential(t, guard, consequentialDoc(nil), &executions, testNow); outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected the resend to be handled, got %q (inFlight=%v)", outcome.Kind, outcome.InFlight)
	}
	if executions != 1 {
		t.Errorf("expected one execution, got %d", executions)
	}
}

// The acceptance window is half-open and ends where the record does: at exactly
// the instant RecordExpiry returns the record may be gone, so the document must
// already be refused. A millisecond earlier it is still a duplicate.
func TestItem11WindowClosesAtTheInstantTheRecordMayBeDropped(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	executions := 0
	issued := testNow
	end, ok := tt.RecordExpiry(consequentialDoc(nil), tt.ConsequentialChecks(guard).Freshness, issued)
	if !ok || !end.Equal(issued.Add(tt.DefaultMaxAge+tt.DefaultSkew)) {
		t.Fatalf("unexpected record expiry %v (bounded=%v)", end, ok)
	}

	consequential(t, guard, consequentialDoc(nil), &executions, issued)

	if outcome := consequential(t, guard, consequentialDoc(nil), &executions, end.Add(-time.Millisecond)); outcome.Kind != tt.OutcomeDuplicate {
		t.Fatalf("expected a duplicate just inside the window, got %q", outcome.Kind)
	}
	atEnd := consequential(t, guard, consequentialDoc(nil), &executions, end)
	if atEnd.Kind != tt.OutcomeRejected || atEnd.Error == nil || atEnd.Error.Payload.Code != string(tt.CodeExpired) {
		t.Fatalf("expected expired at the window's end, got %q (%+v)", atEnd.Kind, atEnd.Error)
	}
	if executions != 1 {
		t.Errorf("expected one execution, got %d", executions)
	}
}

// panickyResponse panics when encoded, standing in for a MarshalJSON with a bug.
type panickyResponse struct{}

func (panickyResponse) MarshalJSON() ([]byte, error) { panic("response cannot be encoded") }

func TestItem11SettlesTheClaimWhenTheResponseEncoderPanics(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	executions := 0

	func() {
		defer func() {
			if recovered := recover(); recovered == nil {
				t.Fatal("expected the encoder's panic to propagate")
			}
		}()
		_, _ = tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, panickyResponse]{
			Transport:     tt.UnauthenticatedTransport{},
			Spec:          relaxedSpec,
			ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
			PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
			Checks:        tt.ConsequentialChecks(guard),
			Doc:           consequentialDoc(nil),
			MyVID:         me,
			Now:           testNow,
			NewErrorID:    func() string { return "err-1" },
			Clock:         fixedClock,
			Handler: func(_ context.Context, accepted *tt.Document[payload], _ tt.ResolvedParties) (*tt.Document[panickyResponse], error) {
				executions++
				return tt.RespondWith[payload, panickyResponse](accepted, "resp-1", panickyResponse{}, fixedClock), nil
			},
		})
	}()

	// The handler returned, so the effect happened: a resend is absorbed as a
	// settled duplicate, neither left in flight nor executed again.
	outcome := consequential(t, guard, consequentialDoc(nil), &executions, testNow)
	if outcome.Kind != tt.OutcomeDuplicate || outcome.InFlight {
		t.Fatalf("expected a settled duplicate, got %q (inFlight=%v)", outcome.Kind, outcome.InFlight)
	}
	if executions != 1 {
		t.Errorf("expected one execution, got %d", executions)
	}
}
