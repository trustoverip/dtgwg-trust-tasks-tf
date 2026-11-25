# HCP demo: the scenario every port follows

This file is the language-neutral script for the HCP-over-Trust-Tasks demo. The TypeScript
version (`examples/hcp-demo-ts`) is the first implementation. The Rust (`examples/hcp-demo`), Go
(`examples/hcp-demo-go`) and Dart (`examples/hcp-demo-dart`) ports follow it. Every port prints
**the same transcript, byte for byte**: [`transcript.golden.txt`](transcript.golden.txt) in this
directory is the shared golden file, and each port's test compares its own output with it.

It is a demonstration, not a reference provider. The keys are DEMO ONLY.

## Pinned tasks

`trust-tasks.toml` pins, and `trust-tasks.lock` resolves, exactly these:

| Task | Version | Used for |
|---|---|---|
| `trust-task-discovery` | 0.4 | step 1 |
| `hcp/grants/request` | 0.1 | step 2 |
| `trust-task-next-step` | 0.1 | step 2 (the hold) |
| `hcp/entries/query` | 0.1 | steps 3 and 5 |
| `hcp/grants/revoke` | 0.1 | step 4 |
| `hcp/grants/revoked` | 0.1 | step 6 |

`trust-task-error` 0.6 is used too, but has no bundle: every runtime models it by hand.

The Rust port pins only the four `hcp/*` tasks: `trust-tasks-build` refuses to generate a
framework specification, and `trust-task-discovery` and `trust-task-next-step` ship with the Rust
runtime (`trust_tasks::specs`). The versions used are the same.

Every port's vendored bundles (`trust-tasks/`) and lock must match a registry built from this
repository's `specs/`: `scripts/check-demo-bundles.mjs` checks it in CI. After a change under
`specs/hcp/`, run `npm run build-bundles` at the repository root, then `trust-tasks update` in each
port with `TRUST_TASKS_REGISTRY=<repo>/website`, and reset the lock's `registry =` line to
`https://trusttasks.org`.

## Parties and keys (DEMO ONLY)

Each key is 32 bytes, every byte the same value. Never use such keys for anything real.

