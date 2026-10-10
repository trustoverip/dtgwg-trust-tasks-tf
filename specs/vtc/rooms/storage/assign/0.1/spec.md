---
slug: vtc/rooms/storage/assign
version: "0.1"
title: "VTC Rooms — Storage — Assign"
summary: A room-hosting administrator points one hosted data room at another storage config, so its new uploads go there; files it already holds stay where they are and keep working.
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
    The request decides which backend and which account receive a room's files from now on — a different cloud, region or public store. It is parked for other administrators' consent and must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed earlier assignment would silently move a room's new uploads back to a config an administrator has since moved it away from.
sideEffects:
  level: mutating
  rationale: >-
    Changes the room's assigned config. No blob moves; reassigning back reverses it.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a room, a config and a reason; the response echoes the assignment and how many of the room's blobs remain on other configs.
retention:
  class: durable
  rationale: >-
    The assignment persists, and the accepted document is the record of who moved a room's storage and why.
errorCodes:
  - code: vtc/rooms/storage/assign:roomNotFound
    meaning: "The community hosts no room with this identifier."
    retryable: false
  - code: vtc/rooms/storage/assign:configNotFound
    meaning: "The community has no storage config with this identifier."
    retryable: false
  - code: vtc/rooms/storage/assign:configNotActive
    meaning: "The config is `draining` or `retired`, and accepts no new rooms."
    retryable: false
  - code: vtc/rooms/storage/assign:credentialRequired
    meaning: "The config cannot authenticate to its backend — a `sealed` config whose credential has not been set — so the room could not store anything there."
    retryable: false
related:
  - vtc/rooms/storage/migrate
  - vtc/rooms/get
  - vtc/storage/configs/get
  - vtc/admin/actions/show
---

## Abstract

Every hosted room is assigned to exactly one storage config, which receives its new uploads; one config serves any number of rooms. The **VTC Rooms — Storage — Assign** Trust Task points a room at another config. **New uploads go there from the moment it executes.** The room's existing files stay where they are and keep working, because every blob remembers the config it was written to. Moving them is [`vtc/rooms/storage/migrate`](../../../../../vtc/rooms/storage/migrate/0.1/spec.md).

The assignment is held by the community beside the room, never in the room's credentials: a room that moves to another host keeps its credentials, and that host chooses its own storage.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`, an unknown room with `roomNotFound`, and an unknown config with `configNotFound`.
2. **MUST** refuse a config that is not `active` with `configNotActive`, and one with no usable credential with `credentialRequired`.
3. **MUST** treat assigning a room to the config it is already on as a success that changes nothing.
4. **MUST** direct every upload begun after execution to the new config, and **MUST NOT** move, copy or delete an existing blob on account of this task.
5. **MUST** write an audit record carrying the requester, the approvers, the room, the previous and new config, and `reason`.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

**Second-administrator consent.** This operation changes where the community keeps its members' files or what it uses to reach them, so a community implementing the administrator action list **parks** it rather than executing it: it answers with the framework-reserved [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry whose `typeUri` is `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` and whose `hint` is `{"actionId": "…"}`, exactly as [`vtc/admin/actions/show/0.2`](../../../../../vtc/admin/actions/show/0.2/spec.md) describes. The requester does **not** re-submit; the community executes the parked payload itself, exactly once, when its threshold is met, re-running every check in this specification against the community as it is then, and answers on the original `threadId` with this task's `#response` or a `trust-task-error`. An approval is signed by the approver's own DID; nobody approves their own request. Where the community has no other administrator to consent and runs in a single-administrator mode, it may waive the consent on its own stated terms. Whether, and by how many, an operation must be approved is the community's policy; this specification describes the model it is designed for.

## Definitions

- **Storage config** — a named backend; see `StorageConfig` in [`vtc/_shared/0.1/room-storage.schema.json`](../../../../../vtc/_shared/0.1/room-storage.schema.json).
- **Assigned config** — the config that receives the room's new uploads.

## Request

A room-hosting administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Move a room's new uploads to the EU bucket

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/storage/assign/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000004ff",
  "payload": {
    "roomId": "room:z6MkRoomExample0001",
    "configId": "eu-s3-primary",
    "reason": "Board rooms belong on the EU account."
  }
}
```

## Response

When the assignment executes, the community answers on the request's `threadId` with the sub-schema reachable via `$anchor: "response"`. Before that, a parking community answers with `trust-task-next-step` (see [Authorization](#authorization)). A refusal, including one at execution, is a `trust-task-error`.

### Assigned

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/storage/assign/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000004ff",
  "payload": {
    "roomId": "room:z6MkRoomExample0001",
    "configId": "eu-s3-primary",
    "previousConfigId": "local-default",
    "blobsOnEarlierConfigs": 12
  }
}
```

## Security & Privacy

### Data carried

A room, a config identifier and a reason in; the assignment out. No credential, and nothing about the room's contents.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. A config's provider learns, from where blobs start to arrive, that one more room's prefix is in use — never the room's identifier, since blobs are stored under a digest of it.

### Retention

The assignment persists until changed. The accepted document and its approvals **SHOULD** be retained with the community's administrative audit records.

### Consent/purpose

The purpose is placing a room's files with the backend the community has chosen for it. Choosing a backend changes what members can be promised — files on Walrus are public ciphertext and their removal is cryptographic, not physical — so a community **SHOULD** tell a room's owner when its room moves between kinds of backend.
