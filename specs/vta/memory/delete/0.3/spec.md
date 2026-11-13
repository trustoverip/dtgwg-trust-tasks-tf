---
slug: vta/memory/delete
version: "0.3"
extends: "0.2"
title: VTA Memory — Delete
summary: An agent forgets a memory item, leaving a tombstone so its other devices learn of the deletion, optionally only if the item is still at the version it last read.
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords:
  - memory
  - agent
  - forget
  - tombstone
  - optimistic-concurrency
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
  requirement: REQUIRED
  rationale: Forgetting is a mutation of durable per-context state the VTA audits; the audit record must name the agent key that asked, independent of the transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A delete replayed after the item was written again would forget the newer item. The issue time is what lets the VTA refuse a stale copy.
sideEffects:
  level: destructive
  rationale: "The item's value is discarded and cannot be recovered from the VTA. The tombstone that remains carries the key and version only."
subjectPath: /contextId
exposure:
  discloses: secret
  ingests: personal
  actsAsSubject: false
  rationale: "`key` is the agent-chosen label from `vta/memory/put` and is descriptive in its own right. A success discloses nothing back, but a `versionConflict` refusal carries the item's current value — the edit the caller was about to discard — which `vta/memory/list` classifies as secret."
retention:
  class: durable
  rationale: The tombstone outlives the item for the VTA's tombstone retention window, so that every device syncing the context learns of the deletion; it is then reaped.
errorCodes:
  - code: vta/memory/delete:contextForbidden
    meaning: The caller is not permitted to forget memory in the named context.
    retryable: false
  - code: vta/memory/delete:versionConflict
    meaning: The `expectedVersion` precondition failed. The details carry the maintainer's current version and value, so the caller can see the edit it was about to discard.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["reason"]
      properties:
        reason:
          type: string
          enum: ["versionMismatch", "recordAbsent"]
        currentVersion:
          type: integer
          minimum: 1
        currentValue:
          type: string
        currentDeleted:
          type: boolean
  - code: vta/memory/delete:purposeNotGranted
    meaning: The request names a purpose the caller's ACL entry does not grant. A delete discloses nothing, so a request that names *no* purpose is never refused for it; but one that names a purpose is held to it. The refusal discloses no memory content and is distinct from `contextForbidden`, which refuses access itself (VTI-ACL-104).
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
  - vta/memory/put
  - vta/memory/list
  - vta/memory/get
  - vta/app-state/delete
---

## Abstract

The **VTA Memory — Delete** Trust Task forgets one memory item. `0.1` deleted
the item outright, which left every other copy — an agent's cache on a second
device, a sync peer — holding it forever, because nothing told them it was
gone. In this version the value is discarded and a **tombstone** takes its
place: the key, the version the delete took, and when. A device syncing with
[`vta/memory/list`](../../list/0.3/spec.md)'s change feed receives the
tombstone and drops its copy, and the tombstone is reaped after the VTA's
retention window.

An optional `expectedVersion` makes the delete conditional, so an agent cannot
forget an edit it never saw. Deleting a key that holds nothing is a success:
the caller wanted the item gone, and it is.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.2

- **`purpose`** (optional) is added to the request: the purpose the caller will use the content a conflict discloses for (VTI-ACL-100 – 110, `MemoryPurpose`).
- **`vta/memory/delete:purposeNotGranted`** is new, for a request naming a purpose its caller's ACL entry does not grant.
- A `versionConflict` carries `currentValue` only where the request's purpose is granted.
- `extends: "0.2"`: a 0.2 consumer reads a 0.3 delete as a 0.2 delete, ignoring `purpose`.

## Changes from 0.1 to 0.2

- A delete leaves a **tombstone** instead of removing every trace.
- **`expectedVersion`** (optional, positive) is added, with
  `vta/memory/delete:versionConflict`.
- The **response** gains `contextId` and `existed` (required) and `version` and
  `deletedAt` (optional).
- Deleting a key that holds nothing **succeeds** with `existed: false`.
- `sideEffects.level` is `destructive`: the value cannot be recovered.

A `0.1` delete against a VTA serving both versions **MUST** leave a tombstone
too, since the store is one store and a `0.2` reader syncing it must converge.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the agent) **MUST**:

