---
slug: vtc/rooms/community/remove
version: "0.1"
title: "VTC Rooms — Community — Remove"
summary: "An administrator removes an archived data room the community owns: every credential is withdrawn at its host, and after a grace period — or at once — its content is purged from the host and from storage, its identity retired, and only a record that it existed is kept."
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
    Schedules the destruction of a room's content, at once when `graceDays` is `0`. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed removal could destroy a room whose scheduled removal administrators have since cancelled.
sideEffects:
  level: destructive
  rationale: >-
    Withdraws every credential at the room's host if that is not already done; records the room `removalScheduled` and, when the grace period ends, purges every record, epoch, commit, revocation list and file of the room from its host and storage, drops every credential the community keeps for its members, and asks the community's VTA to delete the room's identity. Cancellable until the purge begins; never after.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a room, a grace period and a reason; the response is the room as administrators see it and the revocation list's head. No member DID.
retention:
  class: durable
  rationale: >-
    The room's content is destroyed; the community keeps the owner's record of the room in state `removed`, its members' records without credentials, and the audit trail, as its history.
errorCodes:
  - code: vtc/rooms/community/remove:notFound
    meaning: "The community owns no room with this identifier. A room it only hosts is `notFound` here too: its owner is someone else."
    retryable: false
  - code: vtc/rooms/community/remove:wrongState
    meaning: "The room cannot be removed in its state. An `active` room is archived first (vtc/rooms/community/archive): removal is the second of two deliberate acts. A room being purged or removed is already going or gone. `details.state` is the room's state."
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
            - purging
            - removed
  - code: vtc/rooms/community/remove:keyHolderUnavailable
    meaning: "The community's VTA, or the room's host behind it, did not answer. If the room's credentials were already reset, the room shows a `pendingChange` and the community finishes it by itself; repeating the request finishes it too. Otherwise nothing changed."
    retryable: true
  - code: vtc/rooms/community/remove:keyHolderRefused
    meaning: "The community's VTA refused a step. Its own code and message are carried in `details`. If the room's credentials were already reset, the room shows a `pendingChange`; otherwise nothing changed."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        code:
          type: string
          maxLength: 128
        message:
          type: string
          maxLength: 500
related:
  - vtc/rooms/community/archive
  - vtc/rooms/community/removal/cancel
  - vtc/rooms/community/unarchive
  - vtc/rooms/community/get
  - vtc/rooms/memberships/list
  - rooms/owner/revoke
  - rooms/revocations/put
  - vta/webvh/dids/delete
---

## Abstract

**VTC Rooms — Community — Remove** ends a room the community owns. It is the second of two deliberate acts: a room is removed only once it is archived ([`vtc/rooms/community/archive`](../../../../../vtc/rooms/community/archive/0.1/spec.md)), so that closing a room and destroying it are never one gesture.

Removal runs in three stages, each visible in the room's `status`:

1. **`removalScheduled`** — nobody but the community reaches the room. If it was archived `readOnly`, the community's VTA first resets it exactly as a `noAccess` archival does ([`rooms/owner/revoke`](../../../../../rooms/owner/revoke/0.1/spec.md), one `room` withdrawal), and every admitted member becomes `suspended`. The content is kept until `purgeAt`, the end of a **grace period** — `graceDays` from now, or the community's default — during which [`vtc/rooms/community/removal/cancel`](../../../../../vtc/rooms/community/removal/cancel/0.1/spec.md) returns the room to `archived`, intact. A second removal of a scheduled room reschedules it: `graceDays: 0` on a scheduled room purges it now.
2. **`purging`** — the grace period ended, or `graceDays` was `0`. The community deletes the room's content from its host and from every storage config holding its files, drops what it kept for the room's members, stops serving the room's identity, and asks its VTA to delete it. Some of these steps are remote and can fail; the community retries until each has succeeded, and shows how it is going (`status.purge`). **A purge is not reversible**, and cannot be cancelled once begun.
3. **`removed`** — the content is gone. The community keeps a record that the room existed, and nothing in it.

