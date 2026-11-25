# HCP over Trust Tasks: a demonstration (TypeScript)

This example runs the Human Context Protocol (HCP 2.0-draft-3) flow as Trust Tasks, between three
parties in one process:

- a toy, in-memory **HCP Provider**,
- a **Context Client** (a booking agent that wants the person's travel preferences), and
- a **Management Client** (the person's own app).

Each party has its own did:key and TSP keys. Every document is built with the generated types,
signed with `@openvtc/trust-tasks-proof`, and sealed as a TSP message with
`@openvtc/trust-tasks-tsp`. The receiving end then runs the SPEC §7.2 pipeline on it: proof
verified over the document as received, payload validated against its schema. Only then does
anything act on it. Nothing touches the network.

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

From the repository root, build the four TypeScript packages once:

```sh
for d in trust-tasks-ts trust-tasks-ts-gen trust-tasks-ts-proof trust-tasks-ts-tsp; do (cd $d && npm ci && npm run build); done
```

Then:

```sh
cd examples/hcp-demo-ts
npm ci
npm run all        # generate from trust-tasks.lock, build, run the demo, run the tests
```

`npm start` runs the demo again. `npm test` runs the tests: the golden transcript, a query after
revoke, a tampered document and a schema-invalid document. If you change the scenario on purpose,
run `npm run build && npm run update-golden` and update [SCENARIO.md](SCENARIO.md), which the Rust,
Go and Dart ports follow.

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
hcp-demo-ts: discover, grant request (held, then granted), query, revoke, notice and post-revoke refusal all OK
```

The full run is in [transcript.golden.txt](transcript.golden.txt).

## What is simplified

- **The provider is a toy.** It has three entries in memory, a grant table, no storage, no
  minimization beyond category filtering, no audit log and no directives. It is not an HCP
  implementation.
- **There is no consent UI.** The person's decision is one function call, and it always approves
  what was asked. The demo shows the shape of the hold: a next step first, then the grant on
  resubmission.
- **Keys and identities are fixed.** Every party knows the others' public keys up front, so there
  is no DID resolution and no discovery of the provider's DID document.
- **Clock and identifiers are fixed** (see [SCENARIO.md](SCENARIO.md)), so every run, in every
  language, prints the same transcript.
- **Tokens are opaque strings** the toy provider mints and looks up. There is no OAuth
  Authorization Service.
- **The revocation notice is sent once.** HCP's retry schedule (HCP 9.9) is not shown.
- **The specs are drafts**, and several points are pending HCP confirmation. Each spec lists them
  under "Pending HCP confirmation".

## Layout

As in [`consumer-app-ts`](../consumer-app-ts) and [`getting-started-ts`](../getting-started-ts):
`trust-tasks.toml` pins the tasks, `trust-tasks.lock` and `trust-tasks/` hold the vendored bundles,
and `npm run generate` writes `src/trust-tasks/` (git-ignored). The framework tasks
(`trust-task-discovery`, `trust-task-next-step`) ship with `@openvtc/trust-tasks` and are imported
from it. The files:

- `src/demo.ts`: the parties, the toy provider and the five steps.
- `src/validator.ts`: payload validation with ajv. The getting-started example's structural
  stand-in ignores `pattern`, which the HCP schemas rely on.
- `src/main.ts`: runs the demo and prints the transcript.
- `test/demo.test.ts`: the tests.
