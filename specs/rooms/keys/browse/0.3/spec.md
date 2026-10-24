---
slug: rooms/keys/browse
version: "0.3"
title: "Rooms Keys — Browse"
summary: "A member asks their own agent to list a room's records at its host, to check the listing against what the host committed to, and to say whether the room's records have been pruned."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords:
  - room
  - record
  - list
  - commitment
  - completeness
authors:
  - Glenn Gore (https://github.com/stormer78)
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
    The request makes the recipient act outward in its principal's name — presenting their room credentials to a party this request names. One whose origin depended on the transport would let a compromised channel aim a member's credentials at a host of somebody else's choosing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The response carries a verdict about a host's honesty, and a verdict that cannot be placed in time cannot be compared with a later one — which is the whole mechanism. An undated listing is also one a duplicate-suppression window cannot absorb.
sideEffects:
  level: none
  rationale: >-
    Returns metadata. The recipient makes an outbound request and MAY record what the host asserted, but the room is unchanged.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: true
  rationale: >-
    `actsAsSubject` is the point: the recipient presents its principal's membership and authority to the named host, as them. `discloses: metadata` rather than `secret` because a listing carries no bodies — which is a property of the task and not of the tier, and is why browsing and reading are two tasks.
retention:
  class: transient
  rationale: >-
    The listing is returned and not kept; the presentation minted for the fetch is single-purpose. What the recipient MAY retain is what the host asserted about the room — see `verification.priorRoots`.
errorCodes:
  - code: rooms/keys/browse:notAMember
    meaning: "The recipient holds no group state for this room, so it has nothing to present."
    retryable: false
  - code: rooms/keys/browse:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer."
    retryable: true
  - code: rooms/keys/browse:hostRefused
    meaning: "The host answered and declined. Its own code and reason are carried in `details` — commonly that it does not serve this room."
    retryable: false
related:
  - rooms/records/prune
  - rooms/keys/read
  - rooms/keys/present
  - rooms/records/list
---

## Abstract

A member asks their **key holder** to list a room's records at its host, and to check the listing against what the host committed to.

It is [`rooms/records/list`](../../../records/list/0.1/spec.md) made from the party that can both present and check. The presentation only the agent can mint; the checking needs the room's state and, for the part that matters most, a memory of what this host has said before — which a tab does not have and a CLI does not keep.

Bodies are never returned. That is a property of this task rather than of the tier: [`rooms/keys/read`](../../read/0.1/spec.md) is how a member opens something, and it is also the only shape a trace can be produced for. Browsing and reading are two tasks so that looking at a room's shelf does not decrypt the room.

Version 0.2 passes on the room's `lastPrune`. A prune removes records outright, tombstones
included, so a member's agent syncing a room incrementally through this task has no other way
to learn that records it holds are gone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

One optional response member, `lastPrune` ([`RecordPrune`](../../../_shared/0.2/room.schema.json)),
which a recipient fills from [`rooms/records/list`](../../../records/list/0.3/spec.md) `0.3`. The
response is a closed object, so a `0.1` caller would refuse it; a new member on a closed
payload is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). A recipient
implementing `0.2` **SHOULD** continue to serve `0.1`; answering a `0.1` request carrying
`sinceVersion` for a room it knows has been pruned past it, it **SHOULD** ignore `sinceVersion`
and list the whole room, so that a `0.1` caller is at least handed the full set — though it
cannot be told to drop what is missing from it.

## Changes from 0.2

* The shared `RecordPrune` member (response `lastPrune`) **`at`** → **`prunedAt`**. Same type (`string`, `date-time`, required) and meaning.

The rename brings this member into line with the registry's naming convention — an instant that records an event is named `<event>At` — so the same concept has one spelling across the registry. All of the schema's `$ref`s into `rooms/_shared/0.1/room.schema.json` move to `rooms/_shared/0.2/room.schema.json`, which differs from 0.1 only by this rename; nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is being the recipient's own principal.** A key holder holds one member's room keys; this asks it to use their credentials to look.

A conforming consumer **MUST** accept this only from the party whose keys it holds, **MUST** refuse it for a room it holds no group state for, and **MUST** present a `read` grant scoped to this room and granted to its principal, in `rooms/records/list` documents issued and signed by the principal — never one conferring more. As in [`rooms/keys/read`](../../read/0.1/spec.md), the recipient acts as its principal and its own identifier **MUST NOT** reach the host.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

Everything [`rooms/keys/read`](../../read/0.1/spec.md) says about a caller-named `host` applies unchanged: presenter binding means naming a host transfers no standing, what it transfers is **sight** of the presentation, and a recipient **SHOULD** bound the outbound request to transports the host's DID document advertises.

## The room's prunes

A recipient serving `0.2` **MUST** list with `rooms/records/list` `0.3` where the host serves
it, and **MUST** return `lastPrune` exactly as the host served it, omitting it when the host
did. A caller that keeps its own copy of the room **MUST** apply the reader rules of
[`rooms/records/list`](../../../records/list/0.3/spec.md) `0.3` (*After a prune*) to it: a
`lastPrune.version` above its watermark means rebuild from a browse with no `sinceVersion`,
and drop what that no longer holds.

## What the recipient checks

1. **The reply is signed by the host that was addressed** — verified, and the proven signer bound to the `host` named. An unsigned or misattributed reply is `hostRefused`, never a success with a caveat.
2. **The root against what it has seen before, at this `headVersion`** — `verification.priorRoots`.
3. **The length against the count the host committed to** — `verification.count`, and this is the check that only exists here.
4. **The room's own witnessed anchor**, where it has published one — `verification.anchor`. The only one of these a first-time reader can make, and the only place a **rollback** is visible: a host serving a state older than the room's own published statement is `behind`.

