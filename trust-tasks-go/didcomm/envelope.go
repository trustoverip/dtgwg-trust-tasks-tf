package didcomm

import (
	"crypto/ecdh"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"strings"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// BindingURI is the binding's stable identifier (bindings/didcomm/0.2 §1). It
// is never on the wire. The identifier moved with the minor; the envelope
// [EnvelopeType] deliberately did not, so a 0.1 and a 0.2 implementation stay
// mutually intelligible on the wire (binding §7.1).
const BindingURI = "https://trusttasks.org/binding/didcomm/0.2"

// EnvelopeType is the DIDComm message `type` a Trust Task envelope carries
// (binding §2). A consumer rejects any other `type`.
const EnvelopeType = "https://trusttasks.org/binding/didcomm/0.1/envelope"

// mediaTypeEncrypted is the JWE `typ` header for a DIDComm v2 encrypted message.
const mediaTypeEncrypted = "application/didcomm-encrypted+json"

// Transport is the [tt.TransportHandler] for one DIDComm v2 exchange: Peer is
// the authcrypt-verified sender DID, Local the DID the message was sealed to.
type Transport struct {
	Local string
	Peer  string
}

// BindingURI implements [tt.TransportHandler].
func (Transport) BindingURI() string { return BindingURI }

// DeriveParties implements [tt.TransportHandler]: the verified peer is the
// issuer, the local DID the recipient; the core applies SPEC §4.8.1 precedence.
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

// Failure is why a DIDComm envelope was refused before a document reached the
// §7.2 pipeline. Binding §4 keeps the first four out of the pipeline entirely —
// there is no authenticated party to route a trust-task-error to.
type Failure int

const (
	// FailNotAuthcrypt: the envelope is plaintext, anoncrypt, or otherwise not
	// an authcrypt JWE. No authenticated sender (binding §2, §4).
	FailNotAuthcrypt Failure = iota
	// FailNotForThisRecipient: no recipient entry this identity can open.
	FailNotForThisRecipient
	// FailSenderUnresolved: the sender's key could not be resolved.
	FailSenderUnresolved
	// FailDecrypt: key unwrap or content decryption failed — a forged skid,
	// wrong recipient, or tampering.
	FailDecrypt
	// FailWrongEnvelopeType: the DIDComm `type` is not [EnvelopeType].
	FailWrongEnvelopeType
	// FailInvalidBody: the message body is not a JSON object / Trust Task shape.
	FailInvalidBody
)

func (f Failure) String() string {
	switch f {
	case FailNotAuthcrypt:
		return "notAuthcrypt"
	case FailNotForThisRecipient:
		return "notForThisRecipient"
	case FailSenderUnresolved:
		return "senderUnresolved"
	case FailDecrypt:
		return "decrypt"
	case FailWrongEnvelopeType:
		return "wrongEnvelopeType"
	case FailInvalidBody:
		return "invalidBody"
	default:
		return "unknown"
	}
}

// EnvelopeError is a DIDComm message refused by [UnpackTrustTask].
type EnvelopeError struct {
	Failure Failure
	Detail  string
	// Sender is the authenticated sender DID URL where authentication got that
	// far; empty otherwise.
	Sender string
}

func (e *EnvelopeError) Error() string {
	return fmt.Sprintf("didcomm: %s: %s", e.Failure, e.Detail)
}

// Identity is a local X25519 key-agreement identity: the DID it belongs to, the
// DID URL (kid) that names the key, and the private key itself.
type Identity struct {
	// DID is the bare DID, e.g. did:example:alice.
	DID string
	// Kid is the key-agreement DID URL, e.g. did:example:alice#key-agreement-1.
	Kid  string
	priv *ecdh.PrivateKey
}

// NewIdentity builds an [Identity] from a 32-byte X25519 private key.
func NewIdentity(did, kid string, x25519Priv []byte) (*Identity, error) {
	priv, err := ecdh.X25519().NewPrivateKey(x25519Priv)
	if err != nil {
		return nil, fmt.Errorf("didcomm: invalid X25519 private key: %w", err)
	}
	return &Identity{DID: did, Kid: kid, priv: priv}, nil
}

// PeerKey is a remote party's X25519 key-agreement public key and the DID URL
// that names it.
type PeerKey struct {
	Kid string
	pub *ecdh.PublicKey
}

// NewPeerKey builds a [PeerKey] from a 32-byte X25519 public key.
func NewPeerKey(kid string, x25519Pub []byte) (*PeerKey, error) {
	pub, err := ecdh.X25519().NewPublicKey(x25519Pub)
	if err != nil {
		return nil, fmt.Errorf("didcomm: invalid X25519 public key: %w", err)
	}
	return &PeerKey{Kid: kid, pub: pub}, nil
}

// didcommMessage is the DIDComm v2 plaintext message (binding §2). Only the
// members the binding populates or reads are modelled; a producer MAY add more,
// and this struct preserves none it does not name.
type didcommMessage struct {
	ID    string          `json:"id"`
	Type  string          `json:"type"`
	From  string          `json:"from,omitempty"`
	To    []string        `json:"to,omitempty"`
	Thid  string          `json:"thid,omitempty"`
	Pthid string          `json:"pthid,omitempty"`
	Body  json.RawMessage `json:"body"`
}

// threadHeaders is the subset of a Trust Task document the binding cross-checks
// against the DIDComm thread headers (binding §3.1).
type threadHeaders struct {
	ID             string `json:"id"`
	ThreadID       string `json:"threadId"`
	ParentThreadID string `json:"parentThreadId"`
}

// Unpacked is a Trust Task document taken out of a DIDComm envelope.
type Unpacked struct {
	// Document is the body, exactly as the sender serialised it.
	Document json.RawMessage
	// Transport is the authenticated sender and opening recipient.
	Transport Transport
	// Thid and Pthid are the DIDComm thread headers, for the §3.1 cross-check
	// the consumer performs against the document's threadId / parentThreadId.
	Thid  string
	Pthid string
}

// PackTrustTask seals doc into a DIDComm v2.1 authcrypt envelope from sender to
// recipient (binding §2, §3.1). doc is the serialised Trust Task document; its
// threadId (or, failing that, its id) becomes the DIDComm `thid`, and its
// parentThreadId becomes `pthid`.
func PackTrustTask(doc json.RawMessage, sender *Identity, recipient *PeerKey) ([]byte, error) {
	var th threadHeaders
	if err := json.Unmarshal(doc, &th); err != nil {
		return nil, fmt.Errorf("didcomm: document is not a JSON object: %w", err)
	}
	thid := th.ThreadID
	if thid == "" {
		thid = th.ID
	}
	msg := didcommMessage{
		ID:    NewMessageID(),
		Type:  EnvelopeType,
		From:  sender.DID,
		To:    []string{bareDID(recipient.Kid)},
		Thid:  thid,
		Pthid: th.ParentThreadID,
		Body:  doc,
	}
	plaintext, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	return packAuthcrypt(plaintext, sender.priv, sender.Kid, recipient.pub, recipient.Kid)
}

// ResolveSender maps an authenticated sender DID to its X25519 key-agreement
// key. The binding does no DID resolution itself (the Go-style model): the
// caller supplies this, looking the DID up in its own store.
type ResolveSender func(senderDID string) (*PeerKey, error)

// UnpackTrustTask opens a DIDComm v2.1 authcrypt envelope addressed to
// recipient and takes out the Trust Task document. The sender's key is resolved
// from the envelope's authenticated skid via resolve. Returns an
// [*EnvelopeError] for a message that never reaches the pipeline (binding §4):
// a non-authcrypt envelope, a wrong `type`, a decryption failure, or an
// unresolvable sender.
func UnpackTrustTask(wire []byte, recipient *Identity, resolve ResolveSender) (*Unpacked, error) {
	inner, err := unpackAuthcrypt(wire, recipient.priv, recipient.Kid, func(skid string) (*ecdh.PublicKey, error) {
		pk, err := resolve(bareDID(skid))
		if err != nil {
			return nil, err
		}
		return pk.pub, nil
	})
	if err != nil {
		return nil, err
	}

	var msg didcommMessage
	if err := json.Unmarshal(inner.plaintext, &msg); err != nil {
		return nil, &EnvelopeError{Failure: FailInvalidBody, Detail: "decrypted payload is not a DIDComm message", Sender: inner.senderKid}
	}
	if msg.Type != EnvelopeType {
		return nil, &EnvelopeError{Failure: FailWrongEnvelopeType, Detail: msg.Type, Sender: inner.senderKid}
	}
	if len(msg.Body) == 0 {
		return nil, &EnvelopeError{Failure: FailInvalidBody, Detail: "message carries no body", Sender: inner.senderKid}
	}

	return &Unpacked{
		Document:  msg.Body,
		Transport: Transport{Local: bareDID(inner.recipientKid), Peer: bareDID(inner.senderKid)},
		Thid:      msg.Thid,
		Pthid:     msg.Pthid,
	}, nil
}

// AdvertisedSender reads the sender DID a JWE envelope's protected header names,
// without opening it — unauthenticated until [UnpackTrustTask] verifies it. Use
// it only to decide whether to open a message, or to resolve the sender's key.
func AdvertisedSender(wire []byte) string {
	var env jweJSON
	if err := json.Unmarshal(wire, &env); err != nil {
		return ""
	}
	protectedJSON, err := b64.DecodeString(env.Protected)
	if err != nil {
		return ""
	}
	var header protectedHeader
	if err := json.Unmarshal(protectedJSON, &header); err != nil {
		return ""
	}
	return bareDID(header.Skid)
}

// bareDID strips the #fragment from a DID URL: did:example:a#key-1 → did:example:a.
func bareDID(didURL string) string {
	if i := strings.IndexByte(didURL, '#'); i >= 0 {
		return didURL[:i]
	}
	return didURL
}

// NewMessageID returns a fresh DIDComm message id (a UUID URN), independent of
// the Trust Task document's own id (binding §2).
func NewMessageID() string {
	return "urn:uuid:" + uuidV4()
}

func uuidV4() string {
	var b [16]byte
	_, _ = rand.Read(b[:])
	b[6] = (b[6] & 0x0f) | 0x40
	b[8] = (b[8] & 0x3f) | 0x80
	const hexd = "0123456789abcdef"
	buf := make([]byte, 36)
	for i, p := range []int{0, 2, 4, 6, 9, 11, 14, 16, 19, 21, 24, 26, 28, 30, 32, 34} {
		buf[p] = hexd[b[i]>>4]
		buf[p+1] = hexd[b[i]&0x0f]
	}
	buf[8], buf[13], buf[18], buf[23] = '-', '-', '-', '-'
	return string(buf)
}
