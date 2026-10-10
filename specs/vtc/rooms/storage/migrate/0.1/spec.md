---
slug: vtc/rooms/storage/migrate
version: "0.1"
title: "VTC Rooms — Storage — Migrate"
summary: A room-hosting administrator moves a hosted data room's existing files onto the storage config it is now assigned to — copy, verify against the committed digest, switch, then delete the old copy — resumably and without any key.
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
    The request moves a room's files between backends and accounts, and ends with the old copies being deleted. It is parked for other administrators' consent and must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Bounding replay keeps a captured request from restarting a migration an administrator has since decided against.
sideEffects:
  level: mutating
  rationale: >-
    Copies blobs to the room's assigned config and switches each blob's location once its copy verifies; the old copies become orphans that the community deletes after its grace window. Every step before that deletion is reversible by migrating back.
consequences:
  - The old copies are deleted from their store after the grace window. On Walrus a deletion is a lapse, and ciphertext already stored there stays public for as long as anyone kept it.
  - Migrating onto Walrus publishes the room's ciphertext permanently.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a room, a reason and a rate; the response reports counts and bytes moved. The bytes moved are ciphertext, and no key is involved.
retention:
  class: durable
  rationale: >-
    The migration's state persists until it completes, so it survives a restart and can be resumed; the accepted document is the record of who moved the room's files and why.
errorCodes:
  - code: vtc/rooms/storage/migrate:roomNotFound
    meaning: "The community hosts no room with this identifier."
    retryable: false
  - code: vtc/rooms/storage/migrate:configNotActive
    meaning: "The room's assigned config is not `active`, so it cannot receive the blobs. Assign the room to an active config first."
    retryable: false
  - code: vtc/rooms/storage/migrate:capacityExceeded
    meaning: "The assigned config's capacity cannot hold the blobs to be moved. `details` states the capacity, its current use and the bytes to move."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capacityBytes, usedBytes, requestedBytes]
      properties:
        capacityBytes:
          type: integer
          minimum: 0
        usedBytes:
          type: integer
          minimum: 0
        requestedBytes:
          type: integer
          minimum: 0
related:
  - vtc/rooms/storage/assign
  - vtc/rooms/get
  - vtc/storage/configs/retire
  - vtc/admin/actions/show
---

## Abstract

Reassigning a room ([`vtc/rooms/storage/assign`](../../../../../vtc/rooms/storage/assign/0.1/spec.md)) sends its new uploads to another storage config and leaves its existing files where they are. The **VTC Rooms — Storage — Migrate** Trust Task moves those existing files too, so that a config can be drained and retired, or a room's files brought under one account.

For each blob of the room on a config other than its assigned one, the community:

1. **copies** the ciphertext to the assigned config;
2. **verifies** the copy against the blob's committed manifest — every chunk digest and the whole-blob digest — so the bytes moved are provably the bytes the uploader committed to;
3. **switches** the blob's location to the new copy;
4. **orphans** the old copy, which its store's deletion — after the community's grace window — removes.

