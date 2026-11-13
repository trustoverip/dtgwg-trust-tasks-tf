package trusttasks

import (
	"encoding/json"
	"os"
	"reflect"
	"testing"
)

// The shared forward-routing vectors (fixtures/consume/extends-routing.json at
// the repository root), which the Rust, TypeScript and Dart runtimes also run:
// a consumer implementing one version reads a later one whose declared extends
// chain reaches it as its own (pruned, answered as the received version), and
// refuses an unreached version unsupportedVersion and another specification or
// variant unsupportedType (SPEC §5.2 "Declared extension", §7.2 item 3).

type extendsVectors struct {
	Implements struct {
		TypeURI       string `json:"typeUri"`
		PayloadSchema string `json:"payloadSchema"`
	} `json:"implements"`
	Extends map[string]string `json:"extends"`
	Cases   []struct {
		Name       string          `json:"name"`
		Document   string          `json:"document"`
		Verdict    string          `json:"verdict"`
		Extended   bool            `json:"extended"`
		Payload    json.RawMessage `json:"payload"`
		AnswerType string          `json:"answerType"`
	} `json:"cases"`
}

func TestSharedExtendsRoutingVectors(t *testing.T) {
	raw, err := os.ReadFile("../../fixtures/consume/extends-routing.json")
	if err != nil {
		t.Fatal(err)
	}
	var vectors extendsVectors
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatal(err)
	}
	if len(vectors.Cases) == 0 {
		t.Fatal("no vectors")
	}
	spec := SpecPolicy{
		TypeURI:       vectors.Implements.TypeURI,
		PayloadSchema: vectors.Implements.PayloadSchema,
	}
	lookup := func(typeURI string) string { return vectors.Extends[typeURI] }
	implemented, err := routingKey(spec.TypeURI)
	if err != nil {
		t.Fatal(err)
	}

	for _, c := range vectors.Cases {
		t.Run(c.Name, func(t *testing.T) {
			reading, reason, err := ReadForSpecWith(json.RawMessage(c.Document), spec, lookup)
			if err != nil {
				t.Fatalf("ReadForSpecWith: %v", err)
			}
			if c.Verdict != "accept" {
				if reason == nil {
					t.Fatalf("accepted, want %s", c.Verdict)
				}
				if string(reason.Code) != c.Verdict {
					t.Fatalf("refused %s, want %s", reason.Code, c.Verdict)
				}
				return
			}
			if reason != nil {
				t.Fatalf("refused %s (%s), want accept", reason.Code, reason.Message)
			}

			var doc Document[json.RawMessage]
			if err := json.Unmarshal(reading, &doc); err != nil {
				t.Fatalf("decode the reading: %v", err)
			}
			var got, want any
			if err := json.Unmarshal(doc.Payload, &got); err != nil {
				t.Fatal(err)
			}
			if err := json.Unmarshal(c.Payload, &want); err != nil {
				t.Fatal(err)
			}
			if !reflect.DeepEqual(got, want) {
				t.Errorf("payload = %s, want %s", doc.Payload, c.Payload)
			}

			received, err := routingKey(doc.Type)
			if err != nil {
				t.Fatal(err)
			}
			extended := received.Major() != implemented.Major() || received.Minor() != implemented.Minor()
			if extended != c.Extended {
				t.Errorf("extended = %v, want %v", extended, c.Extended)
			}

			response := RespondWith[json.RawMessage, struct{}](&doc, "urn:uuid:response", struct{}{}, nil)
			if response.Type != c.AnswerType {
				t.Errorf("answer typed %s, want %s", response.Type, c.AnswerType)
			}
		})
	}
}
