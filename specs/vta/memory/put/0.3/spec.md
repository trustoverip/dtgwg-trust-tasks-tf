---
slug: vta/memory/put
version: "0.3"
extends: "0.2"
title: VTA Memory — Put
summary: An agent stores a memory item in a VTA context, optionally conditional on the version it last read, and receives a typed conflict carrying the current version and value when another writer got there first.
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords:
  - memory
  - agent
  - context
  - cross-session
  - recall
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
  rationale: A memory write mutates durable per-context state the VTA audits and may replay; transport-independent integrity is required, so that an audit record read later names the agent key that wrote rather than the session it arrived on.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A put overwrites what the agent remembers, and the agent acts on it. A stale copy therefore restores a superseded belief rather than merely an old value.
sideEffects:
  level: mutating
  rationale: "Creates or replaces one memory item. Recoverable — the prior value is overwritten but the key remains, and a conditional write cannot clobber a version it did not see."
subjectPath: /contextId
exposure:
  discloses: secret
  ingests: personal
  actsAsSubject: false
  rationale: "`value` is unstructured agent-authored text about whatever the agent was asked to remember, and `key` is an agent-chosen label that is itself descriptive; the VTA reads and stores both in the clear. A success discloses nothing back, but a `versionConflict` refusal carries the item's current value — the stored memory itself, which `vta/memory/list` classifies as secret — to the caller that lost the race."
retention:
  class: durable
  rationale: Surviving the session that wrote it is the entire purpose of the task; an item persists until it is overwritten or removed with `vta/memory/delete`.
errorCodes:
  - code: vta/memory/put:contextForbidden
    meaning: The caller is not permitted to write memory in the named context.
    retryable: false
  - code: vta/memory/put:versionConflict
    meaning: The `expectedVersion` precondition failed. The details carry the maintainer's current version and value, so the caller can resolve without a re-read.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["reason"]
      properties:
        reason:
          type: string
          enum: ["versionMismatch", "recordExists", "recordAbsent"]
        currentVersion:
          type: integer
          minimum: 1
        currentValue:
          type: string
        currentDeleted:
          type: boolean
  - code: vta/memory/put:valueTooLarge
    meaning: The value exceeds the VTA's documented per-item cap. The details carry both the cap and the actual size.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["limitBytes", "actualBytes"]
      properties:
        limitBytes:
          type: integer
          minimum: 0
        actualBytes:
          type: integer
          minimum: 0
  - code: vta/memory/put:purposeNotGranted
    meaning: The request names a purpose the caller's ACL entry does not grant. A write discloses nothing, so a request that names *no* purpose is never refused for it; but one that names a purpose is held to it. The refusal discloses no memory content and is distinct from `contextForbidden`, which refuses access itself (VTI-ACL-104).
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
  - vta/memory/list
  - vta/memory/delete
  - vta/app-state/put
---

## Abstract

The **VTA Memory — Put** Trust Task stores a free-form **memory item** in a VTA
**context**, so an AI agent can *remember across sessions* while its memory
stays **scoped to one context**. Memory is opaque agent text — facts,
summaries, preferences — keyed by an agent-chosen `key`, and distinct from the
password vault (`vault/*`) and the credential store (`vault/credentials/*`).

`0.1` was an unconditional upsert: two agents (or one agent on two devices)
writing the same key overwrote each other, and neither could tell afterwards.
This version gives every item a **version**, assigned by the VTA and returned
on every write, and lets a writer make its write conditional on it. A writer
that read version 47 sends `expectedVersion: 47`; if another writer has moved
the item on, the write is refused, and the refusal — `versionConflict` —
carries the item's *current version and value*. The loser is handed the
winner's view, so it can merge and re-issue without a re-read that would race
the next write. `expectedVersion: 0` writes only if no live item exists:
"remember this unless something is already remembered here".

The semantics are `vta/app-state/put`'s, applied to memory: the same counter,
the same precondition, the same conflict. They are restated here rather than
referenced so that this specification can be implemented on its own.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.2

- **`purpose`** (optional) is added to the request: the purpose the caller will use the content a conflict discloses for (VTI-ACL-100 – 110, `MemoryPurpose`).
- **`vta/memory/put:purposeNotGranted`** is new, for a request naming a purpose its caller's ACL entry does not grant.
- A `versionConflict` carries `currentValue` only where the request's purpose is granted, as well as only to a caller with read access.
- `extends: "0.2"`: a 0.2 consumer reads a 0.3 put as a 0.2 put, ignoring `purpose`.

## Changes from 0.1 to 0.2

- **`expectedVersion`** (optional) is added to the request.
- The **response** gains `contextId`, `version`, `wasCreated` and `updatedAt`
  (required) and `valueBytes` (optional), so a writer can chain conditional
  writes without reading in between.
