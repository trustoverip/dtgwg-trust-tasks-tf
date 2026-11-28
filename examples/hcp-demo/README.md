# HCP over Trust Tasks: a demonstration (Rust)

This example runs the Human Context Protocol (HCP 2.0-draft-3) flow as Trust Tasks, between three
parties in one process:

- a toy, in-memory **HCP Provider**,
- a **Context Client** (a booking agent that wants the person's travel preferences), and
- a **Management Client** (the person's own app).

Each party has its own did:key and TSP keys. Every document is signed with
`trust_tasks::proof` (eddsa-jcs-2022) and sealed as a TSP message with `trust_tasks::tsp`. The
receiving end then runs the SPEC §7.2 pipeline on it (`TspConsumer::receive_dispatched`): proof
verified over the document as received, payload validated against its schema with a JSON Schema
2020-12 validator. Only then does anything act on it. Nothing touches the network.

It is the Rust port of [`hcp-demo-ts`](../hcp-demo-ts). Both follow
[SCENARIO.md](../hcp-demo-ts/SCENARIO.md) and print the same transcript, byte for byte.

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
the held payload (compared in JCS form, as received rather than through the generated type); and a
resubmission after approval gets the same grant with a newly issued token within the hour, never
a second grant, and `permissionDenied` after it. The provider keeps only a SHA-256 digest of each
token it issues, never the token, and accepts every token issued under a grant until the grant
ends. Each side checks a reply before it uses it: from the party asked, on
the same thread, signed (unless the library built it), and valid against its schema
(`trust-task-error` included, against the framework's own schema in `specs/`).

## Run it

From the repository root:

```sh
cargo run -p trust-tasks-example-hcp-demo     # the demo
cargo test -p trust-tasks-example-hcp-demo    # the tests
```

The tests are the 18 listed in [SCENARIO.md](../hcp-demo-ts/SCENARIO.md#tests-every-port-has): the
golden transcript, the query after revoke, a tampered document, a schema-invalid document, the
client's handling of the notice, the provider's token and grant-request rules, the hold's hint and the token's lifetime, the decline and the hold's expiry, and the reply check.
The golden file is shared with the other ports and lives beside the
TypeScript version, [`../hcp-demo-ts/transcript.golden.txt`](../hcp-demo-ts/transcript.golden.txt);
the test reads it from there and never keeps a copy. A change to the scenario starts in SCENARIO.md
and the TypeScript version, which rewrites the golden file.

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
hcp-demo: discover, grant request (held, then granted), query, revoke, post-revoke refusal and notice all OK
```

## What is simplified

- **The provider is a toy.** It has three entries in memory, a grant table, no storage, no
  minimization beyond category filtering, no audit log and no directives (every query result
  carries an empty `directives` list), and a registry of one client. It is not an HCP
  implementation.
- **There is no approval UI.** The person's decision is one function call, which approves what
  was asked (step 2) or declines (step 7); in step 8 the person never decides. The demo shows the
  shape of the hold: a next step first, then the grant or the refusal on resubmission.
- **Keys and identities are fixed.** Every party knows the others' public keys up front, so there
  is no DID resolution and no discovery of the provider's DID document.
- **Clock and identifiers are fixed** (see [SCENARIO.md](../hcp-demo-ts/SCENARIO.md)), so every
  run, in every language, prints the same transcript. `respond_with` and `reject_with` stamp
  `issuedAt` from the system clock, so the demo overwrites it with its own clock before signing.
- **Tokens are opaque strings** the toy provider mints and looks up. There is no OAuth
  Authorization Service.
- **The revocation notice is sent once.** HCP's retry schedule (HCP 9.9) is not shown; a test
  sends a later attempt to show the client deduplicating on `noticeId`.
- **The specs are drafts**, and several points are pending HCP confirmation. Each spec lists them
  under "Pending HCP confirmation".

## Layout

As in [`consumer-app`](../consumer-app) and [`getting-started`](../getting-started):
`trust-tasks.toml` pins the tasks, `trust-tasks.lock` and `trust-tasks/` hold the vendored bundles,
and `build.rs` generates them with `trust-tasks-build` into `$OUT_DIR`. Everything else comes
through the `trust-tasks` facade (features `tsp`, `proof-affinidi` and `validate`), with
`affinidi-tsp` for the TSP keys, `affinidi-secrets-resolver` for the signer, and
`serde_json_canonicalizer` (the RFC 8785 implementation the proof crate signs with) for comparing
resubmissions.

The lock pins the four `hcp/*` tasks only. `trust-task-discovery` and `trust-task-next-step` are
framework specifications, which ship with the runtime (`trust_tasks::specs`) and which
`trust-tasks-build` refuses to generate, so, unlike the TypeScript version, they are not pinned
here.

The files:

- `src/lib.rs`: the parties, the toy provider, the client's state, the handlers and the eight
  steps, one function each.
- `src/main.rs`: runs the demo and prints the transcript.
- `tests/demo.rs`: the tests.
- `tests/any_of_entry.rs`: a Rust-only check that the generated `query` types read a whole
  `Entry` (not one of the shared tests).