**What removal means, and does not.** Removal destroys the room's copy of its content at this host and at the storage it used — and with it, the means for anyone, members and administrators alike, to read it from here again. It does not recall anything a member already read or saved, and it cannot reach a copy a member exported or another host mirrors. On a sealed tier a member's key holder may still hold the keys of epochs they joined; with the ciphertext gone from the host, those keys open nothing it serves.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`, and **MUST** answer `notFound` for a room it does not own.
2. **MUST**, when the room has a `pendingChange`, first finish it as [`vtc/rooms/community/archive`](../../../../../vtc/rooms/community/archive/0.1/spec.md) requires, and **MUST** refuse with `keyHolderUnavailable` or `keyHolderRefused` if it cannot. The request is then judged against the state the room reached.
3. **MUST** answer `wrongState` for a room that is `active`, `purging` or `removed`. It **MUST NOT** archive a room as part of removing it.
4. **MUST**, for a room archived `readOnly`, reset it and re-issue its own membership and authority exactly as an archival with `noAccess` does — the host accepting the reset before anything is recorded, and a `pendingChange` recorded if a later step fails — and record each `admitted` member `suspended`. A room archived `noAccess` is already reset and **MUST NOT** be reset again.
5. **MUST** record the room `removalScheduled` with `purgeAt` set to now plus `graceDays` — or now plus the community's configured default when `graceDays` is absent (30 days in the reference implementation) — and **MUST** do the same for a room already `removalScheduled`, replacing its `purgeAt`. When `graceDays` is `0` it **MUST** instead record the room `purging` and begin the purge; it **MAY** answer before the purge finishes, and **SHOULD**.
6. **MUST** begin the purge of a `removalScheduled` room once `purgeAt` has passed, without a further request, and record the room `purging` when it does.
7. **MUST**, in a purge, delete everything the room's host holds for the room — its records, its epoch links and commits, its revocation list, its host record — and every file the room stored, at every storage config holding one, with the index, usage and transfer records that name them and the room's storage assignment; drop every credential it keeps for the room's members (open offers, kept Welcomes, renewals) and every removal of a member it still owed; remove the room from every group room binding naming it; and stop serving a log it served for a serverless room identity. Before deleting the host record it **MUST** snapshot it, and keep the snapshot with the room's record: it is the `room` a `purging` or `removed` room is shown with, its `lifecycle` as computed when the purge began.
8. **SHOULD**, in a purge, have its VTA delete the room's identity ([`vta/webvh/dids/delete`](../../../../../vta/webvh/dids/delete/1.0/spec.md)). The community's own entry at its VTA may not hold the authority that task requires on the identity's context, and the VTA refuses it while anything there still depends on the identity; so a refusal or failure of this one step **MUST NOT** hold the purge or keep the room from `removed`. The community **MUST** record its outcome with the room — the reference implementation reports it under `ext` — so that an operator can retire the identity at the VTA by hand.
9. **MUST** record the room `removed`, with `removedAt`, only once every step of item 7 has succeeded. A step that fails **MUST** be retried with backoff, recorded in `status.purge` (attempts, the last error, when the next attempt is due), and **MUST NOT** be skipped, or reported done, because it keeps failing. A purge **MUST NOT** be cancelled or reversed once begun.
10. **MUST** keep, for a removed room, the owner's record — its label, description, tags, creation, and `status` — and each member's record with its state and dates but **no** credential, and **MUST NOT** restore a `suspended` member of a room being purged or removed. It **MUST NOT** keep anything of the room's content.
11. **MUST** keep renewing its own authority over a room `removalScheduled`, so that a cancelled removal leaves a room it can still act on, and **MUST** stop once the room is `purging`.
12. **MUST** write one audit record for the removal, carrying the requester, the room, the state and access it left, `purgeAt`, the count of suspended members, and `reason`; one when the purge begins; and one when the room is `removed`, the last two attributed to the administrator who scheduled the removal.

A conforming **producer** (the administrator's surface):

1. **SHOULD** require the requester to type the room's label before the request is signed — and before any step-up gesture bound to it — and **SHOULD** show them `purgeAt`, that the room can be returned to `archived` until then, and what removal does and does not do (*What removal means, and does not*, above).
2. **MUST**, when `graceDays` is `0`, say before the request is signed that the content is purged at once and the removal cannot be undone.
3. **MUST NOT** report a removal as recalling anything a member already read or saved.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, describe them, admit and remove their members, archive and remove them. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.2/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder and as the room's host.** The community resets the room and asks for its identity to be deleted through its own VTA, which holds the room's keys and applies its own gates to each task; it deletes the content as the room's host, at its owner's request. The community holds no room key and no group secret.

**The model this task is designed for.** A community implementing administrator step-up binds this operation to a step-up of the requester's (a gesture bound to this one document), records it in its audit trail with its `reason`, and executes it without a second administrator's consent: the room is the community's own, it was archived first by a separate act, and the grace period is the window in which a mistaken removal is undone. A community that wants more for an act that destroys content — a second administrator's consent, at least for `graceDays: 0` — parks it as it parks any consented act, answering with `trust-task-next-step` naming the action. Whether a step-up or consent is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- `CommunityRoomV3`, `RoomStatus`, `PurgeProgress`, `PendingChange`, `MemberStateV2` — see [`vtc/_shared/0.5/community-room.schema.json`](../../../../../vtc/_shared/0.5/community-room.schema.json); `Reason` — see [`vtc/_shared/0.1/community-room.schema.json`](../../../../../vtc/_shared/0.1/community-room.schema.json); `RevocationHead` — see [`rooms/_shared/0.1/room.schema.json`](../../../../../rooms/_shared/0.1/room.schema.json).
- **Grace period** — the time between a removal and its purge, in which the removal can be cancelled.
- **Purge** — the deletion of item 7.

## Request

An administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Remove after a grace period

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/remove/0.1#request",
  "issuer": "did:example:secretary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-01T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "graceDays": 30,
    "reason": "Superseded by the 2027 board pack; the retention period for the 2026 papers has ended."
  }
}
```

