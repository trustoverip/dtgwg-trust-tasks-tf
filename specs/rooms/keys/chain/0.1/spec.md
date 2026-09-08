---
slug: rooms/keys/chain
version: "0.1"
title: "Rooms Keys — Chain"
summary: "A member hands their key-holding agent the room's epoch key chain, so it can open records sealed before they joined."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong — a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [rooms, keys, chain, a-term-a-searcher-would-use]
#   authors:
#     - Your Name (https://github.com/your-handle)
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request hands the recipient key material to retain and act on. One whose origin depended on the transport would let a compromised channel fill a member's own key holder with rungs of somebody else's choosing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed delivery is harmless — rungs are idempotent by epoch — but an undated one cannot be reasoned about when a member is re-establishing a key holder from more than one source."
sideEffects:
  level: mutating
  rationale: >-
    The recipient retains the rungs. That is the point of the task: they are what lets it open records sealed before its principal joined.
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: >-
    The request carries wrapped key material INTO the recipient — that is what a rung is — so `ingests` is `secret` even though the response discloses only how far back the recipient can now read. The recipient is the member's own key holder, which is the only party this may be sent to.
retention:
  class: durable
  rationale: >-
    The rungs are kept for the life of the room. Discarding them means the recipient can no longer open anything written before its principal joined, and re-fetching them is the only repair.
errorCodes:
  - code: rooms/keys/chain:notAMember
    meaning: "The recipient holds no group state for this room, so there is nothing for the rungs to extend."
    retryable: false
related:
  - rooms/epoch/chain
  - rooms/keys/open
  - rooms/keys/welcome
---

## Abstract

A member hands their **key holder** — the party that holds their room keys and opens records on their agent's behalf — the room's epoch key chain, so that it can open records sealed before they joined.

[`rooms/epoch/chain`](../../../epoch/chain/0.1/spec.md) gets the rungs from the room's host to the member. This gets them the last leg, from the member to the party that will actually use them. The two are not the same delivery and cannot be collapsed: a host is not permitted to speak to a member's key holder, and a key holder does not present the member's credentials to a host.

Without it a member who joins reads the room's history in their own client and their **agent** does not, which is the more confusing half of the same failure: an agent recalling from a room it is plainly a member of, finding nothing older than the day it arrived.

A key holder accrues rungs on its own for every membership change it lives through — each [`rooms/keys/commit`](../../commit/0.1/spec.md) carries one. This task is for the history it did **not** live through.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is being the recipient's own principal.** A key holder holds one member's room keys, and this task extends what it can decrypt — so what authorizes it is not a credential the room issued but the standing relationship between a member and their own key holder. A conforming consumer **MUST** accept this only from the party whose keys it holds, established by the same means it establishes that for every other custody task in this family, and **MUST** refuse it for a room it holds no group state for.

That is deliberately narrower than the grant used to *obtain* the rungs. Fetching them from a host takes a `read` chain the room issued ([`rooms/epoch/chain`](../../../epoch/chain/0.1/spec.md)); handing them onward takes no room credential at all, because the recipient is not being asked to believe anything about the room — it is being handed material it will verify by trying to use it.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome. Here those two happen to coincide, because the entitlement *is* an identity relationship; a consumer must still make the check deliberately rather than treat a valid proof as the answer.

**What a wrongly accepted request costs.** Not disclosure — the sender learns nothing, and the response says only how far back the recipient can read. It costs the recipient's storage, and the risk that its principal's records stop opening if it is fed rungs that displace real ones. That is why a rung already held is never replaced, which is the same rule the host side follows.

## Definitions

**`roomId`** — the room these rungs belong to. The recipient **MUST** already hold group state for it, and **MUST NOT** create any on the strength of this request: a rung is inert without an epoch key, so retaining rungs for a room one is not in is retaining key material for nothing.

**`links`** — the rungs, in any order. Order does not matter because a rung is identified by the epoch it opens under, and the chain is walked on demand rather than on receipt.

