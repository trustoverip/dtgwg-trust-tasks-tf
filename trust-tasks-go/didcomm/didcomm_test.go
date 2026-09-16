package didcomm

import (
	"bytes"
	"context"
	"crypto/ecdh"
	"crypto/rand"
	"encoding/json"
	"errors"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

const echoType = "https://trusttasks.org/spec/example/echo/0.1"

var echoSpec = tt.SpecPolicy{TypeURI: echoType}

type echo struct {
	Text string `json:"text"`
}

func decodeEcho(raw json.RawMessage) (echo, error) {
	var e echo
	err := json.Unmarshal(raw, &e)
	return e, err
}

var fixedNow = time.Date(2026, 6, 1, 12, 0, 0, 0, time.UTC)

const (
	fixedIssuedAt = "2026-06-01T12:00:00Z"
	defaultID     = "urn:uuid:00000000-0000-4000-8000-000000000001"
)

// party builds a local Identity and the matching PeerKey (public view) for a DID.
func party(t *testing.T, did string) (*Identity, *PeerKey) {
	t.Helper()
	k, err := ecdh.X25519().GenerateKey(rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	kid := did + "#key-agreement-1"
	id, err := NewIdentity(did, kid, k.Bytes())
	if err != nil {
		t.Fatal(err)
	}
	pk, err := NewPeerKey(kid, k.PublicKey().Bytes())
	if err != nil {
		t.Fatal(err)
	}
	return id, pk
}

func echoDoc(id, text string, edit func(map[string]any)) json.RawMessage {
	doc := map[string]any{
		"id":       id,
		"type":     echoType,
		"issuedAt": fixedIssuedAt,
		"payload":  map[string]any{"text": text},
	}
	if edit != nil {
		edit(doc)
	}
	b, _ := json.Marshal(doc)
	return b
}

func TestRoundTrip(t *testing.T) {
	alice, alicePub := party(t, "did:example:alice")
	bob, bobPub := party(t, "did:example:bob")
	doc := echoDoc(defaultID, "hello", nil)

	wire, err := PackTrustTask(doc, alice, bobPub)
	if err != nil {
		t.Fatal(err)
	}
	if got := AdvertisedSender(wire); got != alice.DID {
		t.Fatalf("advertised sender = %q, want %q", got, alice.DID)
	}
	unpacked, err := UnpackTrustTask(wire, bob, resolverFor(alice.DID, alicePub))
	if err != nil {
		t.Fatal(err)
	}
	if unpacked.Transport.Peer != alice.DID || unpacked.Transport.Local != bob.DID {
		t.Fatalf("transport = %+v", unpacked.Transport)
	}
	if !jsonEqual(unpacked.Document, doc) {
		t.Fatalf("document round-trip differs:\n got %s\nwant %s", unpacked.Document, doc)
	}
}

func TestConfidentiality(t *testing.T) {
	alice, _ := party(t, "did:example:alice")
	_, bobPub := party(t, "did:example:bob")
	wire, err := PackTrustTask(echoDoc(defaultID, "top-secret", nil), alice, bobPub)
	if err != nil {
		t.Fatal(err)
	}
	if bytes.Contains(wire, []byte("top-secret")) {
		t.Fatal("plaintext leaked into the sealed message")
	}
}

func TestWrongRecipient(t *testing.T) {
	alice, alicePub := party(t, "did:example:alice")
	_, bobPub := party(t, "did:example:bob")
	carol, _ := party(t, "did:example:carol")
	wire, err := PackTrustTask(echoDoc(defaultID, "hello", nil), alice, bobPub)
	if err != nil {
		t.Fatal(err)
	}
	_, err = UnpackTrustTask(wire, carol, resolverFor(alice.DID, alicePub))
	assertFailure(t, err, FailNotForThisRecipient)
}

func TestWrongEnvelopeType(t *testing.T) {
	alice, alicePub := party(t, "did:example:alice")
	bob, bobPub := party(t, "did:example:bob")
	msg := didcommMessage{ID: NewMessageID(), Type: "urn:other", From: alice.DID, To: []string{bob.DID}, Body: echoDoc(defaultID, "x", nil)}
	plaintext, _ := json.Marshal(msg)
	wire, err := packAuthcrypt(plaintext, alice.priv, alice.Kid, bobPub.pub, bob.Kid)
	if err != nil {
		t.Fatal(err)
	}
	_, err = UnpackTrustTask(wire, bob, resolverFor(alice.DID, alicePub))
	e := assertFailure(t, err, FailWrongEnvelopeType)
	if e.Sender != alice.Kid {
		t.Fatalf("sender = %q, want %q", e.Sender, alice.Kid)
	}
}

// ── Consumer ────────────────────────────────────────────────────────────────

type harness struct {
	alice    *Identity
	alicePub *PeerKey
	bob      *Identity
	bobPub   *PeerKey
	consumer *Consumer
	resolve  ResolveSender
	calls    *[]string
}

func newHarness(t *testing.T) harness {
	t.Helper()
	alice, alicePub := party(t, "did:example:alice")
	bob, bobPub := party(t, "did:example:bob")
	c := NewConsumer(bob)
	c.Now = func() time.Time { return fixedNow }
	return harness{
		alice: alice, alicePub: alicePub, bob: bob, bobPub: bobPub,
		consumer: c, resolve: resolverFor(alice.DID, alicePub), calls: &[]string{},
	}
}

func (h harness) seal(t *testing.T, doc json.RawMessage) []byte {
	t.Helper()
	wire, err := PackTrustTask(doc, h.alice, h.bobPub)
	if err != nil {
		t.Fatal(err)
	}
	return wire
}

func (h harness) receive(t *testing.T, wire []byte) *Received[echo] {
	t.Helper()
	handler := func(_ context.Context, doc *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[echo], error) {
		*h.calls = append(*h.calls, doc.Payload.Text)
		return tt.RespondWith[echo, echo](doc, NewMessageID(), echo{Text: upper(doc.Payload.Text)}, nil), nil
	}
	got, err := Receive(context.Background(), h.consumer, wire, h.resolve, echoSpec, decodeEcho, handler)
	if err != nil {
		t.Fatalf("receive: %v", err)
	}
	return got
}

func codeOf(t *testing.T, r *Received[echo]) string {
	t.Helper()
	if r.Reply == nil {
		return ""
	}
	var d struct {
		Payload struct {
			Code string `json:"code"`
		} `json:"payload"`
	}
	if err := json.Unmarshal(r.Reply, &d); err != nil {
		t.Fatal(err)
	}
	return d.Payload.Code
}

func TestConsumerIdentityOmitted(t *testing.T) {
	h := newHarness(t)
	r := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	if r.Outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("outcome = %s, reply=%s", r.Outcome.Kind, r.Reply)
	}
	if r.Transport.Peer != h.alice.DID {
		t.Fatalf("peer = %q", r.Transport.Peer)
	}
	if got := r.Outcome.Response.Payload.Text; got != "HELLO" {
		t.Fatalf("response text = %q", got)
	}
}

func TestConsumerReplyRoundTrip(t *testing.T) {
	h := newHarness(t)
	r := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	wire, err := PackReply(h.bob, h.alicePub, r.Reply)
	if err != nil {
		t.Fatal(err)
	}
	back, err := UnpackTrustTask(wire, h.alice, resolverFor(h.bob.DID, h.bobPub))
	if err != nil {
		t.Fatal(err)
	}
	if back.Transport.Peer != h.bob.DID {
		t.Fatalf("reply peer = %q", back.Transport.Peer)
	}
	var d struct {
		Type     string `json:"type"`
		ThreadID string `json:"threadId"`
	}
	_ = json.Unmarshal(back.Document, &d)
	if d.Type != echoType+"#response" || d.ThreadID != defaultID {
		t.Fatalf("reply doc = %+v", d)
	}
	// The reply's DIDComm thid tracks the document's threadId (binding §3.1).
	if back.Thid != defaultID {
		t.Fatalf("reply thid = %q, want %q", back.Thid, defaultID)
	}
}

func TestConsumerReDeliveryAbsorbed(t *testing.T) {
	h := newHarness(t)
	first := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	again := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	if again.Outcome.Kind != tt.OutcomeDuplicate {
		t.Fatalf("second outcome = %s", again.Outcome.Kind)
	}
	if !jsonEqual(again.Reply, first.Reply) {
		t.Fatal("duplicate reply differs from the first")
	}
	if len(*h.calls) != 1 {
		t.Fatalf("handler ran %d times, want 1", len(*h.calls))
	}
}

func TestConsumerIDConflict(t *testing.T) {
	h := newHarness(t)
	h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	r := h.receive(t, h.seal(t, echoDoc(defaultID, "changed", nil)))
	if codeOf(t, r) != "idConflict" {
		t.Fatalf("code = %q", codeOf(t, r))
	}
	if len(*h.calls) != 1 {
		t.Fatalf("handler ran %d times", len(*h.calls))
	}
}

func TestConsumerIdentityMismatch(t *testing.T) {
	h := newHarness(t)
	doc := echoDoc(defaultID, "hello", func(m map[string]any) { m["issuer"] = "did:web:someone-else.example" })
	r := h.receive(t, h.seal(t, doc))
	if codeOf(t, r) != "identityMismatch" {
		t.Fatalf("code = %q", codeOf(t, r))
	}
	var reply map[string]any
	_ = json.Unmarshal(r.Reply, &reply)
	if reply["recipient"] != h.alice.DID {
		t.Fatalf("reply recipient = %v, want the authenticated sender", reply["recipient"])
	}
	if len(*h.calls) != 0 {
		t.Fatal("handler ran")
	}
}

func TestConsumerStale(t *testing.T) {
	h := newHarness(t)
	doc := echoDoc(defaultID, "hello", func(m map[string]any) { m["issuedAt"] = "2026-05-01T00:00:00Z" })
	if code := codeOf(t, h.receive(t, h.seal(t, doc))); code != "expired" {
		t.Fatalf("code = %q", code)
	}
}

func TestConsumerThreadMismatch(t *testing.T) {
	h := newHarness(t)
	// The document declares one threadId; the DIDComm thid says another. Binding
	// §3.1: both present and unequal is malformedRequest, routed to the sender.
	doc := echoDoc(defaultID, "hello", func(m map[string]any) { m["threadId"] = "urn:thread:a" })
	msg := didcommMessage{ID: NewMessageID(), Type: EnvelopeType, From: h.alice.DID, To: []string{h.bob.DID}, Thid: "urn:thread:b", Body: doc}
	plaintext, _ := json.Marshal(msg)
	wire, err := packAuthcrypt(plaintext, h.alice.priv, h.alice.Kid, h.bobPub.pub, h.bob.Kid)
	if err != nil {
		t.Fatal(err)
	}
	handler := func(_ context.Context, doc *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[echo], error) {
		*h.calls = append(*h.calls, doc.Payload.Text)
		return tt.RespondWith[echo, echo](doc, NewMessageID(), echo{Text: "x"}, nil), nil
	}
	r, err := Receive(context.Background(), h.consumer, wire, h.resolve, echoSpec, decodeEcho, handler)
	if err != nil {
		t.Fatal(err)
	}
	if codeOf(t, r) != "malformedRequest" {
		t.Fatalf("code = %q, want malformedRequest", codeOf(t, r))
	}
	if len(*h.calls) != 0 {
		t.Fatal("handler ran on a thread-inconsistent document")
	}
}

func TestConsumerProofWithoutVerifier(t *testing.T) {
	h := newHarness(t)
	doc := echoDoc(defaultID, "hello", func(m map[string]any) {
		m["issuer"] = h.alice.DID
		m["recipient"] = h.bob.DID
		m["proof"] = map[string]any{
			"type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022",
			"verificationMethod": h.alice.DID + "#k", "proofPurpose": "assertionMethod",
			"created": fixedIssuedAt, "proofValue": "z1",
		}
	})
	if code := codeOf(t, h.receive(t, h.seal(t, doc))); code != "malformedRequest" {
		t.Fatalf("code = %q", code)
	}
}

func TestConsumerFireAndForget(t *testing.T) {
	h := newHarness(t)
	handler := func(_ context.Context, _ *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[json.RawMessage], error) {
		return nil, nil
	}
	got, err := Receive(context.Background(), h.consumer, h.seal(t, echoDoc(defaultID, "hello", nil)), h.resolve, echoSpec, decodeEcho, handler)
	if err != nil {
		t.Fatal(err)
	}
	if got.Outcome.Kind != tt.OutcomeAccepted {
		t.Fatalf("outcome = %s", got.Outcome.Kind)
	}
	if got.Reply != nil {
		t.Fatalf("reply = %s, want nil", got.Reply)
	}
}

// ── helpers ─────────────────────────────────────────────────────────────────

func resolverFor(did string, key *PeerKey) ResolveSender {
	return func(senderDID string) (*PeerKey, error) {
		if senderDID != did {
			return nil, errors.New("unknown sender " + senderDID)
		}
		return key, nil
	}
}

func assertFailure(t *testing.T, err error, want Failure) *EnvelopeError {
	t.Helper()
	var e *EnvelopeError
	if !errors.As(err, &e) {
		t.Fatalf("err = %v, want *EnvelopeError", err)
	}
	if e.Failure != want {
		t.Fatalf("failure = %s, want %s", e.Failure, want)
	}
	return e
}

func jsonEqual(a, b json.RawMessage) bool {
	var av, bv any
	if json.Unmarshal(a, &av) != nil || json.Unmarshal(b, &bv) != nil {
		return false
	}
	aj, _ := json.Marshal(av)
	bj, _ := json.Marshal(bv)
	return string(aj) == string(bj)
}

func upper(s string) string {
	b := []byte(s)
	for i, c := range b {
		if c >= 'a' && c <= 'z' {
			b[i] = c - 32
		}
	}
	return string(b)
}
