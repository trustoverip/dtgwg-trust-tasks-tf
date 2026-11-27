# auth/oob — family conventions

This document records the rules shared by every specification under
`auth/oob/`. Where it could conflict with a specific spec, **that spec's own
front matter and payload schema are authoritative.** How a starter hands a
request to an approver — the `sign-in` trigger link — is in
[`TRIGGER-LINK.md`](TRIGGER-LINK.md).

## 1. Roles and tasks

A **starter** (for `login`, a browser on the portal page) asks a **service**
(for `login`, the community's VTC) to open a request. An **approver** (a member,
using a wallet and their VTA) decides it on another device. For `deviceEnrol`
the starter is a client that holds its own keys — an AI agent, a command-line
tool, an app or a device — the service is the person's own VTA, and the
approver is the person's wallet, acting as an administrator of that VTA. The family has six
tasks a party sends, and two DID-signed documents that are only ever carried
inside another task:

| Task | From → to | Signed by | Carries |
|---|---|---|---|
| [`auth/oob/request`](../../request/0.3/spec.md) | starter → service | starter key `K_b` | — |
| [`auth/oob/claim`](../../claim/0.3/spec.md) | approver → service | lock key `K_a` | — |
| [`auth/oob/prove`](../../prove/0.4/spec.md) | approver → service | `K_a` | `identify` |
| [`auth/oob/identify`](../../identify/0.2/spec.md) | (carried) | approving DID, `authentication` | — |
| [`auth/oob/respond`](../../respond/0.4/spec.md) | approver → service | `K_a` | `grant` |
| [`auth/oob/grant`](../../grant/0.4/spec.md) | (carried) | approving DID, `assertionMethod` | — |
| [`auth/oob/redeem`](../../redeem/0.3/spec.md) | starter → service | `K_b` | — |
| [`auth/oob/cancel`](../../cancel/0.1/spec.md) | starter or approver → service | `K_b` or `K_a` | — |

Carrying a signed document inside another follows the precedent of
[`auth/signing-key/authorize`](../../../signing-key/authorize/0.1/spec.md)
inside `auth/signing-key/enroll`.

## 2. Shared schema components

