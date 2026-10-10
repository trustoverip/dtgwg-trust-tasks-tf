---
slug: vtc/rooms/usage
version: "0.1"
title: "VTC Rooms — Usage"
summary: A room-hosting administrator reports how much the community's hosted data rooms store and serve — grouped by room, member, storage config or day, over a date range of up to 400 days — in sizes and counts only.
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
    Authority is the signer's `vtc.rooms.admin` capability, which the community can establish from a verified proof or a transport-authenticated sender. A proof is recommended so the report is attributable on every transport, relayed ones included — on an `attributed` room it names members.
sideEffects:
  level: none
  rationale: >-
    Reads usage counters and daily history. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses usage counters the host keeps to enforce limits — files, bytes, uploads, downloads and egress — grouped as asked. Grouped by member, it names the DIDs of members of `attributed` rooms who have added files, with their figures: the host learns which member acts on that tier by design, and this is where administrators read it. Never a file name, record or member of a `private` room.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing on its account. The daily history it reads is kept for 400 days.
errorCodes:
  - code: vtc/rooms/usage:invalidRange
    meaning: "`from` is after `to`, the range is longer than 400 days, or it begins before the community's oldest retained day. `details.earliest` states the oldest day it holds."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        earliest:
          type: string
          format: date
  - code: vtc/rooms/usage:memberUsageUnavailable
    meaning: "Grouping or filtering by member was asked for a `private` room, whose host cannot tell members apart."
    retryable: false
  - code: vtc/rooms/usage:invalidCursor
    meaning: "`cursor` was not issued by this community for this filter, grouping and sort."
    retryable: false
related:
  - vtc/rooms/get
  - vtc/rooms/limits/set
  - vtc/storage/configs/get
---

## Abstract

A host already knows everything usage reporting needs: the counters it keeps to enforce limits, and a daily history of them. The **VTC Rooms — Usage** Trust Task reads them, so a community can see what its rooms store and what they cost — growth over a year, the rooms that are largest, the rooms whose files are downloaded a thousand times a day (on most clouds reads are what cost money), and on which storage config it all sits.

Rows are grouped by `room`, `member`, `storageConfig` or `day`, filtered by any of those and a date range, and sorted. Grouping by `day` is the history: one row per day for whatever the filter selects. Every figure is a size or a count. A report never names a file, because the host cannot.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`.
2. **MUST** apply every filter given, and **MUST** compute activity figures (`uploads`, `downloads`, `egressBytes`) over `from`–`to` inclusive, defaulting to the last 30 days. **MUST** refuse with `invalidRange` a range over 400 days, inverted, or starting before its oldest retained day.
3. **MUST** report holdings (`files`, `bytes`, `reservedBytes`) as of the read for `room`, `member` and `storageConfig` groupings, and as of the end of each day for `day`.
4. **MUST** produce `member` rows only from `attributed` rooms, and **MUST** refuse with `memberUsageUnavailable` a member grouping or filter that names a `private` room. A member grouping over several rooms aggregates each member's figures across the `attributed` rooms selected.
5. **MUST** return at most `limit` rows, sorted as asked (`bytes` descending by default; `date` ascending for a `day` grouping), with `nextCursor` present exactly when more remain, and **MUST** refuse a foreign cursor with `invalidCursor`.
6. **MUST NOT** return a file name, record, record key or anything derived from a room's contents.
7. **SHOULD** keep the daily history for 400 days, so a year can be compared with the one before.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

## Definitions

- **Holdings** — `files`, `bytes` and `reservedBytes`, as `Usage` in [`rooms/_shared/0.1/blobs.schema.json`](../../../../rooms/_shared/0.1/blobs.schema.json) defines them: committed blobs not orphaned, and reservations of uploads in progress. Ciphertext bytes.
- **Activity** — uploads committed, downloads begun and chunk bytes served (`egressBytes`).
- **Daily history** — one `DailyUsage` row per room per UTC day; see [`vtc/_shared/0.1/room-storage.schema.json`](../../../../vtc/_shared/0.1/room-storage.schema.json).
- **Member** — the subject at the root of the authority chains a party presents. On `attributed` rooms the host learns it for every upload; on `private` rooms it never does.

## Request

A room-hosting administrator (`issuer`) asks the community (`recipient`) for a report. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The ten largest rooms

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/usage/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "groupBy": "room",
    "sort": {
      "by": "bytes",
      "order": "desc"
    },
    "limit": 10
  }
}
```

### One room's last 90 days, day by day

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/usage/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003fe",
  "payload": {
    "groupBy": "day",
    "filter": {
      "roomId": "room:z6MkRoomExample0001",
      "from": "2026-07-13",
      "to": "2026-10-10"
    },
    "limit": 100
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: `asOf`, the rows, and `nextCursor` when more remain. Each row carries the group it describes — `roomId`, `member`, `storageConfig` or `date` — its holdings and its activity over the range, and, for a `room` or `member` row, the limits it is enforced against. A refusal is a `trust-task-error`.

### Largest rooms

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/rooms/usage/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "asOf": "2026-10-10T10:00:01Z",
    "from": "2026-09-10",
    "to": "2026-10-10",
    "items": [
      {
        "roomId": "room:z6MkRoomExample0001",
        "storageConfig": "eu-s3-primary",
        "files": 312,
        "bytes": 1288490188,
        "reservedBytes": 0,
        "uploads": 41,
        "downloads": 1270,
        "egressBytes": 9126805504,
        "limits": {
          "maxFiles": 10000,
          "maxFileBytes": 104857600,
          "maxBytes": 5368709120
        }
      },
      {
        "roomId": "room:z6MkRoomExample0003",
        "storageConfig": "walrus-main",
        "files": 88,
        "bytes": 734003200,
        "reservedBytes": 0,
        "uploads": 3,
        "downloads": 19,
        "egressBytes": 51380224,
        "limits": {
          "maxFiles": 10000,
          "maxFileBytes": 104857600,
          "maxBytes": 5368709120
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

A filter, a grouping, a sort and paging in; sizes and counts out. On a `member` grouping or filter, member DIDs of `attributed` rooms with their figures. Nothing about what any file is.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. Per-member usage on an `attributed` room tells administrators who adds how much and who downloads how much — the tier's actual privacy property, which a console presenting these rows **SHOULD** state beside them. Usage can never be broken down by member on a `private` room, and the task refuses to try rather than returning an empty column that reads as "nobody". Activity over time is itself a signal — a burst of downloads before a deal closes — which is why it is shown only to `vtc.rooms.admin`.

### Retention

A read. The daily history it reads is kept for 400 days and then discarded; a community **SHOULD NOT** keep it longer, since it is per-room activity over time.

### Consent/purpose

The purpose is capacity planning, cost and enforcing limits. Using member-level activity to monitor what individual members do in a room, beyond the storage they consume, is outside it.
