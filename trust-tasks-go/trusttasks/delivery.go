package trusttasks

import (
	"context"
	"sync/atomic"
)

// Delivery says which delivery of a document a [Handler] is answering, and
// lets it keep its response out of the SPEC §7.2 item 11 record.
// [ConsumeInbound] puts it on the context it passes the handler; read it with
// [DeliveryFrom].
//
// By default a duplicate document is answered from the response its first
// execution produced and never reaches the handler. A specification can rule
// that out: hcp/grants/request 0.1 issues an access token the provider keeps in
// no record, and answers a re-delivered copy of an answered request with a
// newly issued token. Such a handler calls [Delivery.WithholdResponse] while
// answering the first delivery; the id stays recorded as executed, but the
// response is not stored ([WithholdingReplayGuard]). A duplicate of that
// document is then handed to the handler again with [Delivery.Redelivery]
// true — the document has already been executed, so the handler answers it (a
// new token for the same grant) and must not execute it again — and its answer
// is the [OutcomeHandled], [OutcomeRejected] or [OutcomeAccepted] outcome.
//
// A handler that never calls WithholdResponse sees no change. The same
// mechanism exists in every runtime (Rust Delivery::current(), TypeScript
// parties.delivery, Dart Delivery.current); see docs/runtime-parity.md.
type Delivery struct {
	redelivery bool
	withheld   atomic.Bool
}

type deliveryKey struct{}

func withDelivery(ctx context.Context, d *Delivery) context.Context {
	return context.WithValue(ctx, deliveryKey{}, d)
}

// DeliveryFrom returns the [Delivery] [ConsumeInbound] put on the handler's
// context, or nil for a context that did not come from it.
func DeliveryFrom(ctx context.Context) *Delivery {
	d, _ := ctx.Value(deliveryKey{}).(*Delivery)
	return d
}

// Redelivery reports whether this document was executed before and its
// response kept out of the record: answer it, and do not execute it again
// (§7.2 item 11). False on the first execution, and on a nil Delivery.
func (d *Delivery) Redelivery() bool {
	return d != nil && d.redelivery
}

// WithholdResponse keeps the response the handler returns out of the
// duplicate-execution record. The id is still recorded as executed, so a
// duplicate is never executed again; it is handed back to the handler as a
// re-delivery instead of being answered from the record.
//
// Applies to whatever the first execution answers: a success response, a
// non-retryable [Refusal], or nothing. A retryable refusal releases the claim
// whatever this says. No effect on a re-delivery (nothing is recorded for
// one), without a replay guard, or on a nil Delivery.
func (d *Delivery) WithholdResponse() {
	if d != nil {
		d.withheld.Store(true)
	}
}
