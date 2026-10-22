---
slug: rooms/records/retention/set
version: "0.1"
title: "Rooms Records — Retention — Set"
summary: "A room's administrator changes how long the room keeps its records by age — the rule every member can read, and the only one a prune may remove records under."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - record
  - retention
  - policy
  - prune
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
    A shortened retention widens what every later prune removes, for every member. The change must be attributable independently of the transport it arrived on, because it is what a removal is later justified by.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed change would restore a retention an administrator has since replaced — possibly a shorter one, making records eligible that the room had decided to keep. Placing it in time is what lets a host refuse a stale one.
sideEffects:
  level: mutating
  rationale: >-
    Replaces the room's record retention. Nothing is removed by this task; what changes is what `rooms/records/prune` may remove from now on.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a retention rule and a reason; the response, the rule now in force and the one it replaced. A rule every member of the room can read anyway (`rooms/info`).
retention:
  class: durable
  rationale: >-
    The retention is kept for the life of the room, and each change to it — old rule, new rule, reason, requester where the tier discloses one — is kept in the room's audit trail, since a later prune is justified by it.
errorCodes:
  - code: rooms/records/retention/set:retentionConflict
    meaning: "`expectedRetention` does not match the record retention in force. `details.recordRetention` carries the one in force, `{}` when there is none."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [recordRetention]
      properties:
        recordRetention:
          $ref: "../../../../_shared/0.1/room.schema.json#/$defs/RecordRetention"
  - code: rooms/records/retention/set:retentionNotPermitted
    meaning: "The host's governance does not permit this retention — commonly one longer than it is willing to store records for. The host records nothing rather than a retention other than the one asked for."
    retryable: false
  - code: rooms/records/retention/set:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/records/retention/set:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
related:
  - rooms/records/prune
  - rooms/create
  - rooms/info
---

## Abstract

A room's **administrator** changes the room's **record retention** — how long it keeps
retracted records' tombstones, and how long it keeps records at all, by age
([`RecordRetention`](../../../../_shared/0.1/room.schema.json)).

A room states its first record retention at [`rooms/create`](../../../../create/0.2/spec.md).
Unlike the key chain's `retentionPolicy`, which is fixed because key material either exists for
an epoch or it does not, how long a room keeps its meeting notes is a decision it is entitled to
revisit. This is how it does.

Nothing is removed here. The record retention is the **rule**; removal is a separate, deliberate
act by a holder of `admin` ([`rooms/records/prune`](../../../prune/0.1/spec.md)), which removes
only what the rule in force makes eligible.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **host** (`recipient`):

1. **MUST** verify the presentation as [`rooms/records/put`](../../../put/0.2/spec.md) does and
   require that it confer `admin` at this room's scope; a presentation that does not is refused
   with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).
2. **MUST** refuse with `retentionConflict` when `expectedRetention` is present and differs from
   the retention in force (`{}` standing for none), compared member by member.
3. **MUST** refuse with `retentionNotPermitted`, recording nothing, a retention its governance
   will not apply — and **MUST NOT** record a different one instead: a retention silently
   shortened is records removed under a rule nobody stated.
4. **MUST** replace the retention in force with `recordRetention` whole. A member it leaves out
   is no longer stated; `{}` clears the retention, after which no record is eligible by age.
5. **MUST** serve the new retention from then on in [`rooms/info`](../../../../info/0.2/spec.md)
   0.2 and apply it to the next prune, and **MUST** keep in the room's audit trail the retention
   replaced, the one recorded, the `reason`, and the requester where the tier discloses one.
6. **MUST NOT** remove any record because of this task.

A conforming **producer** **SHOULD**, before shortening a retention, show whoever is deciding
what the shorter one would make eligible — a [`rooms/records/prune`](../../../prune/0.1/spec.md)
`dryRun` answers that only for the retention in force, so a surface computes it from a listing,
which carries every record's `updatedAt` and status.

## Authorization

`admin` at the room's scope, the grant [`rooms/records/prune`](../../../prune/0.1/spec.md) takes.
Shortening the retention is how a prune is made to reach further, so the two must take the same
authority; a weaker grant here would let a party who may not prune decide what the next prune
removes. Not `curate`, which every writer can be given and which is judgement about one record.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), a verified `proof` establishes who
asked, not that they may. The entitlement is the chain conferring `admin`, which the room issued.

