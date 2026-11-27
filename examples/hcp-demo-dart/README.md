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
7. **Decline.** The client asks again on a new thread and the person declines (HCP 6.1.7). The
   client's next resubmission is refused with `permissionDenied`, the same fixed refusal an
   unknown person gets, and no grant is issued.
8. **Expired hold.** The client asks again and the person never decides. Once the hold's lifetime
   (300 s, provider policy) has passed, the policy denies: the next resubmission gets the same
   `permissionDenied`, the thread is never reopened (a late approval changes nothing), and a new
   request on a new thread is put to the person again.

The provider also applies the rules the tests exercise: a token presented from another VID gets
`permissionDenied`; only its one registered client may ask for a grant; a resubmission must carry
the held payload (compared in JCS form); and a resubmission after approval gets the same grant
within the hour, never a second one, and `permissionDenied` after it. Each side checks a reply
before it uses it: from the party asked, on the same thread, signed (unless the library built it),
and valid against its schema.

## Run it

Dart 3.10 or later (`affinidi_tsp` 0.2's floor). From this directory:

```sh
dart pub get
dart run trust_tasks:generate --lock trust-tasks.lock --out lib/trust_tasks
dart run bin/main.dart
```

`dart test` runs the tests listed in [SCENARIO.md](../hcp-demo-ts/SCENARIO.md#tests-every-port-has),
the same 18 in every port: the golden transcript, the query after revoke, a tampered document, a
schema-invalid document, the notice and its deduplication, the token from another VID, the
grant-request rules, the reply checks, the hold's hint and the token's lifetime, the decline and the
hold's timeout. The transcript is compared with the shared
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
=== 5. Query after revoke (hcp/entries/query 0.1) fails with hcp:invalidToken ===
…
    "code": "hcp:invalidToken",
…
hcp-demo-dart: discover, grant request (held, then granted), query, revoke, post-revoke refusal and revocation notice all OK
```

The full run is in [transcript.golden.txt](../hcp-demo-ts/transcript.golden.txt).

## What is simplified

- **The provider is a toy.** It has three entries in memory, a grant table, no storage, no
  minimization beyond category filtering, no audit log and no directives (every query result
  carries an empty `directives` list), and a registry of one client. It is not an HCP
  implementation.
- **There is no approval UI.** The person's decision is one function call, which approves what
  was asked (step 2) or declines (step 7); in step 8 the person never decides. The demo shows the
  shape of the hold: a next step first, then the grant or the refusal on resubmission.
- **Keys and identities are fixed.** Every party knows the others' public keys up front (a
  `StaticVidResolver`), so there is no DID resolution and no discovery of the provider's DID
  document.
- **Clock and identifiers are fixed** (see [SCENARIO.md](../hcp-demo-ts/SCENARIO.md)), so every
  run, in every language, prints the same transcript.
- **Tokens are opaque strings** the toy provider mints and looks up. There is no OAuth
  Authorization Service.
- **The revocation notice is sent once.** HCP's retry schedule (HCP 9.9) is not shown; a test
  sends a later attempt to show the client deduplicating on `noticeId`.
- **The specs are drafts**, and several points are pending HCP confirmation. Each spec lists them
  under "Pending HCP confirmation".

## Where the Dart port differs (nothing printed)

- TSP is version 1.0 (`affinidi_tsp` 0.2): `trust_tasks_tsp` seals with HPKE-Base and signs the
  message; the sender is authenticated by the TSP signature and the ESSR sender field. The sealed
  bytes are never printed.
- `TspConsumer` mints the `id` of an error it builds itself (a refusal before the handler runs,
  such as `proofInvalid` or `malformedRequest`) with a random `urn:uuid`; it takes no id hook. Only
  the tampered and schema-invalid tests (3 and 4) meet such refusals, and they check only the
  code, so the shared counter stays in step with the other ports.

## Layout

As in [`consumer-app-dart`](../consumer-app-dart) and
[`getting-started-dart`](../getting-started-dart): `trust-tasks.toml` pins the tasks,
`trust-tasks.lock` and `trust-tasks/` hold the vendored bundles (the same as `hcp-demo-ts`'s), and
`dart run trust_tasks:generate` writes `lib/trust_tasks/` (git-ignored). The framework tasks
(`trust-task-discovery`, `trust-task-next-step`) ship with `package:trust_tasks` and are imported
from it. The files:

- `lib/demo.dart`: the parties, the toy provider, the Context Client's wallet and processed
  notices, the reply checks, and the eight steps, one function each.
- `lib/validator.dart`: payload validation with
  [`package:json_schema`](https://pub.dev/packages/json_schema), a full JSON Schema 2020-12
  engine. The getting-started example's structural stand-in ignores `pattern`, which the HCP
  schemas rely on.
- `bin/main.dart`: runs the demo and prints the transcript.
- `test/demo_test.dart`: the tests.

The provider compares a resubmission with the held request in JCS form through `ssi`'s
`JcsUtil.canonicalize`, the canonicaliser `trust_tasks_proof` signs with (the proof package does
not export one of its own).

`affinidi_tsp` is not on pub.dev yet, so `pubspec.yaml` overrides it with the commit
`trust-tasks-dart-tsp/pubspec_overrides.yaml` pins.
