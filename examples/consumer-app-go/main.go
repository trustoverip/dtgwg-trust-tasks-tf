// An application that speaks three Trust Tasks and compiles only those — the
// Go counterpart of examples/consumer-app.
//
// trust-tasks.lock pins acl/grant/0.2, keys/list/0.2 and vault/get/0.3, and
// their registry bundles are vendored under trust-tasks/. `go generate` runs
// trust-tasks-gen, which checks each bundle against its locked digest and
// writes one package per version into internal/tasks (committed, as go
// generate output usually is; CI regenerates it and fails on drift):
//
//	go generate ./... && go run .
//
// It plays both parties in one process: a producer builds requests and
// serializes them; a consumer parses the bytes, applies each version's policy
// through the trusttasks runtime, and answers.
package main

//go:generate go run github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/cmd/trust-tasks-gen --lock trust-tasks.lock --out internal/tasks

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"

	"example.com/trust-tasks-consumer-app-go/internal/tasks"
	grant "example.com/trust-tasks-consumer-app-go/internal/tasks/acl/grant/v0_2"
	keyslist "example.com/trust-tasks-consumer-app-go/internal/tasks/keys/list/v0_2"
	vaultget "example.com/trust-tasks-consumer-app-go/internal/tasks/vault/get/v0_3"
)

const (
	consumerVID = "did:web:vta.example"
	producerVID = "did:web:root-admin.example"
)

var now = time.Date(2026, 10, 26, 12, 0, 0, 0, time.UTC)

func clock() string { return now.Format(time.RFC3339) }

func ptr[T any](v T) *T { return &v }

func main() {
	if err := run(); err != nil {
		fmt.Fprintln(os.Stderr, "consumer-app-go:", err)
		os.Exit(1)
	}
	fmt.Println("ok")
}

func run() error {
	// The policy each bundle declares is on the generated packages.
	if !grant.Spec.IsProofRequired || keyslist.Spec.IsProofRequired {
		return fmt.Errorf("generated policy does not match the bundles")
	}

	// 1. keys/list — a request, serialized, read back, consumed and answered.
	request, err := json.Marshal(tt.Document[keyslist.Payload]{
		ID:        "req-1",
		Type:      keyslist.TypeURI,
		Issuer:    ptr(producerVID),
		Recipient: ptr(consumerVID),
		IssuedAt:  ptr(clock()),
		Payload:   keyslist.Payload{Status: ptr(keyslist.KeyStatusActive), Limit: ptr(int64(10))},
	})
	if err != nil {
		return err
	}
	outcome, err := consumeKeysList(request)
	if err != nil {
		return err
	}
	if outcome.Kind != tt.OutcomeHandled {
		return fmt.Errorf("keys/list: expected handled, got %s", outcome.Kind)
	}
	reply, err := json.Marshal(outcome.Response)
	if err != nil {
		return err
	}
	var answer tt.Document[keyslist.Response]
	if err := json.Unmarshal(reply, &answer); err != nil {
		return err
	}
	if answer.Type != keyslist.ResponseTypeURI || len(answer.Payload.Keys) != 1 || answer.Payload.Keys[0].KeyType != keyslist.KeyTypeEd25519 {
		return fmt.Errorf("keys/list: unexpected answer %s", reply)
	}
	fmt.Printf("keys/list 0.2: handled → %s\n", answer.Type)

	// 2. A document of a version this application did not lock is refused
	// before it is parsed: keys/list 0.3 declares no extension of 0.2.
	newer := strings.Replace(string(request), keyslist.TypeURI, "https://trusttasks.org/spec/keys/list/0.3", 1)
	if _, refusal, err := tt.ReadForSpecWith(json.RawMessage(newer), keyslist.Spec, tasks.ExtendsOf); err != nil || refusal == nil {
		return fmt.Errorf("keys/list 0.3 was not refused (err %v)", err)
	}
	fmt.Println("keys/list 0.3: refused — not locked, and it does not extend 0.2")

	// 3. acl/grant requires a proof (its front matter, carried by the bundle);
	// a proofless request is refused by the runtime before any handler runs.
	refused, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[grant.Payload, grant.Response]{
		Transport:     tt.StaticTransport{Context: tt.TransportContext{Issuer: ptr(producerVID), Recipient: ptr(consumerVID)}},
		Spec:          grant.Spec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.ConsequentialChecks(tt.NewInMemoryReplayGuard(0)),
		Doc: &tt.Document[grant.Payload]{
			ID: "req-2", Type: grant.TypeURI, Issuer: ptr(producerVID), Recipient: ptr(consumerVID), IssuedAt: ptr(clock()),
			Payload: grant.Payload{Entry: grant.ACLEntry{
				Subject:      "did:web:alice.example",
				Role:         "admin",
				Act:          json.RawMessage(`{"scope":"all"}`),
				Capabilities: json.RawMessage(`{"scope":"ceiling"}`),
			}},
		},
		MyVID:      consumerVID,
		Now:        now,
		NewErrorID: func() string { return "err-1" },
		Clock:      clock,
		Handler: func(context.Context, *tt.Document[grant.Payload], tt.ResolvedParties) (*tt.Document[grant.Response], error) {
			return nil, fmt.Errorf("the handler ran for a proofless acl/grant")
		},
	})
	if err != nil {
		return err
	}
	if refused.Kind != tt.OutcomeRejected {
		return fmt.Errorf("acl/grant: expected rejected, got %s", refused.Kind)
	}
	fmt.Printf("acl/grant 0.2: rejected — %s\n", refused.Error.Payload.Code)

	// 4. vault/get — optional arrays are pointers, so present-and-empty stays
	// distinct from absent on the wire.
	empty, _ := json.Marshal(vaultget.Response{Entry: vaultget.VaultEntry{ID: "e1"}, RedactedFields: &[]string{}})
	absent, _ := json.Marshal(vaultget.Response{Entry: vaultget.VaultEntry{ID: "e1"}})
	if !strings.Contains(string(empty), `"redactedFields":[]`) || strings.Contains(string(absent), "redactedFields") {
		return fmt.Errorf("vault/get: present-but-empty collapsed into absent")
	}
	fmt.Println("vault/get 0.3: redactedFields [] and absent stay distinct")
	return nil
}