### Counting is the check a listing can actually do

A reader **cannot** recompute a room's root from a listing. A leaf commits to a whole record and a listing returns a projection without the body, so the arithmetic is unavailable however complete the listing is.

What is available is the count. A host that omits a record from a listing while committing to a tree that holds it **contradicts itself inside one exchange** — no second party, no anchor, no other member. That is the omission the commitment exists to make detectable, caught at the cheapest possible moment.

The condition is narrow and a consumer **MUST** respect it. The comparison is meaningful only for a listing with **no `prefix`**, **no `sinceVersion`**, and `complete: true`. Anything else legitimately holds fewer records, and comparing it produces a discrepancy the reader manufactured. `notComparable` is the honest answer and will be the common one.

**`complete` is why a page bound is not a short listing.** A recipient that stopped at a `limit` and a host that withheld a record produce the same shorter array, and a consumer that could not tell them apart would either cry wolf on every paged listing or learn to ignore the one that mattered. So a recipient **MUST** set `complete: false` when it stopped for any reason of its own, and **MUST NOT** report `count: "short"` in that case.

### A verdict is never an error

As on [`rooms/keys/read`](../../read/0.1/spec.md), and for the same reason: **the consequence of catching a host belongs on the write path, not on the member's ability to look at their own room.** A consumer **MUST NOT** treat any value in `verification` as a failed listing, **MUST** surface an adverse one in words that name what was observed, and **MUST NOT** reduce it to a status colour — a member who dismisses an icon here is a member who later reads a refused write as their own agent malfunctioning.

## Definitions

**`roomId`** — the room to list. The recipient **MUST** already hold group state for it.

**`host`** — the host to list from, as a DID. Named by the caller because nothing maps a room to its host; see [`rooms/keys/read`](../../read/0.1/spec.md).

**`prefix`**, **`sinceVersion`** — passed through to the host. Either one makes the listing non-comparable against the room's `recordCount`, which is stated here because a caller setting a prefix has usually not thought about the commitment at all.

**`limit`** — the **page size to ask the host for**, never a cap on the result. A recipient **MUST** follow the host's cursor to the end unless it stopped of its own accord, in which case `complete` is `false`. A short page is not the end of a listing, and a producer that infers exhaustion from one has invented a shorter room.

**`complete`** (response) — whether the recipient reached the end. See above; it is what separates a page bound from a withheld record.

## Request

A **Member** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member looks at a room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.3#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community"
  }
}
```

## Response

The **KeyHolder** responds, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### A complete listing that reconciles

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.3#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "records": [
      {
        "key": "giXFLTGBdnnQJRoIsktuIg",
        "version": 412,
        "epoch": 7,
        "status": "active",
        "updatedAt": "2026-01-01T00:00:00Z"
      }
    ],
    "complete": true,
    "verification": {
      "priorRoots": "agree",
      "count": "agrees",
      "head": {
        "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "recordCount": 1,
        "headVersion": 412
      }
    }
  }
}
```

### A host that served fewer records than it committed to

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.3#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "records": [],
    "complete": true,
    "verification": {
      "priorRoots": "noneHeld",
      "count": "short",
      "head": {
        "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "recordCount": 118,
        "headVersion": 412
      }
    }
  }
}
```

The listing is still returned, and `priorRoots: "noneHeld"` shows this was the first read — the host was caught **without** the agent having any history at all, which is the point of carrying a count.

### A room whose records have been pruned

The host removed fourteen records at version 413. A caller whose copy of the room was
synchronised below 413 rebuilds it.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.3#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-02T00:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "records": [
      {
        "key": "giXFLTGBdnnQJRoIsktuIg",
        "version": 412,
        "epoch": 7,
        "status": "active",
        "updatedAt": "2026-09-30T00:00:00Z"
      }
    ],
    "complete": true,
    "lastPrune": { "version": 413, "prunedAt": "2026-10-01T00:00:05Z", "removed": 14 },
    "verification": {
      "priorRoots": "noneHeld",
      "count": "agrees",
      "head": {
        "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "recordCount": 1,
        "headVersion": 413
      }
    }
  }
}
```

`headVersion` is the prune's version, not the highest record's: a prune takes a version like
any other change to the tree (`HeadVersion`).

## Security & Privacy

### Data carried

The request carries identifiers and a query. What crosses the network *because of it* is the recipient presenting its principal's membership and authority to the named host.

The response carries **metadata, never bodies**. On the sealed tiers the keys are opaque by construction, so a listing discloses how much the room holds and when it moved rather than what it says. A producer **MUST NOT** put record content in `ext`.

### Correlation

To the host, this is a member of that room surveying it — which is what the presentation already said. The event is *who has seen what this room holds*, and it is the event a host **SHOULD** log with a retention period of its own rather than one inherited.

Browsing is cheaper to repeat than reading and therefore easier to turn into a poll. A recipient that polls converts an access pattern into a timeline held by the host, so a consumer **SHOULD NOT** poll.

### Retention

The listing is returned and not kept; the presentation minted for the fetch is single-purpose.

What a recipient **MAY** retain, and what `verification.priorRoots` depends on, is `(headVersion, dataCommitment)` — a few dozen bytes per observation. It carries no record content, but it is a record of *when this member looked at this room*, so a recipient **SHOULD** keep it bounded and **SHOULD NOT** keep it beyond the member's membership.

### Consent/purpose

The listing is fetched so that a member's agent can show them the room its principal is a member of. It confers nothing the member did not already hold.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
