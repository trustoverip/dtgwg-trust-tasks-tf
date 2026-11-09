---
slug: vta/memory/list
version: "0.3"
extends: "0.2"
title: VTA Memory — List
summary: An agent recalls the memory items in a VTA context a page at a time, narrowed by key prefix, or syncs a local copy with a change feed that carries deletions.
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords:
  - memory
  - agent
  - recall
  - sync
  - change-feed
  - pagination
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Agent
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: >-
    The request changes no durable state, so the transport's authenticated sender
    is sufficient for it. The response hands the caller stored memory, and the
    SPEC §7.3 item 8 floor for a `discloses: secret` response applies — what was
    released has to stay attributable to the VTA that released it, on any
    transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Listing returns the agent's stored memories, which are secret to their owner. A captured list request keeps returning them for as long as it stays acceptable, so the acceptance window is what bounds the exposure.
sideEffects:
  level: none
  rationale: "Read-only recall of a context's memory items."
subjectPath: /contextId
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response returns stored memory values — free text the agent wrote, which
    routinely holds contact details and notes about people — unless the caller
    asks for the metadata view, and even then returns every key, which is
    descriptive in its own right. The request carries only the context, a key
    prefix and a watermark.
retention:
  class: exchange
  rationale: >-
    The VTA remains the record of the memory; the response is a working copy. A
    copy kept past the session is exactly what the change feed exists to keep
    current, and an agent that keeps one without syncing it has a copy no
    `vta/memory/delete` can reach.
errorCodes:
  - code: vta/memory/list:contextForbidden
    meaning: The caller is not permitted to read memory in the named context.
    retryable: false
  - code: vta/memory/list:filterConflict
    meaning: The supplied combination of members is not answerable — `sinceVersion` with `includeDeleted` set to false.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["reason"]
      properties:
        reason:
          type: string
          enum: ["changeFeedCannotExcludeDeleted"]
  - code: vta/memory/list:watermarkTooOld
    meaning: The supplied `sinceVersion` predates the oldest tombstone the VTA still retains, so a change feed from it would omit deletions and the consumer's copy would not converge. The consumer must rebuild from a snapshot rather than resume.
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
          minimum: 0
  - code: vta/memory/list:cursorInvalid
    meaning: The supplied `cursor` cannot be honoured — expired, malformed, or issued against state that no longer exists. Consumers SHOULD restart the enumeration without a cursor.
    retryable: true
  - code: vta/memory/list:purposeNotGranted
    meaning: The request asks for values (`includeValues` true, the default) for a purpose the caller's ACL entry does not grant, or names no purpose under an entry listing more than one. A listing without values discloses no content and is never refused for its purpose. The refusal discloses no memory content and is distinct from `contextForbidden`, which refuses access itself (VTI-ACL-104).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [purposeNamed]
      properties:
        purposeNamed:
          type: boolean
          description: Whether the request named a purpose. False when it named none under an entry listing more than one.
        purpose:
          type: string
          description: The purpose the request named, when it named one.
related:
  - vta/memory/get
  - vta/memory/put
  - vta/memory/delete
  - vta/app-state/list
---

## Abstract

The **VTA Memory — List** Trust Task enumerates the memory items in one VTA
context. `0.1` returned the whole context in one response, with no way to
narrow it, page it, or learn what had changed. This version has two modes:

- **Snapshot** (no `sinceVersion`): the live items, in key order, optionally
  narrowed to a key `prefix` and paged with `cursor` and `limit`.
- **Change feed** (`sinceVersion` supplied): every item whose version exceeds
  the watermark, **tombstones included**, in version order. An agent keeping a
  local copy on a second device stores the response's `highWatermark`, asks
  again from it later, and applies what comes back in order. Deletions arrive as
  tombstones, so the copy converges instead of keeping forgotten items forever.

The modes and their rules are `vta/app-state/list`'s, over one context instead
of one namespace. To recall items by name, use
[`vta/memory/get`](../../get/0.2/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.2

- **`purpose`** (optional) is added to the request: the purpose the caller will use the values it reads for (VTI-ACL-100 – 110, `MemoryPurpose`).
- **`vta/memory/list:purposeNotGranted`** is new. It refuses a listing **with values** for a purpose the caller's entry does not grant; a listing without values (`includeValues: false`) discloses only keys and versions and is not purpose-bound.
- The audit record of a listing with values carries its purpose.
- `extends: "0.2"`: a 0.2 consumer reads a 0.3 list as a 0.2 list, ignoring `purpose`.

## Changes from 0.1 to 0.2

- New request members: `prefix`, `sinceVersion`, `includeValues`,
  `includeDeleted`, `cursor`, `limit`.
- The response's `items` are `MemoryRecord`s — `key`, `value`, `version`,
  `deleted`, timestamps and `valueBytes` — instead of bare `{key, value}`.
  Values are still returned by default.
- The response gains `highWatermark` (required), `nextCursor` and
  `tombstoneRetentionSeconds`.
- New error codes: `filterConflict`, `watermarkTooOld`, `cursorInvalid`.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the agent) **MUST**:

1. Emit a *Trust Task document* of type `https://trusttasks.org/spec/vta/memory/list/0.3` with itself as `issuer` and the VTA as `recipient`.
2. Treat `cursor` as opaque, and send back only a cursor this VTA issued.
3. When syncing, store `highWatermark` as its next `sinceVersion` only after draining the final page (the one without `nextCursor`), and on `watermarkTooOld` rebuild from a snapshot rather than resume.

