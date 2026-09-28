---
slug: vtc/rooms/list
version: "0.1"
title: "VTC Rooms — List"
summary: A community administrator pages through the data rooms the community hosts — owner, tier, epoch and lifecycle state of each — which is everything a host holds about a room and nothing derived from its contents.
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
    Authority is the signer's administrator standing, which the community can establish from a verified proof or a transport-authenticated sender. A proof is recommended so the read is attributable on every transport, relayed ones included.
sideEffects:
  level: none
  rationale: >-
    Reads the host records of the rooms stored here. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses, to a community administrator, each hosted room's identifier, owner DID, tier and lifecycle — operating metadata the host holds by construction. No record, no member list, nothing derived from content.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes: []
related:
  - rooms/create
  - rooms/epoch/mint
---

## Abstract

A community can host data rooms (the `rooms/*` family): spaces whose records are sealed to their members, whose membership the host cannot see at the `private` tier, and whose every operation is authorized by credentials the room itself issued. What the host *does* hold about each room is what operating one requires — who is accountable for it, which tier it was created at, which epoch it is on, and where it sits on the lifecycle clock.

The **VTC Rooms — List** Trust Task returns exactly that, to the community's administrators, so they can reach an owner about quota or abuse and send the reclamation notice a lapsed room is owed. There is no task here that returns records, and no member list to return.

## Why this is `vtc/rooms/list` and not `rooms/list`

Every `rooms/*` task is authorized by the room: a party presents the authority chain the room issued, and the host decides from that chain and nothing else. That is the rooms design's invariant **I5 — room authorization never uses host ACLs** — and it is what makes a room portable: the same task, with the same authority, works against any host that stores the room.

This task is the opposite. It is authorized by the **host's** access control — the caller's administrator standing at this community — and it enumerates rooms across owners, which no room-issued authority could license. Publishing it under `rooms/*` would put a host-ACL-authorized operation into the family whose defining property is that no operation is, and a host implementing `rooms/*` by that invariant would have no way to answer it. So it lives with the other operator reads of a community, under `vtc/`, where host ACL authorization is the norm.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing at the community with `permissionDenied`. Room-issued authority is neither required nor accepted: this is a host operation.
2. **MUST** return every room it stores, restricted to `lifecycle` when present, at most `limit` per page, with `nextCursor` present exactly when more remain.
3. **MUST** compute `lifecycle` at the time of the read, and **MUST** include `ownerDid` at every tier.
4. **MUST NOT** return anything derived from a room's records, and **MUST NOT** return a membership it does not hold.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's. That this is the *host's* authority, not the room's, is the whole reason for the slug (see above).

## Definitions

- **Host record** — what a community stores about a room it hosts, as distinct from the room's records.
- **`lifecycle`** — `live`, `lapsed`, `dormant` or `reclaimable`, computed from the epoch expiry and retention.
- **`visibility`**, **`retentionPolicy`** — as [`rooms/_shared/0.1/room.schema.json`](../../../../rooms/_shared/0.1/room.schema.json) defines them.

## Request

A community administrator (`issuer`) asks the community (`recipient`) for a page.

### Rooms that have lapsed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "lifecycle": "lapsed"
  }
}
```

### Every room, first page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {}
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One private room

Its owner is visible although its contents and members are not.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/rooms/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "items": [
      {
        "roomId": "room:z6MkRoomExample0001",
        "ownerDid": "did:example:member",
        "visibility": "private",
        "retentionPolicy": "chained",
        "epoch": 3,
        "lifecycle": "live",
        "epochExpiresAt": 1798761600,
        "retentionDays": 30,
        "createdAt": 1767225600,
        "updatedAt": 1790000000
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Room identifiers, owner DIDs, tier, epoch and lifecycle timestamps. No record content, no record metadata, and no members.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The listing ties each owner DID to the rooms it owns here, including `private` ones — disclosed to administrators because the owner is who the host must be able to reach, and to no one else.

### Retention

A read. An administrator acting on a lapsed room **SHOULD** keep its own record of the notice it sent.

### Consent/purpose

The purpose is operating the rooms this community hosts — quota, abuse and the lifecycle notices owners are owed. Using the owner list to learn anything about a room's members or contents is outside it, and not possible from what the task returns.
