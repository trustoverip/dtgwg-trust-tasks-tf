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
| `hcp/entries/query` | 0.1 | steps 3 and 6 |
| `hcp/grants/revoke` | 0.1 | step 4 |
| `hcp/grants/revoked` | 0.1 | step 5 |

`trust-task-error` 0.6 is used too, but has no bundle: every runtime models it by hand.

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
  new time. Instants are whole seconds, written `YYYY-MM-DDTHH:MM:SSZ`. Use the library's clock
  hook (or a fixed clock you inject); never format the system time.
- **Instants inside a payload that mean "now"** (`grantedAt`, `respondedAt`, `revokedAt`) are the
  clock **before** the reply ticks, which is the `issuedAt` of the document being answered (for
  `revokedAt`, the `issuedAt` of the revoke response).
- **Identifiers.** One counter, shared by every party, starting at 1:
  `urn:uuid:00000000-0000-4000-8000-` followed by the counter in **lower-case hex**, zero-padded to
  12 digits. A document's `id` takes the next value when the document is built. A request's
  `threadId` is its own `id`, except a resubmission, which keeps the first request's `threadId`.
  Payload identifiers (`grantId`, `retrievalId`) take the next value *before* the reply's own `id`.
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

A grant table: `grantId`, token, client VID, categories, purpose codes, status (`active` |
`revoked`). A held-request table keyed by `threadId`.

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

Rules: a query whose token is unknown, not the sender's, or under a revoked grant gets
`hcp:invalidToken` ("The access token is not valid.", retryable false). A purpose or category
outside the grant gets `permissionDenied` with the fixed text "The request is not authorized.". A
revoke whose token is not the management token, or whose `grantId` is unknown, gets the same
`permissionDenied`.

## Every receiving end

The Provider and the Context Client each receive through the TSP consumer and run the SPEC §7.2
pipeline: open the TSP message, check the authenticated sender against `issuer`, check freshness
and the replay guard, **verify the proof over the document as received**, **validate the payload
against its schema** (a full JSON Schema 2020-12 validator), and only then call the handler. The
sender opens the sealed reply, verifies its proof where it has one, and validates its payload
against the reply type's schema before it uses it.

## The steps

Payload values not listed here are in the golden transcript.

| # | From → To | Document `type` | id | Payload (summary) | Expected reply |
|---|---|---|---|---|---|
| 1 | Context Client → Provider | `…/trust-task-discovery/0.4` (bare) | #1 | `{"patterns": ["hcp/*"]}` | `#response` #2: `supportedTypes` = the four `hcp/*` Type URIs, bare, in the order request, query, revoke, revoked |
| 2a | Context Client → Provider | `…/hcp/grants/request/0.1#request` | #3 | `access`: one detail, `actions ["read"]`, `categories ["travel_preferences"]`, purpose `https://purposes.example/travel-booking` ("Book trips that match how you like to travel"); `expiresIn 14400`; `subjectHint "acct:alex@provider.example"` | `trust-task-next-step/0.1` #4: `expects` the request Type URI, reason "The request is with the person, who has not decided yet.", `hint {"retryAfter": 5}`; `continuation "resubmit"`; `inResponseTo` #3 |
| — | the person | — | — | prints `[person] approves the grant request on thread <#3> in the provider's app` | — |
| 2b | Context Client → Provider | same, **resubmission** | #5, thread #3 | identical payload | `#response` #7: `accessToken hcp_at_demo_0001`, `grantId` #6, `access` as asked, `grantType temporary`, `grantedAt` 10:00:05, `tokenExpiresAt` +1 h, `grantExpiresAt` +14400 s |
| 3 | Context Client → Provider | `…/hcp/entries/query/0.1#request` | #8 | `query "How does Alex like to fly?"`, `categories ["travel_preferences"]`, the purpose, `maxEntries 5`, the grant token | `#response` #10: the two travel entries, `retrievalId` #9, `minimization {"method": "lexical"}`, `categoriesSearched ["travel_preferences"]`, `respondedAt` 10:00:07 |
| 4 | Management Client → Provider | `…/hcp/grants/revoke/0.1#request` | #11 | `grantId` #6, `accessToken hcp_mt_demo_management` | `#response` #12: `{}` |
| 5 | Provider → Context Client | `…/hcp/grants/revoked/0.1#request` | #13 | `grantId` #6, `revokedAt` 10:00:10 | `#response` #14 from the Context Client: `{}` |
| 6 | Context Client → Provider | `…/hcp/entries/query/0.1#request` | #15 | the same payload as step 3 | `trust-task-error/0.6` #16: `code "hcp:invalidToken"`, `inResponseTo` #15 |

The ids in the table are the counter values (#10 is `…-00000000000a`).

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
5. `5. Revocation notice (hcp/grants/revoked 0.1)`
6. `After revoke: the same query fails with hcp:invalidToken`

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
next step and the resubmission. Nothing else is printed into the transcript. A port's `main`
may print a closing summary line after it; the golden file does not contain it.

## Tests every port has

1. The run's transcript equals `transcript.golden.txt`.
2. Step 6's reply is a `trust-task-error` with code `hcp:invalidToken`.
3. A signed query whose payload is altered after signing is refused with `proofInvalid`.
4. A signed query with no `purpose` is refused with `malformedRequest`.

Tests 3 and 4 run after the whole scenario, on the same world. The refusals in 3 and 4 come from
the library pipeline, not the handler; they are unsigned and their `issuedAt` may come from the
library's own clock, so the tests check only the code.

## Documented differences

No port differs in a line of the golden file. A port that cannot match one says so here, with the
reason.

Differences that print nothing:

- **Dart** (`hcp-demo-dart`): `trust_tasks_tsp` seals with HPKE in Base mode (the only mode
  `affinidi_tsp` offers) and signs the message, rather than HPKE-Auth. Its `TspConsumer` mints the
  `id` of an error the library builds (tests 3 and 4) with a random `urn:uuid`, as it has no id
  hook; those tests check only the code.