A producer **SHOULD** name the `purpose` it will use what it reads for, and **MUST** name one where its ACL entry lists more than one; it **MUST NOT** expect `any` to grant `modelTraining`.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements), and sign the response.
2. Refuse with `vta/memory/list:contextForbidden` when the caller lacks read access to memory in `payload.contextId`.
3. Refuse `sinceVersion` together with `includeDeleted: false` with `filterConflict`.
4. In snapshot mode, return live items (and tombstones only when `includeDeleted` is true) whose key begins with `prefix`, in ascending byte order of key. In change-feed mode, return every item and tombstone with `version > sinceVersion`, narrowed by `prefix` if given, in ascending version order.
5. Return values unless `includeValues` is false; return `valueBytes` on every live item.
6. Return `highWatermark`, the context's current counter value, on every page; it **MUST NOT** be the highest version among the returned items.
7. Refuse with `watermarkTooOld` a `sinceVersion` older than the oldest tombstone it has reaped, rather than answer a feed that silently omits deletions.
8. Issue cursors that resume the same enumeration without repeating or skipping an item that was present throughout, and refuse one it did not issue or can no longer honour with `cursorInvalid`.
9. Treat a listing that returns values as **purpose-bound** (VTI-ACL-102): refuse it with `vta/memory/list:purposeNotGranted` where the request names a purpose the caller's ACL entry does not grant, or names none under an entry listing more than one. `any` grants every purpose but `modelTraining` (VTI-ACL-105). Where the entry lists exactly one purpose, a request naming none is read as naming it. A listing with `includeValues: false` is not purpose-bound.
10. Record in the audit record of every purpose-bound listing, served or refused, the purpose it named, or that it named none (VTI-ACL-106).

## Authorization

Authority is **read access to memory in the named context**, held on the VTA's
context ACL. The boundary is the context, never the item: a caller that may list
one item may list them all, and `prefix` narrows a response, not a grant.

The request's proof is OPTIONAL because the read changes nothing; the caller is
identified by the transport's authenticated sender, and authorization is checked
against that identity.

A listing that returns values is **purpose-bound** (VTI-ACL-102): the
caller's ACL entry states what it may use what it reads for (`purposes`), and
values are returned only for a purpose the entry grants. The purpose is the
caller's declaration — the VTA cannot see what the caller does with what it
read — and its value is that it is recorded, at the moment of the read, in an
audit record the caller cannot alter.

## Request

### Recall one family of items

```json
{
  "id": "8e4a5f1d-3b77-4c54-af43-6d0b5e9a7c23",
  "type": "https://trusttasks.org/spec/vta/memory/list/0.3",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-09T14:00:00Z",
  "payload": {
    "contextId": "finance",
    "prefix": "contact/",
    "limit": 50
  }
}
```

### Resume a sync on a second device

```json
{
  "id": "9f5b6a2e-4c88-4d65-b054-7e1c6f0b8d34",
  "type": "https://trusttasks.org/spec/vta/memory/list/0.3",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-09T14:10:00Z",
  "payload": {
    "contextId": "finance",
    "sinceVersion": 52
  }
}
```

## Response

### Change feed, with the deletion that makes it converge

```json
{
  "id": "c4eb6a3a-9f5d-4a4c-9d7e-3f0a8b2c5d94",
  "type": "https://trusttasks.org/spec/vta/memory/list/0.3#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkFinanceAgent",
  "issuedAt": "2026-11-09T14:10:01Z",
  "threadId": "9f5b6a2e-4c88-4d65-b054-7e1c6f0b8d34",
  "payload": {
    "items": [
      {
        "contextId": "finance",
        "key": "contact/billing",
        "version": 54,
        "deleted": true,
        "updatedAt": "2026-11-09T13:00:01Z",
        "deletedAt": "2026-11-09T13:00:01Z"
      },
      {
        "contextId": "finance",
        "key": "preference/invoice-format",
        "version": 55,
        "value": "PDF, one file per invoice",
        "valueBytes": 25,
        "deleted": false,
        "createdAt": "2026-11-09T13:30:00Z",
        "updatedAt": "2026-11-09T13:30:00Z"
      }
    ],
    "highWatermark": 55,
    "tombstoneRetentionSeconds": 2592000
  }
}
```

## Security & Privacy

**The change feed is a disclosure of history as well as state.** A feed from an
old watermark returns tombstones for items forgotten since, so a caller learns
the keys of things that were deleted. That is the price of convergence and is
bounded by the tombstone retention window.

### Data carried

The request carries the context, an optional key prefix and watermark, and a
cursor the VTA issued. The response carries stored memory values unless
`includeValues` is false, and every returned key in any case; keys are
descriptive in their own right. Tombstones carry a key and timestamps, never a
value.

### Correlation

All items in a context are joinable by construction, which is the point of the
store. Versions and the `highWatermark` reveal how much the context has been
written since a caller last looked. Across contexts nothing joins automatically,
but agents tend to reuse keys (`preference/timezone`), so an operator holding
several contexts for one principal can align them on key alone.

### Retention

The VTA keeps nothing new because of a list. The caller keeps what it was sent,
and an agent that caches it **SHOULD** keep the cache current with the change
feed, so that a `vta/memory/delete` reaches its copy. **Audit.** A VTA **SHOULD**
record each list with the requesting DID, the context, and the number of items
returned and a digest of their keys, so that "which agent saw what" can be
answered later; it **SHOULD NOT** copy values into the audit trail.

### Consent/purpose

The purpose is recall inside one context. A VTA **MUST NOT** return an item
written in one context to a caller scoped to another, whatever `prefix` it
supplies.
