---
slug: vtc/storage/configs/list
version: "0.1"
title: "VTC Storage — Configs — List"
summary: A room-hosting administrator pages through the community's storage configs — the named backends its data rooms' files are kept on — with what each holds, its credential state and its health, and never a credential.
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
    Reads the community's storage configs and their counters. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses each config's backend settings (bucket, region, directory, Walrus endpoints), how it authenticates — by mode, external account name or fingerprint — and its usage and health. Never a credential.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/storage/configs/list:invalidCursor
    meaning: "`cursor` was not issued by this community for this filter."
    retryable: false
related:
  - vtc/storage/configs/get
  - vtc/storage/configs/create
  - vtc/rooms/usage
---

## Abstract

A community keeps its data rooms' files on **storage configs**: named backends — a local directory, an S3-compatible bucket, a Google Cloud Storage bucket or Walrus — each with its own account. It may have as many as it wants; every room is assigned to one, and one serves any number of rooms. The **VTC Storage — Configs — List** Trust Task pages through them, with the facts an administrator needs to run them: rooms assigned, files and bytes held against capacity, deletions pending, whether the config can authenticate and how, and when it last worked.

It is the `list` half of the registry's split pair; [`vtc/storage/configs/get`](../../../../../vtc/storage/configs/get/0.1/spec.md) fetches one config by identifier and answers `notFound` for one that does not exist.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`.
2. **MUST** return every config it holds, `retired` ones included, restricted by `state` and `kind` when given, at most `limit` per page, with `nextCursor` present exactly when more remain; and **MUST** refuse a foreign cursor with `invalidCursor`.
3. **MUST** compute `facts` at the time of the read.
4. **MUST NOT** return a credential, a key or any value from which one could be derived, through this or any other task. A `sealed` credential is reported only by its fingerprint.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

## Definitions

- `StorageConfig`, `StorageConfigFacts`, `StorageKind`, `ConfigState` — see [`vtc/_shared/0.1/room-storage.schema.json`](../../../../../vtc/_shared/0.1/room-storage.schema.json).

## Request

A room-hosting administrator (`issuer`) asks the community (`recipient`) for a page. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Active configs

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000006ff",
  "payload": {
    "state": "active"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One S3 config on a VTA account

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000006ff",
  "payload": {
    "items": [
      {
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
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Filters and paging in; configs and their facts out. Settings name buckets, regions, directories and endpoints, which an attacker would find useful for reconnaissance but cannot use without a credential — and no credential is ever carried.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The listing links the community to the cloud accounts and buckets it stores to, for whoever may read it.

### Retention

A read.

### Consent/purpose

The purpose is operating the community's file storage. The settings are the community's own infrastructure, not anyone's personal data.
