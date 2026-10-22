---
slug: rooms/create
version: "0.2"
title: Rooms — Create
summary: "An owner registers a data room with a host, naming the room's own identifier, its visibility, the party accountable for it, and how long it keeps its records."
status: draft
targetFrameworkVersion: "0.5.0"
category: access-control
keywords:
  - room
  - create
  - visibility
  - owner
  - hosting
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
  rationale: "Room creation establishes durable state a host will serve to others; its authorization must be verifiable independently of transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Creation is time-bounded by the credential authorizing it; a replayed create could re-establish a room its owner had closed."
sideEffects:
  level: mutating
  rationale: "Registers a room with the host."
subjectPath: /roomId
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The host learns the room's identifier, its visibility, its owner, its retention period and its record retention. On `attributed` and `private` rooms it learns nothing further for the life of the room."
retention:
  class: durable
  rationale: "A room persists until it lapses without renewal past its stated retention period, or its owner closes it."
errorCodes:
  - code: rooms/create:alreadyExists
    meaning: "A room with that identifier is already registered with this host."
    retryable: false
  - code: rooms/create:visibilityNotPermitted
    meaning: "The host's governance does not permit rooms of this visibility."
    retryable: false
related:
  - rooms/epoch/mint
  - rooms/records/put
  - rooms/records/prune
  - rooms/records/retention/set
  - rooms/info
---

## Abstract

The **Rooms — Create** Trust Task registers a data room with a host.

**The owner brings the identifier; the host does not assign one.** A room identified by
something its host chose could not move to another host without changing identity, and
portability is the property the whole family rests on: because authorization is decided by
credentials the room issued rather than by host state, re-pointing a room at a different
host reissues nothing.

`visibility` is **fixed at creation and immutable thereafter**. A downgrade cannot un-see
cleartext, and an upgrade would protect only what came after while presenting as though it
protected everything. To change the visibility of some material, make another room and move
it deliberately.

`retentionPolicy` is fixed at creation and immutable for the same reason, and answers a
different question: not what the *host* can see, but whether the room keeps its own past
readable. A room's records are sealed under the key of the epoch current when they were
written, and a group key schedule offers no way to derive an earlier epoch's key from a
later one — which is what makes removing a member mean something. `chained` bridges that
deliberately, by having each advance seal the outgoing epoch's key under the incoming one
(see [`rooms/epoch/mint`](../../epoch/mint/0.1/spec.md)); `fromJoin` does not.

**Absent means `chained`, and that is the choice a room wants unless it has a reason not
to.** Joining a room should mean being able to read it, and a room that advances without
producing rungs silently loses the ability to read everything written before — for every
member, including whoever wrote it. `fromJoin` is for the room that wants exactly that: a
stream rather than a library, or one under a strict forward-secrecy obligation. The cost of
`chained` is real and belongs in the same breath: a member's current key then reaches every
retained epoch, so compromising one reaches the room's whole retained history rather than
only what came after.

`ownerDid` is visible at **every** visibility, including `private`. A room has a party
answerable for it: the controller of its identifier, the issuer of its credentials, and the
party a host addresses about quota, abuse, or lifecycle. A room whose contents no one can
read still has someone responsible for it existing.

`recordRetention` (new in 0.2) answers a third question, and it is the one that is **not**
fixed. Not what the host can see, not whether the key chain reaches back, but how long the
room keeps its records while it is live: records older than it states become eligible for
[`rooms/records/prune`](../../records/prune/0.1/spec.md). A room states it at creation so
that every member can read the rule before any record is removed under it, and a holder of
`admin` changes it later with [`rooms/records/retention/set`](../../records/retention/set/0.1/spec.md),
because how long a room keeps meeting notes is a decision it is entitled to revisit. Absent,
no record is ever eligible by age. See [`RecordRetention`](../../_shared/0.1/room.schema.json)
for why it is not called `retentionPolicy` or `retentionDays`, both of which this family
already uses for something else.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

### What changed from 0.1

One optional request member, `recordRetention`, and its echo in the response. The request is a
closed object, so a 0.1 host refuses a create carrying it rather than registering a room whose
retention it ignored — which is the failure that matters, since an owner who stated a
retention and saw the room created would believe it in force. A new member on a closed
payload is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). A host that
implements 0.2 **SHOULD** continue to accept 0.1, recording no record retention.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the owner) **MUST** mint the room's identifier before calling,
and **MUST** be able to demonstrate control of it.

A conforming **consumer** (the host) **MUST** refuse a visibility its governance does not
permit, **MUST** treat `visibility` and `retentionPolicy` as immutable for the life of the
room, and **MUST** state its retention behaviour at creation rather than applying one
discovered later. A host **MUST NOT** require a member list, at creation or afterwards.