### Remove at once

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/remove/0.1#request",
  "issuer": "did:example:secretary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-01T09:05:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:scratch",
    "graceDays": 0,
    "reason": "Created by mistake; holds nothing."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Scheduled, from `readOnly`

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/remove/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:secretary",
  "issuedAt": "2026-12-01T09:00:02Z",
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
        "state": "removalScheduled",
        "access": "noAccess",
        "changedAt": "2026-12-01T09:00:02Z",
        "changedBy": "did:example:secretary",
        "purgeAt": "2026-12-31T09:00:02Z"
      },
      "tags": ["board"],
      "storageConfig": "local",
      "admitted": 0,
      "offered": 0,
      "suspended": 6,
      "ownerAuthorityValidUntil": "2027-12-01T09:00:01Z",
      "createdBy": "did:example:administrator"
    },
    "revocation": {
      "sequence": 3,
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "revokedAt": "2026-12-01T09:00:01Z"
  }
}
```

### Purging at once

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/remove/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:secretary",
  "issuedAt": "2026-12-01T09:05:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "room": {
      "room": {
        "roomId": "did:webvh:example.com:rooms:scratch",
        "ownerDid": "did:example:community",
        "visibility": "open",
        "retentionPolicy": "fromJoin",
        "epoch": 0,
        "lifecycle": "live",
        "retentionDays": 90,
        "createdAt": "2026-11-30T15:00:00Z",
        "updatedAt": "2026-11-30T15:00:00Z"
      },
      "label": "Scratch",
      "status": {
        "state": "purging",
        "changedAt": "2026-12-01T09:05:01Z",
        "changedBy": "did:example:secretary",
        "purgeAt": "2026-12-01T09:05:01Z",
        "purge": {
          "attempts": 0
        }
      },
      "storageConfig": "local",
      "admitted": 0,
      "offered": 0,
      "ownerAuthorityValidUntil": "2027-11-30T15:00:00Z",
      "createdBy": "did:example:secretary"
    }
  }
}
```

## Security & Privacy

### Data carried

A room identifier, a grace period and a reason in; the room as administrators see it and the revocation list's head out. No member DID. A reset names nobody to the room's host.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. Deleting the room's identity ends its resolution; nothing about its members is published by that.

### Retention

The content is destroyed at the purge. The community keeps the room's record, its members' records without credentials, and the audit trail, as its history of the room; how long it keeps them is its own retention policy.

### Consent/purpose

Removal is the owner's, and needs no consent from the room's members — who, from the room's archival with `noAccess` or from the removal itself, could no longer reach it. A community that wants members to be able to keep a copy archives the room `readOnly` and gives notice before it removes it.
