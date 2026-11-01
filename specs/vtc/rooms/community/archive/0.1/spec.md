---
slug: vtc/rooms/community/archive
version: "0.1"
title: "VTC Rooms — Community — Archive"
summary: "An administrator archives a data room the community owns: it admits no one and its host accepts no writes; members keep reading (readOnly) or have every credential withdrawn at the host until it is reopened (noAccess). The content is kept."
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
    Ends every member's ability to write to a room and, with `noAccess`, to reach it at all. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed archival could close a room administrators have since reopened.
sideEffects:
  level: mutating
  rationale: >-
    Stops the room's host accepting writes; withdraws every open offer; with `noAccess`, has the community's VTA reset the room's credentials at its host and re-issue the community's own; records the room's new state. Reversed by vtc/rooms/community/unarchive.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a room and an access; the response is the room as administrators see it, a count of withdrawn offers and the revocation list's head. No member DID.
retention:
  class: durable
  rationale: >-
    The room's state persists until changed; each change is kept in the community's audit trail with its reason.
errorCodes:
  - code: vtc/rooms/community/archive:notFound
    meaning: "The community owns no room with this identifier. A room it only hosts is `notFound` here too: its owner is someone else."
    retryable: false
  - code: vtc/rooms/community/archive:wrongState
    meaning: "The room is scheduled for removal, being purged, or removed, and cannot be archived. A scheduled removal is cancelled with vtc/rooms/community/removal/cancel, which leaves the room archived with `noAccess`. `details.state` is the room's state."
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
  - code: vtc/rooms/community/archive:keyHolderUnavailable
    meaning: "The community's VTA, or the room's host behind it, did not answer. If the room's credentials were already reset, the room shows a `pendingChange` and the community finishes it by itself; repeating the request finishes it too. Otherwise nothing changed."
    retryable: true
  - code: vtc/rooms/community/archive:keyHolderRefused
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
  - vtc/rooms/community/unarchive
  - vtc/rooms/community/remove
  - vtc/rooms/community/removal/cancel
  - vtc/rooms/community/get
  - vtc/rooms/community/members/remove
  - vtc/rooms/memberships/list
  - rooms/owner/revoke
  - rooms/owner/issue-membership
  - rooms/owner/issue-authority
  - rooms/revocations/put
---

## Abstract

A community's rooms outlive their purpose: the board's 2026 papers, a working group that finished, a members' library replaced by a new one. **VTC Rooms — Community — Archive** closes such a room to change without destroying it. An archived room admits no one, its open offers are withdrawn, and its host accepts no write, upload or curation from any member. The content stays exactly as it was, and [`vtc/rooms/community/unarchive`](../../../../../vtc/rooms/community/unarchive/0.1/spec.md) reopens it.

An administrator chooses what the room's members may still do — its **access**:

- **`readOnly`** — members keep reading. Their credentials stand; the host refuses every task but a read. This is the archive of a room whose record should stay available to the people who made it.
- **`noAccess`** — nobody but the community reaches the room. The community's VTA **resets** the room: it signs a revocation list withdrawing every credential the room issued before now, from everyone, and delivers it to the room's host ([`rooms/owner/revoke`](../../../../../rooms/owner/revoke/0.1/spec.md), one `room` withdrawal). Each admitted member becomes **`suspended`**. When the room leaves `noAccess`, the community has the room re-issue each suspended member's credentials, with the actions and expiry they had, and the member collects them as a renewal ([`vtc/rooms/offers/list`](../../../../../vtc/rooms/offers/list/0.4/spec.md) `renewals`). This is the archive of a room that should be kept but no longer read — the step before removal.

An archived room's access can be changed by archiving it again with the other access. A room is removed only from `archived` ([`vtc/rooms/community/remove`](../../../../../vtc/rooms/community/remove/0.1/spec.md)): archiving is the deliberate first step of removing, and the reversible one.