// consumeKeysList is the consumer side of keys/list: route, parse, run the
// SPEC §7.2 pipeline, answer.
func consumeKeysList(received []byte) (tt.ConsumeOutcome[keyslist.Response], error) {
	var zero tt.ConsumeOutcome[keyslist.Response]
	reading, refusal, err := tt.ReadForSpecWith(received, keyslist.Spec, tasks.ExtendsOf)
	if err != nil {
		return zero, err
	}
	if refusal != nil {
		return zero, fmt.Errorf("refused: %s", refusal.Code)
	}
	var doc tt.Document[keyslist.Payload]
	if err := json.Unmarshal(reading, &doc); err != nil {
		return zero, err
	}
	return tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[keyslist.Payload, keyslist.Response]{
		Transport:     tt.StaticTransport{Context: tt.TransportContext{Issuer: ptr(producerVID), Recipient: ptr(consumerVID)}},
		Spec:          keyslist.Spec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofRejectIfPresent},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.NotConsequentialChecks(),
		Doc:           &doc,
		Received:      received,
		MyVID:         consumerVID,
		Now:           now,
		NewErrorID:    func() string { return "err-2" },
		Clock:         clock,
		Handler: func(_ context.Context, accepted *tt.Document[keyslist.Payload], _ tt.ResolvedParties) (*tt.Document[keyslist.Response], error) {
			return tt.RespondWith[keyslist.Payload, keyslist.Response](accepted, "resp-1", keyslist.Response{
				Keys: []keyslist.KeyRecord{{
					KeyID:     "did:web:vta.example#key-1",
					KeyType:   keyslist.KeyTypeEd25519,
					Status:    keyslist.KeyStatusActive,
					PublicKey: "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
					CreatedAt: clock(),
				}},
			}, clock), nil
		},
	})
}
