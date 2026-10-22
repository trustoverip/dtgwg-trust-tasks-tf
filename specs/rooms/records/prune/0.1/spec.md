---
slug: rooms/records/prune
version: "0.1"
title: "Rooms Records — Prune"
summary: "A room's administrator asks its host to remove the records the room's stated record retention makes eligible — old tombstones, old records, or both — leaving a mark that records went and never what they said."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - record
  - retention
  - prune
  - tombstone
  - erasure
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request removes records from a room for every member, permanently. Authority for that must be verifiable independently of the transport it arrived on, and attributable afterwards, when the records are gone and the request is all that is left.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Eligibility is a function of time — a record becomes eligible as it ages — so a replayed prune is not a repeat of the first one: it removes whatever has aged into eligibility since. Placing the request in time is what lets a host refuse a stale one rather than perform it.
sideEffects:
  level: destructive
  rationale: >-
    Records are removed outright, tombstones included, and the blobs only they named are released. Nothing recovers them from the host; the room keeps only a mark that records went, how many, and when.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The request names a room, what kinds of record to remove and why. The response says how many were removed and under which retention — counts and a rule every member can already read — and never which records, or anything they said.
retention:
  class: durable
  rationale: >-
    The prune's mark (`RecordPrune`) is kept as the room's `lastPrune`, and its audit entry — reason, requester where the tier discloses one, retention applied — is kept as the only account of an act that left nothing else behind.
errorCodes:
  - code: rooms/records/prune:noRetention
    meaning: "The room's record retention states no rule for any of the kinds asked — no `tombstoneDays` for `retracted`, no `recordDays` for `aged`, or no retention at all — so no record is eligible, and none ever was. Set one first with `rooms/records/retention/set`. `details.recordRetention` carries the retention in force, absent when there is none."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        recordRetention:
          $ref: "../../../_shared/0.1/room.schema.json#/$defs/RecordRetention"
  - code: rooms/records/prune:roomNotLive
    meaning: "The room's epoch has lapsed. A prune is a write; a room nobody has renewed is read-only until somebody does."
    retryable: false
  - code: rooms/records/prune:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/records/prune:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
related:
  - rooms/records/retention/set
  - rooms/records/curate
  - rooms/records/list
  - rooms/info
  - rooms/create
  - rooms/epoch/prune
---

## Abstract

A room's **administrator** asks its **host** to remove the records the room's **record
retention** makes eligible: retracted records' tombstones older than it keeps them, records
older than it keeps them, or both. What is removed is removed outright — key, version, body,
tombstone — and every blob only those records named is released.

The rule is the room's, stated in advance and readable by every member
([`RecordRetention`](../../../_shared/0.1/room.schema.json), set at
[`rooms/create`](../../../create/0.2/spec.md) and changed with
[`rooms/records/retention/set`](../../retention/set/0.1/spec.md), served by
[`rooms/info`](../../../info/0.2/spec.md)). The act is the administrator's, made deliberately
with a reason. The host does neither on its own: it measures age and removes what it is asked
to, and nothing on a timer.

What the room keeps is a **mark**: that records were removed, how many, when, and the version
the removal took ([`RecordPrune`](../../../_shared/0.1/room.schema.json)). Never which, and
never anything they said.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **host** (`recipient`):

1. **MUST** verify the presentation as [`rooms/records/put`](../../put/0.2/spec.md) does —
   every link, at most 8, no link widening its parent, no parent dereferenced, the leaf bound
   to the party authenticated for this request, and on a `private` room a `subjectBinding` —
   and **MUST** require that it confer `admin` at this room's scope. A presentation conferring
   `curate` or `write` alone is refused with the standard `permissionDenied`
   ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).
2. **MUST** refuse with `roomNotLive` when the room's epoch has lapsed.
3. **MUST** decide eligibility from the record retention in force when it performs the prune,
   and from nothing else — never from the request, `ext`, or a policy of its own:
   - for `retracted`, a record is eligible when it is `retracted` and its `updatedAt` is more
     than `tombstoneDays` days before the moment of the prune;
   - for `aged`, a record is eligible when it is **not pinned** and its `updatedAt` is more
     than `recordDays` days before the moment of the prune, whatever its standing.

   A kind for which the retention states no rule makes nothing eligible. When that holds for
   every kind asked, the host **MUST** refuse with `noRetention` rather than answer
   `removed: 0`: the caller has asked for something the room never said it would do, which is
   worth saying plainly.
4. On `dryRun: true`, **MUST** report what a prune would remove now and change nothing.
5. Otherwise **MUST**, as one atomic change to the room:
   - remove every eligible record entirely — key, version, status, body or ciphertext,
     tombstone — so that no read, listing or `sinceVersion` returns it again;
   - release every blob a removed record named, exactly as a rewrite or retraction does
     ([`rooms/records/put`](../../put/0.2/spec.md) 0.2, consumer item 8);
   - when at least one record was removed, take the next version from the room's counter, as a
     put or a curation does, and record a [`RecordPrune`](../../../_shared/0.1/room.schema.json)
     carrying it, the time and the number removed as the room's `lastPrune`.

   A prune that removed nothing takes no version and leaves no mark.
