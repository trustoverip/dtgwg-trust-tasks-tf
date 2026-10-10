---
slug: vtc/rooms/get
version: "0.1"
title: "VTC Rooms — Get"
summary: A room-hosting administrator reads one hosted data room in detail — its storage config, effective limits and their source, usage and orphaned blobs — everything a host holds about a room's files and nothing derived from them.
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
    Authority is the signer's `vtc.rooms.admin` capability, which the community can establish from a verified proof or a transport-authenticated sender. A proof is recommended so the read is attributable on every transport, relayed ones included.
sideEffects:
  level: none
  rationale: >-
    Reads the room's host record, its blob index and its counters. Persists nothing.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses operating metadata the host holds by construction — the room's owner, tier and lifecycle, blob counts and sizes, limits and usage. No record, no file name, no member list. On an `attributed` room it states how many members have added files, never who.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/rooms/get:notFound
    meaning: "The community hosts no room with this identifier."
    retryable: false
related:
  - vtc/rooms/list
  - vtc/rooms/limits/set
  - vtc/rooms/usage
  - vtc/rooms/storage/assign
  - vtc/rooms/storage/migrate
---

## Abstract

A community hosting data rooms holds, for each room, what operating it requires: its host record ([`vtc/rooms/list`](../../../../vtc/rooms/list/0.1/spec.md) returns that much for every room), and — once rooms carry files — where the room's blobs are stored, how much it may hold, and how much it does. The **VTC Rooms — Get** Trust Task returns all of that for one room, so an administrator can answer a room owner's question about space, see whether a room still has files on a storage config being drained, and decide whether a limit override is warranted.

It returns sizes and counts, never what the files are. The host cannot know: file names, types and contents are sealed inside records it cannot open (see [`rooms/_shared/0.1/blobs.schema.json`](../../../../rooms/_shared/0.1/blobs.schema.json), `FileManifest`).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`, and **MUST** answer `notFound` for a room it does not host.
2. **MUST** compute every member of the response at the time of the read.
3. **MUST** report in `limits` the limits it actually enforces, with the source of each scope's, bounded by its host-wide ceiling.
4. **MUST** count in `storage.earlierConfigs` every config other than the assigned one that still holds a blob of this room that is not yet deleted.
5. **MUST NOT** return a record, a record key, a file name or anything else derived from a room's contents, and **MUST NOT** return member DIDs. On an `attributed` room it **MAY** return `membersWithUsage`, a count; per-member figures are [`vtc/rooms/usage`](../../../../vtc/rooms/usage/0.1/spec.md)'s, where the disclosure is stated. On a `private` room it **MUST** omit `membersWithUsage`: the host cannot tell members apart.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

## Definitions

- **Host record** — what a community stores about a room it hosts; see `HostedRoom` in [`vtc/_shared/0.1/room-storage.schema.json`](../../../../vtc/_shared/0.1/room-storage.schema.json).
- **Assigned config** — the storage config that receives the room's new uploads.
- **Earlier config** — a config the room was assigned to before, which still holds some of its blobs. Each blob remembers the config it was written to, so a reassignment strands nothing; [`vtc/rooms/storage/migrate`](../../../../vtc/rooms/storage/migrate/0.1/spec.md) moves them.
- **Orphan** — a blob no record names any more. It stopped counting against the room's and members' usage when it was orphaned, and is deleted from its store after a grace window.
- `Usage`, `RoomLimits` — as [`rooms/_shared/0.1/blobs.schema.json`](../../../../rooms/_shared/0.1/blobs.schema.json) defines them. Sizes are ciphertext bytes throughout.

## Request

A room-hosting administrator (`issuer`) names one room to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### One room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/get/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "room:z6MkRoomExample0001"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

- `room` — the host record.
- `storage` — the assigned config and any earlier configs still holding blobs, with the files and bytes on each; `migration` while one is running.
- `limits` — the effective limits, with each scope's source and the host ceiling.
- `usage` — the room's `Usage`, and its orphans awaiting deletion.
- `largestBlobs` — up to ten, by size only.
- `membersWithUsage` — on an `attributed` room, how many members have added files.

### An attributed room mid-migration

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/get/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "room": {
      "roomId": "room:z6MkRoomExample0001",
      "ownerDid": "did:example:owner",
      "visibility": "attributed",
      "retentionPolicy": "chained",
      "epoch": 7,
      "lifecycle": "live",
      "epochExpiresAt": 1798761600,
      "retentionDays": 30,
      "createdAt": 1767225600,
      "updatedAt": 1790000000
    },
    "storage": {
      "configId": "eu-s3-primary",
      "earlierConfigs": [
        {
          "configId": "local-default",
          "files": 12,
          "bytes": 48234496
        }
      ],
      "migration": {
        "migrationId": "6f1c2b3a-4d5e-4f60-8172-93a4b5c6d7e8",
        "state": "running",
        "totalFiles": 12,
        "totalBytes": 48234496,
        "doneFiles": 5,
        "doneBytes": 20971520
      }
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
      "roomSource": "override",
      "memberSource": "policy",
      "hostCeilingBytes": 1073741824
    },
    "usage": {
      "room": {
        "files": 312,
        "bytes": 1288490188,
        "reservedBytes": 4718880,
        "reservedFiles": 1
      },
      "orphans": {
        "files": 3,
        "bytes": 9437184,
        "oldestOrphanedAt": "2026-10-05T08:12:00Z"
      }
    },
    "largestBlobs": [
      {
        "size": 98566144,
        "configId": "eu-s3-primary",
        "createdAt": "2026-09-30T14:02:11Z"
      }
    ],
    "membersWithUsage": 9
  }
}
```

## Security & Privacy

### Data carried

A room identifier in; the room's host record, storage placement, limits, usage counters, orphan totals and up to ten blob sizes out. Nothing about what any file is, and no member identifier.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. Blob sizes can fingerprint a known document whose exact size an observer already knows; rooms that care pad their files (`FileManifest.padding`), and the response gives sizes only to administrators who already hold this index.

### Retention

A read. The community keeps nothing on account of it.

### Consent/purpose

The purpose is operating the storage the community provides its rooms — space, cost and the movement of files between stores. Using these figures to infer what a room holds or who is in it is outside that purpose, and the response is shaped so that it cannot.
