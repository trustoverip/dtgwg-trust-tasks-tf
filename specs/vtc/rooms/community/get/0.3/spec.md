---
slug: vtc/rooms/community/get
version: "0.3"
title: "VTC Rooms — Community — Get"
summary: "An administrator reads one data room the community itself owns: its host record, label, description, tags, storage config, member counts and when the community's own authority over it expires."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Authority is the signer's `vtc.rooms.manage` capability, which the community can establish from a verified proof or a transport-authenticated sender. A proof is recommended so the read is attributable on every transport.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that discloses no member; a bounded `issuedAt` still keeps a captured request from being replayed indefinitely.
sideEffects:
  level: none
  rationale: >-
    Reads one room. Persists nothing.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The room's host record, label, description, tags, the administrators who created it and last changed its details, and counts. No member DIDs.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/rooms/community/get:notFound
    meaning: "The community owns no room with this identifier. A room it only hosts is `notFound` here too: its owner is someone else."
    retryable: false
related:
  - vtc/rooms/community/members/list
  - vtc/rooms/community/list
  - vtc/rooms/community/update
  - vtc/rooms/get
---

## Abstract

**VTC Rooms — Community — Get** returns one room the community owns, as [`vtc/rooms/community/list`](../../../../../vtc/rooms/community/list/0.2/spec.md) describes each, including `ownerAuthorityValidUntil`: when the community's own `admin` authority over the room lapses, after which it can no longer advance the room's epoch, admit or remove. The room's members are [`vtc/rooms/community/members/list`](../../../../../vtc/rooms/community/members/list/0.1/spec.md)'s, a paged read, because a community library may admit the whole community. The host's view of the same room — storage placement, limits, usage — is [`vtc/rooms/get`](../../../../../vtc/rooms/get/0.1/spec.md)'s.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.2

The response's `room` is still a `CommunityRoomV2`, now re-pinned from [`vtc/_shared/0.1/community-room.schema.json`](../../../../../vtc/_shared/0.1/community-room.schema.json) to [`vtc/_shared/0.3/community-room.schema.json`](../../../../../vtc/_shared/0.3/community-room.schema.json). The definition is otherwise identical; what changes is its host record, `room` — `HostedRoom`, now from [`vtc/_shared/0.3/room-storage.schema.json`](../../../../../vtc/_shared/0.3/room-storage.schema.json):

- `room.room.epochExpiresAt`: integer seconds since the Unix epoch → RFC 3339 `date-time` string (still optional; absent still means the room never lapses).
- `room.room.createdAt`: integer seconds → RFC 3339 `date-time` string (still required).
- `room.room.updatedAt`: integer seconds → RFC 3339 `date-time` string (still required).

The names are unchanged: each already ends in `At`. `epoch` (a key-epoch counter) and `retentionDays` (a duration) stay integers. The reason is the registry-wide convention that an instant is an RFC 3339 string named `*At` — the rest of `CommunityRoomV2` (`ownerAuthorityValidUntil`, `detailsUpdatedAt`) already was. Every other `$ref` stays where it was, and nothing else in the request or response moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains published and pinned to `vtc/_shared/0.1`.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`.
2. **MUST** answer `notFound` for a room it does not own, including one it only hosts.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, admit and remove their members. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores), because a community may let someone run its board room without letting them reassign every room's storage, and the reverse; [`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place either in a role's ceiling. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder.** The community is the room's owner, and its own VTA holds the room's signing key and, on a sealed tier, the community's place in the room's group. The community acts on the room by sending its VTA the same `rooms/owner/*` and `rooms/keys/*` tasks any owner's key holder serves, as itself; the VTA applies its own gates to each. The community holds no room key and no group secret.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/get/0.3#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-14T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board"
  }
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/get/0.3#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-14T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "room": {
      "room": {
        "roomId": "did:webvh:example.com:rooms:board",
        "ownerDid": "did:example:community",
        "visibility": "private",
        "retentionPolicy": "chained",
        "epoch": 1,
        "lifecycle": "live",
        "epochExpiresAt": "2026-10-19T00:00:00Z",
        "retentionDays": 90,
        "createdAt": "2025-10-14T10:00:00Z",
        "updatedAt": "2025-10-14T10:00:00Z"
      },
      "label": "Board pack",
      "description": "Papers for the next board meeting: agenda, minutes in draft, and the finance pack.",
      "tags": ["board"],
      "storageConfig": "local",
      "admitted": 0,
      "offered": 0,
      "ownerAuthorityValidUntil": "2027-10-14T10:00:00Z",
      "createdBy": "did:example:administrator",
      "detailsUpdatedAt": "2026-10-15T09:30:00Z",
      "detailsUpdatedBy": "did:example:secretary"
    }
  }
}
```

## Security & Privacy

### Data carried

A host record, label, description, tags and counts for a room the community owns, and the DIDs of the administrators who created it and last changed its details — administrators of the same community, shown to holders of the same capability. No member DID, key or credential.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. The answer links nothing a member did to any room.

### Retention

None; a read.

### Consent/purpose

Shown so the owner can run its rooms.
