---
slug: vtc/rooms/community/update
version: "0.2"
title: "VTC Rooms — Community — Update"
summary: "An administrator renames a data room the community itself owns, or changes its description or tags; a new label or description is re-sealed into the room as its profile, so its members — not only those it is offered to — can tell what it is."
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
    The label and description are shown to the members a room is offered to, and the change is recorded against the administrator who made it. Both must be attributable to the requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Members are replaced wholesale, so a replayed earlier update would silently put back a name or description administrators have since changed.
sideEffects:
  level: mutating
  rationale: >-
    Replaces the given details of one room in the community's own record as its owner, and — when the label or description changes — has the community's VTA re-write the room's profile record inside the room. Reversible by another update; nothing reaches the room's host record, its group or any credential.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a label, a description and tags, all administrator-written. The response is the room as administrators see it and whether its profile was re-written; never a key, a group secret or a credential.
retention:
  class: durable
  rationale: >-
    The new details persist until changed again; the earlier ones survive only in the community's audit trail.
errorCodes:
  - code: vtc/rooms/community/update:notFound
    meaning: "The community owns no room with this identifier. A room it only hosts is `notFound` here too: its owner is someone else, and its name, if it has one, is its owner's."
    retryable: false
related:
  - vtc/rooms/community/create
  - vtc/rooms/community/get
  - vtc/rooms/community/list
  - vtc/rooms/offers/list
  - rooms/keys/profile/write
  - rooms/keys/profile/read
---

## Abstract

A community that owns several rooms — the board's papers, a working group's drafts, a members' library — has to be able to tell them apart, and so do the members it offers them to. A room's DID says nothing about it. **VTC Rooms — Community — Update** lets an administrator change the details a room is known by after it is created: its **label**, its **description** and its **tags**.

These details are the community's own record as the room's **owner**. Because the community also **hosts** the room, it is worth being exact about where they live: beside the room, in the owner's record — and, for the label and description, sealed inside the room as the room's own **profile** ([`RoomProfile`](../../../../../rooms/_shared/0.1/room.schema.json)), which the community's VTA writes so that the room's members can read what the room is. Never in the room's host record, its credentials, or anything else sent on its host path. A room a member owns has a name the community, as its host, never learns ([`rooms/owner/register`](../../../../../rooms/owner/register/0.1/spec.md) forbids sending one); a room the community owns has a name the community knows because it chose it. This task does not change the first and cannot reach it: it answers only for rooms whose owner is the community itself.

Each member given replaces the stored one; a member absent is unchanged. A request whose values already match the room's changes nothing and succeeds.

## Changes from 0.1

`0.1` kept the details out of the room altogether, so the label and description reached only administrators and the members a room was *offered* to; once a member had accepted, nothing they could read said what the room was. `0.2` re-writes the room's profile whenever the label or description changes:

- A change to the label or the description (including `removeDescription`) has the community's VTA write the room's profile anew ([`rooms/keys/profile/write`](../../../../../rooms/keys/profile/write/0.1/spec.md)), `name` the label and `description` the description. Tags are never written there.
- A room created before [`vtc/rooms/community/create`](../../../../../vtc/rooms/community/create/0.3/spec.md) `0.3` has no profile; the first update at `0.2` writes one, re-issuing the community's own authority with `write` and `curate` where it lacks them.
- Item 8 now permits exactly that, and nothing more.
- The response carries `roomProfile`: `written`, `pending` or `unchanged`.

