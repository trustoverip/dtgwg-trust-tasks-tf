// Tests for forward-minor routing along a declared extension (SPEC §5.2).
//
// No specification declares `extends` yet, so these do not use the generated
// index: they hand ReadForSpecWith a stub ExtendsLookup declaring the chain
// widget/0.3 -> 0.2 -> 0.1, and a hand-written SpecPolicy for widget/0.1 whose
// schema exercises every construct the pruner follows. RegistryExtends is the
// same function type over the generated index, so nothing here depends on a
// spec edit.
package trusttasks_test

import (
	"context"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"reflect"
	"strings"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

const widgetBase = "https://trusttasks.org/spec/test/widget/"

// widgetChain declares 0.3 extends 0.2 extends 0.1; 0.4 declares nothing.
func widgetChain(typeURI string) string {
	if i := strings.IndexByte(typeURI, '#'); i >= 0 {
		typeURI = typeURI[:i]
	}
	return map[string]string{
		widgetBase + "0.3": "0.2",
		widgetBase + "0.2": "0.1",
	}[typeURI]
}

// widgetSchema is widget/0.1's payload schema: closed objects, a $ref'd
// definition, an array of them, a oneOf discriminated by `kind`, and a
// free-form object whose members must survive.
const widgetSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "additionalProperties": false,
  "required": ["name"],
  "properties": {
    "name": {"type": "string"},
    "count": {"type": "integer"},
    "item": {"$ref": "#/$defs/Item"},
    "items": {"type": "array", "items": {"$ref": "#/$defs/Item"}},
    "shape": {"oneOf": [
      {"type": "object", "additionalProperties": false, "required": ["kind", "r"],
       "properties": {"kind": {"const": "circle"}, "r": {"type": "number"}}},
      {"type": "object", "additionalProperties": false, "required": ["kind", "side"],
       "properties": {"kind": {"const": "square"}, "side": {"type": "number"}}}
    ]},
    "meta": {"type": "object"}
  },
  "$defs": {
    "Item": {"type": "object", "additionalProperties": false, "required": ["sku"],
             "properties": {"sku": {"type": "string"}}},
    "Response": {"type": "object", "additionalProperties": false,
                 "properties": {"ok": {"type": "boolean"}}}
  }
}`

var widgetSpec = tt.SpecPolicy{
	TypeURI:       widgetBase + "0.1",
	PayloadSchema: widgetSchema,
}

type widgetItem struct {
	SKU string `json:"sku"`
}

type widgetShape struct {
	Kind string   `json:"kind"`
	R    *float64 `json:"r,omitempty"`
	Side *float64 `json:"side,omitempty"`
}

type widgetPayload struct {
	Name  string                     `json:"name"`
	Count *int64                     `json:"count,omitempty"`
	Item  *widgetItem                `json:"item,omitempty"`
	Items *[]widgetItem              `json:"items,omitempty"`
	Shape *widgetShape               `json:"shape,omitempty"`
	Meta  map[string]json.RawMessage `json:"meta,omitempty"`
}

type widgetResponse struct {
	OK bool `json:"ok"`
}

// widget03 is a widget/0.3 document carrying members 0.1 does not know, at the
// top of the payload, inside a $ref'd object, inside array items and inside a
// oneOf branch — plus an unknown envelope member, which is not the pruner's to
// touch.
func widget03(version string) string {
	return `{
  "id": "req-1",
  "type": "` + widgetBase + version + `",
  "issuer": "` + peer + `",
  "recipient": "` + me + `",
  "x-envelope": {"kept": true},
  "payload": {
    "name": "w",
    "count": 9007199254740993,
    "colour": "red",
    "item": {"sku": "a", "weight": 3},
    "items": [{"sku": "b", "tags": []}, {"sku": "c"}],
    "shape": {"kind": "square", "side": 2.50, "rounded": true},
    "meta": {"anything": "goes"}
  }
}`
}

func TestReadForSpecExactVersionIsUnchanged(t *testing.T) {
	received := json.RawMessage(widget03("0.1"))
	reading, refusal, err := tt.ReadForSpecWith(received, widgetSpec, widgetChain)
	if err != nil || refusal != nil {
		t.Fatalf("exact version refused: %v %+v", err, refusal)
	}
	if string(reading) != string(received) {
		t.Fatalf("exact version was rewritten:\n%s", reading)
	}
}

func TestReadForSpecPrunesAnExtendingVersion(t *testing.T) {
	for _, version := range []string{"0.2", "0.3"} { // direct, and along the chain
		reading, refusal, err := tt.ReadForSpecWith(json.RawMessage(widget03(version)), widgetSpec, widgetChain)
		if err != nil || refusal != nil {
			t.Fatalf("%s: refused: %v %+v", version, err, refusal)
		}
		var got map[string]any
		if err := json.Unmarshal(reading, &got); err != nil {
			t.Fatal(err)
		}
		if got["type"] != widgetBase+version {
			t.Fatalf("%s: type rewritten to %v", version, got["type"])
		}
		if _, ok := got["x-envelope"]; !ok {
			t.Fatalf("%s: unknown envelope member dropped", version)
		}
		var want map[string]any
		_ = json.Unmarshal([]byte(`{
		  "name": "w",
		  "count": 9007199254740993,
		  "item": {"sku": "a"},
		  "items": [{"sku": "b"}, {"sku": "c"}],
		  "shape": {"kind": "square", "side": 2.50},
		  "meta": {"anything": "goes"}
		}`), &want)
		if !reflect.DeepEqual(got["payload"], want) {
			t.Fatalf("%s: payload pruned to %v", version, got["payload"])
		}
		// Number literals travel verbatim, not through float64.
		if !strings.Contains(string(reading), "9007199254740993") || !strings.Contains(string(reading), "2.50") {
			t.Fatalf("%s: number literal re-rendered: %s", version, reading)
		}
	}
}

func TestReadForSpecRefusesWhatTheChainDoesNotReach(t *testing.T) {
	cases := map[string]tt.StandardCode{
		widgetBase + "0.4":                            tt.CodeUnsupportedVersion, // later, declares nothing
		widgetBase + "0.0":                            tt.CodeUnsupportedVersion, // earlier
		widgetBase + "1.0":                            tt.CodeUnsupportedVersion, // another MAJOR
		widgetBase + "0.3#response":                   tt.CodeUnsupportedType,    // another variant
		"https://trusttasks.org/spec/test/gadget/0.3": tt.CodeUnsupportedType,
	}
	for typeURI, code := range cases {
		received := json.RawMessage(`{"id":"x","type":"` + typeURI + `","payload":{"name":"w"}}`)
		reading, refusal, err := tt.ReadForSpecWith(received, widgetSpec, widgetChain)
		if err != nil || reading != nil || refusal == nil || refusal.Code != code {
			t.Fatalf("%s: got %s %+v %v, want %s", typeURI, reading, refusal, err, code)
		}
	}
	// With no lookup declaring anything — the registry today — an extending
	// version is refused as before.
	_, refusal, _ := tt.ReadForSpecWith(json.RawMessage(widget03("0.3")), widgetSpec, func(string) string { return "" })
	if refusal == nil || refusal.Code != tt.CodeUnsupportedVersion {
		t.Fatalf("undeclared chain accepted: %+v", refusal)
	}
	if _, _, err := tt.ReadForSpecWith(json.RawMessage(`[1]`), widgetSpec, widgetChain); err == nil {
		t.Fatal("a non-object document was not an error")
	}
}

func TestExtendsChainReaches(t *testing.T) {
	if !tt.ExtendsChainReaches(widgetBase+"0.3#response", widgetBase+"0.1#response", widgetChain) {
		t.Fatal("response variant chain not followed")
	}
	if tt.ExtendsChainReaches(widgetBase+"0.1", widgetBase+"0.3", widgetChain) {
		t.Fatal("chain walked backwards")
	}
	loop := func(string) string { return "0.9" } // 0.9 -> 0.9 -> ... must terminate
	if tt.ExtendsChainReaches(widgetBase+"0.9", widgetBase+"0.1", loop) {
		t.Fatal("looping lookup reached")
	}
	// The generated index ignores a fragment, and knows nothing of an unknown
	// specification. (Its agreement with the generated Extends constants is
	// pinned in ../smoke, which can import a generated package.)
	uri := "https://trusttasks.org/spec/auth/authenticate/0.3"
	if tt.RegistryExtends(uri+"#response") != tt.RegistryExtends(uri) {
		t.Fatal("registry index reads a fragment")
	}
	if got := tt.RegistryExtends(widgetBase + "0.3"); got != "" {
		t.Fatalf("registry index reports %q for an unknown specification", got)
	}
}

// signature is a stand-in for a Data Integrity proof value: a digest of the
// document minus its proof, so any change to the signed members breaks it — as
// pruning or a typed re-encoding does to a real one.
func signature(t *testing.T, document []byte) string {
	t.Helper()
	var members map[string]any
	if err := json.Unmarshal(document, &members); err != nil {
		t.Fatal(err)
	}
	delete(members, "proof")
	canonical, _ := json.Marshal(members) // encoding/json sorts map keys
	sum := sha256.Sum256(canonical)
	return "z" + base64.RawURLEncoding.EncodeToString(sum[:])
}

func signedWidget(t *testing.T, version string) json.RawMessage {
	t.Helper()
	unsigned := []byte(widget03(version))
	var members map[string]json.RawMessage
	_ = json.Unmarshal(unsigned, &members)
	members["proof"], _ = json.Marshal(tt.Proof{
		Type: "DataIntegrityProof", Cryptosuite: "test-2026", VerificationMethod: peer + "#key-1",
		Created: "2026-01-01T00:00:00Z", ProofPurpose: "assertionMethod", ProofValue: signature(t, unsigned),
	})
	signed, _ := json.Marshal(members)
	return signed
}

type digestVerifier struct{ t *testing.T }

func (v digestVerifier) Verify(_ context.Context, doc json.RawMessage) error {
	var d tt.Document[json.RawMessage]
	if err := json.Unmarshal(doc, &d); err != nil || d.Proof == nil {
		return errors.New("no proof")
	}
	if d.Proof.ProofValue != signature(v.t, doc) {
		return errors.New("signature does not match")
	}
	return nil
}

func consumeWidget(t *testing.T, doc *tt.Document[widgetPayload], received json.RawMessage) tt.ConsumeOutcome[widgetResponse] {
	t.Helper()
	outcome, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[widgetPayload, widgetResponse]{
		Transport:     tt.StaticTransport{Context: tt.TransportContext{Issuer: ptr(peer), Recipient: ptr(me)}},
		Spec:          widgetSpec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofVerify, Verifier: digestVerifier{t}},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.NotConsequentialChecks(),
		Doc:           doc,
		Received:      received,
		MyVID:         me,
		Now:           testNow,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(_ context.Context, d *tt.Document[widgetPayload], _ tt.ResolvedParties) (*tt.Document[widgetResponse], error) {
			return tt.RespondWith(d, "resp-1", widgetResponse{OK: true}, fixedClock), nil
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	return outcome
}

// The end-to-end forward case: a signed widget/0.3 document reaches a consumer
// implementing widget/0.1, the proof verifies over the document as received, and
// the response is typed as the version the producer used.
func TestSignedExtendedDocumentIsConsumedAndAnsweredAsReceived(t *testing.T) {
	received := signedWidget(t, "0.3")
	reading, refusal, err := tt.ReadForSpecWith(received, widgetSpec, widgetChain)
	if err != nil || refusal != nil {
		t.Fatalf("refused: %v %+v", err, refusal)
	}
	var doc tt.Document[widgetPayload]
	if err := json.Unmarshal(reading, &doc); err != nil {
		t.Fatal(err)
	}

	outcome := consumeWidget(t, &doc, received)
	if outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("outcome %s %+v", outcome.Kind, outcome.Error)
	}
	if want := widgetBase + "0.3#response"; outcome.Response.Type != want {
		t.Fatalf("response typed %s, want %s", outcome.Response.Type, want)
	}

	// The control: the same document with the proof checked over a re-encoding
	// (no Received) fails, because the reading is not the signed document. This
	// is what Received exists to prevent.
	if outcome := consumeWidget(t, &doc, nil); outcome.Kind != tt.OutcomeRejected ||
		outcome.Error.Payload.Code != string(tt.CodeProofInvalid) {
		t.Fatalf("re-encoded proof check unexpectedly passed: %s", outcome.Kind)
	}
}

// Received vouches only for a Doc that is a reading of it.
func TestReceivedMustBeTheDocumentBeingConsumed(t *testing.T) {
	received := signedWidget(t, "0.3")
	reading, _, _ := tt.ReadForSpecWith(received, widgetSpec, widgetChain)

	var tampered tt.Document[widgetPayload]
	_ = json.Unmarshal(reading, &tampered)
	tampered.Payload.Name = "not what was signed"
	if outcome := consumeWidget(t, &tampered, received); outcome.Kind != tt.OutcomeRejected ||
		outcome.Error.Payload.Code != string(tt.CodeProofInvalid) {
		t.Fatalf("tampered payload accepted: %s", outcome.Kind)
	}

	// A change float64 cannot see (2^53 + 1 -> 2^53) is still a change.
	var nudged tt.Document[widgetPayload]
	_ = json.Unmarshal(reading, &nudged)
	nudged.Payload.Count = ptr(int64(9007199254740992))
	if outcome := consumeWidget(t, &nudged, received); outcome.Kind != tt.OutcomeRejected ||
		outcome.Error.Payload.Code != string(tt.CodeProofInvalid) {
		t.Fatalf("integer nudged past float precision accepted: %s", outcome.Kind)
	}

	var readdressed tt.Document[widgetPayload]
	_ = json.Unmarshal(reading, &readdressed)
	readdressed.ID = "req-2"
	if outcome := consumeWidget(t, &readdressed, received); outcome.Kind != tt.OutcomeRejected ||
		outcome.Error.Payload.Code != string(tt.CodeProofInvalid) {
		t.Fatalf("changed envelope accepted: %s", outcome.Kind)
	}

	// An exact-version document decoded directly is a reading of itself.
	exact := signedWidget(t, "0.1")
	var direct tt.Document[widgetPayload]
	_ = json.Unmarshal(exact, &direct)
	if outcome := consumeWidget(t, &direct, exact); outcome.Kind != tt.OutcomeHandled ||
		outcome.Response.Type != widgetBase+"0.1#response" {
		t.Fatalf("exact version: %s %+v", outcome.Kind, outcome.Error)
	}
}