- **`vta/memory/put:versionConflict`** is new, with details carrying the
  current version and value. `valueTooLarge` now carries `limitBytes` and
  `actualBytes`.
- `contextId` must be non-empty, and `key` is bounded: 1–512 characters, no
  NUL (`MemoryKey`). `0.1` constrained neither.
- `exposure.discloses` is `secret`, not `none`: a conflict returns the stored
  value, which `vta/memory/list` already classifies as secret.

A `0.2` put without `expectedVersion` behaves exactly as a `0.1` put. Both
versions write one store: a VTA serving both **MUST** give an item written by
`0.1` a version like any other write, so a `0.2` reader can condition on it.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the agent) **MUST**:

1. Emit a *Trust Task document* of type `https://trusttasks.org/spec/vta/memory/put/0.3` with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`, `payload.key`, and `payload.value`.
3. Include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A producer that intends a read-modify-write **SHOULD** supply `expectedVersion`
from the read, and **SHOULD** treat a `versionConflict` as data rather than as a
reason to re-read: the details already carry the current version and value.

A producer that wants a conflict to carry the current value **SHOULD** name the `purpose` it will use that value for — under an entry listing more than one purpose a conflict otherwise omits it — and **MUST NOT** expect `any` to grant `modelTraining`.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Refuse with `vta/memory/put:contextForbidden` when the caller lacks write access to memory in `payload.contextId` (the same context ACL that gates keys).
3. Enforce a **documented** per-item cap on `value` and refuse an oversized write with `vta/memory/put:valueTooLarge`, carrying `limitBytes` and `actualBytes`. A VTA **MUST NOT** silently truncate a value. The RECOMMENDED cap is **65536 bytes** over the UTF-8 encoding of `value`.
4. Evaluate `expectedVersion` before writing:
   - absent → unconditional upsert;
   - `0` → apply only if no **live** item exists at the key. A tombstone is not a live item, so the write applies over one and `wasCreated` is true. If a live item exists → `versionConflict` with `reason: "recordExists"`;
   - positive → apply only if the live item's `version` equals it exactly. On mismatch → `versionConflict` with `reason: "versionMismatch"`. If no live item exists → `versionConflict` with `reason: "recordAbsent"`.
5. Populate `currentVersion` and `currentValue` in the conflict details whenever the key holds a live item, and `currentDeleted: true` where it holds a tombstone. This is normative rather than a courtesy: a consumer that omits them reintroduces the re-read race this version exists to remove. The one exception is a caller without read access to the context (see [Authorization](#authorization)): `currentValue` **MUST** then be omitted.
6. Assign the written item the next value of the context's counter (see `MemoryRecord` in [`vta/_shared/0.1/memory-record`](../../../_shared/0.1/memory-record.schema.json)) and return it as `version`. The evaluation in item 4 and the write are one atomic step: no other write in the context may take effect between them.
7. **MUST NOT** interpret, summarise or rewrite `value`.
8. Where the request names a `purpose` the caller's ACL entry does not grant — `any` grants every purpose but `modelTraining`; a list grants exactly its members — refuse with `vta/memory/put:purposeNotGranted` before writing. The write itself is not purpose-bound: a request that names no purpose is not refused for it.
9. Include `currentValue` in a `versionConflict` only where the request's purpose is granted (VTI-ACL-102): where it names none under an entry listing more than one purpose, omit `currentValue` as for a caller without read access. A conflict that carries `currentValue` is a read: record the purpose it was disclosed for in its audit record, or that none was named (VTI-ACL-106).

## Authorization

Authority is **write access to memory in the named context**, held on the VTA's
context ACL — the same boundary that gates keys, vault entries and policy. This
task adds no scope of its own, and an item carries no access control of its own:
a caller that may write one item in a context may write any of them.

A conflict returns the item's current value, which is a read. Where the VTA's
ACL grants memory write without memory read — an agent that may record but not
recall — the VTA **MUST** omit `currentValue` from a conflict sent to such a
caller, and **MAY** still send `currentVersion`, which reveals only that the
item changed. A VTA whose ACL grants one context access for both, as `0.1`
assumed, sends both.

The required `proof` establishes *who authored the write*, so the resulting item
and its audit entry can be attributed to an agent key. It is not the
authorization — a correctly signed request from a caller without write access is
refused, and the access check happens after the signature is settled
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

A conflict that carries `currentValue` is a **purpose-bound** disclosure
(VTI-ACL-102): the caller's entry states what it may use what it reads for
(`purposes`), and the value is disclosed only for a purpose the entry grants.
The write is not purpose-bound, so naming no purpose never blocks it; naming
one the entry does not grant does, because the caller has declared a use it was
not given.

## Request

### Unconditional upsert, as in 0.1

```json
{
  "id": "5b1d2c8a-0e44-4f21-9c10-3a7e2b6d4f90",
  "type": "https://trusttasks.org/spec/vta/memory/put/0.3",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-09T12:00:00Z",
  "payload": {
    "contextId": "finance",
    "key": "contact/billing",
    "value": "billing@acme.example — prefers PDF"
  }
}
```

### Conditional write, from a version a read returned

```json
{
  "id": "6c2e3d9b-1f55-4a32-8d21-4b8f3c7e5a01",
  "type": "https://trusttasks.org/spec/vta/memory/put/0.3",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-09T12:05:00Z",
  "payload": {
    "contextId": "finance",
    "key": "contact/billing",
    "value": "billing@acme.example — prefers PDF; cc ap@acme.example",
    "expectedVersion": 47
  }
}
```

## Response

### Written

```json
{
  "id": "a2c94f1e-7d3b-4e2a-9b5c-1d8e6f0a3b72",
  "type": "https://trusttasks.org/spec/vta/memory/put/0.3#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkFinanceAgent",
  "issuedAt": "2026-11-09T12:05:01Z",
  "threadId": "6c2e3d9b-1f55-4a32-8d21-4b8f3c7e5a01",
  "payload": {
    "contextId": "finance",
    "key": "contact/billing",
    "version": 53,
    "wasCreated": false,
    "updatedAt": "2026-11-09T12:05:01Z",
    "valueBytes": 56
  }
}
```

### Conflict — the loser is handed the winner's view

```json
{
  "id": "d6139fc7-80ea-42b5-9f6c-7e91b3da5264",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkFinanceAgent",
  "issuedAt": "2026-11-09T12:05:01Z",
  "threadId": "6c2e3d9b-1f55-4a32-8d21-4b8f3c7e5a01",
  "payload": {
    "code": "vta/memory/put:versionConflict",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/memory/put/0.3",
      "id": "6c2e3d9b-1f55-4a32-8d21-4b8f3c7e5a01"
    },
    "message": "Item is at version 52; expected 47.",
    "retryable": false,
    "details": {
      "reason": "versionMismatch",
      "currentVersion": 52,
      "currentValue": "billing@acme.example — invoices by post only"
    }
  }
}
```

The agent now has what it needs to decide — keep the other writer's value, or
merge and re-issue with `expectedVersion: 52` — without a re-read.

## Security & Privacy

**A precondition is not a lock.** `expectedVersion` gives compare-and-set, not
mutual exclusion over a sequence of operations.

**Retry safety.** A put without `expectedVersion` is not safely replayable: a
replay writes twice and takes two counter values, so a device syncing the
context sees a change that never happened. A put with `expectedVersion`
converges — the replay fails its own precondition. Implementations **SHOULD**
nonetheless treat the task as requiring a client-supplied idempotency key,
because the class is per task type and a caller reading the classification
cannot see which shape a given request took.

### Data carried

`value` is the sensitive member, and it is unstructured: whatever the agent
decided to remember about a person lands there verbatim. The example stores a
working email address and a correspondence preference, which is the ordinary
case rather than a worst case. `key` is content too — `contact/billing` says
something about the subject even to a party that cannot read the value. Nothing
here is sealed: the VTA necessarily reads `value` in cleartext in order to store
it, and a `versionConflict` hands the current value to the caller that lost.

A producer **SHOULD** store the fact it needs to recall rather than the
conversation it learned it from, and **MUST NOT** use `value` as an overflow
store for credential material that belongs behind a sealed vault write. `ext` is
part of the producer's signed surface; a producer **MUST NOT** place there
anything it would not put in `value`.

### Correlation

Everything under one `contextId` is joinable by construction, and that is the
point of the store; the isolation boundary is the context, never the item. Each
write takes the next value of the context's counter, so the versions a caller
sees reveal how much other memory traffic the context carried in between — to a
caller that may write the context anyway.

The `issuer` DID is deliberately stable: cross-session recall depends on the
same agent being recognised on its next connection. That stability is scoped to
the agent's relationship with its own VTA, which is why the Agent party declares
`identifierScope: pairwise`.

### Retention

Durable, and by design: an item ends only when it is overwritten or
[`vta/memory/delete`](../../delete/0.3/spec.md) removes it. This version adds no
expiry, so a VTA accumulates memory indefinitely unless its operator imposes a
policy of its own, which implementers **SHOULD** expose to the principal.

**Audit.** A VTA **SHOULD** record each write with the requesting DID, the
context, the key, the resulting version and whether the item was created — and
**SHOULD NOT** copy the value into the audit trail, which would give it a second
and usually longer retention.

### Consent/purpose

The purpose is recall inside one context: an agent stores what it needs to do
the work that context exists for. The context ACL is the record of the basis on
which the write happens, and that basis does not travel with the item: a VTA
**MUST NOT** surface an item written in one context to an agent scoped to a
different one. Whether a human is asked before an agent commits something to
long-term memory is a consumer policy question this specification takes no
position on.
