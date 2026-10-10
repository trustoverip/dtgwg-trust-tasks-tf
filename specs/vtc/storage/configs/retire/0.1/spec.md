---
slug: vtc/storage/configs/retire
version: "0.1"
title: "VTC Storage — Configs — Retire"
summary: A room-hosting administrator retires a storage config with no rooms assigned — it accepts no new rooms or uploads, keeps serving the files it holds while they are migrated off, and is never deleted while a blob names it. Parked for consent.
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
    The request withdraws a backend from service. It is parked for other administrators' consent and must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Retirement is final, so a replayed request must not be able to retire a config with the same identifier later.
sideEffects:
  level: mutating
  rationale: >-
    Moves a config to `draining` (blobs still name it) or `retired` (none do). Nothing stored is moved or deleted by this task; a retired identifier is never reused.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a config and a reason; the response echoes the config's new state.
retention:
  class: durable
  rationale: >-
    The config record is kept, retired, for as long as anything refers to it, and its identifier is never reused.
errorCodes:
  - code: vtc/storage/configs/retire:notFound
    meaning: "The community has no storage config with this identifier."
    retryable: false
  - code: vtc/storage/configs/retire:roomsAssigned
    meaning: "Rooms are still assigned to the config. Assign each to another config first (vtc/rooms/storage/assign). `details.rooms` is the count."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [rooms]
      properties:
        rooms:
          type: integer
          minimum: 1
  - code: vtc/storage/configs/retire:isDefault
    meaning: "The config is the community's default. Make another config the default first."
    retryable: false
related:
  - vtc/storage/configs/update
  - vtc/rooms/storage/assign
  - vtc/rooms/storage/migrate
  - vtc/admin/actions/show
---

## Abstract

A storage config is never deleted while a blob names it, because a blob is found through its config. The **VTC Storage — Configs — Retire** Trust Task takes one out of service instead. Draining a config is:

1. assign every room on it elsewhere ([`vtc/rooms/storage/assign`](../../../../../vtc/rooms/storage/assign/0.1/spec.md));
2. retire it with this task — it becomes `draining` if blobs still name it, `retired` if none do;
3. migrate each of those rooms ([`vtc/rooms/storage/migrate`](../../../../../vtc/rooms/storage/migrate/0.1/spec.md)); when the last blob has moved and its old copy is deleted, the community moves the config to `retired` itself.

A `draining` or `retired` config accepts no new room and no new upload, and a `draining` one keeps serving every blob it holds until it holds none.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`, and an unknown config with `notFound`.
2. **MUST** refuse with `roomsAssigned` while any room is assigned to the config, and with `isDefault` while it is the default.
3. **MUST** set `draining` when any blob not yet deleted names the config, and `retired` otherwise; and **MUST** move a `draining` config to `retired` when the last such blob is gone.
4. **MUST** keep serving the blobs of a `draining` config, and **MUST NOT** delete or move any blob on account of this task.
5. **MUST** treat retiring a config already `draining` or `retired` as a success that changes nothing, and **MUST NOT** reuse a retired identifier.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

**Second-administrator consent.** This operation changes where the community keeps its members' files or what it uses to reach them, so a community implementing the administrator action list **parks** it rather than executing it: it answers with the framework-reserved [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry whose `typeUri` is `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` and whose `hint` is `{"actionId": "…"}`, exactly as [`vtc/admin/actions/show/0.2`](../../../../../vtc/admin/actions/show/0.2/spec.md) describes. The requester does **not** re-submit; the community executes the parked payload itself, exactly once, when its threshold is met, re-running every check in this specification against the community as it is then, and answers on the original `threadId` with this task's `#response` or a `trust-task-error`. An approval is signed by the approver's own DID; nobody approves their own request. Where the community has no other administrator to consent and runs in a single-administrator mode, it may waive the consent on its own stated terms. Whether, and by how many, an operation must be approved is the community's policy; this specification describes the model it is designed for.

## Definitions

- `ConfigState` — see [`vtc/_shared/0.1/room-storage.schema.json`](../../../../../vtc/_shared/0.1/room-storage.schema.json).

## Request

A room-hosting administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Retire the local directory

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/retire/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000aff",
  "payload": {
    "configId": "local-default",
    "reason": "All rooms now on eu-s3-primary; the local disk is being decommissioned."
  }
}
```

## Response

When the retirement executes, the community answers on the request's `threadId` with the sub-schema reachable via `$anchor: "response"`. Before that, a parking community answers with `trust-task-next-step` (see [Authorization](#authorization)). A refusal is a `trust-task-error`.

### Draining

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/retire/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000aff",
  "payload": {
    "configId": "local-default",
    "state": "draining",
    "blobsRemaining": 12
  }
}
```

## Security & Privacy

### Data carried

A config identifier and a reason in; its new state and the count of blobs still on it out.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered.

### Retention

The retired config's record is kept for as long as anything refers to it. Its backend's own data — a directory, a bucket — is the operator's to dispose of once the config is `retired`; on Walrus there is nothing to dispose of, since storage simply lapses.

### Consent/purpose

The purpose is withdrawing a backend from the community's storage without stranding any member's file.
