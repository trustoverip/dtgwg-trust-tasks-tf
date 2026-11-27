package trusttasks_test

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"reflect"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// The shared vectors in fixtures/consume/received-payload.json, which the Rust,
// TypeScript and Dart runtimes also run: with Received set, ConsumeInbound
// validates the payload as received — where the typed decode reads a forbidden
// null as absent — and a later version read forward along extends pruned, as
// ReadForSpecWith reads it (SPEC §7.2 item 2).

type receivedPayloadVectors struct {
	Implements struct {
		TypeURI       string `json:"typeUri"`
		PayloadSchema string `json:"payloadSchema"`
	} `json:"implements"`
	Extends  map[string]string `json:"extends"`
	Consumer string            `json:"consumer"`
	Now      string            `json:"now"`
	Cases    []struct {
		Name      string          `json:"name"`
		Document  string          `json:"document"`
		Verdict   string          `json:"verdict"`
		Validated json.RawMessage `json:"validated"`
	} `json:"cases"`
}

// receivedGrant is the implemented version as a generated type reads it: an
// optional role that is nil whether absent or null — the shape of acl/update
// 0.3's Purposes (*PurposeScope with omitempty).
type receivedGrant struct {
	Subject string  `json:"subject"`
	Role    *string `json:"role,omitempty"`
}

// recordingValidator records the payload it is handed and checks the vector
// schema by hand: an object, subject present, every member named in
// properties, every member a string.
type recordingValidator struct{ seen json.RawMessage }

func (v *recordingValidator) Validate(_ string, payload json.RawMessage) error {
	v.seen = append(json.RawMessage(nil), payload...)
	var members map[string]any
	if err := json.Unmarshal(payload, &members); err != nil || members == nil {
		return errors.New("not an object")
	}
	if _, ok := members["subject"]; !ok {
		return errors.New("missing subject")
	}
	for name, value := range members {
		if name != "subject" && name != "role" {
			return fmt.Errorf("unknown member %s", name)
		}
		if _, ok := value.(string); !ok {
			return fmt.Errorf("%s is not a string", name)
		}
	}
	return nil
}

func loadReceivedPayloadVectors(t *testing.T) receivedPayloadVectors {
	t.Helper()
	raw, err := os.ReadFile("../../fixtures/consume/received-payload.json")
	if err != nil {
		t.Fatal(err)
	}
	var vectors receivedPayloadVectors
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatal(err)
	}
	if len(vectors.Cases) == 0 {
		t.Fatal("no vectors")
	}
	return vectors
}

// consumeGrant reads received as the implemented version and consumes it, with
// Received set or not, returning the verdict and the payload validated.
func consumeGrant(t *testing.T, vectors receivedPayloadVectors, received json.RawMessage, passReceived bool) (string, json.RawMessage) {
	t.Helper()
	spec := tt.SpecPolicy{TypeURI: vectors.Implements.TypeURI, PayloadSchema: vectors.Implements.PayloadSchema}
	lookup := func(typeURI string) string { return vectors.Extends[typeURI] }
	now, err := time.Parse(time.RFC3339, vectors.Now)
	if err != nil {
		t.Fatal(err)
	}
	reading, reason, err := tt.ReadForSpecWith(received, spec, lookup)
	if err != nil || reason != nil {
		t.Fatalf("ReadForSpecWith: %v %v", err, reason)
	}
	var doc tt.Document[receivedGrant]
	if err := json.Unmarshal(reading, &doc); err != nil {
		t.Fatalf("decode the reading: %v", err)
	}
	validator := &recordingValidator{}
	opts := tt.ConsumeOptions[receivedGrant, struct{}]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          spec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadValidate, Validator: validator},
		Checks:        tt.NotConsequentialChecks(),
		Doc:           &doc,
		Extends:       lookup,
		MyVID:         vectors.Consumer,
		Now:           now,
		NewErrorID:    func() string { return "urn:uuid:error" },
		Clock:         fixedClock,
		Handler: func(_ context.Context, d *tt.Document[receivedGrant], _ tt.ResolvedParties) (*tt.Document[struct{}], error) {
			return tt.RespondWith[receivedGrant, struct{}](d, "urn:uuid:answer", struct{}{}, fixedClock), nil
		},
	}
	if passReceived {
		opts.Received = received
	}
	outcome, err := tt.ConsumeInbound(context.Background(), opts)
	if err != nil {
		t.Fatal(err)
	}
	switch outcome.Kind {
	case tt.OutcomeHandled:
		return "accept", validator.seen
	case tt.OutcomeRejected:
		return outcome.Error.Payload.Code, validator.seen
	default:
		return string(outcome.Kind), validator.seen
	}
}

func sameJSON(t *testing.T, a, b json.RawMessage) bool {
	t.Helper()
	var x, y any
	if err := json.Unmarshal(a, &x); err != nil {
		return false
	}
	if err := json.Unmarshal(b, &y); err != nil {
		return false
	}
	return reflect.DeepEqual(x, y)
}

func TestSharedReceivedPayloadVectors(t *testing.T) {
	vectors := loadReceivedPayloadVectors(t)
	for _, c := range vectors.Cases {
		t.Run(c.Name, func(t *testing.T) {
			verdict, validated := consumeGrant(t, vectors, json.RawMessage(c.Document), true)
			if verdict != c.Verdict {
				t.Fatalf("verdict %s, want %s", verdict, c.Verdict)
			}
			if !sameJSON(t, validated, c.Validated) {
				t.Fatalf("validated %s, want %s", validated, c.Validated)
			}
		})
	}
}

// Without Received there is only Doc's payload re-encoded to validate, from
// which the forbidden null is gone: the document is accepted.
func TestWithoutReceivedTheForbiddenNullIsNotSeen(t *testing.T) {
	vectors := loadReceivedPayloadVectors(t)
	document := `{"id":"urn:uuid:7b0c2f1e-0000-4000-8000-0000000000a1","type":"https://example.com/spec/vector/grant/0.1","issuer":"did:example:alice","recipient":"did:example:bob","issuedAt":"2026-08-01T12:00:00Z","payload":{"subject":"did:example:carol","role":null}}`
	if verdict, _ := consumeGrant(t, vectors, json.RawMessage(document), true); verdict != string(tt.CodeMalformedRequest) {
		t.Fatalf("with Received: %s, want malformedRequest", verdict)
	}
	verdict, validated := consumeGrant(t, vectors, json.RawMessage(document), false)
	if verdict != "accept" {
		t.Fatalf("without Received: %s, want accept", verdict)
	}
	if !sameJSON(t, validated, json.RawMessage(`{"subject":"did:example:carol"}`)) {
		t.Fatalf("without Received validated %s", validated)
	}
}
