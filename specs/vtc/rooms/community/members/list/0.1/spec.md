---
slug: vtc/rooms/community/members/list
version: "0.1"
title: "VTC Rooms — Community — Members — List"
summary: "An administrator lists who the community has admitted to, or offered admission to, a data room it owns."
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
    Discloses who has access to a room the community owns. A maintainer must be able to attribute the disclosure to a specific administrator.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A request that discloses members is worth replaying; `issuedAt` bounds how old a request a maintainer will answer.
sideEffects:
  level: none
  rationale: >-
    Reads the owner's record of one room's members. Persists nothing.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Member DIDs with their actions, expiries and state, for one room: who has access to it, disclosed to its owner's administrators.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/rooms/community/members/list:notFound
    meaning: "The community owns no room with this identifier."
    retryable: false
related:
  - vtc/rooms/community/get
  - vtc/rooms/community/members/add
  - vtc/rooms/community/members/remove
---

## Abstract

**VTC Rooms — Community — Members — List** returns, a page at a time, the members of one room the community owns — admitted, offered or removed — with their actions and expiry. A separate, paged read rather than part of [`vtc/rooms/community/get`](../../../../../../vtc/rooms/community/get/0.1/spec.md), because a community room may admit the whole community.

This is the **owner's** record, read by the owner's administrators. The community never reads it when it serves the room as host, so a list here grants nothing and gates nothing: who may act in the room is decided by the room's credential chains alone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`, and **MUST** answer `notFound` for a room it does not own.
2. **MUST NOT** return a credential, a key or anything derived from the room's content.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, admit and remove their members. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores), because a community may let someone run its board room without letting them reassign every room's storage, and the reverse; [`vtc/roles/define`](../../../../../../vtc/roles/define/0.1/spec.md) can place either in a role's ceiling. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder.** The community is the room's owner, and its own VTA holds the room's signing key and, on a sealed tier, the community's place in the room's group. The community acts on the room by sending its VTA the same `rooms/owner/*` and `rooms/keys/*` tasks any owner's key holder serves, as itself; the VTA applies its own gates to each. The community holds no room key and no group secret.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/members/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-14T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "state": "admitted",
    "limit": 50
  }
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/members/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-14T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "items": [
      {
        "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "actions": [
          "read",
          "write"
        ],
        "validUntil": "2027-10-01T00:00:00Z",
        "state": "admitted",
        "offeredAt": "2026-10-14T10:00:00Z",
        "admittedAt": "2026-10-14T11:00:00Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Member DIDs, actions, expiries and state for one room.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. On a `private` room this is the membership the room's host cannot see. It is disclosed here only to the room's owner, which issued every credential it lists.

### Retention

None; a read.

### Consent/purpose

Shown so the owner can run the room: renew, narrow, remove.
