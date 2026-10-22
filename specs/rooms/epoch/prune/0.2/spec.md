---
slug: rooms/epoch/prune
version: "0.2"
title: "Rooms Epoch — Prune"
summary: "A room's owner asks its host to drop the epoch key chain below an epoch, making the history before it unrecoverable — and the host keeps the resulting floor, so members' key holders forget what they hold below it."
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
    Rungs are erased at the host, which never had the keys they wrap, so nobody who had not already derived a key below the floor can derive it again. Members' key holders that did derive them discard them on learning the floor; what a member already read or decrypted is not recalled.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The request names a room and an epoch. The response says how far back the chain still reaches and the room's epoch floor, both of which any member can learn from `rooms/epoch/chain` 0.2 anyway.
retention:
  class: durable
  rationale: >-
    The task removes rungs, and keeps one integer in their place: the room's epoch floor, for the life of the room, because it is what tells every member's key holder what to discard and what never to accept again. Beyond that, whatever audit entry the host keeps.
errorCodes:
  - code: rooms/epoch/prune:notAhead
    meaning: "`beforeEpoch` is at or above the room's current epoch, which would drop the chain a member needs to read anything at all. `details.epoch` names the room's current one."
    retryable: false
  - code: rooms/epoch/prune:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
related:
  - rooms/epoch/chain
  - rooms/keys/chain
  - rooms/keys/backfill
  - rooms/epoch/mint
  - rooms/create
---

## Abstract

A room's **owner** asks its **host** to drop the epoch key chain below a given epoch. Rungs below it are erased, and the history sealed under those epochs becomes unopenable — for everyone, permanently.

This is the verb behind [`rooms/create`](../../../create/0.1/spec.md)'s `retentionPolicy`. A room that chose `chained` keeps its whole history readable; this is how such a room decides, later and deliberately, that some of it should stop being.