| Party | Ed25519 signing key bytes | X25519 TSP encryption key bytes | VID (did:key of the Ed25519 public key) |
|---|---|---|---|
| Provider (toy HCP Provider) | `0x11` × 32 | `0x12` × 32 | `did:key:z6MktULudTtAsAhRegYPiZ6631RV3viv12qd4GQF8z1xB22S` |
| Context Client (a booking agent) | `0x21` × 32 | `0x22` × 32 | `did:key:z6MkodJHzY8Bn6PycRu3pon7V3vzfk12nbndWcBz2mxMAsa2` |
| Management Client (the person's app) | `0x31` × 32 | `0x32` × 32 | `did:key:z6MkjJRuJtMxfTErK55eWeV7FQLLxpsvzSUQ3AC6DmdNPFja` |

- The VID is both the Trust Task `issuer`/`recipient` and the TSP VID.
- The proof's `verificationMethod` is `<did>#<multibase key>`, as the library's did:key signer writes it.
- Every party knows the others' public keys up front. There is no DID resolution and no network.

## Deterministic inputs

- **Clock.** It starts at `2026-11-25T10:00:00Z` and advances **one second for every document
  issued**, requests and replies alike. A document's `issuedAt` and its proof's `created` are the
  new time. Instants are whole seconds, written `YYYY-MM-DDTHH:MM:SSZ`, through one `iso()`
  helper over the injected clock; never format the system time (no `toISOString()`,
  `Utc::now()`, `time.Now()`, `DateTime.now()`). Tests may move the clock on without issuing a
  document (`advance(seconds)`); the main run never does.
- **Instants inside a payload that mean "now"** (`grant.issuedAt`, `respondedAt`) are the clock
  **before** the reply ticks, which is the `issuedAt` of the document being answered. Instants
  relative to now (`tokenExpiresAt`, `grant.expiresAt`, `useUntil`) add to that same instant.
  `revokedAt` is the instant the provider revoked the grant: the `issuedAt` of the revoke request
  (step 4), recorded with the grant and copied into the notice (step 6).
- **Identifiers.** One counter, shared by every party, starting at 1:
  `urn:uuid:00000000-0000-4000-8000-` followed by the counter in **lower-case hex**, zero-padded to
  12 digits. A document's `id` takes the next value when the document is built. A request's
  `threadId` is its own `id`, except a resubmission (and a later attempt of a notice), which keeps
  the first document's `threadId`. Payload identifiers (`grantId`, `retrievalId`, `noticeId`) take
  the next value *before* the `id` of the document that carries them: the reply's for `grantId`
  and `retrievalId`, the notice's own for `noticeId`.
- **Proofs.** `DataIntegrityProof`, `eddsa-jcs-2022`, `proofPurpose: authentication`, `created`
  equal to `issuedAt`. Ed25519 over JCS is deterministic, so `proofValue` matches across languages.
  Every document a party *builds* is signed: requests, responses, next steps and the handler's
  error responses.
- **TSP.** Each document travels as a Direct TSP message (HPKE-Auth sealed, signed) carrying the
  binding's envelope `{"type": "https://trusttasks.org/binding/tsp/0.1/envelope", "document": …}`,
  and each reply comes back the same way. The sealed bytes are random (HPKE ephemeral keys) and are
  never printed.
- **Tokens (DEMO ONLY).** The person's Management Client holds the token `hcp_mt_demo_management`,
  issued at setup. The provider mints grant tokens `hcp_at_demo_0001`, `hcp_at_demo_0002`, ….

## The toy provider

In memory. Three entries of the person's context:

| ref | category | value |
|---|---|---|
| `hcp-ref-0001` | `travel_preferences` | Prefers aisle seats and early-morning flights. |
| `hcp-ref-0002` | `travel_preferences` | Avoids overnight layovers. |
| `hcp-ref-0003` | `health_context` | Severe peanut allergy. |

**A client registry with one entry**: the Context Client's VID, registered as name
`Wayfare booking agent`, operator `{"id": "booking.example", "name": "Wayfare Travel Ltd"}`. The
person is known to that client by the pseudonym `hcp-sub-demo-alex`.

**A grant table**: `grantId`, token, `tokenExpiresAt`, client VID, the approved `access`, categories,
purpose codes, `issuedAt`, `expiresAt`, status (`active` | `revoked`), `revokedAt`.

**A hold table** keyed by `threadId`: the held request's `issuer`, its payload and the payload's
JCS form, a state (`pending` → `approved` → `granted`), and once granted the `grantId` and the
instant the response was first sent.

Each entry disclosed in a query response is:

```json
{
  "ref": "<ref>", "version": 1,
  "content": { "profile": "https://hcp.example/spec/profiles/general-context/2.0", "value": "<value>", "language": "en" },
  "assertion": { "declarationType": "person-declared" },
  "classification": { "categories": ["<category>"] },
  "standing": { "assurance": "authenticated", "confirmed": true },
  "useUntil": "<now + 1 hour>"
}
```

Every `permissionDenied` the handler sends has the fixed message "The request is not authorized."
and `retryable: false`; every `hcp:invalidToken` has "The access token is not valid." and
`retryable: false`.

### Rules, in the order the handler applies them

**`hcp/grants/request`** (spec: Authorization, Holding while the person decides, After approval):

1. The `issuer` is not in the client registry → `permissionDenied`. Nothing is held.
2. No hold for the `threadId` → hold it (`pending`, recording `issuer`, payload, JCS form) and
   answer the next step.
3. The hold's `issuer` differs from this `issuer` → `permissionDenied`; the hold is unaffected.
4. The payload's JCS form differs from the held one → `malformedRequest` with message "The
   resubmission differs from the held request."; the hold is unaffected. Compare JCS forms, never
   the bytes or a language's own serialisation: members in another order are the same payload.
5. `pending` → a further next step (same shape, `inResponseTo` the resubmission).
6. `approved` → issue the one grant (approved as asked), set the hold to `granted` with the
   `grantId` and the response time, and answer the response.
7. `granted` → if more than 3600 s have passed since the response was first sent, or the grant is
   not `active` → `permissionDenied`. Otherwise answer a response for **the same grant**: the same
   token, `tokenExpiresAt` and `grant`, with its status as it stands. Never a second grant.

**`hcp/entries/query`** (spec: Authorization, The access token, Errors):

1. No grant has the token → `hcp:invalidToken`.
2. The grant's client VID is not the `issuer` → `permissionDenied` (a token presented from another
   VID; checked before the status, so it tells that VID nothing about the grant).
3. The grant is not `active` → `hcp:invalidToken`.
4. The `purpose` is not one of the grant's, or a requested category is outside the grant →
   `permissionDenied`.
5. Otherwise the response: the matching entries (at most `min(maxEntries ?? 5, 20)`), with
   `directives: []`.

**`hcp/grants/revoke`**: a token other than the management token, or an unknown `grantId` →
`permissionDenied`. Otherwise, if the grant is `active`, set it `revoked` and record `revokedAt`
(now); answer `{}` (a second revoke is idempotent and keeps the first `revokedAt`).

### The Context Client's state

- **A wallet** keyed by `grantId`: the token, and the `ref`s of the copies disclosed under the
  grant (recorded from the step 3 response).
- **Processed notices**: `noticeId` → `grantId`.
- **Forgetting a grant** removes its wallet entry: the token and every copy (HCP 6.2.2, 7.6).
- On `hcp:invalidToken` in step 5 the client forgets the grant at once (hcp/grants/revoked
  "Independence").
