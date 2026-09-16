package trusttasks

// The integration point between the transport-agnostic document model and a
// concrete transport binding (HTTPS + mTLS, DIDComm, TSP, an in-memory test
// loopback, …).
//
// Mirrors transport.rs in trust-tasks-rs and transport.ts in
// @openvtc/trust-tasks. The contract encodes SPEC.md §4.8.1 and §9.2:
//
//   - In-band issuer / recipient are authoritative when present.
//   - Transport-derived identity fills in absent members.
//   - When both are present they MUST agree; a mismatch is a validation failure
//     (identityMismatch, §8.3).

import "fmt"

// ResolvedParties is party identity after §4.8.1 precedence — the values a
// consumer applies for every subsequent framework rule referencing the issuer or
// recipient.
type ResolvedParties struct {
	Issuer    *string
	Recipient *string
}

// TransportContext is what the transport authenticated about an inbound message.
//
// A field is nil when the transport does not authenticate that party — a plain
// HTTPS handler with no client certificate leaves Issuer unset. Returning an
// entirely empty context is valid; the framework then relies on the in-band
// members and any proof they carry.
type TransportContext struct {
	Issuer    *string
	Recipient *string
}

// ConsistencyError is raised when in-band and transport-derived identity
// disagree (§7.2 item 6).
type ConsistencyError struct {
	// Party is "issuer" or "recipient".
	Party     string
	InBand    string
	Transport string
}

func (e *ConsistencyError) Error() string {
	// Local diagnostics only. The wire message is [IdentityMismatchReason]'s,
	// which deliberately names neither value — see the note there.
	return fmt.Sprintf(
		"trusttasks: in-band %s %q does not match transport-derived %q",
		e.Party, e.InBand, e.Transport,
	)
}

// TransportHandler is a transport binding's plug-in for the framework.
type TransportHandler interface {
	// BindingURI is a stable identifier for this binding, for logs and audit
	// (§9.1, §9.2).
	BindingURI() string

	// DeriveParties reports the identities the transport authenticated for the
	// message under consideration.
	DeriveParties() TransportContext
}

// ResolveParties applies §4.8.1 precedence to produce the final
// [ResolvedParties].
//
// Returns a [ConsistencyError] when an in-band member is present and disagrees
// with the transport-derived value for the same party. Callers translate that
// into an identityMismatch error response (§8.3) — or let [ConsumeInbound] do it.
func ResolveParties[P any](
	handler TransportHandler,
	doc *Document[P],
) (ResolvedParties, *ConsistencyError) {
	ctx := handler.DeriveParties()

	for _, pair := range []struct {
		party     string
		inBand    *string
		transport *string
	}{
		{"issuer", doc.Issuer, ctx.Issuer},
		{"recipient", doc.Recipient, ctx.Recipient},
	} {
		if pair.inBand != nil && pair.transport != nil && *pair.inBand != *pair.transport {
			return ResolvedParties{}, &ConsistencyError{
				Party:     pair.party,
				InBand:    *pair.inBand,
				Transport: *pair.transport,
			}
		}
	}

	parties := ResolvedParties{Issuer: doc.Issuer, Recipient: doc.Recipient}
	if parties.Issuer == nil {
		parties.Issuer = ctx.Issuer
	}
	if parties.Recipient == nil {
		parties.Recipient = ctx.Recipient
	}
	return parties, nil
}

// Reject builds the error response for doc, applying the §8.1 routing rules.
//
// Returns nil when the rejection is identityMismatch and the transport
// authenticated no sender. §8.1 is explicit that the consumer SHOULD NOT emit a
// response in that case: the in-band issuer is by definition the contested
// identity, so addressing it would be an oracle, and on any transport that signs
// error responses it would compel the consumer to emit a signed document about a
// party that did not take part in the exchange.
func Reject[P any](
	handler TransportHandler,
	doc *Document[P],
	id string,
	reason RejectReason,
	clock Clock,
) *ErrorResponse {
	var recipient *string
	if reason.Code == CodeIdentityMismatch {
		recipient = handler.DeriveParties().Issuer
		if recipient == nil {
			return nil
		}
	} else {
		recipient = doc.Issuer
	}
	return RejectWithRecipient(doc, id, ToErrorPayload(reason), recipient, clock)
}

// IdentityMismatchReason is the identityMismatch reason for a
// [ConsistencyError].
//
// §8.1 additionally requires the wire message to be sanitized: naming the
// consumer's expected transport-authenticated identity, or echoing the contested
// in-band value, leaks identity information to a possibly hostile sender. The
// standard wire form is the code alone with a non-identifying message — so the
// mismatched values are deliberately not included here. Log them locally
// instead; [ConsistencyError.Error] carries them.
func IdentityMismatchReason(_ *ConsistencyError) RejectReason {
	return RejectReason{
		Code:    CodeIdentityMismatch,
		Message: "identityMismatch: in-band identity does not match transport-derived identity",
	}
}

// UnauthenticatedTransport is a handler for transports that authenticate nothing
// — an unauthenticated HTTP POST, a public queue, paper. Party identity comes
// entirely from the in-band members and whatever proof they carry.
type UnauthenticatedTransport struct {
	// URI overrides the reported binding URI. Empty means the default below.
	URI string
}

// BindingURI implements [TransportHandler].
func (t UnauthenticatedTransport) BindingURI() string {
	if t.URI == "" {
		return "urn:trust-tasks:transport:unauthenticated"
	}
	return t.URI
}

// DeriveParties implements [TransportHandler], authenticating nothing.
func (t UnauthenticatedTransport) DeriveParties() TransportContext {
	return TransportContext{}
}

// StaticTransport is a fixed-identity handler, for tests and for transports
// resolved out-of-band.
type StaticTransport struct {
	Context TransportContext
	// URI overrides the reported binding URI. Empty means the default below.
	URI string
}

// BindingURI implements [TransportHandler].
func (t StaticTransport) BindingURI() string {
	if t.URI == "" {
		return "urn:trust-tasks:transport:static"
	}
	return t.URI
}

// DeriveParties implements [TransportHandler].
func (t StaticTransport) DeriveParties() TransportContext {
	return t.Context
}