## A shorter retention takes effect at the next prune

A change applies to every record already in the room, not only to records written after it: the
retention is a rule about age, measured when a prune is performed. So shortening `recordDays`
from two years to one makes eligible, at the next prune, every unpinned record that has not
changed for a year. That is the intended meaning — a room that decides to keep less keeps less —
and it is why the change is attributable, takes `admin`, and carries a reason.

Lengthening a retention, or clearing it, only narrows what a prune may remove, and protects
nothing a prune has already removed.

**This specification asks for no notice period**, and a deployment may want one: a member who
reads the room's retention today and finds tomorrow that their records went under a shorter one
read the right rule at the wrong time. A host or a room's own governance **MAY** delay applying a
shortened retention to prunes by a period it states; one that does **SHOULD** say so where the
retention is served.

## Definitions

**Record retention** — [`RecordRetention`](../../../../_shared/0.1/room.schema.json): how long
tombstones (`tombstoneDays`) and records (`recordDays`) are kept, by age. Absent or `{}`, no
record is eligible by age.

**`recordRetention`** — the retention from now on, replacing the one in force whole.

**`expectedRetention`** — optional precondition: the retention the caller believes is in force.

**`reason`** — why, for the room's audit trail. REQUIRED.

**`previous`** (response) — the retention replaced.

## Request

An **Administrator** sends this to the room's **Host**. The payload is the top-level schema in
[`payload.schema.json`](payload.schema.json).

### Keeping records one year instead of two

```json
{
  "id": "urn:uuid:4e2b8d61-7c3a-4b9f-a1d0-6f8e2c4b7a01",
  "type": "https://trusttasks.org/spec/rooms/records/retention/set/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-10-15T12:00:00Z",
  "threadId": "urn:uuid:4e2b8d61-7c3a-4b9f-a1d0-6f8e2c4b7aff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.admin-leaf.sig",
        "eyJhbGciOiJFZERTQSJ9.authority-root.sig"
      ]
    },
    "expectedRetention": { "tombstoneDays": 30, "recordDays": 730 },
    "recordRetention": { "tombstoneDays": 30, "recordDays": 365 },
    "reason": "The deal closed; working notes are kept one year under the agreed policy."
  }
}
```

## Response

The **Host** responds with the sub-schema reachable via `$anchor: "response"`. Failures use a
`trust-task-error` document rather than a `#response`, per
[SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Recorded

```json
{
  "id": "urn:uuid:4e2b8d61-7c3a-4b9f-a1d0-6f8e2c4b7a02",
  "type": "https://trusttasks.org/spec/rooms/records/retention/set/0.1#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-15T12:00:01Z",
  "threadId": "urn:uuid:4e2b8d61-7c3a-4b9f-a1d0-6f8e2c4b7aff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "recordRetention": { "tombstoneDays": 30, "recordDays": 365 },
    "previous": { "tombstoneDays": 30, "recordDays": 730 }
  }
}
```

## Security & Privacy

### Data carried

A room identifier, a retention rule, a precondition and a reason in; the rule in force and the
one it replaced out. No record, key or content crosses. A producer **MUST NOT** put a member
identifier or record content in `reason`, which the host keeps for the life of the room.

### Correlation

To the host this is an administrator deciding about retention; on `open` and `attributed`
rooms, which one. A shortened retention shortly before a prune, or shortly after a member
leaves, says something about why; an administrator for whom that matters changes retention on
its own schedule rather than in response to events.

### Retention

The rule is kept for the life of the room, and each change to it in the room's audit trail,
because a later prune is justified by the rule in force when it ran. Nothing a member wrote is
kept or removed by this task.

### Consent/purpose

The retention exists so that a room keeps no more of its past than it has said it would, and so
that every member can read, before any record is removed, the rule it will be removed under. A
change to it is a change to the terms every member writes under, which is why it takes the
grant a prune takes, carries a reason, and is served to every member — and why a surface
**SHOULD** show a member the room's record retention where they write.

This specification describes purpose only. Per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that
consent, approval or a step-up is required.
