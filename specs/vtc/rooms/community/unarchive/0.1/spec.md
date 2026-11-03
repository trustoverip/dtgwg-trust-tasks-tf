---
slug: vtc/rooms/community/unarchive
version: "0.1"
title: "VTC Rooms — Community — Unarchive"
summary: "An administrator reopens an archived data room the community owns: its host accepts its members' writes again, and members whose credentials a noAccess archival withdrew have the room re-issue them, with the actions and expiry they had."
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
    Restores members' write access to a room and, after a `noAccess` archival, has credentials signed in the room's name. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed unarchival could reopen a room administrators have since archived again.
sideEffects:
  level: mutating
  rationale: >-
    Lifts the room's write freeze at its host; records the room `active`; has the community's VTA re-issue each suspended member's credentials, in the background, for the member to collect as a renewal.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a room; the response is the room as administrators see it, with counts. No member DID, and no credential.
retention:
  class: durable
  rationale: >-
    The room's state persists until changed; each change is kept in the community's audit trail with its reason.
errorCodes:
  - code: vtc/rooms/community/unarchive:notFound
    meaning: "The community owns no room with this identifier. A room it only hosts is `notFound` here too: its owner is someone else."
    retryable: false
  - code: vtc/rooms/community/unarchive:wrongState
    meaning: "The room is not archived. `active` needs nothing; a room scheduled for removal is first returned to `archived` with vtc/rooms/community/removal/cancel; a room being purged or removed cannot be reopened. `details.state` is the room's state."
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
            - removalScheduled
            - purging
            - removed
  - code: vtc/rooms/community/unarchive:keyHolderUnavailable
    meaning: "A change the room was already part-way through (its `pendingChange`) could not be finished because the community's VTA, or the room's host behind it, did not answer. The community keeps trying by itself; nothing of this request changed."
    retryable: true
  - code: vtc/rooms/community/unarchive:keyHolderRefused
    meaning: "The community's VTA refused a step of a change the room was already part-way through (its `pendingChange`). Its own code and message are carried in `details`. Nothing of this request changed."
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
  - vtc/rooms/community/remove
  - vtc/rooms/community/removal/cancel
  - vtc/rooms/community/get
  - vtc/rooms/offers/list
  - vtc/rooms/memberships/list
  - rooms/owner/issue-membership
  - rooms/owner/issue-authority
---

## Abstract

**VTC Rooms — Community — Unarchive** reopens a room archived with [`vtc/rooms/community/archive`](../../../../../vtc/rooms/community/archive/0.1/spec.md). The room becomes `active`: its host accepts its members' writes again, and administrators can admit members to it again.

What it gives back depends on how the room was archived:

- **`readOnly`** — nothing was withdrawn from an admitted member, so nothing needs restoring: the write freeze lifts and the members who could read can write again, as their credentials say.
- **`noAccess`** — the archival reset the room, withdrawing every credential it had issued, and recorded each admitted member **`suspended`**. Unarchiving has the community's VTA re-issue each suspended member's membership and authority ([`rooms/owner/issue-membership`](../../../../../rooms/owner/issue-membership/0.1/spec.md), [`rooms/owner/issue-authority`](../../../../../rooms/owner/issue-authority/0.2/spec.md)), with the **same actions and expiry** they had. The member collects them as a renewal ([`vtc/rooms/offers/list`](../../../../../vtc/rooms/offers/list/0.4/spec.md) `renewals`), the way they collect any renewal of their admission, and is `admitted` again. On a sealed tier they never left the room's group, so new credentials are all they need. An admission whose expiry has passed meanwhile is not restored: it is recorded `removed`, and an administrator admits the member again if they are still wanted.

Restoring is done **in the background**. A room may admit the whole community, and each restoration is a signature at the community's VTA; the answer does not wait for them. `status.restoring` counts the admissions still to restore, and [`vtc/rooms/community/members/list`](../../../../../vtc/rooms/community/members/list/0.2/spec.md) shows each member's state.

