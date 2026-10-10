---
slug: vtc/rooms/limits/set
version: "0.1"
title: "VTC Rooms — Limits — Set"
summary: A room-hosting administrator overrides how much one hosted data room may store — for the room, for its members by default, or for one named member — or clears an override, with a reason that is audited.
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
    The request changes what a room's members may store, and its reason is the audit record of why. Both must be attributable to the administrator after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. An override is replaced wholesale; a replayed earlier override would silently restore limits an administrator has since changed.
sideEffects:
  level: mutating
  rationale: >-
    Stores or clears one limit override on a room. Lowering a limit below current use refuses new uploads and deletes nothing, so the change is reversible by setting the limit again.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a room, an optional member DID, numeric limits and a reason. The response carries the effective limits and current usage at the target scope — counters the host holds by construction.
retention:
  class: durable
  rationale: >-
    The override persists until replaced or cleared, and the accepted document with its reason is the audit record of who changed a room's allowance and why.
errorCodes:
  - code: vtc/rooms/limits/set:notFound
    meaning: "The community hosts no room with this identifier."
    retryable: false
  - code: vtc/rooms/limits/set:aboveCeiling
    meaning: "A `maxFileBytes` exceeds the community's host-wide ceiling. `details` names the measure and the ceiling. Limits are never raised past it, here or by policy."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [measure, ceiling]
      properties:
        measure:
          type: string
          enum: [maxFileBytes, maxBytes, maxFiles]
        ceiling:
          type: integer
          minimum: 0
  - code: vtc/rooms/limits/set:memberLimitsUnavailable
    meaning: "The target is `memberDefault` or `member` and the room is `private`, where the host cannot tell members apart and so cannot enforce a per-member limit. Room and object limits still apply."
    retryable: false
  - code: vtc/rooms/limits/set:invalidCombination
    meaning: "The request names none of `limits`, `clear` and `filesEnabled`; names `limits` with `clear: true`; or names `filesEnabled` with a target other than `room`."
    retryable: false
related:
  - vtc/rooms/get
  - vtc/rooms/usage
  - rooms/blobs/upload/begin
---

## Abstract

How much a hosted room may store is a hosting decision, made by the community and never by anything in the room's credentials. It is set at three scopes — the room, each member in the room, and each object — each measured as file count, largest file and total bytes ([`RoomLimits`](../../../../../rooms/_shared/0.1/blobs.schema.json)). Ordinarily the community's rooms policy decides them when a room is created. The **VTC Rooms — Limits — Set** Trust Task lets a room-hosting administrator override them for one room: for the room as a whole, for its members by default, or for one named member, or clear an override so the policy's value applies again.

