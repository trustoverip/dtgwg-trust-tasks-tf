---
slug: rooms/epoch/commits
version: "0.1"
title: "Rooms Epoch — Commits"
summary: "A member fetches the MLS commits a room made while they were away, so they can catch up without the owner sending to each of them."
status: draft
targetFrameworkVersion: "0.5.0"
category: access-control
keywords:
  - room
  - epoch
  - commit
  - mls
  - catch-up
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "A member presents an authority chain to fetch these; the presentation authorizing it must be verifiable independently of transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A read presentation is time-bounded, and accepting a stale one would extend access past the window its credentials describe."
sideEffects:
  level: none
  rationale: "Returns commits the host already holds. A member applying them changes their own group state and nothing at the host."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns MLS commits, which are opaque to anyone without the group state. The host learns that a member is catching up from some epoch."
retention:
  class: transient
  rationale: "The task returns existing state and stores nothing of its own."
errorCodes:
  - code: rooms/epoch/commits:notAuthorized
    meaning: "The presentation does not confer `read` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/epoch/commits:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
related:
  - rooms/epoch/mint
  - rooms/epoch/chain
  - rooms/keys/commit
---

## Abstract

A **member** asks a room's **host** for the MLS commits the room made while they were away.

Every epoch change is a commit, and every member has to apply it or fall out of the group. Without somewhere to fetch them from, the only way a commit reaches a member is the owner sending it — **O(n) deliveries per renewal**, to parties the owner must have addresses for, all of which have to be online or have inboxes. A room whose members are people with browsers does not have that.

So the host holds them and members fetch. This is the delivery half of [`rooms/epoch/mint`](../../mint/0.1/spec.md)'s `commit`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Why a host may relay a commit when it may not relay a Welcome

This family deliberately keeps the host off the Welcome path: a Welcome names the party joining, and on a `private` room the membership is the thing the tier withholds. It would be reasonable to assume commits inherit that rule. **They do not, and the difference is worth stating rather than leaving to be re-derived.**

A commit is ciphertext plus a leaf index. It **names nobody**. A host relaying commits learns that the room moved, which it already knew from `epoch` on the room row, and that some member is behind — which is what fetching anything already tells it.

So this is authorised the same way a record read is: a `read` chain, and on a `private` room **no host session**, because authorising by session would hand the host a member identifier on every catch-up and a period of those logs reconstructs the membership.

## Ordered, and with no gaps

MLS commits apply **in sequence**. One applied out of order, or over a gap, is rejected by the group — so a host that returned an unordered or sparse set would produce a member who cannot catch up and cannot say why.

A host **MUST** return the commits above `sinceEpoch` in ascending epoch order. Where it is **missing** one, it **MUST** return the run it holds *up to* the gap and stop, rather than skipping past it. A short answer is recoverable; a set with a hole in it is a member stuck at an epoch with no explanation.

## `roomEpoch` is not arithmetic

It is deliberately **not** `sinceEpoch` plus the number of commits returned, and a consumer **MUST NOT** compute it that way.

The two differ whenever the host is missing a commit or the page ended early, and **that difference is the useful part**: a member who applies everything served and finds themselves still behind knows a delivery is missing, rather than concluding their own group state is broken. That is the distinction [`rooms/keys/list`](../../../keys/list/0.1/spec.md) already draws between "a commit was not delivered" and "the key chain has not arrived", and it needs the same care.

## What this does not do

**It does not make a commit reach an offline member.** It makes one *fetchable*, which is a different and smaller promise: a member who never comes back never catches up, and a room that removes them is doing the right thing.

**It is not a substitute for the epoch key chain.** A commit moves a member forward; a rung ([`rooms/epoch/chain`](../../chain/0.1/spec.md)) lets them read backwards. A member who applies every commit and fetches no rungs can write to the room and read nothing written before they arrived.

## Definitions

**`sinceEpoch`** — the epoch this member is at. The host returns what is above it. A member asks for what they are missing rather than for a count, because only they know where they are — a host that guessed would be guessing from the last time it saw them, which on a `private` room is a thing it should not be tracking.

**`limit`** — page size. A member far behind reads to the end by asking again from the epoch they reached.

**`roomEpoch`** — where the room is now. See above.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/epoch/commits/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:host",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "sinceEpoch": 5,
    "presentation": {
      "membership": "urn:uuid:11111111-1111-1111-1111-111111111111",
      "authority": [
        "urn:uuid:22222222-2222-2222-2222-222222222222",
        "urn:uuid:33333333-3333-3333-3333-333333333333"
      ]
    }
  }
}
```

## Response

Two commits served, and the room is further ahead than they reach — so a delivery is missing and the member is told rather than left to infer it.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/epoch/commits/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "commits": [
      { "epoch": 6, "commit": "AAECAwQFBgcICQoLDA0ODw" },
      { "epoch": 7, "commit": "EBESExQVFhcYGRobHB0eHw" }
    ],
    "roomEpoch": 9
  }
}
```

## Security & Privacy

### Data carried

MLS commits, which are opaque to anyone without the group state. A member who is not in the group learns nothing from one, which is what makes relaying them through a host acceptable at all.

A producer **MUST NOT** put member identifiers or record content in `ext`.

### Correlation

The host learns that a member is catching up, and from roughly where. `sinceEpoch` is the informative part: a member fetching from epoch 1 has just restored an agent or joined, and one fetching from the epoch before last was briefly away. Neither names them, and on `open` and `attributed` the presentation does that anyway.

A member for whom the pattern matters can ask from further back than they need — the extra commits are ones they can already apply, and asking for them costs bandwidth rather than security.

### Retention

The task stores nothing. What the **host** retains is `rooms/epoch/mint`'s concern: a commit kept forever lets a member restore from any point, and a commit dropped early makes a long-absent member unrecoverable. That is the same trade `RetentionPolicy` already makes for rungs and should be made the same way.

### Consent/purpose

Commits are fetched so that a member's group state can catch up with the room they are a member of. It confers nothing they did not already hold, and a party without the group state gains nothing from holding one.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
