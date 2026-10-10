---
slug: rooms/usage
version: "0.1"
title: Rooms — Usage
summary: "A room's owner reads, from the room's host, how much storage each member who has added files is using against their allowance — authorized by a room-issued chain conferring `admin`, and refused on `private` rooms, where the host cannot tell members apart."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - quota
  - usage
  - storage
  - owner
parties:
  - role: Room owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: "A read that discloses which members add how much. Authority is the room-issued chain in the payload, bound to whoever the host authenticated; a proof makes that binding hold on relayed transports, and makes each disclosure attributable."
sideEffects:
  level: none
  rationale: "Reads the host's per-member usage counters for one room. Persists nothing."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Discloses, to the room's owner, the DIDs of members who have added files to the room and how many files and bytes each holds. That is personal data about those members' activity, which the `attributed` tier already lets the host see; the task hands it to the room's owner and to nobody else. Never file names or contents, which the host does not have."
retention:
  class: transient
  rationale: "A read; the host keeps nothing beyond the counters it already maintains."
errorCodes:
  - code: rooms/usage:notAuthorized
    meaning: "The presentation does not confer `admin` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/usage:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/usage:notAvailable
    meaning: "The room is `private`. The host does not know which member uploaded what, so there is no per-member usage to report; room totals are in rooms/info."
    retryable: false
related:
  - rooms/info
  - rooms/owner/issue-authority
  - vtc/rooms/usage
  - vtc/rooms/limits/set
---

## Abstract

A host applies per-member limits inside a room: how many files each member may add, how
large, and how much in total. When a room fills, its owner needs to know **who** is using
the space — to ask a member to tidy up, or to ask the host to raise one member's allowance.

The **Rooms — Usage** Trust Task returns that: one entry per member who has added files,
with their usage and the limits that apply to them. It is the room owner's read, authorized
by a chain the room issued conferring `admin`, and answered by the host from counters it
keeps anyway in order to enforce the limits.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **host** (`recipient`) **MUST**:

1. Verify the presentation as every `rooms/*` task does — every link, at most 8, no link
   widening its parent, no parent dereferenced, the leaf bound to the party authenticated
   for this request — and require that it confer **`admin`** at this room's scope.
   `read`, `write` and `curate` do not suffice: knowing what other members do is the
   owner's concern, not a member's.
2. Refuse a `private` room with `notAvailable`. The host cannot attribute uploads there, by
   design, and **MUST NOT** approximate (by session, network origin or timing) to produce
   an answer.
3. Return one entry per member who currently holds or has reserved storage in the room, or
   for whom an override is set, keyed by the **root subject** of the chains they uploaded
   with. Uploads made by a member's agents are counted under the member.
4. Report, for each, the usage counted against limits (reservations included) and the
   limits that actually apply, marking an override with `overridden`.
5. Page with `cursor` and `limit`, returning `nextCursor` exactly when more remain.
6. **Never** return a member who has stored nothing and holds no override — the response
   is not a roster, and the host has no roster to give — and never a file name, a record
   key, or anything derived from content.

## Authorization

Authority is **conferred by the room**: the presentation must confer `admin` at this room's
scope — the action [`rooms/owner/issue-authority`](../../owner/issue-authority/0.2/spec.md)
grants to whoever runs the room. The host's own access control is neither required nor
consulted; a host's administrators read usage across rooms through
[`vtc/rooms/usage`](../../../vtc/rooms/usage/0.1/spec.md), on the host's authority, which is
a different question asked by a different party.

## Definitions

- **Usage**, **limits** — as in [`rooms/_shared/0.1/blobs.schema.json`](../../_shared/0.1/blobs.schema.json).
  Sizes are ciphertext bytes.
- **Root subject** — the subject of the credential at the room end of an authority chain.
- **Override** — limits set for one member in one room by the host's administrators,
  replacing the room's per-member default for that member.

## Request

The room's owner (`issuer`) asks the room's host (`recipient`). The payload is the top-level
schema in [`payload.schema.json`](payload.schema.json).

### The owner pages through member usage

```json
{
  "id": "urn:uuid:9e2f4d71-0b3c-4d5e-8f6a-1b2c3d4e5f01",
  "type": "https://trusttasks.org/spec/rooms/usage/0.1#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-10-10T11:00:00Z",
  "threadId": "urn:uuid:9e2f4d71-0b3c-4d5e-8f6a-1b2c3d4e5fff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.owner-authority.sig"
      ]
    },
    "limit": 50
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`.

### Two members have added files; one has an override

```json
{
  "id": "urn:uuid:9e2f4d71-0b3c-4d5e-8f6a-1b2c3d4e5f02",
  "type": "https://trusttasks.org/spec/rooms/usage/0.1#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-10-10T11:00:01Z",
  "threadId": "urn:uuid:9e2f4d71-0b3c-4d5e-8f6a-1b2c3d4e5fff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "members": [
      {
        "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "usage": { "files": 41, "bytes": 209600000, "reservedBytes": 0 },
        "limits": { "maxFiles": 2000, "maxFileBytes": 104857600, "maxBytes": 209715200 },
        "overridden": true,
        "lastUploadAt": "2026-10-10T09:30:01Z"
      },
      {
        "member": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "usage": { "files": 3, "bytes": 15728640, "reservedBytes": 4718880, "reservedFiles": 1 },
        "limits": { "maxFiles": 2000, "maxFileBytes": 104857600, "maxBytes": 1073741824 },
        "lastUploadAt": "2026-10-09T16:12:44Z"
      }
    ]
  }
}
```

## Security & Privacy

**This discloses member activity, deliberately and narrowly.** On an `attributed` room the
host already knows which member uploaded each blob; that is the tier's stated privacy
property. This task gives the room's owner a view of it, and nothing more — counts and
bytes, never names or contents.

**`admin` and nothing less.** A chain conferring `read` or `write` makes a party a member,
and a member's own figures are in [`rooms/info`](../../info/0.1/spec.md). Answering this to
any member would let every member watch every other.

**`private` is refused, not approximated.** A host that guessed attribution on a `private`
room from timing or origin would be reconstructing exactly the membership the tier exists
to withhold.

### Data carried

In: a room identifier, a presentation naming the owner, and paging. Out: member DIDs with
file counts, byte totals, limits and last-upload times.

### Correlation

The response links member DIDs to activity volumes in one room, for a party — the room's
owner — who already issued those members their credentials. It adds nothing that links
members across rooms.

### Retention

The host keeps the counters it needs to enforce limits; this task creates nothing new. An
owner who stores the response holds personal data about their members and is responsible
for it.

### Consent/purpose

Members accept the room's tier, and its disclosure of their activity to the host, when they
accept an invitation. The purpose here is to administer the room's storage — to find who
is using the space, and to ask for or grant more. Nothing in the response is content.
