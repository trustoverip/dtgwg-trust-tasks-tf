---
slug: rooms/epoch/prune
version: "0.1"
title: "Rooms Epoch — Prune"
summary: "A room's owner asks its host to drop the epoch key chain below an epoch, making the history before it unrecoverable."
status: draft
targetFrameworkVersion: "0.5.0"
category: access-control
keywords:
  - room
  - epoch
  - retention
  - forget
  - chain
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request destroys the room's ability to open a span of its own history, for every member, permanently. Authority for that must be verifiable independently of the transport it arrived on.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed prune is not idempotent in the way it looks: a second one arriving after the epoch has advanced names a different span than the first did. Placing it in time is what makes that distinguishable.
sideEffects:
  level: destructive
  rationale: >-
    Rungs are erased. Nothing recovers them — no member retained the outgoing keys and the host never had them — so the history below the pruned epoch becomes unopenable for everyone, including the member who wrote it.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The request names a room and an epoch. The response says how far back the chain still reaches, which the caller could have learned from `rooms/epoch/chain` anyway.
retention:
  class: transient
  rationale: >-
    The task removes state and stores nothing of its own beyond whatever audit entry the host keeps.
errorCodes:
  - code: rooms/epoch/prune:notAuthorized
    meaning: "The presentation does not confer `admin` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/epoch/prune:notAhead
    meaning: "`beforeEpoch` is at or above the room's current epoch, which would drop the chain a member needs to read anything at all. `details.epoch` names the room's current one."
    retryable: false
  - code: rooms/epoch/prune:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
related:
  - rooms/epoch/chain
  - rooms/epoch/mint
  - rooms/create
---

## Abstract

A room's **owner** asks its **host** to drop the epoch key chain below a given epoch. Rungs below it are erased, and the history sealed under those epochs becomes unopenable — for everyone, permanently.

This is the verb behind [`rooms/create`](../../../create/0.1/spec.md)'s `retentionPolicy`. A room that chose `chained` keeps its whole history readable; this is how such a room decides, later and deliberately, that some of it should stop being.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## What is actually destroyed, and what is not

**The rungs, not the records.** A pruned room still holds every record it held before, and a host still serves them. What is gone is the *ability to derive the keys those records were sealed under*, for anyone who has not already derived them.

That distinction matters in both directions and is easy to get wrong on screen:

- A member who **already walked the chain** and holds an old epoch's key keeps it. A key someone has read is a key they have, and nothing here reaches into a member's agent. Pruning withholds *deriving it again* — after a restart, on another device, or on joining.
- A member who **joins afterwards** can never read that span, however much authority they are granted. The host will serve them the ciphertext and no key exists to open it.

So a prune is not a deletion and does not behave like one. It is closer to losing a key than to shredding a document, and an implementation that describes it as "deleting old records" is describing something else.

## Authorization

`admin` at the room's scope. Deliberately not `curate` and not `write`.

Pruning makes no statement about any record — it does not retract, demote or pin anything — so `curate` is the wrong shape as well as the wrong strength. And every member who can write can curate, which would put "end the room's readable history" within reach of every writer.

It sits beside [`rooms/epoch/mint`](../../mint/0.1/spec.md) because it is the same class of act: a decision about the room's key material that every member lives with.

A conforming consumer **MUST** apply the chain-verification, depth and no-dereference rules of [`rooms/records/put`](../../../records/put/0.1/spec.md).

## `beforeEpoch` must be below the room's current epoch

A host **MUST** refuse with `notAhead` where `beforeEpoch` is at or above the room's current epoch.

Pruning to the current epoch would drop every rung, which does not merely shorten the history — it leaves a member who restarts unable to derive **anything below the epoch they are handed next**, which is most of the room. That is a plausible typo (`beforeEpoch: currentEpoch` reads like "keep from here") with a consequence nobody would choose, so it is refused rather than performed.

## The response reports reach, not the request

`earliestRung` is **not** a restatement of `beforeEpoch`, and a host **MUST NOT** treat it as one.

A chain can already have a gap — rungs are delivered per commit, and a delivery that never happened leaves one. A prune below an existing gap changes nothing about how far back a member can actually walk, and echoing the request back would tell an operator they had achieved something they had not.

`pruned: 0` is likewise a **success**: the chain already went back no further. A caller that read it as a failure would retry an operation with nothing left to do.

## Definitions

**`beforeEpoch`** — drop every rung *below* this epoch. The chain walks backwards, so this is a floor rather than a ceiling.

**`reason`** — why, for the room's audit trail. A prune is irreversible and unattributable after the fact — the rungs are simply gone — so the only record of intent is the one made at the time.

**`pruned`** — how many rungs were dropped.

**`earliestRung`** — how far back the chain still reaches. See above.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/epoch/prune/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:host",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "beforeEpoch": 5,
    "reason": "retention: the first four epochs are past the agreed window",
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

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/epoch/prune/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "pruned": 4,
    "earliestRung": 5
  }
}
```

## Security & Privacy

### Data carried

The request names a room, an epoch and a reason. Nothing sensitive crosses; what crosses is an instruction.

A producer **SHOULD** keep `reason` free of member identifiers and record content. It is written to a host's audit log, which on a `private` room is a place the room otherwise puts nothing readable.

### Correlation

To the host, this is an owner deciding about retention. The **timing** is the informative part: a prune shortly after a removal says something about why the removal happened, and a host correlating the two learns more than either alone. An owner for whom that matters should not prune immediately after a membership change.

### Retention

This *is* the retention mechanism, and it is one-way. A host that kept a copy of the pruned rungs would defeat the entire operation while reporting success, so a conforming host **MUST** erase them rather than marking them unavailable.

There is no soft delete here on purpose. A rung retained "just in case" is a rung that can be produced under compulsion, which is exactly the state the owner was trying to leave.

### Consent/purpose

The rungs are dropped so that a room can stop being able to read part of its own past. It is destructive, irreversible, and affects every member — so an implementation whose surface offers it **SHOULD** show what will become unreadable, and **SHOULD NOT** offer it as an incidental control beside reversible ones.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required — though `sideEffects: destructive` is what a consent surface keys off, and this is a strong candidate for one.
