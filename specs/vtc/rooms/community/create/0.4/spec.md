---
slug: vtc/rooms/community/create
version: "0.4"
title: "VTC Rooms — Community — Create"
summary: "An administrator creates a data room the community itself owns, with a label and a description: the community's VTA mints its identity and holds its keys, the community hosts it, and the label and description are sealed into the room as its profile for members."
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
    Creates a room the community will answer for, at a tier fixed for its life, and has the community's key holder mint an identity and sign credentials in its name. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed create would mint a second room.
sideEffects:
  level: mutating
  rationale: >-
    Has the community's VTA mint a DID and sign credentials, founds a group on a sealed tier, stores the room, and has the VTA write the room's profile record into it.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a label, an optional description and tags, a tier and a retention choice. The response is the room's host record and counts, and whether the room's profile was written; never a key, a group secret or a credential.
retention:
  class: durable
  rationale: >-
    The room persists under the lifecycle every hosted room follows.
errorCodes:
  - code: vtc/rooms/community/create:visibilityNotPermitted
    meaning: "The community's rooms policy does not let it own a room at this tier."
    retryable: false
  - code: vtc/rooms/community/create:unknownStorageConfig
    meaning: "`storageConfig` names no config, or one that is not `active`."
    retryable: false
  - code: vtc/rooms/community/create:noKeyHolder
    meaning: "The community has no link to a VTA that can hold the room's keys. Its host's operator configures one."
    retryable: false
  - code: vtc/rooms/community/create:keyHolderUnavailable
    meaning: "The community's VTA did not answer. Nothing was stored; a DID it may have minted is named in `details.roomId` so an operator can retire it."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        roomId:
          type: string
          maxLength: 512
  - code: vtc/rooms/community/create:keyHolderRefused
    meaning: "The community's VTA refused a step. Its own code and message are carried in `details`. Nothing was stored."
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
  - vtc/rooms/community/get
  - vtc/rooms/community/update
  - vtc/rooms/community/members/add
  - rooms/owner/register
  - rooms/owner/group/create
  - rooms/owner/issue-authority
  - rooms/keys/profile/write
  - vtc/rooms/list
---

## Abstract

A community hosts rooms its members own, and it can also **own** rooms itself — the board's working papers, a working group's drafts, a members' library — run by its administrators and outliving any of them. **VTC Rooms — Community — Create** makes one.

The community is the room's owner, so the room's identity and its keys must be held by a key holder acting for the community, and that is **the community's own VTA**. A room's owner is whoever controls its DID and issues its credentials. Nothing about that changes because the owner is a community, and nothing new is needed to do it: the VTA serves the same `rooms/owner/*` tasks for its community as a member's VTA serves for them.

## The steps, and their order

The community, on a valid request:

1. Has its VTA mint the room's DID (the `room` template), in a context the community may act in, as a DID that is **not** the context's own.
2. On an `attributed` or `private` room, has its VTA found the room's group with the community as its only member ([`rooms/owner/group/create`](../../../../../rooms/owner/group/create/0.1/spec.md)).
3. Has its VTA issue the community's own membership credential, and an authority credential conferring `admin`, `write` and `curate`, in the room's name ([`rooms/owner/issue-membership`](../../../../../rooms/owner/issue-membership/0.1/spec.md), [`rooms/owner/issue-authority`](../../../../../rooms/owner/issue-authority/0.2/spec.md)) and keep them in its vault, so it can present the community's authority when the room's epoch advances and when it writes the room's profile.
4. **Then**, and only then, stores the room as its host, with `ownerDid` the community's own DID.
5. Has its VTA write the room's **profile** — `name` the label, `description` the description, and nothing else — into the room ([`rooms/keys/profile/write`](../../../../../rooms/keys/profile/write/0.1/spec.md), with itself as `host`), so the room's members can see what the room is.

Every remote step that the room depends on comes before the one local write. A failure in steps 1–3 leaves no room behind; at most a DID the VTA minted and nobody uses, which `keyHolderUnavailable` names. A room that existed at its host with no key holder behind it would admit nobody and could not be repaired.

Step 5 comes after, and its failure does not undo the room: a room with no profile yet is a complete room, and its members' surfaces fall back to the label of the offer that brought them in. The answer says which it was (`roomProfile`), and a community that has not written the profile keeps trying.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.3

The response's `room` is still a `CommunityRoomV2`, now re-pinned from [`vtc/_shared/0.1/community-room.schema.json`](../../../../../vtc/_shared/0.1/community-room.schema.json) to [`vtc/_shared/0.3/community-room.schema.json`](../../../../../vtc/_shared/0.3/community-room.schema.json). The definition is otherwise identical; what changes is its host record, `room` — `HostedRoom`, now from [`vtc/_shared/0.3/room-storage.schema.json`](../../../../../vtc/_shared/0.3/room-storage.schema.json):

- `room.room.epochExpiresAt`: integer seconds since the Unix epoch → RFC 3339 `date-time` string (still optional; absent still means the room never lapses).
- `room.room.createdAt`: integer seconds → RFC 3339 `date-time` string (still required).
- `room.room.updatedAt`: integer seconds → RFC 3339 `date-time` string (still required).