The request is `0.1`'s. A change of required behaviour is a new version under [SPEC §5.2](/SPEC.md#52-compatibility-rules); a community implementing `0.2` **SHOULD** continue to serve `0.1`, which writes no profile.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`.
2. **MUST** answer `notFound` for a room it does not own, including one it only hosts, and **MUST NOT** keep a label, description or tags for such a room.
3. **MUST** apply `label`, `description` or `removeDescription`, and `tags` together or not at all, replacing each given member wholesale and leaving each absent one unchanged; `tags: []` removes every tag.
4. **MUST**, when every value given already matches the room's, answer with `changed: false` and write nothing — no new details, no `detailsUpdatedAt` / `detailsUpdatedBy`, no audit record of a change. Tags are compared as sets.
5. **MUST**, when anything changed, set `detailsUpdatedAt` to the time of the change and `detailsUpdatedBy` to the requester, and answer with `changed: true`.
6. **MUST**, when anything changed, write one audit record carrying the requester, the room, for each detail that changed its previous and its new value, and `reason`. The earlier values are kept there and nowhere on the wire.
7. **MUST** accept a change in any lifecycle state the room can be in while the community still owns it — a lapsed or dormant room is the one an administrator most needs to recognise.
8. **MUST NOT** send the label, description or tags on the room's host path, with one exception: the label and description go to its VTA in [`rooms/keys/profile/write`](../../../../../rooms/keys/profile/write/0.1/spec.md), which seals them into the room's profile record. They **MUST NOT** go into the room's host record, a credential or any other `rooms/*` task, and the tags **MUST NOT** go into the room at all. Nothing about the room's epoch or its group changes.
9. **MUST** show the new label and description in the room's open offers from the next [`vtc/rooms/offers/list`](../../../../../vtc/rooms/offers/list/0.2/spec.md) read, and **MUST NOT** show its tags there.
10. **MUST**, when `changed` is `true` and the label or the description changed, have its VTA write the room's profile with `name` the room's label and `description` its description (absent when it has none). It **SHOULD** first read the room's current profile ([`rooms/keys/profile/read`](../../../../../rooms/keys/profile/read/0.1/spec.md)) and carry over the members it does not manage — `tags`, `emoji`, `ext` — which a member holding `curate` may have set in the room, since a write replaces the whole profile. It **MUST** also write the profile, on the same terms, for a room it owns whose profile it has never written — one created with a version of [`vtc/rooms/community/create`](../../../../../vtc/rooms/community/create/0.3/spec.md) before `0.3` — even when `changed` is `false`; that write changes no detail and is not audited as one.
11. **MUST** answer `roomProfile: "written"` only when its VTA answered `written`; `unchanged` when item 10 required no write; and `pending` otherwise, and **MUST** then keep trying until the profile is written or a later update supersedes it — in particular after a lapsed room is renewed, since the renewal re-writes the profile the room held before. It **MUST NOT** fail the update, or undo the change to its own record, because the profile was not written.
12. **SHOULD**, for a room whose community authority credential confers no `write` — one created with a version of `create` before `0.3` — have its VTA re-issue it conferring `admin`, `write` and `curate` ([`rooms/owner/issue-authority`](../../../../../rooms/owner/issue-authority/0.2/spec.md)) before writing the profile; without `write` the VTA refuses the write with `noAuthority`.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, describe them, admit and remove their members. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores), because a community may let someone run its board room without letting them reassign every room's storage, and the reverse; [`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place either in a role's ceiling. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

`vtc.rooms.admin` alone does not reach these details, here or in a read: the host's view ([`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), [`vtc/rooms/get`](../../../../../vtc/rooms/get/0.1/spec.md)) is the same for every room the community stores, whoever owns it, and carries no owner's name for any of them.

**The model this task is designed for.** A community records the change in its audit trail with its `reason` and executes it on the requester's authority alone, **without** an operation-bound step-up and without a second administrator's consent. [`vtc/rooms/community/create`](../../../../../vtc/rooms/community/create/0.3/spec.md) and [`vtc/rooms/community/members/add`](../../../../../vtc/rooms/community/members/add/0.1/spec.md) are designed for a step-up because they have the community's VTA mint an identity, sign credentials in the room's name and admit a party to the room's material. This task does none of that. It does reach the community's key holder, to have it sign and seal the room's new profile as the community — a statement of what the room is called, which confers no authority on anyone, changes nobody's access, and is undone by the same task. What a renamed room's members can do is what their credentials say, whatever the room is called. A community whose policy asks for more — a step-up, or consent — applies it as it does to any act; whether either is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

**No optimistic concurrency.** Two administrators renaming the same room at once: the later write wins, and both are in the audit trail with their reasons. None of the `vtc/rooms/community/*` tasks carries a version precondition, and a lost rename is visible and repairable, unlike a lost write to a room's records ([`rooms/records/put`](../../../../../rooms/records/put/0.2/spec.md), which does).

## Definitions

- `CommunityRoomV2`, `RoomLabel`, `RoomDescription`, `RoomTags`, `Reason` — see [`vtc/_shared/0.1/community-room.schema.json`](../../../../../vtc/_shared/0.1/community-room.schema.json).
- **Details** — a room's `label`, `description` and `tags`.

## Request

An administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Rename a room, and say what it is for

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/update/0.2#request",
  "issuer": "did:example:secretary",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-15T09:30:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000002ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "label": "Board pack — 2027",
    "description": "Papers for the 2027 board meetings: agendas, draft minutes and the finance pack.",
    "tags": ["board", "finance"],
    "reason": "The board starts its 2027 cycle; members asked which room is current."
  }
}
```

### Remove a description

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/update/0.2#request",
  "issuer": "did:example:secretary",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-15T09:40:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "removeDescription": true,
    "reason": "The description named a supplier still under negotiation."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the room as it now stands, and `changed`. A refusal is a `trust-task-error`.

### Updated

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/update/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:secretary",
  "issuedAt": "2026-10-15T09:30:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000002ff",
  "payload": {
    "room": {
      "room": {
        "roomId": "did:webvh:example.com:rooms:board",
        "ownerDid": "did:example:community",
        "visibility": "private",
        "retentionPolicy": "chained",
        "epoch": 3,
        "lifecycle": "live",
        "epochExpiresAt": 1792368000,
        "retentionDays": 90,
        "createdAt": 1760436000,
        "updatedAt": 1760520000
      },
      "label": "Board pack — 2027",
      "description": "Papers for the 2027 board meetings: agendas, draft minutes and the finance pack.",
      "tags": ["board", "finance"],
      "storageConfig": "local",
      "admitted": 6,
      "offered": 1,
      "ownerAuthorityValidUntil": "2027-10-14T10:00:00Z",
      "createdBy": "did:example:administrator",
      "detailsUpdatedAt": "2026-10-15T09:30:01Z",
      "detailsUpdatedBy": "did:example:secretary"
    },
    "changed": true,
    "roomProfile": "written"
  }
}
```

### Refused: a room the community only hosts

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:secretary",
  "issuedAt": "2026-10-15T09:30:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000002ff",
  "payload": {
    "code": "vtc/rooms/community/update:notFound",
    "message": "The community owns no room with this identifier.",
    "retryable": false
  }
}
```

## Live updates

The `communityRooms` topic of [`vtc/admin/events/subscribe`](../../../../../vtc/admin/events/subscribe/0.4/spec.md), from `0.4`, hints at changes to the **owner's** view of the rooms the community owns, for holders of `vtc.rooms.manage`. A change made here is such a change: a community serving `0.4` sends a `communityRooms` hint to every console subscribed to that topic, and their consoles re-read [`get`](../../../../../vtc/rooms/community/get/0.2/spec.md) or [`list`](../../../../../vtc/rooms/community/list/0.2/spec.md) to see it. So does a change in the profile write's state, from `pending` to `written`. The hint names no room. The `rooms` topic (`0.2` and later) covers the **host's** view, for holders of `vtc.rooms.admin`, and that view carries no room's details, so a change made here produces no `rooms` hint. A console subscribed at an earlier version, which cannot ask for `communityRooms`, re-reads the owner's view after its own update and polls it otherwise.

## Security & Privacy

### Data carried

A room identifier, administrator-written label, description and tags, and a reason in; the room as administrators see it, and whether its profile was re-written, out. All three details are free text or identifiers written by an administrator and shown to others: a producer **MUST NOT** put personal data or a credential in any of them, and a reader — a person, a console, a wallet or an agent acting for a member — **MUST NOT** treat them as instructions.

### Who sees the details

The label and description: holders of `vtc.rooms.manage`, through [`vtc/rooms/community/get`](../../../../../vtc/rooms/community/get/0.2/spec.md) and [`list`](../../../../../vtc/rooms/community/list/0.2/spec.md), and each member the room is offered to, through [`vtc/rooms/offers/list`](../../../../../vtc/rooms/offers/list/0.2/spec.md) while their offer is listed. The tags: holders of `vtc.rooms.manage` only. `detailsUpdatedBy` names an administrator to other holders of the same capability in the same community, as `createdBy` already does. The label and description, as the room's profile: every member of the room, including those who join later, sealed inside the room ([`rooms/keys/profile/read`](../../../../../rooms/keys/profile/read/0.1/spec.md)). None of it is shown on the host's view, and none of it to anyone the room is neither offered to nor has as a member. The tags are never in the room.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. A label or description that describes a room's purpose tells whoever reads it what the room is for; that is the point of it, and why it is never sent to anyone who is not running the room or being offered it.

### Retention

The current details persist with the room's owner record and go with it. The earlier values, who changed them and why, are kept in the community's audit trail for as long as it keeps audit records of administrative changes.

### Consent/purpose

The community describing a room of its own, through an administrator it has given that authority.