Version 0.2 makes the prune **reach members**. Each member's key holder keeps its own copy of
the chain — that is what lets it open old records without asking anybody — so erasing the
host's copy alone left every key holder able to derive what the owner had decided nobody
should. A 0.2 host keeps the room's **epoch floor** and serves it
([`rooms/epoch/chain`](../../chain/0.2/spec.md) 0.2), and a key holder that learns it discards
everything below it and refuses to derive there again ([`EpochFloor`](../../../_shared/0.1/room.schema.json)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

- The host **MUST** keep the room's epoch floor, raise it at every prune, and serve it — here,
  in the new required response member `epochFloor`, and in `rooms/epoch/chain` 0.2.
- The host **MUST NOT** store, accept or serve a rung at or below the floor afterwards.
- `beforeEpoch` keeps exactly its 0.1 meaning; its relation to what stays readable — `N − 1`
  — is now stated, because 0.1 left it to be inferred and inferred wrongly in two places
  (below).
- *What is actually destroyed* now says what reaches members and what does not.

A new required response member and new host obligations are not editorial, so this is a
`MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). A host implementing 0.2
**SHOULD** continue to answer 0.1, and **MUST** raise the floor for a prune it performs under
either version — the floor is a property of the room, not of the request that moved it.

### Two corrections to 0.1's prose

0.1's sideEffects rationale said that "no member retained the outgoing keys". Members' key
holders do retain them: every commit a key holder applies leaves it a rung, and
[`rooms/keys/chain`](../../../keys/chain/0.1/spec.md) and
[`rooms/keys/backfill`](../../../keys/backfill/0.1/spec.md) exist to give it the rest. Erasing
the host's copy denied the history only to parties that had never derived it.

0.1 also said that `beforeEpoch` at the current epoch "would drop every rung". It would drop
every rung **below** the current epoch, and rung `N` — sealed under the current key, yielding
`N − 1` — would survive. The refusal (`notAhead`) is kept for the reason it gives, which is
real: such a prune leaves a restarted member nothing to read below the epoch it is handed next.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A presentation that does not confer `admin` at the room's scope, or whose chain does not reach the room, is refused with the framework's standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)). This specification declares no task-specific code for it.

## What is actually destroyed, and what is not

**The rungs, not the records.** A pruned room still holds every record it held before, and a host still serves them. What is gone is the *ability to derive the keys those records were sealed under*, for anyone who has not already derived them.

That distinction matters in both directions and is easy to get wrong on screen:

- A member's **key holder** that already walked the chain discards what it holds below the floor when it learns of it ([`EpochFloor`](../../../_shared/0.1/room.schema.json)): its rungs, the storage keys and file keys it derived or cached, and whatever it derived in passing while catching up through those epochs. It refuses to derive there again, and never accepts such material back — from a backfill, a repair, its principal, or a mirror that has not been pruned. It learns the floor the next time it reads the chain, which a conforming key holder does whenever it catches up and at least as often as it otherwise reads the room.
- What a member **already read** is not recalled. Plaintext a key holder returned, records a member opened, files downloaded and decrypted, exports, and keys a member's own software extracted are beyond any host and any floor. A key someone has read is a key they have. Pruning withholds *deriving it again* — after a restart, on another device, by an agent's next recall, or on joining.
- A key holder that does not conform keeps what it held. The floor is honoured, not enforced; what is guaranteed cryptographically is that a party that had not derived a key below it before the host erased its rungs never can.
- A member who **joins afterwards** can never read that span, however much authority they are granted. The host will serve them the ciphertext and no key exists to open it.

### The floor, and the off-by-one

A rung numbered `e` opens with epoch `e`'s key and yields epoch `e − 1`'s. `beforeEpoch: N`
drops the rungs numbered below `N` — as 0.1 did — so rung `N` survives and still yields
`N − 1`. The earliest epoch anybody can derive afterwards is `N − 1`: that is the room's
**epoch floor**, and the host **MUST** raise its floor to the higher of the one it held and
`N − 1`, never lower it, and report it as `epochFloor`.

| `beforeEpoch` | Rungs dropped | Earliest readable epoch (`epochFloor`) |
|---|---|---|
| 2 | none (no rung is numbered below 2) | 1 |
| 5 | 2, 3, 4 | 4 |
| `E + 1` | 2 … `E` | `E` |

To make history sealed under epochs before `E` unreadable, send `beforeEpoch: E + 1`.

A host **MUST** raise the floor even when it drops no rung — a chain with a gap, or a `fromJoin`
room, which produces none. On a `fromJoin` room the host holds nothing to erase, but members'
key holders may still hold keys or file keys they cached for epochs they lived through, and the
floor is how they learn to discard them.

### Mirrors

A room served by more than one host keeps a chain, and a floor, at each. A prune at one tells
the others nothing, and a key holder backfilling from a mirror that has not been pruned would
be handed the rungs back — which is why a key holder keeps the highest floor it has learnt from
any host and refuses rungs below it whatever their source. The owner **SHOULD** prune at every
host serving the room.

So a prune is not a deletion and does not behave like one. It is closer to losing a key than to shredding a document, and an implementation that describes it as "deleting old records" is describing something else.

## Authorization

`admin` at the room's scope. Deliberately not `curate` and not `write`.

Pruning makes no statement about any record — it does not retract, demote or pin anything — so `curate` is the wrong shape as well as the wrong strength. And every member who can write can curate, which would put "end the room's readable history" within reach of every writer.

It sits beside [`rooms/epoch/mint`](../../mint/0.1/spec.md) because it is the same class of act: a decision about the room's key material that every member lives with.

A conforming consumer **MUST** apply the chain-verification, depth and no-dereference rules of [`rooms/records/put`](../../../records/put/0.1/spec.md).

## `beforeEpoch` must be below the room's current epoch

A host **MUST** refuse with `notAhead` where `beforeEpoch` is at or above the room's current epoch.

Pruning to the current epoch would drop every rung, which does not merely shorten the history — it leaves a member who restarts unable to derive **anything below the epoch they are handed next**, which is most of the room. That is a plausible typo (`beforeEpoch: currentEpoch` reads like "keep from here") with a consequence nobody would choose, so it is refused rather than performed.

## The response reports reach, and the floor

`epochFloor` is what the owner decided; `earliestRung` is what the host can still serve. They
differ when the chain already had a gap above the floor, and both are worth reporting: the first
is what every key holder discards below, the second what a member can actually walk.

### Reach, not the request

`earliestRung` is **not** a restatement of `beforeEpoch`, and a host **MUST NOT** treat it as one.

A chain can already have a gap — rungs are delivered per commit, and a delivery that never happened leaves one. A prune below an existing gap changes nothing about how far back a member can actually walk, and echoing the request back would tell an operator they had achieved something they had not.

`pruned: 0` is likewise a **success**: the chain already went back no further. A caller that read it as a failure would retry an operation with nothing left to do.

## Definitions

**`beforeEpoch`** — drop every rung *below* this epoch. The chain walks backwards, so this is a floor rather than a ceiling.

**`reason`** — why, for the room's audit trail. A prune is irreversible and unattributable after the fact — the rungs are simply gone — so the only record of intent is the one made at the time.

**`pruned`** — how many rungs were dropped.

**`earliestRung`** — how far back the chain still reaches. See above.

**`epochFloor`** — the room's epoch floor after this prune ([`EpochFloor`](../../../_shared/0.1/room.schema.json)).

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/epoch/prune/0.2#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:host",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "beforeEpoch": 5,
    "reason": "retention: the first three epochs are past the agreed window",
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
  "type": "https://trusttasks.org/spec/rooms/epoch/prune/0.2#response",
  "issuer": "did:example:host",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "pruned": 3,
    "earliestRung": 5,
    "epochFloor": 4
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

This *is* the retention mechanism, and it is one-way. A host that kept a copy of the pruned rungs would defeat the entire operation while reporting success, so a conforming host **MUST** erase them rather than marking them unavailable. It keeps one integer in their place — the floor — for the life of the room.

There is no soft delete here on purpose. A rung retained "just in case" is a rung that can be produced under compulsion, which is exactly the state the owner was trying to leave.

### Consent/purpose

The rungs are dropped so that a room can stop being able to read part of its own past. It is destructive, irreversible, and affects every member — so an implementation whose surface offers it **SHOULD** show what will become unreadable, and **SHOULD NOT** offer it as an incidental control beside reversible ones.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required — though `sideEffects: destructive` is what a consent surface keys off, and this is a strong candidate for one.