- On a notice (step 6): if its `noticeId` is already processed, acknowledge `{}` and do nothing
  else; otherwise record `noticeId` → `grantId`, forget the grant, and acknowledge `{}`. The
  client never sends the grant's token after it has forgotten the grant.

## Every receiving end

The Provider and the Context Client each receive through the TSP consumer and run the SPEC §7.2
pipeline: open the TSP message, check the authenticated sender against `issuer`, check freshness
and the replay guard, **verify the proof over the document as received**, **validate the payload
against its schema** (a full JSON Schema 2020-12 validator), and only then call the handler.

The sender opens the sealed reply and, before it uses it, checks in this order:

1. the reply's `issuer` is the VID the document was sent to;
2. the reply's `threadId` is the sent document's `threadId`;
3. the reply carries a `proof`, unless it is a `trust-task-error` whose `code` is one the
   library pipeline raises before any handler runs (`malformedRequest`, `unsupportedType`,
   `unsupportedVersion`, `unsupportedExtension`, `expired`, `proofRequired`, `proofInvalid`,
   `wrongRecipient`, `identityMismatch`, `idConflict`), which the library builds unsigned;
4. the proof, where present, verifies;
5. the payload validates against the reply type's schema (`trust-task-error` included).

Any failure is fatal to the run (an exception, a panic, an error return).

## The steps

One function per step, each commented with the HCP section and Trust Task type it shows. Payload
values not listed here are in the golden transcript. The ids are the counter values (#10 is
`…-00000000000a`); times are on 2026-11-25.

| # | From → To | Document `type` | id | Payload (summary) | Expected reply |
|---|---|---|---|---|---|
| 1 | Context Client → Provider | `…/trust-task-discovery/0.4` (bare) | #1, 10:00:01 | `{"patterns": ["hcp/*"]}` | `#response` #2: `supportedTypes` = the four `hcp/*` Type URIs, bare, in the order request, query, revoke, revoked |
| 2a | Context Client → Provider | `…/hcp/grants/request/0.1#request` | #3, 10:00:03 | `access`: one detail, `type https://hcp.example/spec/authorization/context-access`, `actions ["read"]`, `categories ["travel_preferences"]`, purposes `[{code https://purposes.example/travel-booking, description "Book trips that match how you like to travel"}]`; `expiresIn 14400`; `subjectHint "acct:alex@provider.example"` | `trust-task-next-step/0.1` #4: `expects [{typeUri <request Type URI>#request, reason "The request is with the person, who has not decided yet.", hint {"retryAfter": 5}}]`; `continuation "resubmit"`; `inResponseTo {typeUri …#request, id #3}` |
| — | the person | — | — | prints `[person] approves the grant request on thread <#3> in the provider's app`; the hold becomes `approved` | — |
| 2b | Context Client → Provider | same, **resubmission** | #5, thread #3, 10:00:05 | identical payload | `#response` #7: `accessToken hcp_at_demo_0001`, `tokenExpiresAt` 11:00:05, `grant` = `{grantId #6, subject {id hcp-sub-demo-alex}, authorizingPrincipal {id hcp-sub-demo-alex}, client {id <Context Client VID>, name, operator}, access <as asked>, grantType temporary, issuedAt 10:00:05, expiresAt 14:00:05, status active}`. The client puts the token in its wallet under #6 |
| 3 | Context Client → Provider | `…/hcp/entries/query/0.1#request` | #8, 10:00:07 | `query "How does Alex like to fly?"`, `categories ["travel_preferences"]`, `purpose` the purpose code, `maxEntries 5`, `accessToken` from the wallet | `#response` #10: the two travel entries (`useUntil` 11:00:07), `retrievalId` #9, `minimization {"method": "lexical"}`, `categoriesSearched ["travel_preferences"]`, `directives []`, `respondedAt` 10:00:07. The client records refs `hcp-ref-0001`, `hcp-ref-0002` |
| 4 | Management Client → Provider | `…/hcp/grants/revoke/0.1#request` | #11, 10:00:09 | `grantId` #6, `accessToken hcp_mt_demo_management` | `#response` #12: `{}`; the grant is `revoked`, `revokedAt` 10:00:09 |
| 5 | Context Client → Provider | `…/hcp/entries/query/0.1#request` | #13, 10:00:11 | the same payload as step 3 (the token still in the wallet: no notice has arrived) | `trust-task-error/0.6` #14: `code "hcp:invalidToken"`, `message "The access token is not valid."`, `retryable false`, `inResponseTo` #13. The client forgets grant #6 |
| 6 | Provider → Context Client (`byVid(grant.client)`) | `…/hcp/grants/revoked/0.1#request` | #16, 10:00:13 | `grantId` #6, `noticeId` #15 (HCP's `jti`, the same on every delivery attempt), `revokedAt` 10:00:09 | `#response` #17 from the Context Client: `{}`. The client records #15 → #6 |