A host **MUST** treat an absent `retentionPolicy` as `chained`, and **MUST** reject an
epoch link presented for a room whose policy is `fromJoin` rather than storing one the room
declared it would not have. Silently discarding it instead would let a producer believe the
room's history was being retained when it was not, which is the failure this member exists
to make explicit.

A host **MUST** record `recordRetention` as given, serve it back in `rooms/info` (0.2 and
later) and in this response, and **MUST NOT** remove a record by age except by
`rooms/records/prune` under the record retention in force. A host whose governance does not
permit the retention asked for — one longer than it is willing to store records, say —
**MUST** refuse the create rather than record a shorter one: a retention silently shortened is
records removed under a rule nobody stated.

A caller who may not create rooms on this host is refused with the framework's standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)). This specification declares no task-specific code for it.

## Authorization

The owner's control of the room's identifier, which the producer must be able to demonstrate,
and the host's own policy over who may create rooms on it. Nothing in this request is
authorized by a credential the room issued — the room's credentials do not yet reach a host
that has never heard of it — and per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements) a
verified `proof` establishes who asked, not that they may. `recordRetention` takes no further
authority: the party registering the room is the one deciding its first policy.

## Request

An **Owner** sends this to the **Host**. The payload is the top-level schema in
[`payload.schema.json`](payload.schema.json).

### A room that keeps tombstones a month and records two years

```json
{
  "id": "urn:uuid:3c6f1a2e-5b0d-4a7e-9f1c-2d8e4b6a0c11",
  "type": "https://trusttasks.org/spec/rooms/create/0.2#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-10-20T08:00:00Z",
  "threadId": "urn:uuid:3c6f1a2e-5b0d-4a7e-9f1c-2d8e4b6a0cff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "visibility": "attributed",
    "ownerDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "retentionPolicy": "chained",
    "retentionDays": 90,
    "recordRetention": { "tombstoneDays": 30, "recordDays": 730 }
  }
}
```

## Response

The **Host** responds with the sub-schema reachable via `$anchor: "response"`, echoing what it
recorded. Failures use `trust-task-error`.

### Registered

```json
{
  "id": "urn:uuid:3c6f1a2e-5b0d-4a7e-9f1c-2d8e4b6a0c12",
  "type": "https://trusttasks.org/spec/rooms/create/0.2#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-20T08:00:01Z",
  "threadId": "urn:uuid:3c6f1a2e-5b0d-4a7e-9f1c-2d8e4b6a0cff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "epoch": 1,
    "recordRetention": { "tombstoneDays": 30, "recordDays": 730 }
  }
}
```

## Security & Privacy

**A host verifies chains; it does not keep a roster.** Authorization is decided entirely by
credentials the room issued. A host that consults state of its own has made the room
unmovable and has made itself part of the membership.

**Chain verification is the security of this family.** Anyone can mint a well-formed
authority credential naming any scope and any actions; what makes it worthless is that its
chain does not reach the room. A host that verifies only the credential it was handed
accepts a self-issued grant of arbitrary authority. Every link is verified, and any link
that widens actions, scope, or validity beyond its parent invalidates the chain.

**Chain depth is a denial-of-service surface.** Verification is linear in length and runs on
every operation, hence the maximum of 8. The known uses need 2 to 3.

**Parents are never dereferenced.** A producer presents every link. Resolving one over the
network would make verification depend on availability, turn an identifier into a request
the host can be induced to make against an address the *producer* chooses, and signal
credential use to whoever hosts that identifier. Identifiers in a chain are identifiers, not
locators.

**Credential pooling.** Where membership and authority are presented with the subject
withheld, a host that does not require proof that both describe the same subject lets two
parties combine one's membership with the other's authority and present as a single party
holding both.

**What a host learns anyway.** Sealing content does not hide activity. A host observes room
identifiers, owners, record counts, sizes, epochs, and timing, and — unless traffic is
routed — the network origin of whoever is acting. A room whose adversary can correlate on
those wants a different host, not a different visibility.

### Data carried

The room's identifier, its visibility, its owner, its retention period and its record
retention. Nothing about
its membership, then or later.

### Correlation

A host learns that this owner runs a room. An owner running several rooms is correlatable by
that fact alone, which is a reason a party may prefer a distinct identifier per room.

### Retention

The registration is durable and governs how long the host holds the room after its epoch
lapses without renewal. Stated here rather than discovered later. `recordRetention` governs
how long the room's records may be kept while it is live, and is likewise stated before it
can apply to anything.

### Consent/purpose

The caller asserts control of the room identifier they bring. A host's governance decides
which visibilities it will accept, and may decline.
