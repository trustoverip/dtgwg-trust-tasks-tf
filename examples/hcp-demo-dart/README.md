# HCP over Trust Tasks: a demonstration (Dart)

This example runs the Human Context Protocol (HCP 2.0-draft-3) flow as Trust Tasks, between three
parties in one process:

- a toy, in-memory **HCP Provider**,
- a **Context Client** (a booking agent that wants the person's travel preferences), and
- a **Management Client** (the person's own app).

It is the Dart port of [`hcp-demo-ts`](../hcp-demo-ts), and follows its
[SCENARIO.md](../hcp-demo-ts/SCENARIO.md), the script every language's port follows. Each party has
its own did:key and TSP keys. Every document is built with the generated types, signed with
`trust_tasks_proof`, and sealed as a TSP message with `trust_tasks_tsp`. The receiving end then runs
the SPEC §7.2 pipeline on it: proof verified over the document as received, payload validated
against its schema. Only then does anything act on it. Nothing touches the network.

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

Dart 3.8 or later (`trust_tasks_tsp`'s floor). From this directory:

```sh
dart pub get
dart run trust_tasks:generate --lock trust-tasks.lock --out lib/trust_tasks
dart run bin/main.dart
```

`dart test` runs the tests: the golden transcript, a query after revoke, a tampered document and a
schema-invalid document. The transcript is compared with the shared
[`../hcp-demo-ts/transcript.golden.txt`](../hcp-demo-ts/transcript.golden.txt), so this port and
the others print the same bytes. If the scenario changes, it changes in `hcp-demo-ts` first.

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
hcp-demo-dart: discover, grant request (held, then granted), query, revoke, notice and post-revoke refusal all OK
```

The full run is in [transcript.golden.txt](../hcp-demo-ts/transcript.golden.txt).

## What is simplified

- **The provider is a toy.** It has three entries in memory, a grant table, no storage, no
  minimization beyond category filtering, no audit log and no directives. It is not an HCP
  implementation.
- **There is no consent UI.** The person's decision is one function call, and it always approves
  what was asked. The demo shows the shape of the hold: a next step first, then the grant on
  resubmission.
- **Keys and identities are fixed.** Every party knows the others' public keys up front (a
  `StaticVidResolver`), so there is no DID resolution and no discovery of the provider's DID
  document.
- **Clock and identifiers are fixed** (see [SCENARIO.md](../hcp-demo-ts/SCENARIO.md)), so every
  run, in every language, prints the same transcript.
- **Tokens are opaque strings** the toy provider mints and looks up. There is no OAuth
  Authorization Service.
- **The revocation notice is sent once.** HCP's retry schedule (HCP 9.9) is not shown.
- **The specs are drafts**, and several points are pending HCP confirmation. Each spec lists them
  under "Pending HCP confirmation".

## Where the Dart port differs (nothing printed)

- `trust_tasks_tsp` seals with HPKE in Base mode and signs the message; the sender is authenticated
  by the TSP signature rather than by HPKE-Auth. The sealed bytes are never printed.
- `TspConsumer` mints the `id` of an error it builds itself (a refusal before the handler runs,
  such as `proofInvalid` or `malformedRequest`) with a random `urn:uuid`; it takes no id hook. Only
  the tampered and schema-invalid tests meet such refusals, and they check only the code, so the
  shared counter stays in step with the other ports.

## Layout

As in [`consumer-app-dart`](../consumer-app-dart) and
[`getting-started-dart`](../getting-started-dart): `trust-tasks.toml` pins the tasks,
`trust-tasks.lock` and `trust-tasks/` hold the vendored bundles (the same as `hcp-demo-ts`'s), and
`dart run trust_tasks:generate` writes `lib/trust_tasks/` (git-ignored). The framework tasks
(`trust-task-discovery`, `trust-task-next-step`) ship with `package:trust_tasks` and are imported
from it. The files:

- `lib/demo.dart`: the parties, the toy provider and the five steps.
- `lib/validator.dart`: payload validation with
  [`package:json_schema`](https://pub.dev/packages/json_schema), a full JSON Schema 2020-12
  engine. The getting-started example's structural stand-in ignores `pattern`, which the HCP
  schemas rely on.
- `bin/main.dart`: runs the demo and prints the transcript.
- `test/demo_test.dart`: the tests.

`affinidi_tsp` is not on pub.dev yet, so `pubspec.yaml` overrides it with the commit
`trust-tasks-dart-tsp/pubspec_overrides.yaml` pins.
