---
slug: vta/app-state/list
version: "1.0"
title: VTA Application State — List
summary: An application enumerates its state records in a VTA context — a prefix-scoped snapshot, or an incremental change feed since a watermark that includes tombstones so a consumer's local copy converges.
status: draft
targetFrameworkVersion: "0.4"
category: data-exchange
keywords:
  - vta
  - application-state
  - incremental-sync
  - tombstone
  - pagination
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Application
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Enumeration makes no durable change and the maintainer has already authenticated the caller on the transport that carried the request. A proof lets a maintainer attribute an enumeration to a specific application key on transports with no prior handshake, which matters because enumeration is the task that most exposes the shape of a namespace.
sideEffects:
  level: none
  rationale: "Enumerates records; nothing at the maintainer changes."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/app-state:contextNotFound
    meaning: >-
      OPTIONAL diagnostic, for a maintainer whose authorization model can tell "no such
      context" from "not permitted to reach it". Where it cannot — an ACL that enumerates
      the contexts a caller may act in answers both the same way — the framework's
      standard `permissionDenied` (SPEC §8.3) is the conforming answer to both, and this
      code is never emitted. Refusing an unauthorized caller is NOT this code.
    retryable: false
  - code: vta/app-state/list:filterConflict
    meaning: The supplied combination of members is not answerable — `sinceVersion` without `namespace`, or `sinceVersion` with `includeDeleted` set to false.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["reason"]
      properties:
        reason:
          type: string
          enum: ["sinceVersionRequiresNamespace", "changeFeedCannotExcludeDeleted"]
  - code: vta/app-state/list:watermarkTooOld
    meaning: The supplied `sinceVersion` predates the oldest tombstone the maintainer still retains, so a change feed from it would omit deletions and the consumer's copy would not converge. The consumer must rebuild from a snapshot rather than resume.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        oldestRetainedVersion:
          type: integer
          minimum: 1
        highWatermark:
          type: integer
          minimum: 1
  - code: vta/app-state/list:cursorInvalid
    meaning: The supplied `cursor` cannot be honoured — expired, malformed, or issued against maintainer state that no longer exists. Consumers SHOULD restart the enumeration without a cursor.
    retryable: true
related:
  - vta/app-state/get
  - vta/app-state/put
  - vta/app-state/delete
  - vta/app-state/get-many
  - vta/app-state/put-many
---

## Abstract

**VTA Application State — List** enumerates records in the VTA's
application-state store. It has two modes, and which one a request is in is
determined by a single member.

Without `sinceVersion` it is a **snapshot**: the live records in a context,
optionally narrowed to one namespace and one key prefix. This is what a UI
renders and what a cold start rebuilds from.

With `sinceVersion` it is a **change feed**: every record in the namespace
whose version exceeds the watermark, **tombstones included**. The tombstones
are the point. Without them a consumer pulling from a watermark learns about
every create and update and never learns about a delete — so deleted records
resurrect on its next rebuild and it disagrees with peers that saw the delete
live. This is the property most often omitted from a store like this and the
most expensive to add afterwards, because retrofitting it silently invalidates
every existing consumer's watermark.

That is also why a change feed cannot be asked to exclude deletions: it is a
contradiction, not a preference, and it is refused rather than honoured.