No key is involved at any step: the ciphertext is identical on both sides, and the host never holds a room key. Readers notice nothing, because a blob is named by its digest, not its location.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied` and an unknown room with `roomNotFound`; and **MUST** refuse with `configNotActive` when the room's assigned config is not `active`.
2. **MUST** refuse with `capacityExceeded`, before copying anything, when the assigned config's capacity cannot hold the bytes to move.
3. **MUST** switch a blob's location only after its copy has verified against every chunk digest and the whole-blob digest of its manifest, and **MUST** leave a blob whose copy does not verify on its original config, report the migration `failed`, and audit it.
4. **MUST** serve every blob throughout, from whichever location it currently has.
5. **MUST** persist the migration's progress so that a restart resumes it, and **MUST** answer a request for a room that already has a migration `running` or `paused` with that migration — resuming a `paused` one — rather than starting a second.
6. **SHOULD** throttle copying, to `maxBytesPerSecond` when given, so that a migration does not starve the room's readers or exhaust the source's egress budget.
7. **MUST** complete with zero blobs, as a success, for a room with nothing to move.
8. **MUST** write audit records when the migration starts, completes, pauses or fails, carrying the requester, the approvers, the room, the configs involved, and `reason`.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

**Second-administrator consent.** This operation changes where the community keeps its members' files or what it uses to reach them, so a community implementing the administrator action list **parks** it rather than executing it: it answers with the framework-reserved [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry whose `typeUri` is `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` and whose `hint` is `{"actionId": "…"}`, exactly as [`vtc/admin/actions/show/0.2`](../../../../../vtc/admin/actions/show/0.2/spec.md) describes. The requester does **not** re-submit; the community executes the parked payload itself, exactly once, when its threshold is met, re-running every check in this specification against the community as it is then, and answers on the original `threadId` with this task's `#response` or a `trust-task-error`. An approval is signed by the approver's own DID; nobody approves their own request. Where the community has no other administrator to consent and runs in a single-administrator mode, it may waive the consent on its own stated terms. Whether, and by how many, an operation must be approved is the community's policy; this specification describes the model it is designed for.

Progress is read with [`vtc/rooms/get`](../../../../../vtc/rooms/get/0.1/spec.md), whose `storage.migration` reports it, or by issuing this task again, which returns the migration in progress.

## Definitions

- **Assigned config**, **earlier config** — as in [`vtc/rooms/get`](../../../../../vtc/rooms/get/0.1/spec.md).
- **Manifest** — a blob's committed `BlobManifest` ([`rooms/_shared/0.1/blobs.schema.json`](../../../../../rooms/_shared/0.1/blobs.schema.json)), whose digest is the blob's name.
- **Migration states** — `running`; `paused` (halted by a capacity or backend failure; resumed by issuing this task again); `complete`; `failed` (a copy did not verify).

## Request

A room-hosting administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Drain a room off the local directory

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/storage/migrate/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000005ff",
  "payload": {
    "roomId": "room:z6MkRoomExample0001",
    "maxBytesPerSecond": 10485760,
    "reason": "Retiring local-default; this room is now on eu-s3-primary."
  }
}
```

## Response

When the migration starts, the community answers on the request's `threadId` with the sub-schema reachable via `$anchor: "response"`: the migration and its progress so far. It does not wait for completion. Before that, a parking community answers with `trust-task-next-step` (see [Authorization](#authorization)). A refusal is a `trust-task-error`.

### Started

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/storage/migrate/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000005ff",
  "payload": {
    "migrationId": "6f1c2b3a-4d5e-4f60-8172-93a4b5c6d7e8",
    "roomId": "room:z6MkRoomExample0001",
    "toConfig": "eu-s3-primary",
    "fromConfigs": [
      "local-default"
    ],
    "state": "running",
    "totalFiles": 12,
    "totalBytes": 48234496,
    "doneFiles": 0,
    "doneBytes": 0
  }
}
```

## Security & Privacy

### Data carried

A room, a reason and a rate in; counts, bytes and config identifiers out. The bytes the community moves are ciphertext it already holds, verified against digests the uploader committed to.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. Both stores see the same ciphertext under the same digest-derived prefix during the move, which they can link to each other; neither learns the room's identifier.

### Retention

The migration's state persists until it completes. The old copies are deleted after the grace window — except on Walrus, where deletion is a lapse of paid storage and ciphertext once stored there is assumed to be kept by someone. Migrating a room **off** Walrus therefore does not withdraw its files from public reach; only the room's epoch chain controls who can read them.

### Consent/purpose

The purpose is moving a room's files between backends the community operates. A migration **onto** Walrus makes the room's ciphertext public and permanent; a community **SHOULD** obtain the room owner's agreement before doing so, and a console **SHOULD** say so before the request is sent.
