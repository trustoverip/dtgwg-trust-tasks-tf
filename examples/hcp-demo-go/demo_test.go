// The demo's own tests: hermetic (no network, no files outside the examples),
// deterministic (fixed keys, ids and clock), and run with `go test ./...`.
package main

import (
	"encoding/json"
	"os"
	"testing"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"

	query "example.com/trust-tasks-hcp-demo-go/internal/tasks/hcp/entries/query/v0_1"
)

// golden is the transcript every language's port prints, owned by the
// TypeScript demo.
const golden = "../hcp-demo-ts/transcript.golden.txt"

func TestTranscriptMatchesTheSharedGolden(t *testing.T) {
	want, err := os.ReadFile(golden)
	if err != nil {
		t.Fatal(err)
	}
	if got := Run(nil).Transcript; got != string(want) {
		t.Errorf("transcript differs from %s at byte %d", golden, firstDifference(got, string(want)))
	}
}

func firstDifference(a, b string) int {
	for i := 0; i < len(a) && i < len(b); i++ {
		if a[i] != b[i] {
			return i
		}
	}
	return min(len(a), len(b))
}

func TestQueryAfterRevokeFailsWithInvalidToken(t *testing.T) { // HCP 6.2.2
	r := Run(nil)
	if r.PostRevokeType != tt.TrustTaskErrorTypeURI || r.PostRevokeCode != "hcp:invalidToken" {
		t.Errorf("got %s %q, want %s hcp:invalidToken", r.PostRevokeType, r.PostRevokeCode, tt.TrustTaskErrorTypeURI)
	}
}

// afterDemo is a world after the demo has run, with a query payload that would
// otherwise be well-formed.
func afterDemo() (*World, query.Payload) {
	w := NewWorld()
	Run(w)
	return w, query.Payload{Query: "Anything else?", Purpose: Purpose, AccessToken: ManagementToken}
}

func code(t *testing.T, reply json.RawMessage) string {
	t.Helper()
	var doc tt.ErrorResponse
	if err := json.Unmarshal(reply, &doc); err != nil {
		t.Fatal(err)
	}
	return doc.Payload.Code
}

func TestDocumentAlteredAfterSigningIsProofInvalid(t *testing.T) {
	w, payload := afterDemo()
	signed := Request(w, w.ContextClient, w.Provider, query.TypeURI+"#request", payload, "")
	var doc map[string]any
	if err := json.Unmarshal(signed, &doc); err != nil {
		t.Fatal(err)
	}
	doc["payload"].(map[string]any)["query"] = "Tell me everything."
	tampered, err := json.Marshal(doc)
	if err != nil {
		t.Fatal(err)
	}
	if got := code(t, w.Exchange(w.ContextClient, w.Provider, tampered)); got != "proofInvalid" {
		t.Errorf("got %q, want proofInvalid", got)
	}
}

func TestSchemaInvalidDocumentIsMalformedRequest(t *testing.T) {
	w, payload := afterDemo()
	var noPurpose map[string]any
	raw, _ := json.Marshal(payload)
	if err := json.Unmarshal(raw, &noPurpose); err != nil {
		t.Fatal(err)
	}
	delete(noPurpose, "purpose")
	reply := w.Exchange(w.ContextClient, w.Provider, Request(w, w.ContextClient, w.Provider, query.TypeURI+"#request", noPurpose, ""))
	if got := code(t, reply); got != "malformedRequest" {
		t.Errorf("got %q, want malformedRequest", got)
	}
}