Agent memory's `list` returns *every entry in the context, in ascending key
order* — no prefix, no cursor. Application state and agent memory grow
independently, so a store shaped that way makes every application read pay for
the agent's memory. This task is prefix-scoped and cursor-paginated for that
reason.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the application) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/app-state/list/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`.
3. Populate `payload.namespace` whenever it supplies `payload.sinceVersion`.
4. **MUST NOT** set `includeDeleted` to false together with `sinceVersion`.
5. Treat `cursor` as opaque and re-send it verbatim.

A conforming producer performing incremental sync **MUST** persist
`highWatermark` from the response as its next `sinceVersion`, and **MUST NOT**
derive that watermark from the maximum `version` among the returned records —
which is wrong whenever a `prefix` filtered a later change out of the page, and
undefined when the page is empty. It **MUST NOT** advance its stored watermark
until it has drained the final page of the feed.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements). Where a `proof` is present, verify it.
2. Refuse a caller that lacks read access to application state in `contextId` with the framework's standard `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)), and **MUST NOT** degrade an unreachable context to an empty result. A maintainer whose authorization model can distinguish "no such context" from "not permitted to reach it" **MAY** answer the former with `vta/app-state:contextNotFound`; one whose ACL enumerates the contexts a caller may act in cannot, and answers `permissionDenied` to both.
4. Refuse with `vta/app-state/list:filterConflict` and `reason: "sinceVersionRequiresNamespace"` when `sinceVersion` is present and `namespace` is not; and with `reason: "changeFeedCannotExcludeDeleted"` when `sinceVersion` is present and `includeDeleted` is false.
5. In **snapshot** mode, return live records matching `namespace` and `prefix`, in ascending `key` order, and include tombstones still inside the retention window when `includeDeleted` is true.
6. In **change-feed** mode, return every record in the namespace — live and tombstoned — whose `version` is strictly greater than `sinceVersion`, filtered by `prefix` where supplied, in **ascending `version` order**, so that a consumer applying them in order reaches the same state the maintainer holds.
7. Refuse with `vta/app-state/list:watermarkTooOld` when `sinceVersion` predates the oldest tombstone it still retains. Resuming from such a watermark would silently omit deletions; the consumer must rebuild from a snapshot instead. A maintainer that never reaps tombstones never emits this code.
8. Populate `highWatermark` with the namespace's current counter value whenever `namespace` was supplied, and keep it stable across the pages of one enumeration.
9. Return only the metadata view unless `includeValues` is set — address, `version`, `deleted`, timestamps and `valueBytes`, with no `value`.
10. Apply its own page-size ceiling, set `truncated` and supply a `cursor` when more records remain, and refuse a cursor it cannot honour with `vta/app-state/list:cursorInvalid`.

A conforming consumer **SHOULD** retain tombstones for at least **30 days**, and **SHOULD** report the window it uses in `tombstoneRetentionSeconds` so a consumer can schedule syncs to stay inside it.

## Definitions

* **Snapshot mode.** No `sinceVersion`. Live records, optionally with tombstones under `includeDeleted`. Ordered by key.
* **Change-feed mode.** `sinceVersion` present. Every change after the watermark, tombstones always included. Ordered by version.
* **Watermark.** A value of the namespace's monotonic counter, marking how far a consumer has consumed. Defined in [`_shared/0.1/app-state-record.schema.json`](../../../_shared/0.1/app-state-record.schema.json); the counter is per `(contextId, namespace)`, which is why a change feed is namespace-scoped.
* **Tombstone.** A versioned marker recording that a record was deleted, reaped after the maintainer's retention window.
* **Metadata view.** A record without its `value`, carrying `valueBytes` so a consumer can decide what to fetch without fetching it.

## Request

### Snapshot of one record family

```json
{
  "id": "1f3a5c70-8b92-4d14-a6e8-0c2d4f6a8b91",
  "type": "https://trusttasks.org/spec/vta/app-state/list/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T11:00:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "prefix": "community/",
    "pageSize": 100
  }
}
```

### One round trip instead of a scan plus N gets

```json
{
  "id": "2a4b6d81-9ca3-4e25-b7f9-1d3e5a7b9c02",
  "type": "https://trusttasks.org/spec/vta/app-state/list/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T11:00:10Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "prefix": "contact/",
    "includeValues": true,
    "pageSize": 50
  }
}
```

### Resume an incremental sync

```json
{
  "id": "3b5c7e92-0db4-4f36-c80a-2e4f6b8c0d13",
  "type": "https://trusttasks.org/spec/vta/app-state/list/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T11:05:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "sinceVersion": 40,
    "includeValues": true
  }
}
```

## Response

The VTA responds with `type: https://trusttasks.org/spec/vta/app-state/list/1.0#response`, whose payload is `{ records, truncated, cursor?, highWatermark?, tombstoneRetentionSeconds? }`.

### Snapshot page, metadata only

```json
{
  "id": "4c6d8fa3-1ec5-4047-d91b-3f507c9d1e24",
  "type": "https://trusttasks.org/spec/vta/app-state/list/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T11:00:01Z",
  "threadId": "1f3a5c70-8b92-4d14-a6e8-0c2d4f6a8b91",
  "payload": {
    "records": [
      {
        "contextId": "personal",
        "namespace": "openvtc",
        "key": "community/acme",
        "version": 52,
        "deleted": false,
        "valueBytes": 95,
        "createdAt": "2026-07-02T14:10:00Z",
        "updatedAt": "2026-08-22T10:01:01Z"
      },
      {
        "contextId": "personal",
        "namespace": "openvtc",
        "key": "community/borealis",
        "version": 31,
        "deleted": false,
        "valueBytes": 88,
        "createdAt": "2026-05-19T07:20:00Z",
        "updatedAt": "2026-08-01T09:14:00Z"
      }
    ],
    "truncated": false,
    "highWatermark": 52,
    "tombstoneRetentionSeconds": 2592000
  }
}
```

