---
slug: vtc/rooms/community/update
version: "0.1"
title: "VTC Rooms — Community — Update"
summary: "An administrator renames a data room the community itself owns, or changes its description or tags, so administrators can keep track of what each room is and members can tell what they are offered."
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
    Replaces the given details of one room in the community's own record as its owner. Reversible by another update; nothing reaches the room, its key holder, its group or any credential.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a label, a description and tags, all administrator-written. The response is the room as administrators see it; never a key, a group secret or a credential.
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
---

## Abstract

A community that owns several rooms — the board's papers, a working group's drafts, a members' library — has to be able to tell them apart, and so do the members it offers them to. A room's DID says nothing about it. **VTC Rooms — Community — Update** lets an administrator change the details a room is known by after it is created: its **label**, its **description** and its **tags**.

These details are the community's own record as the room's **owner**. Because the community also **hosts** the room, it is worth being exact about where they live: beside the room, in the owner's record, and never in the room's host record, its records, its credentials or anything sent on its host path. A room a member owns has a name the community, as its host, never learns ([`rooms/owner/register`](../../../../../rooms/owner/register/0.1/spec.md) forbids sending one); a room the community owns has a name the community knows because it chose it. This task does not change the first and cannot reach it: it answers only for rooms whose owner is the community itself.

Each member given replaces the stored one; a member absent is unchanged. A request whose values already match the room's changes nothing and succeeds.

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
8. **MUST NOT** send the label, description or tags on the room's host path: not to its VTA, not in any `rooms/*` task, and not in the room's records. Nothing about the room itself, its epoch, its group or any credential changes.
9. **MUST** show the new label and description in the room's open offers from the next [`vtc/rooms/offers/list`](../../../../../vtc/rooms/offers/list/0.2/spec.md) read, and **MUST NOT** show its tags there.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, describe them, admit and remove their members. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores), because a community may let someone run its board room without letting them reassign every room's storage, and the reverse; [`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place either in a role's ceiling. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

`vtc.rooms.admin` alone does not reach these details, here or in a read: the host's view ([`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), [`vtc/rooms/get`](../../../../../vtc/rooms/get/0.1/spec.md)) is the same for every room the community stores, whoever owns it, and carries no owner's name for any of them.

**The model this task is designed for.** A community records the change in its audit trail with its `reason` and executes it on the requester's authority alone, **without** an operation-bound step-up and without a second administrator's consent. [`vtc/rooms/community/create`](../../../../../vtc/rooms/community/create/0.2/spec.md) and [`vtc/rooms/community/members/add`](../../../../../vtc/rooms/community/members/add/0.1/spec.md) are designed for a step-up because they have the community's VTA mint an identity, sign credentials in the room's name and admit a party to the room's material. This task does none of that: it reaches no key holder, confers no authority on anyone, changes nobody's access, and is undone by the same task. What a renamed room's members can do is what their credentials say, whatever the room is called. A community whose policy asks for more — a step-up, or consent — applies it as it does to any act; whether either is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

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
  "type": "https://trusttasks.org/spec/vtc/rooms/community/update/0.1#request",
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
  "type": "https://trusttasks.org/spec/vtc/rooms/community/update/0.1#request",
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
  "type": "https://trusttasks.org/spec/vtc/rooms/community/update/0.1#response",
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
    "changed": true
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

The `rooms` topic of [`vtc/admin/events/subscribe`](../../../../../vtc/admin/events/subscribe/0.2/spec.md) hints at changes to the **host's** view of rooms, for holders of `vtc.rooms.admin`, and that view carries no room's details. A change made here therefore produces no hint at 0.2, consistently with what that topic covers. A console showing the owner's view re-reads it after its own update; a hint for other administrators' consoles would belong to a topic for the owner's view, in a later version of that specification.

## Security & Privacy

### Data carried

A room identifier, administrator-written label, description and tags, and a reason in; the room as administrators see it out. All three details are free text or identifiers written by an administrator and shown to others: a producer **MUST NOT** put personal data or a credential in any of them, and a reader — a person, a console, a wallet or an agent acting for a member — **MUST NOT** treat them as instructions.

### Who sees the details

The label and description: holders of `vtc.rooms.manage`, through [`vtc/rooms/community/get`](../../../../../vtc/rooms/community/get/0.2/spec.md) and [`list`](../../../../../vtc/rooms/community/list/0.2/spec.md), and each member the room is offered to, through [`vtc/rooms/offers/list`](../../../../../vtc/rooms/offers/list/0.2/spec.md) while their offer is listed. The tags: holders of `vtc.rooms.manage` only. `detailsUpdatedBy` names an administrator to other holders of the same capability in the same community, as `createdBy` already does. None of it is shown on the host's view, in the room, or to anyone the room is not offered to.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. A label or description that describes a room's purpose tells whoever reads it what the room is for; that is the point of it, and why it is never sent to anyone who is not running the room or being offered it.

### Retention

The current details persist with the room's owner record and go with it. The earlier values, who changed them and why, are kept in the community's audit trail for as long as it keeps audit records of administrative changes.

### Consent/purpose

The community describing a room of its own, through an administrator it has given that authority.
