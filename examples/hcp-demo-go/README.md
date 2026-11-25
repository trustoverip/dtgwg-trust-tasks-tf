# HCP over Trust Tasks: a demonstration (Go)

The Go port of [`hcp-demo-ts`](../hcp-demo-ts). It runs the Human Context Protocol
(HCP 2.0-draft-3) flow as Trust Tasks, between three parties in one process:

- a toy, in-memory **HCP Provider**,
- a **Context Client** (a booking agent that wants the person's travel preferences), and
- a **Management Client** (the person's own app).

Each party has its own did:key and TSP keys. Every document is built with the generated types,
signed with [`trust-tasks-go/proof`](../../trust-tasks-go/proof), and sealed as a TSP message with
[`trust-tasks-go/tsp`](../../trust-tasks-go/tsp). The receiving end then runs the SPEC §7.2
pipeline on it: proof verified over the document as received, payload validated against its
schema. Only then does anything act on it. Nothing touches the network.

It follows the language-neutral script in [`../hcp-demo-ts/SCENARIO.md`](../hcp-demo-ts/SCENARIO.md)
and prints the same transcript, byte for byte, as every other port.

**This is a demonstration, not a reference provider.** The keys are DEMO ONLY: 32 repeated bytes
per key. Never reuse them.

## The flow

1. **Discover.** The client asks with `trust-task-discovery` 0.4 whether the provider serves
   `hcp/*`.
2. **Grant request.** The client sends [`hcp/grants/request`](../../specs/hcp/grants/request/0.1/spec.md)
   0.1. The provider holds it with a `trust-task-next-step` while the person decides. The client
   resubmits on the same thread and receives the grant and its access token.
3. **Query.** [`hcp/entries/query`](../../specs/hcp/entries/query/0.1/spec.md) 0.1 runs under the
   grant's token and returns the person's travel entries.
4. **Revoke.** The person's Management Client sends
   [`hcp/grants/revoke`](../../specs/hcp/grants/revoke/0.1/spec.md) 0.1.
5. **Revocation notice.** The provider sends
   [`hcp/grants/revoked`](../../specs/hcp/grants/revoked/0.1/spec.md) 0.1 to the client, which
   acknowledges it.

After step 5, the same query fails with `hcp:invalidToken`, because the grant is checked on every
request (HCP 6.2.2).

## Run it

Go 1.27 or later (the floor of `trust-tasks-go/tsp`). From this directory:

```sh
go generate ./... && go run . && go test ./...
```

`go generate` regenerates `internal/tasks` from `trust-tasks.lock` (the committed tree; CI fails if
it drifts). `go run .` runs the demo. `go test ./...` runs the tests: the golden transcript, a
query after revoke, a tampered document and a schema-invalid document. The golden transcript is
shared with the other ports and owned by the TypeScript demo,
[`../hcp-demo-ts/transcript.golden.txt`](../hcp-demo-ts/transcript.golden.txt); this port compares
its output with it and never writes it.

Expected output (an excerpt; access tokens are redacted to their first 6 characters):

```
=== 2. Grant request (hcp/grants/request 0.1), held with trust-task-next-step ===
--> Context Client -> Provider (TSP) https://trusttasks.org/spec/hcp/grants/request/0.1#request
…
--> Provider -> Context Client (TSP) https://trusttasks.org/spec/trust-task-next-step/0.1
…
[person] approves the grant request on thread urn:uuid:00000000-0000-4000-8000-000000000003 in the provider's app
--> Context Client -> Provider (TSP) https://trusttasks.org/spec/hcp/grants/request/0.1#request
…
--> Provider -> Context Client (TSP) https://trusttasks.org/spec/hcp/grants/request/0.1#response
{
  …
  "payload": {
    "accessToken": "hcp_at…",
…
=== After revoke: the same query fails with hcp:invalidToken ===
…
    "code": "hcp:invalidToken",
…
hcp-demo-go: discover, grant request (held, then granted), query, revoke, notice and post-revoke refusal all OK
```

## What is simplified

- **The provider is a toy.** It has three entries in memory, a grant table, no storage, no
  minimization beyond category filtering, no audit log and no directives. It is not an HCP
  implementation.
- **There is no consent UI.** The person's decision is one function call, and it always approves
  what was asked. The demo shows the shape of the hold: a next step first, then the grant on
  resubmission.
- **Keys and identities are fixed.** Every party knows the others' public keys up front, so there
  is no DID resolution and no discovery of the provider's DID document.
- **Clock and identifiers are fixed** (see [SCENARIO.md](../hcp-demo-ts/SCENARIO.md)), so every
  run, in every language, prints the same transcript.
- **Tokens are opaque strings** the toy provider mints and looks up. There is no OAuth
  Authorization Service.
- **The revocation notice is sent once.** HCP's retry schedule (HCP 9.9) is not shown.
- **The specs are drafts**, and several points are pending HCP confirmation. Each spec lists them
  under "Pending HCP confirmation".

## Layout

As in [`consumer-app-go`](../consumer-app-go) and [`getting-started-go`](../getting-started-go):
`trust-tasks.toml` pins the tasks, `trust-tasks.lock` and `trust-tasks/` hold the vendored bundles,
and `go generate` writes `internal/tasks/` (committed). The framework tasks
(`trust-task-discovery`, `trust-task-next-step`, `trust-task-error`) are imported from
`trust-tasks-go/specs`, as `trust-tasks-gen` directs. `go.mod` replaces the Trust Tasks modules
with this checkout, so the demo tests them before they are released. The files:

- `demo.go`: the parties, the toy provider and the five steps.
- `validator.go`: payload validation with a full JSON Schema 2020-12 engine,
  [`santhosh-tekuri/jsonschema`](https://github.com/santhosh-tekuri/jsonschema) v6. The
  getting-started example's structural stand-in ignores `pattern`, which the HCP schemas rely on.
- `main.go`: runs the demo and prints the transcript.
- `demo_test.go`: the tests.
