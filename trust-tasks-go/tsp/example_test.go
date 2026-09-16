package tsp_test

import (
	"context"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"time"

	atsp "github.com/affinidi/affinidi-tsp-go"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp"
)

// Example shows alice sealing a request to bob over TSP and bob absorbing a
// re-forward of the same document instead of executing it twice.
func Example() {
	alice, _ := atsp.GenerateIdentity("did:example:alice", atsp.SigKeyEd25519, atsp.EncKeyX25519, rand.Reader)
	bob, _ := atsp.GenerateIdentity("did:example:bob", atsp.SigKeyEd25519, atsp.EncKeyX25519, rand.Reader)

	// One consumer for the process: its replay guard is the record. The clock is
	// pinned here only so the example's fixed issuedAt stays in the freshness
	// window; a real consumer leaves Now at its default.
	consumer := tsp.NewConsumer(bob)
	consumer.Now = func() time.Time { return fixedNow }

	// alice's request need not name an issuer: TSP authenticates her.
	doc, _ := json.Marshal(map[string]any{
		"id":       "urn:uuid:11111111-1111-4111-8111-111111111111",
		"type":     "https://trusttasks.org/spec/example/echo/0.1",
		"issuedAt": fixedIssuedAt,
		"payload":  map[string]any{"text": "hello"},
	})

	handle := func(_ context.Context, d *tt.Document[echo], _ tt.ResolvedParties) (*tt.Document[echo], error) {
		return tt.RespondWith[echo, echo](d, "urn:uuid:22222222-2222-4222-8222-222222222222", echo{Text: "HELLO"}, nil), nil
	}

	for _, label := range []string{"delivery", "re-forward"} {
		// A re-forward is the same document resealed with fresh TSP material.
		wire, _ := tsp.PackTrustTask(json.RawMessage(doc), alice, bob.Public())
		got, _ := tsp.Receive(context.Background(), consumer, wire, alice.Public(),
			tt.SpecPolicy{TypeURI: "https://trusttasks.org/spec/example/echo/0.1"}, decodeEcho, handle)
		fmt.Printf("%s: %s\n", label, got.Outcome.Kind)
	}
	// Output:
	// delivery: handled
	// re-forward: duplicate
}