1. Emit a *Trust Task document* of type `https://trusttasks.org/spec/vta/memory/delete/0.3` with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` and `payload.key`.
3. Include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A producer that wants a conflict to carry the current value **SHOULD** name the `purpose` it will use that value for — under an entry listing more than one purpose a conflict otherwise omits it — and **MUST NOT** expect `any` to grant `modelTraining`.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Refuse with `vta/memory/delete:contextForbidden` when the caller lacks write access to memory in `payload.contextId`.
3. Evaluate `expectedVersion`, when present, before deleting: apply only if a live item exists at exactly that version. On mismatch → `versionConflict` with `reason: "versionMismatch"`; with no live item → `versionConflict` with `reason: "recordAbsent"`. `currentVersion`, `currentValue` and `currentDeleted` are populated on the terms of [`vta/memory/put`](../../put/0.3/spec.md) item 5, including its read-access exception.
4. On a live item: discard its value, write a tombstone at the context's next counter value, and answer `existed: true` with that `version` and `deletedAt`. The evaluation in item 3 and this write are one atomic step.
5. On a key already holding a tombstone: change nothing and answer `existed: false` with the tombstone's existing `version` and `deletedAt`. On a key holding nothing: answer `existed: false` with neither.
6. Retain tombstones for a documented window (see `vta/memory/list`'s `tombstoneRetentionSeconds`) and then reap them. A VTA **MUST NOT** keep the deleted value, in the tombstone or elsewhere, beyond what its backups already hold.
7. Where the request names a `purpose` the caller's ACL entry does not grant, refuse with `vta/memory/delete:purposeNotGranted` before deleting. The delete itself is not purpose-bound: a request that names no purpose is not refused for it.
8. Include `currentValue` in a `versionConflict` only where the request's purpose is granted (VTI-ACL-102), and record the purpose in the audit record of a conflict that carries it (VTI-ACL-106).

## Authorization

Authority is **write access to memory in the named context**, the same grant
`vta/memory/put` requires: forgetting is a write. The read-access rule for a
conflict's `currentValue` is `vta/memory/put`'s.

The required `proof` attributes the deletion to an agent key for the audit
record; it is not the authorization, which is checked after the signature is
settled ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

A conflict that carries `currentValue` is a **purpose-bound** disclosure
(VTI-ACL-102), granted only for a purpose the caller's entry grants
(`purposes`). The delete is not purpose-bound, so naming no purpose never
blocks it; naming one the entry does not grant does.

## Request

```json
{
  "id": "7d3f4e0c-2a66-4b43-9e32-5c9a4d8f6b12",
  "type": "https://trusttasks.org/spec/vta/memory/delete/0.3",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-09T13:00:00Z",
  "payload": {
    "contextId": "finance",
    "key": "contact/billing",
    "expectedVersion": 53
  }
}
```

## Response

```json
{
  "id": "b3da5f2f-8e4c-4f3b-8c6d-2e9f7a1b4c83",
  "type": "https://trusttasks.org/spec/vta/memory/delete/0.3#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkFinanceAgent",
  "issuedAt": "2026-11-09T13:00:01Z",
  "threadId": "7d3f4e0c-2a66-4b43-9e32-5c9a4d8f6b12",
  "payload": {
    "contextId": "finance",
    "key": "contact/billing",
    "existed": true,
    "version": 54,
    "deletedAt": "2026-11-09T13:00:01Z"
  }
}
```

## Security & Privacy

**Forgetting at the VTA is not forgetting everywhere.** The tombstone tells
cooperating devices to drop their copies; it cannot reach a copy an agent
exported, summarised into another item, or sent elsewhere. A principal asking
for an item to be forgotten should be told that this task covers the VTA and
the devices that sync from it.

**Retry safety.** A delete converges: a replay finds the tombstone and answers
`existed: false`. A replay of an old delete after the key was written again
would forget the newer item, which is what `issuedAt` and `expectedVersion`
guard against.

### Data carried

The request carries the context and the key — the name of the thing being
forgotten, which is descriptive in its own right. A success carries no value. A
`versionConflict` carries the item's current value, subject to the read-access
rule above. `ext` is part of the producer's signed surface and **MUST NOT** carry
anything the producer would not put in a memory value.

### Correlation

A tombstone keeps the key for its retention window, so a caller listing the
context with `includeDeleted` learns that an item with that name existed and
when it was forgotten. The key of a forgotten item is therefore not forgotten
until the tombstone is reaped; an agent that must not leave even the name
behind should choose keys that do not describe their subject.

### Retention

The value is discarded at once. The tombstone — key, version, timestamps — is
kept for the documented window and then reaped. **Audit.** A VTA **SHOULD**
record each delete with the requesting DID, the context, the key and the
tombstone's version, and **MUST NOT** record the discarded value.

### Consent/purpose

The purpose is to let an agent, or a principal through it, withdraw something
from the agent's memory. A VTA **SHOULD** make deletion available to the
principal directly, not only to the agents writing the context, since the
principal is the one with a right to have it forgotten.