**`earliestReadableEpoch`** (response) — the earliest epoch the recipient can now derive a key for, having walked what it holds.

**`stored`** (response) — how many rungs were new to the recipient.

### The response answers a question the request cannot

`earliestReadableEpoch` is not a restatement of what was sent. A rung extends reach only if every rung above it is present too, so a caller that supplied a set with a gap in it learns so here — rather than at the first record that will not open, which reads like corruption.

This is also why the recipient **MUST NOT** reject rungs it cannot currently use. A rung below a gap is not wrong; it is early, and it becomes useful the moment the gap is filled by a later delivery. Refusing it would make the natural repair — fetch more and send again — fail on the attempt that was about to succeed.

Note what the recipient can honestly claim here and a host could not. The host serving [`rooms/epoch/chain`](../../../epoch/chain/0.1/spec.md) can say how many rungs it holds, and nothing about whether they *work*; a member cannot check that claim without walking them. The key holder **is** the party that walks them, so `earliestReadableEpoch` is computed from keys it actually derived.

## Request

A **Member** sends this to their **KeyHolder** — their own key-holding agent, and no one else. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member who has just joined delivers the chain they fetched

Three rungs, taking the key holder from the epoch its Welcome arrived at back to the room's first.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/chain/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "links": [
      { "epoch": 4, "wrapped": "Lp3wXc9TgYw2mQnR4vBk8Q", "nonce": "c1Au9Rn3Zr2tWwS1" },
      { "epoch": 3, "wrapped": "Tf6yNb2VhZx5pRoS7wCl9Q", "nonce": "d2Bv0So4As3uXxT2" },
      { "epoch": 2, "wrapped": "Wq8zMd4XjBy7rTqU9yEn1Q", "nonce": "e3Cw1Tp5Bt4vYyU3" }
    ]
  }
}
```

## Response

The **KeyHolder** responds, using the sub-schema reachable via `$anchor: "response"`.

Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### The whole history is now reachable

`earliestReadableEpoch: 1` is the answer a joining member is looking for. Had one rung been missing, this would name the epoch the walk stopped at instead — and `stored` would still count what arrived.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/chain/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "earliestReadableEpoch": 1,
    "stored": 3
  }
}
```

## Security & Privacy

### Data carried

The request carries **wrapped key material**: each rung is one epoch's storage key sealed under the next. The response carries two integers.

**Who this may be sent to is the whole of its security.** A rung is inert to anyone holding no epoch key, so the chain is safe to fetch from a host and safe to carry over the network — but the recipient here is by definition a party that *does* hold one, and handing it the chain extends what it can decrypt to the room's whole retained history. A member **MUST** send this only to their own key holder. There is no version of this task addressed to somebody else's.

A producer **MUST NOT** put record content, member identifiers, or anything descriptive in `ext`. Everything the room seals is sealed so that exchanges like this one do not disclose it.

### Correlation

The recipient is the member's own infrastructure and already knows which rooms it holds keys for, so this discloses nothing to it that it did not have.

To an observer, the shape is the informative part: this is sent about **once** — on joining, or after restoring a key holder — so its presence says a member has just joined or just recovered. It is worth sending over the same transport as the member's ordinary traffic with their key holder rather than a distinguishable one.

`links` discloses how many epochs the room has had to anyone who sees the request, which is what the room's epoch number already discloses to anyone who can read it.

### Retention

The recipient keeps the rungs for the life of the room; discarding them means it can no longer open anything written before its principal joined, and re-fetching is the only repair. They are exactly as sensitive as the keys they wrap and belong wherever that key holder's other room key material lives.

The member's own copy is not needed once delivered — the response says how far the recipient can now read, which is the only thing the member wanted to know.

### Consent/purpose

The chain is delivered so that a member's agent can read the room its principal is a member of. It confers nothing the member did not already hold: they fetched these rungs with a `read` grant the room issued, and this hands them to the party that acts for them.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
