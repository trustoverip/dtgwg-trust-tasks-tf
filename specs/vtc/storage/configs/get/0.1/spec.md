---
slug: vtc/storage/configs/get
version: "0.1"
title: "VTC Storage — Configs — Get"
summary: A room-hosting administrator reads one storage config — its settings, how it authenticates, the rooms assigned to it, what it holds against its capacity and its health — and never a credential.
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
    Reads one config and its counters. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses one config's settings, its authentication by mode, account name or fingerprint, the identifiers of the rooms assigned to it, and its usage and health. Never a credential.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/storage/configs/get:notFound
    meaning: "The community has no storage config with this identifier."
    retryable: false
related:
  - vtc/storage/configs/list
  - vtc/storage/configs/update
  - vtc/storage/configs/probe
  - vtc/rooms/storage/assign
---

## Abstract

The **VTC Storage — Configs — Get** Trust Task returns one storage config: what [`vtc/storage/configs/list`](../../../../../vtc/storage/configs/list/0.1/spec.md) returns for it, plus the rooms assigned to it. It answers `notFound` for an identifier the community does not hold, which is the definite answer a list filtered to one config cannot give.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`, and **MUST** answer `notFound` for an unknown config.
2. **MUST** compute `facts` and `assignedRooms` at the time of the read, returning at most 100 room identifiers and setting `assignedRoomsTruncated` when there are more (`facts.rooms` is the full count).
3. **MUST NOT** return a credential, a key or any value from which one could be derived, through this or any other task. A `sealed` credential is reported only by its fingerprint.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

## Definitions

- `StorageConfig`, `StorageConfigFacts` — see [`vtc/_shared/0.1/room-storage.schema.json`](../../../../../vtc/_shared/0.1/room-storage.schema.json).

## Request

A room-hosting administrator (`issuer`) names one config to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### One config

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/get/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000007ff",
  "payload": {
    "configId": "eu-s3-primary"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Found

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/get/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000007ff",
  "payload": {
    "config": {
      "id": "eu-s3-primary",
      "label": "EU — S3 (main account)",
      "settings": {
        "kind": "s3",
        "region": "eu-west-1",
        "bucket": "northwind-rooms-eu"
      },
      "auth": {
        "mode": "vta-account",
        "account": "eu-s3-primary"
      },
      "capacityBytes": 1099511627776,
      "state": "active",
      "isDefault": false,
      "createdBy": "did:example:administrator",
      "createdAt": "2026-10-01T09:00:00Z"
    },
    "facts": {
      "rooms": 14,
      "files": 4210,
      "bytes": 182536110080,
      "pendingDeleteBytes": 9437184,
      "credential": {
        "status": "vta-account",
        "account": "eu-s3-primary",
        "roomsHoldingCredentials": 3
      },
      "health": {
        "lastProbeAt": "2026-10-10T06:00:00Z",
        "lastProbeOk": true,
        "lastPutAt": "2026-10-10T09:41:12Z",
        "lastGetAt": "2026-10-10T09:58:40Z",
        "lastDeleteAt": "2026-10-09T23:00:05Z"
      }
    },
    "assignedRooms": [
      "room:z6MkRoomExample0001",
      "room:z6MkRoomExample0004"
    ],
    "assignedRoomsTruncated": false
  }
}
```

## Security & Privacy

### Data carried

A config identifier in; the config, its facts and the identifiers of rooms assigned to it out. Never a credential.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The response links rooms to the cloud account that stores them, for whoever may read it.

### Retention

A read.

### Consent/purpose

The purpose is operating the community's file storage.
