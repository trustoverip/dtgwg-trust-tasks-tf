---
slug: vtc/rooms/community/list
version: "0.1"
title: "VTC Rooms — Community — List"
summary: "An administrator lists the data rooms the community itself owns, with each room's label, tier and how many members it has admitted or offered."
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
    Reads the community's own rooms. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Each room's host record, label and counts. No member DIDs: those are vtc/rooms/community/members/list's.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes: []
related:
  - vtc/rooms/community/get
  - vtc/rooms/community/create
  - vtc/rooms/list
---

## Abstract

**VTC Rooms — Community — List** returns the rooms the community owns ([`vtc/rooms/community/create`](../../../../../vtc/rooms/community/create/0.1/spec.md)), a page at a time: each room's host record, the label its administrators gave it, its storage config, and how many members are admitted and how many offers are open. It is the owner's view, and is distinct from [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), the host's view of every room the community stores, most of which it does not own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`.
2. **MUST** list only rooms whose owner is the community itself.
3. **MUST NOT** return member DIDs, keys or credentials.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, admit and remove their members. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores), because a community may let someone run its board room without letting them reassign every room's storage, and the reverse; [`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place either in a role's ceiling. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder.** The community is the room's owner, and its own VTA holds the room's signing key and, on a sealed tier, the community's place in the room's group. The community acts on the room by sending its VTA the same `rooms/owner/*` and `rooms/keys/*` tasks any owner's key holder serves, as itself; the VTA applies its own gates to each. The community holds no room key and no group secret.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-14T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "limit": 50
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: one page, and `nextCursor` while more remain.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-14T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "items": [
      {
        "room": {
          "roomId": "did:webvh:example.com:rooms:board",
          "ownerDid": "did:example:community",
          "visibility": "private",
          "retentionPolicy": "chained",
          "epoch": 1,
          "lifecycle": "live",
          "epochExpiresAt": 1792368000,
          "retentionDays": 90,
          "createdAt": 1760436000,
          "updatedAt": 1760436000
        },
        "label": "Board pack",
        "storageConfig": "local",
        "admitted": 0,
        "offered": 0,
        "ownerAuthorityValidUntil": "2027-10-14T10:00:00Z",
        "createdBy": "did:example:administrator"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Host records, labels and counts for rooms the community owns. No member DID, key or credential.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. The answer links nothing a member did to any room.

### Retention

None; a read.

### Consent/purpose

Shown so the owner can run its rooms.