The names are unchanged: each already ends in `At`. `epoch` (a key-epoch counter) and `retentionDays` (a duration) stay integers. The reason is the registry-wide convention that an instant is an RFC 3339 string named `*At` — the rest of `CommunityRoomV2` (`ownerAuthorityValidUntil`, `detailsUpdatedAt`) already was. Every other `$ref` stays where it was, and nothing else in the request or response moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.3` remains published and pinned to `vtc/_shared/0.1`.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`.
2. **MUST** evaluate its rooms policy for a room the community itself owns, at the requested tier, and refuse with `visibilityNotPermitted` when it does not permit one.
3. **MUST** refuse with `unknownStorageConfig` a `storageConfig` that is unknown or not `active`.
4. **MUST** perform the steps above in that order, store the room only after steps 1–3 succeed, and store it with `ownerDid` its own DID.
5. **MUST** refuse with `noKeyHolder` when it has no VTA to hold the room's keys, and **MUST NOT** fall back to holding any room key itself.
6. **MUST** issue the community's own authority credential with an expiry, conferring `admin`, `write` and `curate`, and **SHOULD** renew it well before it lapses.
7. **MUST NOT** return or log a key, a group secret or a credential.
8. **MUST** keep `label`, `description` and `tags` as its own record as the room's owner, beside the room and outside its host record. It **MUST NOT** send any of them on the room's host path, with one exception: the label and description go to its VTA in [`rooms/keys/profile/write`](../../../../../rooms/keys/profile/write/0.1/spec.md), which seals them into the room's profile record (step 5). They **MUST NOT** go into the room's host record, a credential, [`rooms/owner/register`](../../../../../rooms/owner/register/0.1/spec.md) or any other `rooms/*` task, and the tags **MUST NOT** go into the room at all.
9. **MUST** attempt step 5 once the room is stored, **MUST** answer `roomProfile: "written"` only when the VTA answered `written`, and otherwise **MUST** answer `pending` and keep trying until the profile is written or a later update supersedes it. It **MUST NOT** fail the request, or remove the room, because step 5 did not complete.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, admit and remove their members. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores), because a community may let someone run its board room without letting them reassign every room's storage, and the reverse; [`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place either in a role's ceiling. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder.** The community is the room's owner, and its own VTA holds the room's signing key and, on a sealed tier, the community's place in the room's group. The community acts on the room by sending its VTA the same `rooms/owner/*` and `rooms/keys/*` tasks any owner's key holder serves, as itself; the VTA applies its own gates to each. The community holds no room key and no group secret.

**The model this task is designed for.** A community implementing administrator step-up binds this operation to a step-up of the requester's (a gesture bound to this one document), records it in its audit trail with its `reason`, and executes it without a second administrator's consent: a room the community owns is the community's own, and what this changes is reversible. A community whose approvals rules require consent for it parks it as it parks any consented act, answering with `trust-task-next-step` naming the action. Whether a step-up or consent is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- `CommunityRoomV2` — see [`vtc/_shared/0.3/community-room.schema.json`](../../../../../vtc/_shared/0.3/community-room.schema.json); `RoomLabel`, `RoomDescription`, `RoomTags` — see [`vtc/_shared/0.1/community-room.schema.json`](../../../../../vtc/_shared/0.1/community-room.schema.json).
- **The community's VTA** — the key holder the community reaches over its own messaging link, as it reaches it for its other VTA-held identities.

## Request

An administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A private room for the board

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/create/0.4#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-14T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "label": "Board pack",
    "description": "Papers for the next board meeting: agenda, minutes in draft, and the finance pack.",
    "tags": ["board"],
    "visibility": "private",
    "retentionDays": 90,
    "reason": "The board's working papers."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the room it now owns and hosts. A refusal is a `trust-task-error`.

### Created

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/create/0.4#response",
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
      "createdBy": "did:example:administrator"
    },
    "roomProfile": "written"
  }
}
```

## Security & Privacy

### Data carried

A label, an optional description and tags, a tier and retention in; a host record, counts and whether the profile was written out. The room's signing key and the group's secrets are created inside the community's VTA and never leave it.

### Where the label and description go

To the community's own records as owner, as in `0.2`; and now also to the community's VTA, which seals them into the room as its profile. The VTA is the community's own key holder and already holds the room's keys and every record it seals, so it learns nothing it is not already trusted with. On a sealed tier the profile is ciphertext at a key the host cannot compute — and although the host here *is* the community, which knows the label already, the rule is the same for every room so that a room moved to another host carries nothing that host can read. On an `open` room the profile is cleartext, as everything there is. The tags never go into the room: they are administrators' filing, shown to holders of `vtc.rooms.manage` only.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. The room's DID is minted for this room only, so rooms the community owns are not linkable to one another through their identifiers.

### Retention

The room persists as every hosted room does. The community also keeps, as owner, the record of whom it admits ([`vtc/rooms/community/members/add`](../../../../../vtc/rooms/community/members/add/0.1/spec.md)); that record is never read on the host path.

### Consent/purpose

The community creating a room of its own, through an administrator it has given that authority.