6. **MUST** return `lastPrune` thereafter wherever a response can carry it
   ([`rooms/records/list`](../../list/0.3/spec.md) 0.3, [`rooms/keys/browse`](../../../keys/browse/0.2/spec.md)
   0.2 through it, [`rooms/info`](../../../info/0.2/spec.md) 0.2).
7. **MUST** keep the audit trail below, and **MUST NOT** keep anything else of a removed
   record.
8. **MUST NOT** remove a record by age in any other way — on a timer, at a quota, on a
   lapse short of reclamation — and **MUST NOT** prune on its own initiative. Reclaiming a whole
   room after `retentionDays` ([`rooms/create`](../../../create/0.2/spec.md)) is a different
   act and is unaffected.

A conforming **producer** **SHOULD** make a dry run first and show its result to whoever is
deciding, and **SHOULD NOT** offer a prune as an incidental control beside reversible ones.

## Authorization

`admin` at the room's scope. Deliberately not `curate` and not `write`.

`curate` is judgement about **one record**, and it leaves a tombstone so that every member
learns of it and the room's history records that judgement was exercised. A prune removes
records — tombstones included — for every member at once, and leaves only a count. Every
member who can write can be granted `curate`; "make the room's past disappear" must not arrive
with it. A prune sits with [`rooms/epoch/prune`](../../../epoch/prune/0.1/spec.md) and
[`rooms/records/retention/set`](../../retention/set/0.1/spec.md): decisions about the room's
retention that every member lives with.

The entitlement has two parts, and both are required. The **rule** is the room's record
retention, stated in advance and readable by every member; the **act** is the administrator's
request, made with a reason. A prune performs neither alone: the request cannot remove what the
retention does not make eligible, and the retention removes nothing until somebody holding
`admin` asks.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`,
`recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they
were entitled to the outcome. The entitlement is the chain conferring `admin`, which the room
issued.

## What a prune keeps, and what it does not

### The audit trail

For every prune that removed anything, a host keeps:

- the [`RecordPrune`](../../../_shared/0.1/room.schema.json) mark — version, time, count;
- the `reason`;
- the requester, **where the tier discloses one** — the member at the root of the presented
  chain on `open` and `attributed` rooms; on a `private` room, only that a holder of `admin`
  asked, which is all the host knows;
- the record retention it was made under, and the kinds asked.

A host **MUST NOT** keep anything a removed record said or carried — body, ciphertext,
cleartext, title, description, author, file manifest or blob — and **SHOULD NOT** keep the
removed records' keys. The count is the audit; a list of keys on an `open` room, where keys may
be descriptive, is content by another name, and on the sealed tiers it is a list of nothing
anyone can use. A host that kept a copy of what it removed would defeat the prune while
reporting success.

### The record tree

The room's data commitment ([`DataCommitment`](../../../_shared/0.1/room.schema.json)) is over
the records the room holds, so a prune removes leaves: the root changes and
[`RecordCount`](../../../_shared/0.1/room.schema.json) falls by `removed`. The prune's version
moves [`HeadVersion`](../../../_shared/0.1/room.schema.json) past the removal, so a root taken
before and a root taken after are never labelled with one version — which would read as a host
caught equivocating, and is not. A witnessed anchor taken before the prune stays true of the
moment it describes; a reader comparing against it sees the room `ahead`, as after any write.

A reader holding a listing from before the prune **can check it**. Every record missing
afterwards should have been eligible, at `prune.at`, under the response's `recordRetention`:
retracted and old enough, or unpinned and old enough. A record that disappeared without being
eligible is a host that removed what the room did not ask it to, and is evidence against it in
the same way a short listing is.

### Members' copies

**A prune removes records from the host, not from the world.** A member who read a record has
it; a member's agent that cached it holds it until it next synchronises, when
[`rooms/records/list`](../../list/0.3/spec.md) 0.3's `lastPrune` tells it to rebuild and drop
what is gone. An export is beyond the room entirely. On the sealed tiers the ciphertext of a
removed record may survive in a backup, a mirror or a store that publishes ciphertext; to make
such copies unreadable, the room prunes its key chain
([`rooms/epoch/prune`](../../../epoch/prune/0.1/spec.md)), which is the only erasure that
reaches a copy somebody else holds. A surface **MUST NOT** describe a prune as more than this.

### Mirrors

A room served by more than one host is pruned at each. A prune at one host says nothing to the
others, and a member reading from a mirror that has not been pruned still reads what was
removed elsewhere.

### What eligibility is not

- **Not a record's own `validUntil`.** Expiry is sealed inside the record body
  ([`SealedRecordBody`](../../../_shared/0.1/room.schema.json)) on every tier but `open`, and a
  host never sees it; it is a reader's rule. A room that wants expired records gone has a
  curator's agent retract them ([`rooms/records/curate`](../../curate/0.1/spec.md)) — which a
  reader can do, because it can read `validUntil` — and their tombstones then age out under
  `tombstoneDays`.
