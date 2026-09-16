package tsp

import (
	"crypto/rand"
	"encoding/json"
	"errors"
	"fmt"

	tsp "github.com/affinidi/affinidi-tsp-go"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// BindingURI is the binding's stable identifier (bindings/tsp/0.1 §1).
const BindingURI = "https://trusttasks.org/binding/tsp/0.1"

// EnvelopeType is the type member of the envelope object a TSP message carries
// (§1, §2). A TSP VID is a framework VID, so — unlike DIDComm — no identifier is
// transformed on the way in or out.
const EnvelopeType = "https://trusttasks.org/binding/tsp/0.1/envelope"

// Transport is the [tt.TransportHandler] for one TSP exchange: Peer is the VID
// TSP authenticated as the sender, Local the VID that opened the message (§3).
type Transport struct {
	Local string
	Peer  string
}

// BindingURI implements [tt.TransportHandler].
func (Transport) BindingURI() string { return BindingURI }

// DeriveParties implements [tt.TransportHandler].
func (t Transport) DeriveParties() tt.TransportContext {
	ctx := tt.TransportContext{}
	if t.Peer != "" {
		ctx.Issuer = &t.Peer
	}
	if t.Local != "" {
		ctx.Recipient = &t.Local
	}
	return ctx
}

// Failure classifies why a TSP message was refused before a document could be
// produced.
type Failure uint8

const (
	// FailNotForThisReceiver: not a TSP message this VID can open — bad framing,
	// wrong receiver, failed signature or HPKE authentication.
	FailNotForThisReceiver Failure = iota
	// FailNotSealed: the message opened but is signed-only, not sealed. §2
	// requires the payload sealed as well as signed.
	FailNotSealed
	// FailNotFinalRecipient: a routed or nested message this VID only relays
	// (§5.2); the document is not opened here.
	FailNotFinalRecipient
	// FailWrongEnvelopeType: the envelope object's type is not [EnvelopeType].
	FailWrongEnvelopeType
	// FailInvalidBody: the payload is not a Trust Task envelope object, or its
	// document is not a Trust Task document.
	FailInvalidBody
)

func (f Failure) String() string {
	switch f {
	case FailNotForThisReceiver:
		return "notForThisReceiver"
	case FailNotSealed:
		return "notSealed"
	case FailNotFinalRecipient:
		return "notFinalRecipient"
	case FailWrongEnvelopeType:
		return "wrongEnvelopeType"
	case FailInvalidBody:
		return "invalidBody"
	}
	return fmt.Sprintf("Failure(%d)", uint8(f))
}

// EnvelopeError is a TSP message refused by [UnpackTrustTask]. None of these can
// be answered with a trust-task-error: TSP has no anonymous sender, so a refusal
// here leaves nothing authenticated to reply about (§4).
type EnvelopeError struct {
	Failure Failure
	// Sender is the authenticated sender where authentication got that far, else
	// empty. Detail may name VIDs and is for logs, not the wire.
	Sender string
	Detail string
	cause  error
}

func (e *EnvelopeError) Error() string {
	return fmt.Sprintf("tsp: %s: %s", e.Failure, e.Detail)
}

func (e *EnvelopeError) Unwrap() error { return e.cause }

// envelope is the object a TSP message's payload carries (§2).
type envelope struct {
	Type     string          `json:"type"`
	Document json.RawMessage `json:"document"`
}

// PackTrustTask seals document into a Direct TSP message from sender to receiver
// (§2, §5.1), returning the wire bytes.
//
// The message payload is the envelope object {"type": EnvelopeType, "document":
// document}, sealed with HPKE authenticated encryption and signed from sender's
// VID. receiver must carry an encryption key.
//
// document is marshalled with encoding/json; pass a *tt.Document[P] or any value
// that serialises to a Trust Task document.
func PackTrustTask(document any, sender *tsp.PrivateIdentity, receiver *tsp.Identity) ([]byte, error) {
	if receiver == nil || len(receiver.EncryptionKey) == 0 {
		return nil, fmt.Errorf("tsp: receiver has no encryption key; this binding seals every message (§2)")
	}
	docJSON, err := json.Marshal(document)
	if err != nil {
		return nil, fmt.Errorf("tsp: marshal document: %w", err)
	}
	body, err := json.Marshal(envelope{Type: EnvelopeType, Document: docJSON})
	if err != nil {
		return nil, fmt.Errorf("tsp: marshal envelope: %w", err)
	}
	packed, err := tsp.Pack(sender, receiver, &tsp.SCS{Data: body}, &tsp.PackOptions{
		Scheme: tsp.SchemeHPKEBase,
		Rand:   rand.Reader,
	})
	if err != nil {
		return nil, fmt.Errorf("tsp: seal: %w", err)
	}
	return packed.Message, nil
}

// AdvertisedSender is the sender VID a TSP message names, read without verifying
// anything. Unauthenticated until [UnpackTrustTask] opens the message — use it
// only to decide whether to open it (for an allowlist, or to resolve the
// sender's identity first). Empty when the bytes are not a TSP message.
func AdvertisedSender(wire []byte) string {
	env, err := tsp.Peek(wire)
	if err != nil {
		return ""
	}
	return env.Sender
}

// Unpacked is a Trust Task document taken out of a TSP message.
type Unpacked struct {
	// Document is the document as JSON — exactly as the sender serialised it,
	// which is what a proof over it was computed on.
	Document json.RawMessage
	// Transport is the authenticated sender and the opening recipient.
	Transport Transport
}

// UnpackTrustTask opens a Direct TSP message addressed to recipient and takes
// out the Trust Task document. sender is the resolved public identity whose
// signature and HPKE authentication are checked; it must be the identity the
// message names.
//
// It returns an *EnvelopeError for anything that stops before a document is in
// hand.
func UnpackTrustTask(wire []byte, recipient *tsp.PrivateIdentity, sender *tsp.Identity) (*Unpacked, error) {
	msg, err := tsp.Open(recipient, sender, wire)
	if err != nil {
		// Every open failure — bad framing, wrong receiver, wrong sender key,
		// failed signature or AEAD — is "not for this receiver to act on", and
		// none leaves an authenticated document to reply about.
		return nil, &EnvelopeError{
			Failure: FailNotForThisReceiver,
			Detail:  err.Error(),
			cause:   err,
		}
	}

	// §2: the payload MUST be sealed, not merely signed.
	if !msg.Scheme.Confidential() {
		return nil, &EnvelopeError{
			Failure: FailNotSealed,
			Sender:  msg.Sender,
			Detail:  fmt.Sprintf("message is %s, not HPKE-sealed", msg.Scheme),
		}
	}

	scs, ok := msg.Payload.(*tsp.SCS)
	if !ok {
		// A routed/nested layer this VID only relays, or a TSP control message.
		fail := FailInvalidBody
		if _, isHop := msg.Payload.(*tsp.HOP); isHop {
			fail = FailNotFinalRecipient
		}
		return nil, &EnvelopeError{
			Failure: fail,
			Sender:  msg.Sender,
			Detail:  fmt.Sprintf("payload %T is not a Trust Task envelope", msg.Payload),
		}
	}

	var env envelope
	if err := json.Unmarshal(scs.Data, &env); err != nil {
		return nil, &EnvelopeError{
			Failure: FailInvalidBody,
			Sender:  msg.Sender,
			Detail:  "payload is not a JSON envelope object",
			cause:   err,
		}
	}
	if env.Type != EnvelopeType {
		return nil, &EnvelopeError{
			Failure: FailWrongEnvelopeType,
			Sender:  msg.Sender,
			Detail:  fmt.Sprintf("envelope type %q", env.Type),
		}
	}
	if !isTrustTaskDocument(env.Document) {
		return nil, &EnvelopeError{
			Failure: FailInvalidBody,
			Sender:  msg.Sender,
			Detail:  "envelope carries no Trust Task document",
		}
	}

	local := msg.Receiver
	if local == "" {
		// The NULL-VID form, which this binding does not send; fall back to the
		// opening identity.
		local = recipient.VID
	}
	return &Unpacked{
		Document:  env.Document,
		Transport: Transport{Local: local, Peer: msg.Sender},
	}, nil
}

// isTrustTaskDocument reports whether raw is a JSON object with string id and
// type members — the minimum a Trust Task document carries (SPEC §4.1).
func isTrustTaskDocument(raw json.RawMessage) bool {
	var probe struct {
		ID   *string `json:"id"`
		Type *string `json:"type"`
	}
	if err := json.Unmarshal(raw, &probe); err != nil {
		return false
	}
	return probe.ID != nil && probe.Type != nil
}

// NewURNUUID returns a fresh urn:uuid: identifier (UUID version 4), for response
// and error documents.
func NewURNUUID() string {
	var b [16]byte
	if _, err := rand.Read(b[:]); err != nil {
		panic(err) // crypto/rand does not fail in practice; a document id needs entropy
	}
	b[6] = (b[6] & 0x0f) | 0x40
	b[8] = (b[8] & 0x3f) | 0x80
	return fmt.Sprintf("urn:uuid:%x-%x-%x-%x-%x", b[0:4], b[4:6], b[6:8], b[8:10], b[10:16])
}

// AsEnvelopeError reports whether err is an [EnvelopeError], and returns it.
func AsEnvelopeError(err error) (*EnvelopeError, bool) {
	var e *EnvelopeError
	if errors.As(err, &e) {
		return e, true
	}
	return nil, false
}
