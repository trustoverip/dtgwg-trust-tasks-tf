// The HCP-over-Trust-Tasks demo, in Go: run the eight steps and print every
// document. The Go port of examples/hcp-demo-ts, printing the same transcript
// byte for byte.
//
//	go generate ./... && go run . && go test ./...
//
// trust-tasks.lock pins the four hcp/* tasks and the two framework tasks the
// demo uses, and their registry bundles are vendored under trust-tasks/. `go
// generate` runs trust-tasks-gen, which checks each bundle against its locked
// digest and writes one package per hcp/* version into internal/tasks
// (committed, as go generate output usually is; CI regenerates it and fails on
// drift). The framework tasks come from trust-tasks-go/specs.
//
// A DEMONSTRATION, not a reference provider. See README.md and
// ../hcp-demo-ts/SCENARIO.md.
package main

//go:generate go run github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/cmd/trust-tasks-gen --lock trust-tasks.lock --out internal/tasks

import (
	"fmt"
	"os"
)

func main() {
	result := RunDemo(nil)
	fmt.Print(result.Transcript)
	if code := result.PostRevokeError.Payload.Code; code != "hcp:invalidToken" {
		fmt.Fprintf(os.Stderr, "hcp-demo-go: expected hcp:invalidToken after revoke, got %s\n", code)
		os.Exit(1)
	}
	fmt.Println("\nhcp-demo-go: discover, grant request (held, then granted), query, revoke, post-revoke refusal, notice, decline and expired hold all OK")
}