- **Not pinned records, by age.** Pinning says what matters in the room. A pinned record is
  removed only once it is unpinned, or — if it is also retracted — as a tombstone.
- **Not the room's profile, unless it is left unprotected.** The room's profile is an ordinary
  record to its host (`RoomProfileStatement` is located so that a host cannot tell it from any
  other), so `recordDays` reaches it like any record that has not changed for that long. A room
  that states `recordDays` **SHOULD** keep its profile record pinned — the party writing the
  profile holds `curate` or `admin` and can pin it — or re-write it within the retention.

## Definitions

**Record retention** — the room's [`RecordRetention`](../../../_shared/0.1/room.schema.json):
how long tombstones and records are kept, by age.

**Eligible** — a record the record retention in force allows a prune to remove, by the rules of
Conformance item 3.

**`kinds`** — what to remove: `retracted` tombstones, `aged` records, or both.

**`reason`** — why, for the room's audit trail. REQUIRED.

**`dryRun`** — report what would be removed, and remove nothing.

**`removed`**, **`blobsReleased`** (response) — what the prune removed, or would.

**`recordRetention`** (response) — the retention eligibility was decided under.

**`prune`** (response) — the mark the prune left, now the room's `lastPrune`.

## Request

An **Administrator** sends this to the room's **Host**. The payload is the top-level schema in
[`payload.schema.json`](payload.schema.json).

### Removing old tombstones and old records

```json
{
  "id": "urn:uuid:9a3e7c10-2b4d-4f6a-8e1c-5d7f9b0a2c01",
  "type": "https://trusttasks.org/spec/rooms/records/prune/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-10-01T00:00:00Z",
  "threadId": "urn:uuid:9a3e7c10-2b4d-4f6a-8e1c-5d7f9b0a2cff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.admin-leaf.sig",
        "eyJhbGciOiJFZERTQSJ9.authority-root.sig"
      ]
    },
    "kinds": ["retracted", "aged"],
    "reason": "Quarterly run of the room's agreed retention."
  }
}
```

## Response

The **Host** responds with the sub-schema reachable via `$anchor: "response"`. Failures use a
`trust-task-error` document rather than a `#response`, per
[SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Fourteen records removed

```json
{
  "id": "urn:uuid:9a3e7c10-2b4d-4f6a-8e1c-5d7f9b0a2c02",
  "type": "https://trusttasks.org/spec/rooms/records/prune/0.1#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-01T00:00:05Z",
  "threadId": "urn:uuid:9a3e7c10-2b4d-4f6a-8e1c-5d7f9b0a2cff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "removed": 14,
    "blobsReleased": 2,
    "recordRetention": { "tombstoneDays": 30, "recordDays": 730 },
    "prune": { "version": 413, "at": "2026-10-01T00:00:05Z", "removed": 14 }
  }
}
```

### Refused: the room states no retention for what was asked

```json
{
  "id": "urn:uuid:9a3e7c10-2b4d-4f6a-8e1c-5d7f9b0a2c03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-01T00:00:05Z",
  "threadId": "urn:uuid:9a3e7c10-2b4d-4f6a-8e1c-5d7f9b0a2cff",
  "payload": {
    "code": "rooms/records/prune:noRetention",
    "message": "The room keeps records indefinitely; no record is eligible by age.",
    "retryable": false,
    "details": { "recordRetention": { "tombstoneDays": 30 } }
  }
}
```

## Security & Privacy

### Data carried

The request names a room, the kinds of record to remove and a reason. The response carries
counts, the retention applied and the prune's mark. No record, no key, no content crosses in
either direction, on any tier — on the sealed tiers the host could not read what it removes,
and on `open` it does not need to say.

A producer **MUST NOT** put a member identifier or record content in `reason`. It is kept in
the host's audit trail for as long as the room, which on a `private` room is a place the room
otherwise puts nothing readable.

### Correlation

To the host this is an administrator deciding about retention; on `open` and `attributed`
rooms, which one. The **timing** is informative: a prune shortly after a member leaves, or
shortly after a retention was shortened, says something about why. An administrator for whom
that matters runs prunes on a regular schedule rather than in response to events.

`removed` and `lastPrune` tell every member how many records went and when — the room's own
retention made public to its own members, which is the intent.

### Retention

This **is** a retention mechanism, and it is one-way. A host that kept a copy of what it
removed — a soft delete, a "just in case" — would defeat the operation while reporting success,
and a record retained that way is a record that can be produced under compulsion, which is the
state the room was trying to leave. So a conforming host erases, and keeps only the audit trail
described above.

### Consent/purpose

Records are removed so that a room keeps no more of its past than it has said it would. The
rule is stated in advance, readable by every member, and changeable only by a holder of
`admin`; the act is a holder of `admin` applying it, with a reason. What a writer consented to
is the room's retention as it stood when they wrote, and as an administrator may change it —
which is why the retention is served to every member, and why a surface **SHOULD** show a
member the room's record retention where they write.

This specification describes purpose only. Per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that
consent, approval or a step-up is required — though `sideEffects: destructive` is what a
consent surface keys off, and this is a strong candidate for one.