## Transcript format

UTF-8, `\n` line ends. For each step a header line, preceded by a blank line:

```
\n=== <title> ===\n
```

The titles are exactly:

1. `1. Discover (trust-task-discovery 0.4)`
2. `2. Grant request (hcp/grants/request 0.1), held with trust-task-next-step`
3. `3. Query (hcp/entries/query 0.1)`
4. `4. Revoke (hcp/grants/revoke 0.1) from the Management Client`
5. `5. Query after revoke (hcp/entries/query 0.1) fails with hcp:invalidToken`
6. `6. Revocation notice (hcp/grants/revoked 0.1)`

Then, for each document sent and each reply, in order:

```
--> <From name> -> <To name> (TSP) <document type>\n<document>\n
```

Party names are `Provider`, `Context Client` and `Management Client`. `<document>` is the whole
document (proof included) as JSON with **object keys sorted by code point** at every level
(JCS order), two-space indentation, `": "` between key and value, arrays and objects one element
per line, empty arrays `[]` and empty objects `{}`, and strings escaped as `JSON.stringify` does
(non-ASCII left as is). **Redaction:** the value of every member named `accessToken` is printed as
its first 6 characters followed by `…` (U+2026). The person's decision prints one line,
`[person] approves the grant request on thread <threadId> in the provider's app`, between the
next step and the resubmission. Nothing else is printed into the transcript: the client's wallet
and notice record change silently. A port's `main` may print a closing summary line after it;
the golden file does not contain it.

## Tests every port has

"After the demo" means on the same world, once all six steps have run; "after the grant" means on
a fresh world after steps 1 and 2 only. Refusals the library pipeline builds (tests 3 and 4) are
unsigned and their `issuedAt` may come from the library's own clock, so those tests check only the
code.

1. The run's transcript equals `transcript.golden.txt`.
2. Step 5's reply is a `trust-task-error` with code `hcp:invalidToken`.
3. After the demo: a signed query whose payload is altered after signing is refused with
   `proofInvalid`.
4. After the demo: a signed query with no `purpose` is refused with `malformedRequest`.
5. After the demo: the wallet has no entry for the grant, the processed notices are exactly
   {`noticeId` of step 6 → `grantId`}; a later delivery attempt (a new document from the Provider,
   the notice's `threadId` and payload) is acknowledged with `{}` and the processed notices still
   hold one entry.
6. After the grant: step 4 (revoke), then step 6 (notice) without step 5: the wallet entry is
   there before the notice and gone after it.
7. After the grant: the Management Client sends a query with the grant's token
   (`hcp_at_demo_0001`) and otherwise step 3's payload → `permissionDenied`.
8. After the demo: the Management Client (not a registered client) sends a new grant request with
   step 2's payload → `permissionDenied`; the hold table still holds one entry.
9. After the grant: a resubmission on thread #3 with `expiresIn 3600` → `malformedRequest`.
10. After the grant: a resubmission on thread #3 with step 2's payload, members in another order
    → the grant response, `grant.grantId` #6, and the grant table still holds one grant.
11. After the grant: move the clock on 3601 s, then a resubmission on thread #3 with step 2's
    payload → `permissionDenied`.
12. After the demo: a resubmission on thread #3 with step 2's payload → `permissionDenied`; the
    grant table still holds one grant.
13. The reply check, on hand-built replies: an unsigned `proofInvalid` error from the Provider on
    the thread is accepted; a signed `permissionDenied` is accepted; an unsigned
    `permissionDenied` is refused; a reply whose `issuer` is not the party asked is refused; a
    reply on another `threadId` is refused.

## Documented differences

No port differs in a line of the golden file. A port that cannot match one says so here, with the
reason.

Differences that print nothing:

- **Dart** (`hcp-demo-dart`): `trust_tasks_tsp` seals with HPKE in Base mode (the only mode
  `affinidi_tsp` offers) and signs the message, rather than HPKE-Auth. Its `TspConsumer` mints the
  `id` of an error the library builds (tests 3 and 4) with a random `urn:uuid`, as it has no id
  hook; those tests check only the code.
- **Go** (`hcp-demo-go`): `tsp.Consumer.Now` drives the pipeline's checks (so test 11's freshness
  check runs on the injected clock), but the consumer has no hook for the clock that stamps the
  refusals its pipeline builds (tests 3 and 4): those carry the library's system clock, and the
  tests check only the code. The provider compares grant-request payloads with
  `trusttasks.CanonicalJSON` (members ordered recursively, no whitespace), which is
  equality-equivalent to JCS but not byte-exact JCS: `trust-tasks-go/proof`'s JCS is not exported.
