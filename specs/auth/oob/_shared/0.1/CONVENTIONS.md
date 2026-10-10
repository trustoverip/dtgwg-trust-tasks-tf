# auth/oob — family conventions

This document records the rules shared by every specification under
`auth/oob/`. Where it could conflict with a specific spec, **that spec's own
front matter and payload schema are authoritative.** How a starter hands a
request to an approver — the `sign-in` trigger link — is in
[`TRIGGER-LINK.md`](TRIGGER-LINK.md).

## 1. Roles and tasks

A **starter** (for `login`, a browser on the portal page) asks a **service**
(for `login`, the community's VTC) to open a request. An **approver** (a member,
using a wallet and their VTA) decides it on another device. The family has six
tasks a party sends, and two DID-signed documents that are only ever carried
inside another task:

| Task | From → to | Signed by | Carries |
|---|---|---|---|
| [`auth/oob/request`](../../request/0.1/spec.md) | starter → service | starter key `K_b` | — |
| [`auth/oob/claim`](../../claim/0.1/spec.md) | approver → service | lock key `K_a` | — |
| [`auth/oob/prove`](../../prove/0.1/spec.md) | approver → service | `K_a` | `identify` |
| [`auth/oob/identify`](../../identify/0.1/spec.md) | (carried) | approving DID, `authentication` | — |
| [`auth/oob/respond`](../../respond/0.1/spec.md) | approver → service | `K_a` | `grant` |
| [`auth/oob/grant`](../../grant/0.1/spec.md) | (carried) | approving DID, `assertionMethod` | — |
| [`auth/oob/redeem`](../../redeem/0.1/spec.md) | starter → service | `K_b` | — |
| [`auth/oob/cancel`](../../cancel/0.1/spec.md) | starter or approver → service | `K_b` or `K_a` | — |

Carrying a signed document inside another follows the precedent of
[`auth/signing-key/authorize`](../../../signing-key/authorize/0.1/spec.md)
inside `auth/signing-key/enroll`.

## 2. Shared schema components

[`oob.schema.json`](oob.schema.json) defines `Purpose`, `Mode`, `RequestId`,
`OobKey`, `MatchNumber`, `Origin`, `Service`, `SameNetwork` and `Requester`,
referenced by every task in the family
([SPEC §6.6](/SPEC.md#66-shared-schema-components)).

## 3. Closed lists

**`purpose` is a closed list.** Each purpose has a fixed row below covering
what it reveals at each step, what the grant contains, what `redeem` returns and
the window lengths. A service refuses any other value with
`auth/oob/request:purposeUnsupported`. Adding a purpose is a new version of
`oob.schema.json` and of the tasks that reference it.

| Purpose | Status | Step 1 (claim) reveals | Step 2 (prove) reveals | Grant authorizes | `redeem` returns | Windows |
|---|---|---|---|---|---|---|
| `login` | defined | service, portal origin, purpose, decision deadline | step 1 plus `sessionKey`, `requester`, `identifiedAs` | `sessionKey` acts as the approving DID at `origin` until `notAfter` | a `Session` for the DID, bound to the starter key | claim ≤ 180 s, decision ≤ 180 s; 120 s each RECOMMENDED |
| `stepUp` | reserved | — | — | — | — | — |
| `deviceEnrol` | reserved | — | — | — | — | — |

**`mode` is a closed list.** A service refuses any other value with
`auth/oob/request:modeUnsupported`.

| Mode | Status | Meaning |
|---|---|---|
| `scan` | defined | The starter shows the `sign-in` trigger link as a QR code that is also a link ([`TRIGGER-LINK.md`](TRIGGER-LINK.md)). |
| `code` | reserved | A typed code of at least 8 characters, under strict rate limits. |
| `push` | reserved | Sent to the approver's enrolled devices. |

Reserved names are spelled in lowerCamelCase, the registry's casing for
enumerated values ([SPEC §4.10](/SPEC.md#410-naming-conventions)).

## 4. Request state

A service keeps one record per request:
`pending → claimed → identified → approved → consumed`. `declined`,
`cancelled` and `expired` are final. Every change of state is a
compare-and-set, so only one caller can make it.

- A request must be claimed by its **claim deadline** (`claimDeadline`). A
  successful claim starts a fresh **decision window** (`decisionDeadline`)
  covering prove, respond and redeem.
- A failed claim leaves the request as it was. A failed `prove` or `respond`
  from the lock holder moves it to `declined`: each request allows one claim,
  one proof attempt, one decision and one redemption.
- A request's purpose, mode, origin, starter key and lock cannot change once
  set.

## 5. Rules for the whole family

1. **Keys.** The starter key and the approver lock key are Ed25519 `did:key`s
   (`OobKey`), each generated for one request and used nowhere else. A service
   refuses any other with `auth/oob:keyUnsupported`.
2. **Disclosure ladder.** Nothing about the starter is revealed before a
   successful `prove`. Step 1 (the `claim` response) names only the service, the
   origin, the purpose and the deadline.
3. **Authority goes to the starter key, through `redeem` only.** Responses to the
   approver never carry session material.
4. **Proof purposes.** The starter and lock keys sign with `authentication`.
   `identify` is signed by the approving DID for `authentication`. `grant` is
   an attestation and is signed for `assertionMethod`. The service's `claim` and
   `prove` responses are attestations the approver relies on (and the grant
   cites by digest), and are signed with the service's `assertionMethod` key
   ([SPEC §4.7.3](/SPEC.md#473-proof-purpose-and-verification-relationship)).
5. **No caching.** Every response, including every error, is sent with
   `Cache-Control: no-store` where the transport has such a header.
6. **Generic failures.** A non-member and a bad signature at `prove` or
   `respond` get the same `auth/oob:notAuthorized`, so the family cannot be used
   to probe membership.
7. **Rate limits.** A service applies per-network limits on `request`, `claim`
   and `prove`, caps pending requests per network, and allows one open `redeem`
   poll per request. Refusals use `auth/oob:rateLimited`.

## 6. Family error codes

These codes mean the same in every task that declares them, so they use the
family namespace ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)
rule 2):

| Code | Meaning |
|---|---|
| `auth/oob:keyUnsupported` | The starter or lock key is not an Ed25519 `did:key`. |
| `auth/oob:rateLimited` | A rate limit was hit. Retryable after `retryAfter`. |
| `auth/oob:requestNotFound` | No request has this `requestId`. |
| `auth/oob:requestExpired` | The request has ended (its window passed, or it was already consumed) and can no longer be acted on. |
| `auth/oob:notClaimant` | The document is not signed by the request's lock key. |
| `auth/oob:notStarter` | The document is not signed by the request's starter key. |
| `auth/oob:notAuthorized` | A carried document failed verification, or its issuer is not entitled. Deliberately generic. |
| `auth/oob:alreadyDecided` | The request has already been approved, declined or cancelled. |

Task-specific codes (`purposeUnsupported`, `modeUnsupported`, `alreadyClaimed`,
`numberMismatch`, `contextMismatch`, `pending`, `declined`) are declared on the
task they belong to.