**What archiving means, and does not.** It does not recall anything a member already read or saved. On a sealed tier (`attributed`, `private`) it removes nobody from the room's group and advances no epoch: a suspended member still holds the keys to what they could open before, and is refused only the ciphertext — which is what makes restoring them a matter of new credentials rather than a new group. A reset is the only withdrawal a `private` room's host can apply, because it verifies membership without knowing who a member is; it is why `noAccess` is a reset on every tier rather than a withdrawal per member.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`, **MUST** answer `notFound` for a room it does not own, and **MUST** answer `wrongState` for a room scheduled for removal, being purged or removed.
2. **MUST**, when the room is already archived with `access` and no change is pending, answer `changed: false` and write, withdraw and audit nothing.
3. **MUST**, from the moment it answers `archived`, refuse at the room's host every task on the room but a read — writes, uploads, curation and profile writes — from every presenter, whatever their credentials confer, except the room's owner's own `admin` acts (an epoch mint, the delivery of a revocation list), which removing a member still needs. It **MUST NOT** delete, re-seal or hide anything the room holds.
4. **MUST** withdraw every open offer, so that it is never listed or accepted again, and record it `removed`. On an `open` or `attributed` room archived `readOnly`, it **MUST** first have its VTA withdraw the offered parties' credentials at the room's host — one [`rooms/owner/revoke`](../../../../../rooms/owner/revoke/0.1/spec.md) carrying a `subject` withdrawal for each, delivered to every host serving the room — and **MUST NOT** record anything until the host has accepted that list. On a `private` room archived `readOnly` an offer's credentials cannot be withdrawn one at a time; they are honoured at the host until their `validUntil`, and admit their holder to ciphertext they hold no key for.
5. **MUST**, for `noAccess`, have its VTA reset the room — [`rooms/owner/revoke`](../../../../../rooms/owner/revoke/0.1/spec.md) with a single `room` withdrawal, delivered to every host serving the room — and **MUST NOT** record the room `archived` with `noAccess` until the host has accepted it. A reset withdraws the community's own membership and `admin` authority too; the community **MUST** then have its VTA re-issue both ([`rooms/owner/issue-membership`](../../../../../rooms/owner/issue-membership/0.1/spec.md), [`rooms/owner/issue-authority`](../../../../../rooms/owner/issue-authority/0.2/spec.md)) issued after the reset's `revokedAt`, before it records the change. It **MUST** then record each `admitted` member `suspended`, and each open offer `removed` (item 4: the reset withdrew its credentials).
6. **MUST**, when a step at its VTA fails after the host accepted a reset, record a `pendingChange` naming the state and access the room is moving to, answer `keyHolderUnavailable` or `keyHolderRefused`, and finish the change itself — retrying with backoff — without a further request; a repeat of the request **MUST** resume the change where it stopped and **MUST NOT** reset the room a second time. While a change is pending, a request for any change of state on the room — by this task or any other — **MUST** first finish the pending change, and **MUST** be refused with `keyHolderUnavailable` or `keyHolderRefused` if it cannot; it is then judged against the state the room reached.
7. **MUST**, archiving `noAccess` → `readOnly`, restore every `suspended` member as [`vtc/rooms/community/unarchive`](../../../../../vtc/rooms/community/unarchive/0.1/spec.md) does, and **MUST NOT** restore an offer. A room moving `readOnly` → `noAccess` is reset as in item 5.
8. **MUST** keep renewing the community's own authority over an archived room, as for an active one, so that it can still be unarchived, have members removed, and be removed. It **SHOULD** keep renewing `admitted` members' credentials on a `readOnly` room, so that reading keeps working, and **MUST NOT** renew a `suspended` member's.
9. **MUST NOT** write the room's profile while it is archived; a profile write owed from before is written when it is unarchived.
10. **MUST**, when anything changed, set `status.changedAt` and `status.changedBy`, write one audit record carrying the requester, the room, the previous and new state and access, the count of withdrawn offers and of suspended members, and `reason`, and answer `changed: true`.

A conforming **producer** (the administrator's surface):

1. **SHOULD** show the requester, before the request is signed — and before any step-up gesture bound to it — what archiving does with the chosen access and what it does not (*What archiving means, and does not*, above), and how many open offers it withdraws.
2. **MUST NOT** report an archival as recalling anything a member already read, or, on a sealed tier, as removing anyone from the room's group.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, describe them, admit and remove their members, archive and remove them. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.2/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder.** The community acts on the room by sending its own VTA the `rooms/owner/*` tasks any owner's key holder serves, as itself; the VTA applies its own gates to each. The community holds no room key and no group secret. That the community is also the room's host is what lets it stop accepting writes: it does so as the host, for the room's owner, and never by consulting the owner's record of who is a member.

**The model this task is designed for.** A community implementing administrator step-up binds this operation to a step-up of the requester's (a gesture bound to this one document), records it in its audit trail with its `reason`, and executes it without a second administrator's consent: the room is the community's own, and archiving is reversible. A community whose approvals rules require consent for it parks it as it parks any consented act, answering with `trust-task-next-step` naming the action. Whether a step-up or consent is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- `CommunityRoomV3`, `RoomStatus`, `ArchiveAccess`, `PendingChange`, `MemberStateV2` — see [`vtc/_shared/0.5/community-room.schema.json`](../../../../../vtc/_shared/0.5/community-room.schema.json); `Reason` — see [`vtc/_shared/0.1/community-room.schema.json`](../../../../../vtc/_shared/0.1/community-room.schema.json); `RevocationHead` — see [`rooms/_shared/0.1/room.schema.json`](../../../../../rooms/_shared/0.1/room.schema.json).
- **Reset** — a revocation list carrying a `room` withdrawal: every credential the room issued before its `revokedAt`, from everyone.

## Request

An administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/archive/0.1#request",
  "issuer": "did:example:secretary",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-02T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "access": "noAccess",
    "reason": "The 2026 board cycle closed; the papers are kept but no longer read."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/archive/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:secretary",
  "issuedAt": "2026-11-02T10:00:02Z",
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
        "changedAt": "2026-11-02T10:00:02Z",
        "changedBy": "did:example:secretary"
      },
      "tags": ["board"],
      "storageConfig": "local",
      "admitted": 0,
      "offered": 0,
      "suspended": 6,
      "ownerAuthorityValidUntil": "2027-11-02T10:00:01Z",
      "createdBy": "did:example:administrator"
    },
    "changed": true,
    "withdrawnOffers": 1,
    "revocation": {
      "sequence": 3,
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "revokedAt": "2026-11-02T10:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

A room identifier, an access and a reason in; the room as administrators see it, counts, and the revocation list's head out. No member DID. A reset names nobody to the room's host; a `readOnly` archival of an `open` or `attributed` room names the offered parties to it in the list that withdraws their credentials — parties it would have seen act had they accepted.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks.

### Retention

The room's state persists until changed; each change, with its reason, is in the community's audit trail.

### Consent/purpose

Archiving is the owner's, and needs no consent from the room's members. What it withdraws is restored when the room is unarchived, except open offers, which an administrator makes again if they are still wanted.
