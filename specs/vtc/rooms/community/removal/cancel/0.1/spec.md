---
slug: vtc/rooms/community/removal/cancel
version: "0.1"
title: "VTC Rooms — Community — Removal — Cancel"
summary: "An administrator cancels the scheduled removal of a data room the community owns, before its grace period ends: the room returns to archived with noAccess, its content intact, and nothing is purged."
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
  requirement: REQUIRED
  rationale: >-
    Keeps a room's content that an administrator scheduled for destruction. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed cancellation could keep a room administrators have since scheduled for removal again.
sideEffects:
  level: mutating
  rationale: >-
    Records the room `archived` with `noAccess` and clears its `purgeAt`. Changes nothing at the room's host or its VTA.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a room; the response is the room as administrators see it. No member DID.
retention:
  class: durable
  rationale: >-
    The room's state persists until changed; each change is kept in the community's audit trail with its reason.
errorCodes:
  - code: vtc/rooms/community/removal/cancel:notFound
    meaning: "The community owns no room with this identifier. A room it only hosts is `notFound` here too: its owner is someone else."
    retryable: false
  - code: vtc/rooms/community/removal/cancel:wrongState
    meaning: "The room is not scheduled for removal: there is nothing to cancel in `active` or `archived`, and a purge cannot be cancelled once it has begun. `details.state` is the room's state."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required:
        - state
      properties:
        state:
          type: string
          enum:
            - active
            - archived
            - purging
            - removed
related:
  - vtc/rooms/community/remove
  - vtc/rooms/community/archive
  - vtc/rooms/community/unarchive
  - vtc/rooms/community/get
---

## Abstract

A removal ([`vtc/rooms/community/remove`](../../../../../../vtc/rooms/community/remove/0.1/spec.md)) with a grace period leaves the room `removalScheduled` until `purgeAt`, its content intact and nobody but the community able to reach it. **VTC Rooms — Community — Removal — Cancel** is the window's other end: it keeps the room. The room returns to **`archived`** with **`noAccess`** — the state a scheduled removal holds it in — and its suspended members stay suspended. To reopen the room to them, an administrator then archives it `readOnly` or unarchives it ([`vtc/rooms/community/archive`](../../../../../../vtc/rooms/community/archive/0.1/spec.md), [`vtc/rooms/community/unarchive`](../../../../../../vtc/rooms/community/unarchive/0.1/spec.md)), each a separate, deliberate act.

Cancelling is possible only until the purge begins. Once the room is `purging`, its content is being destroyed, and nothing undoes that.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`, and **MUST** answer `notFound` for a room it does not own.
2. **MUST** answer `wrongState` for a room that is not `removalScheduled` — including one whose `purgeAt` has passed and whose purge has begun. The decision whether the purge has begun and the cancellation are one atomic step: a room is never both cancelled and purged.
3. **MUST** record the room `archived` with `access` `noAccess`, clear `purgeAt`, and **MUST NOT** begin its purge.
4. **MUST NOT** restore any suspended member, lift the room's write freeze, or ask the room's host or its VTA for anything: a cancellation returns the room to the state the removal held it in, and nothing more.
5. **MUST** set `status.changedAt` and `status.changedBy`, and write one audit record carrying the requester, the room, the `purgeAt` it cancelled and `reason`.

A conforming **producer** (the administrator's surface):

1. **SHOULD** show the requester, before the request is signed, that the room returns to `archived` with `noAccess`, and that reopening it to its members is a separate act.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, describe them, admit and remove their members, archive and remove them. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../../auth/signing-key/enroll/0.2/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**The model this task is designed for.** A cancellation destroys nothing and grants nothing — it keeps what a removal would have destroyed, in a state no member can reach. A community records it in its audit trail with its `reason` and executes it on the requester's authority alone, **without** an operation-bound step-up and without a second administrator's consent, so that stopping a mistaken removal is never harder than making it. A community whose policy asks for more applies it as it does to any act; whether either is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- `CommunityRoomV3`, `RoomStatus` — see [`vtc/_shared/0.5/community-room.schema.json`](../../../../../../vtc/_shared/0.5/community-room.schema.json); `Reason` — see [`vtc/_shared/0.1/community-room.schema.json`](../../../../../../vtc/_shared/0.1/community-room.schema.json).

## Request

An administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/removal/cancel/0.1#request",
  "issuer": "did:example:secretary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-10T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "reason": "The auditors asked for the 2026 papers to be kept until the audit closes."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/removal/cancel/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:secretary",
  "issuedAt": "2026-12-10T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "room": {
      "room": {
        "roomId": "did:webvh:example.com:rooms:board",
        "ownerDid": "did:example:community",
        "visibility": "attributed",
        "retentionPolicy": "chained",
        "epoch": 4,
        "lifecycle": "live",
        "epochExpiresAt": "2027-10-14T10:00:00Z",
        "retentionDays": 90,
        "createdAt": "2025-10-14T10:00:00Z",
        "updatedAt": "2026-10-30T16:00:00Z"
      },
      "label": "Board pack — 2026",
      "status": {
        "state": "archived",
        "access": "noAccess",
        "changedAt": "2026-12-10T09:00:01Z",
        "changedBy": "did:example:secretary"
      },
      "tags": ["board"],
      "storageConfig": "local",
      "admitted": 0,
      "offered": 0,
      "suspended": 6,
      "ownerAuthorityValidUntil": "2027-12-01T09:00:01Z",
      "createdBy": "did:example:administrator"
    }
  }
}
```

## Security & Privacy

### Data carried

A room identifier and a reason in; the room as administrators see it out. No member DID.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. A cancellation sends nothing to the room's host or its VTA.

### Retention

The room's state persists until changed; each change, with its reason, is in the community's audit trail.

### Consent/purpose

Keeping a room is the owner's, and needs no consent from its members, who cannot reach it until it is reopened.