[`oob.schema.json`](oob.schema.json) defines `Purpose` (named `OobPurpose` from 0.2), `Mode`, `RequestId`,
`OobKey`, `MatchNumber`, `Origin`, `Service`, `SameNetwork`, `Requester`,
`TaskFamily` and `SigningScope`, and from 0.2 `EnrolClient`, `RequestedScope`
and `AclScope` for `deviceEnrol`, referenced by every task in the family
([SPEC §6.6](/SPEC.md#66-shared-schema-components)).

## 3. Closed lists

**`purpose` is a closed list.** Each purpose has a fixed row below covering
what it reveals at each step, what the grant contains, what `redeem` returns and
the window lengths. A service refuses any other value with
`auth/oob/request:purposeUnsupported`. Adding a purpose is a new version of
`oob.schema.json` and of the tasks that reference it.

| Purpose | Status | Step 1 (claim) reveals | Step 2 (prove) reveals | Grant authorizes | `redeem` returns | Windows |
|---|---|---|---|---|---|---|
| `login` | defined | service, portal origin, purpose, decision deadline | step 1 plus `sessionKey`, `requester`, `identifiedAs` | `sessionKey` acts as the approving DID at `origin` until `sessionEndsAt`; from grant 0.2, and only where step 2 offered a `signingScope`, it also signs documents of those task families as that DID for the session | a `Session` for the DID, bound to the starter key | claim ≤ 180 s, decision ≤ 180 s; 120 s each RECOMMENDED |
| `deviceEnrol` | defined (from 0.2) | service (the VTA), the VTA's Trust Task endpoint origin, purpose, decision deadline | step 1 plus `sessionKey`, `requester`, `identifiedAs`, and the `client` (its self-asserted kind and name) and the `requestedScope` it asks for | the VTA creates an ACL entry whose subject is `sessionKey`, with the `aclScope` the approver chose within `requestedScope`, until its `expiresAt`; the approver must be an administrator of the VTA ([§7](#7-pending-confirmation) A7) | the VTA's DID, the ACL entry it created, and `swapClosesAt`, by which the client must swap `sessionKey` for its own DID | as `login`, with a four-digit match number (A12); swap ≤ 300 s after redeem, 120 s RECOMMENDED ([§7](#7-pending-confirmation) A3) |
| `stepUp` | reserved | — | — | — | — | — |

**`mode` is a closed list.** A service refuses any other value with
`auth/oob/request:modeUnsupported`.

| Mode | Status | Meaning |
|---|---|---|
| `scan` | defined | The starter shows the `sign-in` trigger link as a QR code that is also a link ([`TRIGGER-LINK.md`](TRIGGER-LINK.md)). |
| `code` | reserved | A typed code of at least 8 characters, under strict rate limits. |
| `push` | reserved | Sent to the approver's enrolled devices. |

**What `deviceEnrol` is for.** A client that holds its own key asks for
scoped, long-lived access to a person's VTA, and the person approves it from
their wallet with the same QR code, number match and user verification as
`login`. `login` cannot do this: it arms a short session in which the starter
key acts as the whole person. On approval the VTA itself writes the ACL entry
for the client's key — the approver never sends `acl/grant` itself: its VTA
signs the grant on user verification, and the VTA, as service, writes the
entry — and `redeem` hands the client the entry. The client then sends its own
[`acl/swap-key`](../../../../acl/swap-key/0.1/spec.md) to replace the starter
key with its own DID, and its own
[`device/register`](../../../../device/register/0.2/spec.md); neither is part
of this family. Listing and revoking enrolled clients use the existing
`device/*` and `acl/*` tasks.

Reserved names are spelled in lowerCamelCase, the registry's casing for
enumerated values ([SPEC §4.10](/SPEC.md#410-naming-conventions)).

## 4. Request state

A service keeps one record per request:
`pending → claimed → identified → approved → consumed`. `declined`,
`cancelled` and `expired` are final. Every change of state is a
compare-and-set, so only one caller can make it.

- A request must be claimed by the time its **claim window** closes (`claimClosesAt`). A
  successful claim starts a fresh **decision window**, closing at `decisionClosesAt`,
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
1a. **The `deviceEnrol` exception to rule 1.** For `deviceEnrol` the starter
   key outlives the request: once `redeem` succeeds it stays the subject of the
   ACL entry the VTA created, until the client swaps it for its own DID with
   `acl/swap-key`, which it **MUST** do before `swapClosesAt`. Until then the entry
   **MUST** authorize the starter key for nothing but that swap — the key still
   signs this family's own `redeem` and `cancel` for the request, as rule 1
   allows, and nothing else — and the VTA **MUST**
   remove the entry if no swap has happened by `swapClosesAt` ([§7](#7-pending-confirmation) A5). The
   lock key is unaffected: it is used for one request only. Rule 3 still holds.
2. **Disclosure ladder.** Nothing about the starter is revealed before a
   successful `prove`. Step 1 (the `claim` response) names only the service, the
   origin, the purpose and the deadline. For `deviceEnrol`, the client and the
   scope it asks for are revealed at step 2 too, never at step 1. In the other
   direction, after `redeem` the client learns the approving administrator's
   DID: the entry it receives names it as `delegatedBy` and `createdBy`.
3. **Authority goes to the starter key, through `redeem` only.** Responses to the
   approver never carry session material.
3a. **Delegated signing is a scope the member signs, and ends with the session.**
   From [`auth/oob/prove/0.2`](../../prove/0.2/spec.md),
   [`auth/oob/grant/0.2`](../../grant/0.2/spec.md) and
   [`auth/oob/respond/0.2`](../../respond/0.2/spec.md), a service may offer in
   step 2 the task families the starter key would sign documents for as the
   member; the grant copies the offer, and the service then accepts the key's
   documents of those families as the member's for the session `redeem`
   creates, within the exclusions `auth/oob/grant/0.2` states. Prove, grant and
   respond are versioned together: a 0.2 grant travels in a 0.2 respond and
   copies a 0.2 step 2, and a 0.3 grant travels in a 0.3 respond and copies a
   0.3 step 2. A signing scope is `login` only.
3b. **An enrolment scope is a ceiling the client asks for and a scope the
   approver chooses.** From [`auth/oob/request/0.3`](../../request/0.3/spec.md),
   a `deviceEnrol` request names a `requestedScope` — the most capabilities,
   purposes and keys the client asks for, and the longest it asks to be enrolled.
   Step 2 repeats it, and the grant carries an `aclScope` the approver chose:
   the contexts, role, capabilities, purposes, keys and expiry of the entry. The
   service refuses a grant whose scope exceeds the ceiling. This is the reverse
   of rule 3a, where the grant copies the service's offer: here the approver
   picks and the service enforces the bound. The ceiling has no `role`: the
   approver chooses the entry's role, and that is no path to escalation,
   because a role is itself a ceiling on capabilities in the ACL family and
   the approver can confer none it could not write itself. The grant's issuer
   **MUST** be an administrator of the VTA — a DID that could write that entry
   itself — and the entry is delegated by it and never wider than its own
   authority ([§7](#7-pending-confirmation) A7). Prove 0.4, grant 0.4 and
   respond 0.4 are versioned together.
3c. **A request is handled only by task versions whose purpose list contains
   its purpose.** This binds a service that implements these versions. The
   earlier versions are unchanged: a service that implements only them never
   holds a `deviceEnrol` request. Such a service **MUST** refuse any
   earlier-version family task for a `deviceEnrol` request — request 0.2,
   claim 0.2, prove 0.3, respond 0.3 and redeem 0.2, or before — with
   `auth/oob/request:purposeUnsupported` at `request` (or the framework's
   payload-validation refusal, `malformedRequest`, which comes first: the
   closed purpose list of request 0.2 and before cannot carry `deviceEnrol`),
   and elsewhere with `auth/oob:requestNotFound` where that version declares
   it and `auth/oob:notAuthorized` where it does not; and it **MUST NOT**
   answer a `deviceEnrol` request with a `login`-shaped response. At `prove`
   and `respond`, which only the lock holder can send, the refusal also moves
   the request to `declined`; at `claim` and `redeem` the request's state is
   unchanged. A service that serves both old and new versions applies this
   per request, from the purpose the request was opened with.
4. **Proof purposes.** The starter and lock keys sign with `authentication`.
   `identify` is signed by the approving DID for `authentication`. `grant` is
   an attestation and is signed for `assertionMethod`. The service's `claim` and
   `prove` responses are attestations the approver relies on (and the grant
   cites by digest), and are signed with the service's `assertionMethod` key
   ([SPEC §4.7.3](/SPEC.md#473-proof-purpose-and-verification-relationship)).
5. **Times.** `claimClosesAt` (request), `decisionClosesAt` (claim and prove)
   and `sessionEndsAt` (grant and redeem) are RFC 3339 date-time strings. The
   trigger link's `_exp` stays integer UTC epoch seconds, as VTI-LNK defines it,
   and is computed from `claimClosesAt` when the link is built. The earlier
   versions — request 0.1, claim 0.1, grant 0.1 and 0.2, prove 0.1 and 0.2,
   redeem 0.1 — carried these instants as integer UTC epoch seconds under the
   names `claimDeadline`, `decisionDeadline` and `notAfter`. For
   `deviceEnrol`, the entry's `expiresAt` (grant and redeem) and `swapClosesAt`
   (redeem) are RFC 3339 date-time strings too.
6. **No caching.** Every response, including every error, is sent with
   `Cache-Control: no-store` where the transport has such a header.
7. **Generic failures.** A non-member and a bad signature at `prove` or
   `respond` get the same `auth/oob:notAuthorized`, so the family cannot be used
   to probe membership. For `deviceEnrol`, an identity that is not an
   administrator of the VTA gets the same code, so it cannot be used to probe
   who administers a VTA.
8. **Rate limits.** A service applies per-network limits on `request`, `claim`
   and `prove`, caps pending requests per network, and allows one open `redeem`
   poll per request. Refusals use `auth/oob:rateLimited`. A VTA serving
   `deviceEnrol` exposes this unauthenticated surface where it had none, so
   for that purpose the limits are a **MUST**, and the VTA **MUST** also cap
   the unclaimed `deviceEnrol` requests pending at once across all networks
   ([§7](#7-pending-confirmation) A4). That global cap is a trade-off: it
   bounds the unclaimed requests an attacker can make the VTA hold — claimed
   ones, including those an attacker claims itself with a throwaway lock key,
   are bounded by the per-network limits on `request` and `claim` and by the
   decision window, not by the global cap — but an attacker who
   keeps it full from many networks denies enrolment to the owner until the
   requests expire. A low per-network cap, a global cap sized well above it,
   not counting claimed requests, and an administrator's view of pending
   requests keep that denial short and visible.

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

Task-specific codes (`purposeUnsupported`, `modeUnsupported`,
`clientKindUnsupported`, `scopeUnsupported`, `alreadyClaimed`,
`numberMismatch`, `contextMismatch`, `scopeExceedsRequest`, `pending`,
`declined`) are declared on the task they belong to.

**Refusals a `deviceEnrol` client sees.** At `request`:
`purposeUnsupported` (the service is not a VTA serving `deviceEnrol`, or the
request was sent as an earlier version), `clientKindUnsupported`,
`scopeUnsupported`, `keyUnsupported` and `rateLimited`. At `redeem`:
`pending`, `declined` (`details.state` `declined`
when the person declined, the number did not match, the scope exceeded the
request, or the approver's authority failed the re-check at redeem;
`cancelled` when cancelled), `requestExpired`, `requestNotFound` (no such
request, or a poll sent as redeem 0.2, rule 3c), `notStarter` and
`rateLimited`. The reason a
request was declined is not told to the client, as for `login`.

## 7. Pending confirmation

The `deviceEnrol` purpose was drafted from a design review, and the design
leaves the following points open. This version takes the most conservative
reading stated for each, and will follow the family authors' answer when they
give one. Each later point is a tightening a new version can relax additively.

- **A1 — one context per enrolment.** `aclScope.contexts` is a list of
  exactly one context (`maxItems: 1`), never `all`. Several contexts would
  be a loosened bound in a later version.
- **A2 — no contacts in this version.** The approver does not sign into the
  grant the DIDs the client is to talk to (an owner persona, the wallet's chat
  DID, its mediator), and `redeem` returns none. The `redeem` response is not
  signed, so the client could not check that such identifiers came from the
  approver; they are deferred to a follow-up (keyring plan §11 Q7 (a), below).
- **A3 — windows.** The claim and decision windows are `login`'s
  (≤ 180 s each, 120 s RECOMMENDED). The swap window is at most 300 s after
  `redeem` (120 s RECOMMENDED). An enrolment lasts at most 365 days
  (`maxLifetimeDays`), counted from the grant's `issuedAt`; a wallet
  **SHOULD** offer 30 days by default.
- **A4 — rate limits on `request`.** For `deviceEnrol` the rule 8 limits are a
  **MUST**. A VTA **SHOULD** allow no more than 1 unclaimed `deviceEnrol`
  request pending per network and no more than 10 per network per hour, and
  **SHOULD** size its global cap on unclaimed pending requests well above the
  per-network one (at least 10). A request stops counting against the global
  cap once it is claimed, so the owner's own claimed request cannot be pushed
  out. The same holds for a request an attacker claims itself: claimed
  requests are bounded only by the per-network limits on `request` and
  `claim` and by the decision window, and a VTA **MAY** also cap claimed
  pending `deviceEnrol` requests. A VTA **SHOULD** let an administrator see and cancel pending
  `deviceEnrol` requests. No task in this family does that —
  [`auth/oob/cancel`](../../cancel/0.1/spec.md) is sent by the starter or the
  lock holder only — so a VTA provides it by its own means for now (see the
  open questions below). The exact numbers are its policy.
- **A5 — what the starter key may do before the swap.** Under the entry, only
  `acl/swap-key` of that entry; outside it, only this family's `redeem` (to
  receive the enrolment again) and `cancel`. The entry's scope is exercised by the swapped-in DID, never
  by the starter key, and an entry not swapped by `swapClosesAt` is removed.
  For that swap the VTA **SHOULD** require `linkProof`, so the entry lands on a
  key the client controls.
- **A6 — the origin for `deviceEnrol`.** The origin of the `serviceEndpoint`
  of the VTA's `TrustTaskHTTPS` service, the endpoint the client opened the
  request at. A VTA that has no such service cannot serve `deviceEnrol` in
  this version (see the open questions below).
- **A7 — who may approve (settled).** An administrator, for `deviceEnrol`, is
  a DID that could write the entry by
  [`acl/grant/0.3`](../../../../acl/grant/0.3/spec.md) under the ACL
  family's existing delegation bounds
  ([ACL CONVENTIONS §9](../../../../acl/_shared/0.3/CONVENTIONS.md#9-bounds-on-the-granter-and-delegation)):
  the subject of a live entry at the VTA that may write ACL entries in the
  chosen context, and that the entry would not exceed on any axis of that
  table. The entry the VTA writes carries `delegatedBy` and `createdBy` set to
  that DID, is re-evaluated when the approver's own entry narrows, and never
  carries `approve` or `approveCapabilities` authority.
- **A8 — capability shapes.** `aclScope.capabilities` is `none` or `listed`;
  `ceiling` and `additive` grants are refused, so the approver, not the role,
  decides every capability. The schema refuses both.
- **A9 — the client's kind.** `aclScope.consumerKind` must equal the kind the
  request named. The ACL entry has no such member: the VTA keeps it beside the
  entry, and **SHOULD** refuse a later `device/register` for the entry that
  names another kind.
- **A10 — no `extends`.** Only respond 0.4 passes the build's `extends`
  check (`npm run extends-candidates`). The other new versions cannot declare
  it, because a required member becomes conditional (grant, redeem) or a
  closed value set widens (request, claim, prove, identify). Respond 0.4 does
  not declare it, so that a consumer that implements only `login` 0.3 never
  routes a `deviceEnrol` grant to a `login` handler, and so that the version
  rule (§5 rule 3c) stays simple. A later version could revisit this.
- **A11 — `push` stays reserved.** Enrolment with no QR code needs
  push-fatigue rules this version does not write.
- **A12 — four digits for `deviceEnrol`.** A link sent without its number
  gives a person who guesses a 1-in-100 chance per request of matching a
  two-digit number, too weak for access that lasts days; a live relay that
  forwards the real number is not stopped by any length, and is what the
  `requester` signals are for. This version proposes four digits for
  `deviceEnrol` (`MatchNumber` 0.2 allows two or four; the purpose fixes
  which); `login` keeps two. **An explicit ask of the family authors:**
  confirm four digits, or name another length.
- **A13 — no confirmation at the client.** The client asks its own user
  nothing: there is no approve-or-refuse prompt at the starter. Only an
  administrator of the VTA can approve, and the number match is that person's
  check. Nothing in the family requires the starter to confirm.
- **A14 — no `keys: all` for `deviceEnrol`.** `keys` is `none` or `listed`,
  never `all`, in both `RequestedScope` and `AclScope`; the schema refuses
  `all`. A self-asserted program never reaches the signing oracle on every key
  in its context. `purposes: any` is allowed only where the wallet shows it to
  the person as any purpose.

Open questions:

- **Keyring plan §11 Q7 (a) — contacts.** Deferred to a follow-up (A2). The
  follow-up would need a VTA-signed `redeem` response, so that the client can
  check that the identifiers came from the approver's grant.
- **Q7 (b) — a credential and its acceptance.** Not part of this family; a
  credential-exchange task after enrolment is the likely route.
- **Q7 (c) — the wallet learning of the swap.** The wallet can read the entry
  with `acl/show`, `acl/list` or `device/list`; whether the VTA also notifies
  the approver is open.
- **Q7 (d) — refusal codes** are settled by §6.
- **Which family version a service speaks.** The trigger link names no
  version. A wallet sends the newest `claim` version it implements and, on
  `unsupportedType` or `unsupportedVersion`, retries with the next older one.
  A `deviceEnrol` request needs `claim` 0.3 or later; an older claim of one is
  refused (rule 3c), so a wallet that has only 0.2 cannot approve it. A
  version hint in the link is an alternative the family authors may prefer.
- **`sameNetwork: false` for cloud agents.** A legitimate AI agent often runs
  in the cloud, so for `deviceEnrol` `sameNetwork` is often `false` and the
  warning [`auth/oob/prove/0.4`](../../prove/0.4/spec.md) approver item 5
  requires will fire on most genuine enrolments. The warning stays mandatory
  in this version; a wallet **MAY** let the person say the agent runs
  remotely and then show the signal as expected rather than as an alarm.
  The wallet copy is open.
- **DIDComm-only VTAs (A6).** The `origin`, the trigger link and the claim
  endpoint all rest on the VTA's `TrustTaskHTTPS` service, so a VTA reachable
  only over DIDComm, through a mediator, cannot serve `deviceEnrol` here.
  Whether `origin` may instead be the VTA DID's own service endpoint is open.
- **An administrator's view of pending requests (A4).** No existing task lists
  or cancels another party's pending `auth/oob` requests. Whether a task for
  that is defined, and in which family, is open.
