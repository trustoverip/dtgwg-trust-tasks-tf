package tsp_test

import (
	"context"
	"crypto/rand"
	"encoding/json"
	"errors"
	"testing"
	"time"

	atsp "github.com/affinidi/affinidi-tsp-go"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp"
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

const fixedIssuedAt = "2026-06-01T12:00:00Z"

func identity(t *testing.T, vid string) *atsp.PrivateIdentity {
	t.Helper()
	id, err := atsp.GenerateIdentity(vid, atsp.SigKeyEd25519, atsp.EncKeyX25519, rand.Reader)
	if err != nil {
		t.Fatalf("generate %s: %v", vid, err)
	}
	return id
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

const defaultID = "urn:uuid:00000000-0000-4000-8000-000000000001"

func TestRoundTrip(t *testing.T) {
	alice := identity(t, "did:example:alice")
	bob := identity(t, "did:example:bob")
	doc := echoDoc(defaultID, "hello", nil)

	wire, err := tsp.PackTrustTask(json.RawMessage(doc), alice, bob.Public())
	if err != nil {
		t.Fatal(err)
	}
	if got := tsp.AdvertisedSender(wire); got != alice.VID {
		t.Fatalf("advertised sender = %q, want %q", got, alice.VID)
	}
	unpacked, err := tsp.UnpackTrustTask(wire, bob, alice.Public())
	if err != nil {
		t.Fatal(err)
	}
	if unpacked.Transport.Peer != alice.VID || unpacked.Transport.Local != bob.VID {
		t.Fatalf("transport = %+v", unpacked.Transport)
	}
	if !jsonEqual(unpacked.Document, doc) {
		t.Fatalf("document round-trip differs:\n got %s\nwant %s", unpacked.Document, doc)
	}
}

func TestSealedInnerPayloadMatchesRustShape(t *testing.T) {
	// trust-tasks-tsp (Rust) and trust_tasks_tsp (Dart) seal exactly
	// {"type": EnvelopeType, "document": ...}. Open a Go-sealed message at the
	// TSP layer and assert the same two-key object, so all three interoperate.
	alice := identity(t, "did:example:alice")
	bob := identity(t, "did:example:bob")
	doc := echoDoc(defaultID, "hello", nil)
	wire, err := tsp.PackTrustTask(json.RawMessage(doc), alice, bob.Public())
	if err != nil {
		t.Fatal(err)
	}
	msg, err := atsp.Open(bob, alice.Public(), wire)
	if err != nil {
		t.Fatal(err)
	}
	var env map[string]json.RawMessage
	if err := json.Unmarshal(msg.Payload.(*atsp.SCS).Data, &env); err != nil {
		t.Fatal(err)
	}
	if len(env) != 2 {
		t.Fatalf("envelope has %d keys, want 2 (type, document)", len(env))
	}
	var typ string
	_ = json.Unmarshal(env["type"], &typ)
	if typ != tsp.EnvelopeType {
		t.Fatalf("envelope type = %q", typ)
	}
	if !jsonEqual(env["document"], doc) {
		t.Fatalf("envelope document differs")
	}
}

func TestConfidentiality(t *testing.T) {
	alice := identity(t, "did:example:alice")
	bob := identity(t, "did:example:bob")
	wire, err := tsp.PackTrustTask(echoDoc(defaultID, "top-secret", nil), alice, bob.Public())
	if err != nil {
		t.Fatal(err)
	}
	if containsSub(wire, []byte("top-secret")) {
		t.Fatal("plaintext leaked into the sealed message")
	}
}

func TestRefusals(t *testing.T) {
	alice := identity(t, "did:example:alice")
	bob := identity(t, "did:example:bob")
	carol := identity(t, "did:example:carol")
	seal := func(t *testing.T) []byte {
		wire, err := tsp.PackTrustTask(echoDoc(defaultID, "hello", nil), alice, bob.Public())
		if err != nil {
			t.Fatal(err)
		}
		return wire
	}

	t.Run("wrong receiver", func(t *testing.T) {
		_, err := tsp.UnpackTrustTask(seal(t), carol, alice.Public())
		assertFailure(t, err, tsp.FailNotForThisReceiver)
	})
	t.Run("wrong sender", func(t *testing.T) {
		_, err := tsp.UnpackTrustTask(seal(t), bob, carol.Public())
		assertFailure(t, err, tsp.FailNotForThisReceiver)
	})
	t.Run("tampered", func(t *testing.T) {
		wire := seal(t)
		wire[len(wire)-1] ^= 0xff
		_, err := tsp.UnpackTrustTask(wire, bob, alice.Public())
		assertFailure(t, err, tsp.FailNotForThisReceiver)
	})
	t.Run("signed-only", func(t *testing.T) {
		body, _ := json.Marshal(map[string]any{"type": tsp.EnvelopeType, "document": echoDoc(defaultID, "x", nil)})
		packed, err := atsp.Pack(alice, bob.Public(), &atsp.SCS{Data: body}, &atsp.PackOptions{Scheme: atsp.SchemeSignedOnly})
		if err != nil {
			t.Fatal(err)
		}
		_, err = tsp.UnpackTrustTask(packed.Message, bob, alice.Public())
		assertFailure(t, err, tsp.FailNotSealed)
	})
	t.Run("foreign envelope type names the sender", func(t *testing.T) {
		body, _ := json.Marshal(map[string]any{"type": "urn:other", "document": echoDoc(defaultID, "x", nil)})
		packed, _ := atsp.Pack(alice, bob.Public(), &atsp.SCS{Data: body}, &atsp.PackOptions{Scheme: atsp.SchemeHPKEBase})
		_, err := tsp.UnpackTrustTask(packed.Message, bob, alice.Public())
		e := assertFailure(t, err, tsp.FailWrongEnvelopeType)
		if e.Sender != alice.VID {
			t.Fatalf("sender = %q", e.Sender)
		}
	})
	t.Run("not a Trust Task document", func(t *testing.T) {
		body, _ := json.Marshal(map[string]any{"type": tsp.EnvelopeType, "document": map[string]any{"x": 1}})
		packed, _ := atsp.Pack(alice, bob.Public(), &atsp.SCS{Data: body}, &atsp.PackOptions{Scheme: atsp.SchemeHPKEBase})
		_, err := tsp.UnpackTrustTask(packed.Message, bob, alice.Public())
		assertFailure(t, err, tsp.FailInvalidBody)
	})
	t.Run("no encryption key", func(t *testing.T) {
		signOnly := &atsp.Identity{VID: bob.VID, SigKeyType: atsp.SigKeyEd25519, SigningKey: bob.SigningKey}
		_, err := tsp.PackTrustTask(echoDoc(defaultID, "x", nil), alice, signOnly)
		if err == nil {
			t.Fatal("want an error packing to a receiver with no encryption key")
		}
	})
}

// ── Consumer ────────────────────────────────────────────────────────────────

type harness struct {
	alice    *atsp.PrivateIdentity
	bob      *atsp.PrivateIdentity
	consumer *tsp.Consumer
	calls    *[]string
}

func newHarness(t *testing.T) harness {
	t.Helper()
	alice := identity(t, "did:example:alice")
	bob := identity(t, "did:example:bob")
	c := tsp.NewConsumer(bob)
	c.Now = func() time.Time { return fixedNow }
	calls := &[]string{}
	return harness{alice: alice, bob: bob, consumer: c, calls: calls}
}

func (h harness) seal(t *testing.T, doc json.RawMessage) []byte {
	t.Helper()
	wire, err := tsp.PackTrustTask(doc, h.alice, h.bob.Public())
	if err != nil {
		t.Fatal(err)
	}
	return wire
}

func (h harness) receive(t *testing.T, wire []byte) *tsp.Received[echo] {
	t.Helper()
	handler := func(_ context.Context, doc *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[echo], error) {
		*h.calls = append(*h.calls, doc.Payload.Text)
		if doc.Payload.Text == "crash" {
			return nil, errors.New("boom")
		}
		return tt.RespondWith[echo, echo](doc, tsp.NewURNUUID(), echo{Text: upper(doc.Payload.Text)}, nil), nil
	}
	got, err := tsp.Receive(context.Background(), h.consumer, wire, h.alice.Public(), echoSpec, decodeEcho, handler)
	if err != nil {
		t.Fatalf("receive: %v", err)
	}
	return got
}

func codeOf[R any](t *testing.T, r *tsp.Received[R]) string {
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
	if r.Transport.Peer != h.alice.VID {
		t.Fatalf("peer = %q", r.Transport.Peer)
	}
	if got := r.Outcome.Response.Payload.Text; got != "HELLO" {
		t.Fatalf("response text = %q", got)
	}
}

func TestConsumerReplyRoundTrip(t *testing.T) {
	h := newHarness(t)
	r := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	wire, err := tsp.PackReply(h.bob, h.alice.Public(), r.Reply)
	if err != nil {
		t.Fatal(err)
	}
	back, err := tsp.UnpackTrustTask(wire, h.alice, h.bob.Public())
	if err != nil {
		t.Fatal(err)
	}
	if back.Transport.Peer != h.bob.VID {
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
}

func TestConsumerReForwardAbsorbed(t *testing.T) {
	h := newHarness(t)
	first := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	again := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))
	if again.Outcome.Kind != tt.OutcomeDuplicate {
		t.Fatalf("second outcome = %s", again.Outcome.Kind)
	}
	if !jsonEqual(again.Reply, first.Reply) {
		t.Fatalf("duplicate reply differs from the first")
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
	if reply["recipient"] != h.alice.VID {
		t.Fatalf("reply recipient = %v, want the authenticated sender", reply["recipient"])
	}
	if len(*h.calls) != 0 {
		t.Fatal("handler ran")
	}
}

func TestConsumerRecipientMismatch(t *testing.T) {
	h := newHarness(t)
	doc := echoDoc(defaultID, "hello", func(m map[string]any) { m["recipient"] = "did:web:elsewhere.example" })
	r := h.receive(t, h.seal(t, doc))
	if r.Outcome.Kind != tt.OutcomeRejected {
		t.Fatalf("outcome = %s", r.Outcome.Kind)
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

func TestConsumerProofWithoutVerifier(t *testing.T) {
	h := newHarness(t)
	doc := echoDoc(defaultID, "hello", func(m map[string]any) {
		m["issuer"] = h.alice.VID
		m["recipient"] = h.bob.VID
		m["proof"] = map[string]any{
			"type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022",
			"verificationMethod": h.alice.VID + "#k", "proofPurpose": "assertionMethod",
			"created": fixedIssuedAt, "proofValue": "z1",
		}
	})
	if code := codeOf(t, h.receive(t, h.seal(t, doc))); code != "malformedRequest" {
		t.Fatalf("code = %q", code)
	}
}

func TestConsumerMalformedPayload(t *testing.T) {
	h := newHarness(t)
	doc := echoDoc(defaultID, "hello", func(m map[string]any) { m["payload"] = map[string]any{"text": 7} })
	if code := codeOf(t, h.receive(t, h.seal(t, doc))); code != "malformedRequest" {
		t.Fatalf("code = %q", code)
	}
}

func TestConsumerHandlerErrorReleasesClaim(t *testing.T) {
	h := newHarness(t)
	handler := func(_ context.Context, doc *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[echo], error) {
		*h.calls = append(*h.calls, doc.Payload.Text)
		return nil, errors.New("boom")
	}
	run := func() {
		_, err := tsp.Receive(context.Background(), h.consumer, h.seal(t, echoDoc(defaultID, "x", nil)), h.alice.Public(), echoSpec, decodeEcho, handler)
		if err == nil {
			t.Fatal("want the handler error surfaced")
		}
	}
	run()
	run()
	if len(*h.calls) != 2 {
		t.Fatalf("handler ran %d times, want 2 (claim released between)", len(*h.calls))
	}
}

func TestConsumerStoreFailsClosed(t *testing.T) {
	h := newHarness(t)
	h.consumer.Guard = brokenGuard{}
	if code := codeOf(t, h.receive(t, h.seal(t, echoDoc(defaultID, "hello", nil)))); code != "unavailable" {
		t.Fatalf("code = %q", code)
	}
	if len(*h.calls) != 0 {
		t.Fatal("handler ran")
	}
}

func TestConsumerFireAndForget(t *testing.T) {
	h := newHarness(t)
	handler := func(_ context.Context, _ *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[json.RawMessage], error) {
		return nil, nil
	}
	got, err := tsp.Receive(context.Background(), h.consumer, h.seal(t, echoDoc(defaultID, "hello", nil)), h.alice.Public(), echoSpec, decodeEcho, handler)
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

type brokenGuard struct{}

func (brokenGuard) Claim(context.Context, string, string, time.Time, time.Time) (tt.ReplayVerdict, error) {
	return tt.ReplayVerdict{}, errors.New("store down")
}
func (brokenGuard) RecordResponse(context.Context, string, json.RawMessage) error { return nil }
func (brokenGuard) Release(context.Context, string, string) error                 { return nil }

func assertFailure(t *testing.T, err error, want tsp.Failure) *tsp.EnvelopeError {
	t.Helper()
	e, ok := tsp.AsEnvelopeError(err)
	if !ok {
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

func containsSub(haystack, needle []byte) bool {
	for i := 0; i+len(needle) <= len(haystack); i++ {
		if string(haystack[i:i+len(needle)]) == string(needle) {
			return true
		}
	}
	return false
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

// Two members differing only by case would be one field to encoding/json and
// two to every other reader, so the document is refused before decode.
func TestConsumerCaseFoldedDuplicateMembers(t *testing.T) {
	for name, edit := range map[string]func(map[string]any){
		"payload":  func(m map[string]any) { m["payload"] = map[string]any{"text": "reader", "Text": "admin"} },
		"envelope": func(m map[string]any) { m["IssuedAt"] = fixedIssuedAt },
	} {
		t.Run(name, func(t *testing.T) {
			h := newHarness(t)
			if code := codeOf(t, h.receive(t, h.seal(t, echoDoc(defaultID, "hello", edit)))); code != "malformedRequest" {
				t.Fatalf("code = %q, want malformedRequest", code)
			}
		})
	}
}

// An authenticated body whose id and type are strings but whose other members
// do not parse is answered malformedRequest (binding §4) — the sender is known —
// rather than returned as an error with no reply.
func TestConsumerUnparsableBody(t *testing.T) {
	for name, edit := range map[string]func(map[string]any){
		"issuedAt": func(m map[string]any) { m["issuedAt"] = 7 },
		"threadId": func(m map[string]any) { m["threadId"] = []int{1} },
	} {
		t.Run(name, func(t *testing.T) {
			h := newHarness(t)
			r := h.receive(t, h.seal(t, echoDoc(defaultID, "hello", edit)))
			if r.Outcome.Kind != tt.OutcomeRejected {
				t.Fatalf("outcome = %s, want rejected", r.Outcome.Kind)
			}
			if code := codeOf(t, r); code != "malformedRequest" {
				t.Fatalf("code = %q, want malformedRequest", code)
			}
			var reply struct {
				ThreadID string `json:"threadId"`
				Payload  struct {
					InResponseTo struct {
						ID      string `json:"id"`
						TypeURI string `json:"typeUri"`
					} `json:"inResponseTo"`
				} `json:"payload"`
			}
			if err := json.Unmarshal(r.Reply, &reply); err != nil {
				t.Fatal(err)
			}
			if reply.Payload.InResponseTo.ID != defaultID || reply.Payload.InResponseTo.TypeURI != echoType || reply.ThreadID != defaultID {
				t.Fatalf("reply does not name the request: %s", r.Reply)
			}
			if len(*h.calls) != 0 {
				t.Fatalf("handler ran: %v", *h.calls)
			}
		})
	}
}

// SPEC §5.2 (#1117): a consumer of rooms/records/put/0.4 reads a 0.5 document —
// which declares extends: "0.4" in the registry — as 0.4, its payload pruned to
// what 0.4's schema knows, and answers it as 0.5. An undeclared later version
// of the same slug is unsupportedVersion; another slug is unsupportedType.
func TestConsumerReadsAnExtendingVersionForward(t *testing.T) {
	const base = "https://trusttasks.org/spec/rooms/records/put/"
	spec := tt.SpecPolicy{
		TypeURI:       base + "0.4",
		PayloadSchema: `{"type":"object","additionalProperties":false,"properties":{"text":{"type":"string"}}}`,
	}
	type note struct {
		Text  string  `json:"text"`
		Extra *string `json:"extra,omitempty"`
	}
	run := func(typeURI string) (*tsp.Received[note], *note) {
		h := newHarness(t)
		var seen *note
		handler := func(_ context.Context, doc *tt.Document[note], _ tt.ResolvedParties) (*tt.Document[note], error) {
			seen = &doc.Payload
			return tt.RespondWith[note, note](doc, tsp.NewURNUUID(), note{Text: "ok"}, nil), nil
		}
		wire := h.seal(t, echoDoc(defaultID, "hello", func(d map[string]any) {
			d["type"] = typeURI
			d["payload"] = map[string]any{"text": "hello", "extra": "0.5 only"}
		}))
		decode := func(raw json.RawMessage) (note, error) {
			var n note
			err := json.Unmarshal(raw, &n)
			return n, err
		}
		got, err := tsp.Receive(context.Background(), h.consumer, wire, h.alice.Public(), spec, decode, handler)
		if err != nil {
			t.Fatalf("receive: %v", err)
		}
		return got, seen
	}

	got, seen := run(base + "0.5")
	if got.Outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("0.5 outcome = %s, reply=%s", got.Outcome.Kind, got.Reply)
	}
	if seen == nil || seen.Text != "hello" || seen.Extra != nil {
		t.Fatalf("the handler saw %+v, want the payload pruned to 0.4's members", seen)
	}
	if want := base + "0.5#response"; got.Outcome.Response.Type != want {
		t.Fatalf("response typed %s, want %s", got.Outcome.Response.Type, want)
	}

	if got, _ := run(base + "0.99"); codeOf(t, got) != string(tt.CodeUnsupportedVersion) {
		t.Fatalf("undeclared later version: reply=%s", got.Reply)
	}
	if got, _ := run("https://trusttasks.org/spec/rooms/records/get/0.4"); codeOf(t, got) != string(tt.CodeUnsupportedType) {
		t.Fatalf("another slug: reply=%s", got.Reply)
	}
}
