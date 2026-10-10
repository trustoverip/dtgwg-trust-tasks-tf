---
slug: rooms/info
version: "0.1"
title: Rooms — Info
summary: "A member asks a room's host what it holds about the room — tier, current epoch, the limits applied to its files, and how much of them the room and the member have used — authorized by the room's own credentials."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - quota
  - limits
  - usage
  - storage
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: "A read of hosting metadata. Authority is the room-issued chain in the payload, bound to whoever the host authenticated; a proof makes that binding hold on relayed transports too, which is why it is recommended rather than left optional."
sideEffects:
  level: none
  rationale: "Reads the host's record of the room and its usage counters. Persists nothing."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Discloses, to a member, the room's tier, epoch and file limits, the room's total use, and the member's own use and allowance — hosting metadata the host holds by construction. Never another member's figures, and nothing derived from content."
retention:
  class: transient
  rationale: "A read; the host keeps nothing beyond what it held already."
errorCodes:
  - code: rooms/info:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/info:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
related:
  - rooms/usage
  - rooms/blobs/upload/begin
  - rooms/records/put
  - vtc/rooms/limits/set
---

## Abstract

A data room's host decides how much it is willing to store for the room: whether the room
may hold files at all, how large one may be, how many and how much in total, for the room
and for each member in it. A member needs those numbers **before** an upload — to be told
that a file will not fit, and by which limit, rather than finding out halfway through a
transfer — and needs to see how much of their own allowance they have spent.

The **Rooms — Info** Trust Task returns them: the room's tier and current epoch, the limits
the host applies to it, the room's total usage, and the asking member's own usage and
effective allowance. Like every task in the `rooms/*` family, it is authorized by a
presentation of credentials the room issued, never by an account at the host.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **host** (`recipient`) **MUST**:

1. Verify the presentation exactly as [`rooms/records/get`](../../records/get/0.1/spec.md)
   does — every link of the chain, at most 8, no link widening its parent, no parent
   dereferenced, the leaf bound to the party authenticated for this request, and on a
   `private` room a `subjectBinding` — and require that it confer `read` at this room's
   scope.
2. Return the room's **effective** limits: an administrator's override for the room where
   one exists, else the room's limits at creation, with every byte measure capped by the
   host-wide ceiling. A limit the host would not actually enforce **MUST NOT** be reported.
3. Return `usage.room` from the same counters an upload is checked against, reservations
   included, so a member who sees room for a file is not then refused for lack of it.
4. On `open` and `attributed` rooms, return `usage.member` for the **subject at the root of
   the presented chain** — the member, not the agent or key that signed — with the limits
   that apply to that member in this room.
5. On a `private` room, **omit** `usage.member`. The host does not know which member is
   asking, and the tier exists so that it cannot.
6. **Never** return another member's figures, a list of members, or anything derived from
   record or file content.

## Authorization

A presentation that does not confer `read` at this room's scope, whose chain does not reach the room, or whose leaf is not the party the host authenticated, is refused with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)); this specification declares no task-specific code for it.

Authority is **conferred by the room**: the presentation must confer `read` at this room's
scope. The host's own access control is neither required nor consulted — a community
administrator reads hosting figures for every room through the host's own operator task
([`vtc/rooms/usage`](../../../vtc/rooms/usage/0.1/spec.md)), not through this one.

## Definitions

- **Limits**, **usage**, **scope** — as in
  [`rooms/_shared/0.1/blobs.schema.json`](../../_shared/0.1/blobs.schema.json):
  `RoomLimits`, `ScopeLimits`, `Usage`. All sizes are ciphertext bytes, padding included —
  what is actually stored.
- **Effective member limits** — the limits a member's upload is checked against at the
  member scope: an override set for that member in this room if there is one, else the
  room's per-member default.
- **Root subject** — the subject of the credential at the room end of the authority chain;
  usage is charged to it so that a member cannot multiply their allowance by minting agents.

## Request

A member (`issuer`) asks the room's host (`recipient`). The payload is the top-level schema
in [`payload.schema.json`](payload.schema.json).

### A member checks the room before adding a file

```json
{
  "id": "urn:uuid:5b1d8c2e-6a4f-4f0e-9c3b-2a7d1e0f4a01",
  "type": "https://trusttasks.org/spec/rooms/info/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-10-10T09:29:00Z",
  "threadId": "urn:uuid:5b1d8c2e-6a4f-4f0e-9c3b-2a7d1e0f4aff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.authority-root.sig"
      ]
    }
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`.

### An `attributed` room, with the member's own figures

```json
{
  "id": "urn:uuid:5b1d8c2e-6a4f-4f0e-9c3b-2a7d1e0f4a02",
  "type": "https://trusttasks.org/spec/rooms/info/0.1#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T09:29:01Z",
  "threadId": "urn:uuid:5b1d8c2e-6a4f-4f0e-9c3b-2a7d1e0f4aff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "visibility": "attributed",
    "epoch": 7,
    "limits": {
      "filesEnabled": true,
      "room": { "maxFiles": 10000, "maxFileBytes": 104857600, "maxBytes": 5368709120 },
      "member": { "maxFiles": 2000, "maxFileBytes": 104857600, "maxBytes": 1073741824 }
    },
    "usage": {
      "room": { "files": 312, "bytes": 1288490188, "reservedBytes": 4718880, "reservedFiles": 1 },
      "member": {
        "usage": { "files": 41, "bytes": 209600000, "reservedBytes": 0 },
        "limits": { "maxFiles": 2000, "maxFileBytes": 104857600, "maxBytes": 209715200 }
      }
    }
  }
}
```

## Security & Privacy

**Room authorization only.** This task is answered from the room's credentials, so it works
against any host that stores the room, and a host that answers it from an account of its own
has made itself part of the membership.

**The member's figures are the member's.** `usage.member` describes the root subject of the
chain that was presented and nobody else. Returning another member's usage would turn a
read every member may make into a census of who adds what, which on `attributed` is the
room owner's to see ([`rooms/usage`](../../usage/0.1/spec.md)) and on `private` is nobody's.

**Reported limits are enforced limits.** A member decides whether to upload from these
numbers. A host reporting a limit it does not enforce — higher or lower — makes the
member's client wrong in a way the member cannot detect.

### Data carried

In: a room identifier and a presentation, which names the member on `open` and `attributed`
rooms and only proves membership on `private`. Out: tier, epoch, limits and counts — no
record, no file name, no member list.

### Correlation

On `open` and `attributed`, the host learns that this member looked, and when — as it learns
for every read. On `private`, only that some member did. The response itself carries
nothing about other members.

### Retention

Nothing new is stored. Hosts that log reads keep what they keep for every read.

### Consent/purpose

The member asks about a room they belong to, for the purpose of using it. The figures are
the host's own operating data about storage it provides; nothing personal is disclosed
beyond the member's own use, to the member.