### Change feed, with the deletion that makes it converge

The tombstone for `community/defunct` is why this feed is usable. A consumer
that received only the two live records would keep a record its peers deleted,
and would keep it through every subsequent incremental pull.

```json
{
  "id": "5d7e90b4-2fd6-4158-ea2c-406182ae2f35",
  "type": "https://trusttasks.org/spec/vta/app-state/list/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T11:05:01Z",
  "threadId": "3b5c7e92-0db4-4f36-c80a-2e4f6b8c0d13",
  "payload": {
    "records": [
      {
        "contextId": "personal",
        "namespace": "openvtc",
        "key": "community/defunct",
        "version": 44,
        "deleted": true,
        "createdAt": "2026-06-11T08:00:00Z",
        "updatedAt": "2026-08-18T16:05:00Z",
        "deletedAt": "2026-08-18T16:05:00Z"
      },
      {
        "contextId": "personal",
        "namespace": "openvtc",
        "key": "community/acme",
        "version": 52,
        "deleted": false,
        "value": {
          "label": "Acme Engineering",
          "joinedAt": "2026-07-02T14:10:00Z",
          "role": "admin"
        },
        "valueBytes": 95,
        "createdAt": "2026-07-02T14:10:00Z",
        "updatedAt": "2026-08-22T10:01:01Z"
      }
    ],
    "truncated": false,
    "highWatermark": 52,
    "tombstoneRetentionSeconds": 2592000
  }
}
```

The consumer stores 52. Note that it stores the `highWatermark`, not
`max(version)` over the records — here they coincide, but they do not when a
`prefix` filtered a later change out of the page.

### The watermark has aged out

```json
{
  "id": "6e80a1c5-30e7-4269-fb3d-51729bbf3046",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T11:06:00Z",
  "threadId": "3b5c7e92-0db4-4f36-c80a-2e4f6b8c0d13",
  "payload": {
    "code": "vta/app-state/list:watermarkTooOld",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/app-state/list/1.0",
      "id": "3b5c7e92-0db4-4f36-c80a-2e4f6b8c0d13"
    },
    "message": "Tombstones before version 38 have been reaped; resume from a snapshot.",
    "retryable": false,
    "details": {
      "oldestRetainedVersion": 38,
      "highWatermark": 52
    }
  }
}
```

## Security & Privacy

**Enumeration is the disclosure.** A caller that can list a namespace learns
the existence, count, key names and change timing of every record in it, with
no value ever leaving the maintainer. Key names in particular are chosen by
applications and often descriptive, so a metadata-only listing is not an
anonymised one. A maintainer granting read access to a third-party application
**SHOULD** scope the grant to a context, and **SHOULD** treat the ability to
enumerate as equivalent in sensitivity to the ability to read.

**Timing as a side channel.** `updatedAt` and the version ordering together
reveal how often an application writes and in what bursts. Where the requesting
caller is lower-trust, a maintainer **MAY** coarsen `updatedAt`; it **MUST
NOT** coarsen `version`, which the sync protocol depends on being exact.

**Cursor stability.** Cursors **MUST NOT** encode secret state, and a
maintainer **MUST** assume a cursor may be persisted and replayed much later —
returning `cursorInvalid` rather than silently reinterpreting it against
different state. The workspace convention is an opaque, MAC-signed token
carrying a last-key and snapshot identifier, so that one maintainer's cursor
cannot be replayed against another.

**Watermarks make deletion durable in a way values are not.** A tombstone
records that a record existed and was removed, and it is visible to every
consumer that can list the namespace for as long as it is retained. An
application deleting a record for privacy reasons should understand that the
key name survives the value, until the tombstone is reaped.

**Replay is benign.** The task is read-only and idempotent. A maintainer
**SHOULD** still apply the framework's `issuedAt` freshness check.

The optional `ext` member is part of the producer's signed surface; producers **MUST NOT** place data in `ext` they would not be comfortable signing.