An upload must fit every scope at once, and the host-wide ceiling bounds all of them. Lowering a limit below what a room already holds **deletes nothing**: it refuses new uploads until enough is removed.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** names exactly one change: `limits` (store an override), `clear: true` (remove it), or — for a `room` target only — `filesEnabled`, alone or with `limits`. It gives a `reason`.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`, and **MUST** answer `notFound` for a room it does not host.
2. **MUST** refuse with `malformedRequest` a `member` target without `member`, or another target with one; and with `invalidCombination` a request violating the producer rule above.
3. **MUST** refuse a `memberDefault` or `member` target on a `private` room with `memberLimitsUnavailable`.
4. **MUST** refuse a `maxFileBytes` above its host-wide ceiling with `aboveCeiling`, and **MUST NOT** clamp it silently.
5. **MUST** replace the target's override wholesale with `limits` — a member absent from `limits` sets no limit of that kind at this scope, so the next wider scope or the host ceiling governs — and **MUST** apply it to the next upload begun after the response. Uploads already reserved complete under the limits they were reserved against.
6. **MUST NOT** delete, orphan or refuse to serve anything because a new limit is below current use.
7. **MUST** write an audit record carrying the requester, the room, the target, the previous and new values, and `reason`.
8. **MUST** answer with the effective limits after the change and the target scope's current usage, and **MUST** set `overLimit` when that usage already exceeds the new limits.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key. The change is not parked for a second administrator: an allowance is reversible, nothing is lost by it, and the room's owner can see their own limits. A community MAY park it nonetheless, as its policy decides.

## Definitions

- **Target** — `room`, `memberDefault` (every member of the room without an override of their own) or `member` (one member, by DID). See `LimitTarget` in [`vtc/_shared/0.1/room-storage.schema.json`](../../../../../vtc/_shared/0.1/room-storage.schema.json).
- **Member** — the subject at the root of the authority chain a party presents, never the key that signed, so an agent's uploads count against its member.
- **Override** — a limit set by this task, as opposed to one the rooms policy returned at creation or the community's defaults.
- `ScopeLimits`, `RoomLimits`, `Usage` — as [`rooms/_shared/0.1/blobs.schema.json`](../../../../../rooms/_shared/0.1/blobs.schema.json) defines them; ciphertext bytes.

## Request

A room-hosting administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Give one member more room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/limits/set/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000002ff",
  "payload": {
    "roomId": "room:z6MkRoomExample0001",
    "target": {
      "kind": "member",
      "member": "did:example:member"
    },
    "limits": {
      "maxFiles": 4000,
      "maxFileBytes": 104857600,
      "maxBytes": 2147483648
    },
    "reason": "Owner asked for more space for the due-diligence uploads."
  }
}
```

### Turn files off for a room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/limits/set/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000002fe",
  "payload": {
    "roomId": "room:z6MkRoomExample0002",
    "target": {
      "kind": "room"
    },
    "filesEnabled": false,
    "reason": "Open room; the community does not host files for open rooms."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the room's effective limits after the change, the target scope's current usage, and whether that usage already exceeds them. A refusal is a `trust-task-error`.

### Override stored

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/rooms/limits/set/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000002ff",
  "payload": {
    "roomId": "room:z6MkRoomExample0001",
    "target": {
      "kind": "member",
      "member": "did:example:member"
    },
    "limits": {
      "limits": {
        "filesEnabled": true,
        "room": {
          "maxFiles": 10000,
          "maxFileBytes": 104857600,
          "maxBytes": 5368709120
        },
        "member": {
          "maxFiles": 2000,
          "maxFileBytes": 104857600,
          "maxBytes": 1073741824
        }
      },
      "roomSource": "policy",
      "memberSource": "policy",
      "hostCeilingBytes": 1073741824
    },
    "targetLimits": {
      "maxFiles": 4000,
      "maxFileBytes": 104857600,
      "maxBytes": 2147483648
    },
    "usage": {
      "files": 1830,
      "bytes": 1020054732,
      "reservedBytes": 0
    },
    "overLimit": false
  }
}
```

### Refused: above the ceiling

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000002fd",
  "payload": {
    "code": "vtc/rooms/limits/set:aboveCeiling",
    "message": "maxFileBytes 4294967296 is above this community's 1 GiB per-file ceiling.",
    "retryable": false,
    "details": {
      "measure": "maxFileBytes",
      "ceiling": 1073741824
    }
  }
}
```

## Security & Privacy

### Data carried

A room, a target (with a member DID when it names one), numeric limits and a reason in; effective limits and the target's usage counters out. The reason is free text written for approvers and auditors: a producer **MUST NOT** put personal data or a credential in it.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. A `member` target links a member DID to a room in the community's audit trail. That link is no new disclosure — the host already learns which member acts on an `attributed` room — and a member target cannot be set on a `private` room, where it would be.

### Retention

The override persists until replaced or cleared. The accepted document and its audit record **SHOULD** be retained for as long as the community keeps audit records of administrative changes.

### Consent/purpose

The purpose is to decide how much storage the community provides a room and its members. A limit is not a sanction and is not to be used as one against a member for what they wrote: the host cannot read what they wrote, and the task carries nothing that would let it.
