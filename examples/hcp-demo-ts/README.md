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
   resubmits on the same thread and receives an access token and the grant, as HCP's grant object.
3. **Query.** [`hcp/entries/query`](../../specs/hcp/entries/query/0.1/spec.md) 0.1 runs under the
   grant's token and returns the person's travel entries.
4. **Revoke.** The person's Management Client sends
   [`hcp/grants/revoke`](../../specs/hcp/grants/revoke/0.1/spec.md) 0.1.
5. **Query after revoke.** Before any notice arrives, the same query fails with
   `hcp:invalidToken`: the grant is checked on every request, so revocation takes effect at once
   (HCP 6.2.2). The client drops the token and the copies it holds.
6. **Revocation notice.** The provider sends
   [`hcp/grants/revoked`](../../specs/hcp/grants/revoked/0.1/spec.md) 0.1 to the grant's client,
   which records the `noticeId` and the grant, stops using the grant, and acknowledges. It never
   sends the old token again.

The provider also applies the rules the tests exercise: a token presented from another VID gets
`permissionDenied`; only its one registered client may ask for a grant; a resubmission must carry
the held payload (compared in JCS form); and a resubmission after approval gets the same grant
within the hour, never a second one, and `permissionDenied` after it. Each side checks a reply
before it uses it: from the party asked, on the same thread, signed (unless the library built it),
and valid against its schema.

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

`npm start` runs the demo again. `npm test` runs the tests listed in
[SCENARIO.md](SCENARIO.md#tests-every-port-has): the golden transcript, the query after revoke, a
tampered document, a schema-invalid document, the client's handling of the notice, and the
provider's token and grant-request rules. If you change the scenario on purpose,
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
=== 5. Query after revoke (hcp/entries/query 0.1) fails with hcp:invalidToken ===
…
    "code": "hcp:invalidToken",
…
hcp-demo-ts: discover, grant request (held, then granted), query, revoke, post-revoke refusal and notice all OK
```

The full run is in [transcript.golden.txt](transcript.golden.txt).

## What is simplified

- **The provider is a toy.** It has three entries in memory, a grant table, no storage, no
  minimization beyond category filtering, no audit log and no directives (every query result
  carries an empty `directives` list), and a registry of one client. It is not an HCP
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
- **The revocation notice is sent once.** HCP's retry schedule (HCP 9.9) is not shown; a test
  sends a later attempt to show the client deduplicating on `noticeId`.
- **The specs are drafts**, and several points are pending HCP confirmation. Each spec lists them
  under "Pending HCP confirmation".

## Layout

As in [`consumer-app-ts`](../consumer-app-ts) and [`getting-started-ts`](../getting-started-ts):
`trust-tasks.toml` pins the tasks, `trust-tasks.lock` and `trust-tasks/` hold the vendored bundles,
and `npm run generate` writes `src/trust-tasks/` (git-ignored). The framework tasks
(`trust-task-discovery`, `trust-task-next-step`) ship with `@openvtc/trust-tasks` and are imported
from it. The files:

- `src/demo.ts`: the parties, the toy provider, the client's state and the six steps, one
  function each.
- `src/validator.ts`: payload validation with ajv. The getting-started example's structural
  stand-in ignores `pattern`, which the HCP schemas rely on.
- `src/main.ts`: runs the demo and prints the transcript.
- `test/demo.test.ts`: the tests.