**What unarchiving does not restore.** An archival withdrew every open offer, and they stay withdrawn: an offer is an invitation an administrator made at a moment, and reopening a room is not the same decision. Nothing a member was removed from while the room was archived is restored either.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`, and **MUST** answer `notFound` for a room it does not own.
2. **MUST**, when the room has a `pendingChange`, first finish it as [`vtc/rooms/community/archive`](../../../../../vtc/rooms/community/archive/0.1/spec.md) requires, and **MUST** refuse with `keyHolderUnavailable` or `keyHolderRefused`, changing nothing of this request, if it cannot. The request is then judged against the state the room reached.
3. **MUST**, when the room is already `active`, answer `changed: false` and write and audit nothing; and **MUST** answer `wrongState` for a room scheduled for removal, being purged or removed.
4. **MUST**, from the moment it answers, accept at the room's host the writes, uploads and curation its members' credentials confer, as for any `active` room.
5. **MUST**, for each member `suspended`, have its VTA re-issue their membership and authority credentials — the authority granting exactly the actions their admission granted, until the same `validUntil` — and record them as the member's renewal, collectable from the next [`vtc/rooms/offers/list`](../../../../../vtc/rooms/offers/list/0.4/spec.md) read, and only then record the member `admitted`. It **MUST NOT** restore an admission whose `validUntil` has passed; it **MUST** record it `removed` instead. It **MUST NOT** issue an invitation, and **MUST NOT** restore an offer.
6. **MAY** restore suspended members after it answers, and **SHOULD** for a room with more than a few; it **MUST** then count those still to restore in `status.restoring`, keep trying with backoff — a VTA that does not answer delays a restoration and never drops it — and **MUST NOT** report the room as having none to restore until every one is `admitted` or `removed`. A member removed while their restoration is pending is not restored.
7. **MUST** write the room's profile if a write was owed, as [`vtc/rooms/community/update`](../../../../../vtc/rooms/community/update/0.3/spec.md) requires, and keep it current from then on.
8. **MUST** set `status.changedAt` and `status.changedBy`, write one audit record carrying the requester, the room, the state and access it left, the count of suspended members it is restoring and `reason`, and answer `changed: true`.

A conforming **producer** (the administrator's surface):

1. **SHOULD** show the requester, before the request is signed — and before any step-up gesture bound to it — what unarchiving restores and what it does not: on a `noAccess` room, how many suspended members it restores and that each collects new credentials as a renewal; on any room, that the offers the archival withdrew stay withdrawn.
2. **SHOULD**, while `status.restoring` is present, show how many admissions are still being restored.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, describe them, admit and remove their members, archive and remove them. It is distinct from `vtc.rooms.admin`, the community's authority as a **host**. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.2/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder.** The credentials restored are signed by the community's own VTA, which holds the room's signing key, on `rooms/owner/*` tasks it serves to any owner; the VTA applies its own gates to each. The community holds no room key and no group secret.

**The model this task is designed for.** A community implementing administrator step-up binds this operation to a step-up of the requester's (a gesture bound to this one document), records it in its audit trail with its `reason`, and executes it without a second administrator's consent: it gives back what an archival of the same room took, to the same members, and nothing more. A community whose approvals rules require consent for it parks it as it parks any consented act, answering with `trust-task-next-step` naming the action. Whether a step-up or consent is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- `CommunityRoomV3`, `RoomStatus`, `PendingChange`, `MemberStateV2` — see [`vtc/_shared/0.5/community-room.schema.json`](../../../../../vtc/_shared/0.5/community-room.schema.json); `Reason` — see [`vtc/_shared/0.1/community-room.schema.json`](../../../../../vtc/_shared/0.1/community-room.schema.json).
- **Restoring** a member — having the room re-issue a suspended member's membership and authority with the actions and expiry of their admission, recorded as their renewal.

## Request

An administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/unarchive/0.1#request",
  "issuer": "did:example:secretary",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-20T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "reason": "The auditors need the 2026 papers; the board reopens the room for them."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/unarchive/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:secretary",
  "issuedAt": "2026-11-20T09:00:01Z",
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
        "state": "active",
        "changedAt": "2026-11-20T09:00:01Z",
        "changedBy": "did:example:secretary",
        "restoring": 6
      },
      "tags": ["board"],
      "storageConfig": "local",
      "admitted": 0,
      "offered": 0,
      "suspended": 6,
      "ownerAuthorityValidUntil": "2027-11-02T10:00:01Z",
      "createdBy": "did:example:administrator"
    },
    "changed": true
  }
}
```

## Security & Privacy

### Data carried

A room identifier and a reason in; the room as administrators see it, with counts, out. No member DID and no credential: restored credentials are kept for each member to collect, never returned to the administrator.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. Restoring sends the room's host nothing that names a member: re-issued credentials reach it only when their holders present them.

### Retention

The room's state persists until changed; each change, with its reason, is in the community's audit trail. A restored member's renewal is kept until they collect it and the credentials it replaces have expired, as any renewal is.

### Consent/purpose

Reopening is the owner's, and needs no consent from the room's members: it restores to each exactly what they held before the archival.
