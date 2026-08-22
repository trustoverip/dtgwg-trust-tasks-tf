---
slug: vta/app-state/put
version: "1.0"
title: VTA Application State — Put
summary: An application writes one versioned state record to a VTA, optionally conditional on the version it last read, and receives a typed conflict carrying the maintainer's current version and value when that precondition fails.
status: draft
targetFrameworkVersion: "0.4"
category: data-exchange
keywords:
  - vta
  - application-state
  - versioned
  - optimistic-concurrency
  - merge-patch
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
  requirement: REQUIRED
  rationale: A write mutates durable state that an account's recoverability depends on, and the maintainer audits it. Attribution must survive the transport that carried the request, so that an audit record read later names the application key that wrote, not merely the session it arrived on.
sideEffects:
  level: mutating
  rationale: "Creates or replaces one record. Recoverable — the prior value is overwritten but the address remains, and a conditional write cannot clobber a version it did not see."
subjectPath: /contextId
exposure:
  discloses: none
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
  - code: vta/app-state/put:versionConflict
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
        currentValue: {}
        currentDeleted:
          type: boolean
  - code: vta/app-state/put:valueTooLarge
    meaning: The value exceeds the maintainer's documented per-record cap. The details carry both the cap and the actual size, so the caller learns how far over it is rather than only that it failed.
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
  - code: vta/app-state/put:notFound
    meaning: A `mergePatch` write named an address holding no live record. A patch has nothing to apply to; the caller must send a whole `value`.
    retryable: false
related:
  - vta/app-state/get
  - vta/app-state/list
  - vta/app-state/delete
  - vta/app-state/get-many
  - vta/app-state/put-many
  - vta/memory/put
---

## Abstract

**VTA Application State — Put** writes one record to the VTA's
application-state store, at the address `(contextId, namespace, key)`.

Two things distinguish it from the upsert an application would otherwise
hand-roll on agent memory, and both exist because a store without them cannot
be shared safely by two instances of the same application.

The first is the **version**, which the maintainer assigns and the response
returns. `MemoryItem` is `{key, value}` — no version, no timestamp, nothing to
hang a precondition on — so two writers overwrite each other and, worse,
neither can detect it afterwards. Here a writer may supply `expectedVersion`
and the write applies only if the record is still at the version the writer
read.

The second is what a **failed precondition returns**. A bare rejection obliges
the caller to re-read, and between the rejection and the re-read the record can
change again; the pattern has no fixed point under contention. So
`vta/app-state/put:versionConflict` carries the maintainer's *current version
and current value* with the rejection. The loser is handed the winner's view,
which removes the race rather than narrowing it.

`expectedVersion: 0` — "create only, fail if a live record exists" — is what
makes lease acquisition safe. Without it two instances can each read "absent",
each write, and each believe it won.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the application) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/app-state/put/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`, `payload.namespace`, `payload.key`, and **exactly one** of `payload.value` or `payload.mergePatch`.
3. Include a `proof` per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).
4. **MUST NOT** place secret material in `value` or `mergePatch` — see [Security & Privacy](#security--privacy).

A conforming producer that intends a read-modify-write **SHOULD** supply `expectedVersion` from the read, and **SHOULD** treat a `versionConflict` as data rather than as a reason to re-read: the details already carry the maintainer's current version and value.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Refuse a caller that lacks write access to application state in `contextId` with the framework's standard `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)). A maintainer whose authorization model can distinguish "no such context" from "not permitted to reach it" **MAY** answer the former with `vta/app-state:contextNotFound`; one whose ACL enumerates the contexts a caller may act in cannot, and answers `permissionDenied` to both.
3. Enforce a **documented** per-record cap on the stored value and refuse an oversized write with `vta/app-state/put:valueTooLarge`, carrying both `limitBytes` and `actualBytes`. A maintainer **MUST NOT** silently truncate or drop an oversized write — refusing loudly at a knowable cap is the whole requirement, and a limit that drops a write silently has already cost a real deployment a lost join. The RECOMMENDED cap is **65536 bytes** measured over the UTF-8 encoding of the [[RFC8785]](https://www.rfc-editor.org/rfc/rfc8785) canonicalization of the value.
4. Evaluate `expectedVersion` before writing:
   - absent → unconditional upsert;
   - `0` → apply only if no **live** record exists. A tombstone is not a live record, so the write applies over one, and `created` is true. If a live record exists → `versionConflict` with `reason: "recordExists"`.
   - positive → apply only if the live record's `version` equals it exactly. On mismatch → `versionConflict` with `reason: "versionMismatch"`. If no live record exists → `versionConflict` with `reason: "recordAbsent"`.
5. Populate `currentVersion` and `currentValue` in the conflict details whenever the address holds a live record, and `currentDeleted: true` where it holds a tombstone. This is normative rather than a courtesy: a consumer that omits them reintroduces the re-read race this task is specified to remove.
6. Apply `mergePatch` per [[RFC7386]](https://www.rfc-editor.org/rfc/rfc7386) to the record's current value, and refuse with `vta/app-state/put:notFound` when no live record exists at the address.
7. Assign the written record the next value of the `(contextId, namespace)` counter, and return it as `version`.
8. **MUST NOT** interpret, validate, migrate, or rewrite the stored value beyond applying a supplied `mergePatch`. The store is schema-agnostic: a dumb store needs no migration when a consumer's model changes.

## Authorization

Authority is **write access to application state in the named context**, held
on the maintainer's ACL. It is the same context boundary that gates keys,
vault entries and policy; this task adds no scope of its own.

Nothing in this version grants at namespace granularity. The address supports
it — that is why `namespace` is part of the address rather than a prefix
convention on `key` — but a namespace in 1.0 is a collision-avoidance
partition, not a trust boundary. Two applications writing to one context can
read and overwrite each other's namespaces, and a maintainer that needs them
isolated **MUST** put them in different contexts. Deployments should choose
namespace names on the assumption that a future per-namespace grant will name
this exact string.

The required `proof` establishes *who authored the write*, so the resulting
record and its audit entry can be attributed to an application key. It is not
the authorization — a correctly signed request from a caller without write
access is refused, and the access check happens after the signature is settled
([SPEC §7.2 item 10](../../../../../SPEC.md#72-consumer-requirements)).

## Request

### Unconditional upsert

```json
{
  "id": "8c1e4a72-3b95-4d60-8a17-2f4c6e8b0d19",
  "type": "https://trusttasks.org/spec/vta/app-state/put/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T10:00:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/acme",
    "value": {
      "label": "Acme Engineering",
      "joinedAt": "2026-07-02T14:10:00Z",
      "role": "member"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-22T10:00:00Z",
    "verificationMethod": "did:key:z6MkOpenVtcClient#z6MkOpenVtcClient",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

### Conditional write, from a version a read returned

```json
{
  "id": "9d2f5b83-4ca6-4e71-9b28-3a5d7f9c1e20",
  "type": "https://trusttasks.org/spec/vta/app-state/put/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T10:01:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/acme",
    "expectedVersion": 47,
    "value": {
      "label": "Acme Engineering",
      "joinedAt": "2026-07-02T14:10:00Z",
      "role": "admin"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-22T10:01:00Z",
    "verificationMethod": "did:key:z6MkOpenVtcClient#z6MkOpenVtcClient",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

### Acquire a lease — create only

`expectedVersion: 0` is what makes this safe. Two instances issuing this
concurrently produce exactly one winner; the loser receives a
`versionConflict` with `reason: "recordExists"` naming the holder.

```json
{
  "id": "a3e06c94-5db7-4f82-ac39-4b6e80ad2f31",
  "type": "https://trusttasks.org/spec/vta/app-state/put/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T10:02:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "lease/reconcile",
    "expectedVersion": 0,
    "value": {
      "holder": "did:key:z6MkOpenVtcClient",
      "expiresAt": "2026-08-22T10:07:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-22T10:02:00Z",
    "verificationMethod": "did:key:z6MkOpenVtcClient#z6MkOpenVtcClient",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

### Edit one member without colliding

Two instances patching different members of the same record both succeed, where
two whole-value writes would have serialised behind `expectedVersion` and one
would have had to retry.

```json
{
  "id": "b4f17da5-6ec8-4093-bd4a-5c7f91be3042",
  "type": "https://trusttasks.org/spec/vta/app-state/put/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T10:03:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/acme",
    "mergePatch": {
      "label": "Acme Engineering (EMEA)",
      "role": null
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-22T10:03:00Z",
    "verificationMethod": "did:key:z6MkOpenVtcClient#z6MkOpenVtcClient",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

Per [[RFC7386]](https://www.rfc-editor.org/rfc/rfc7386), `"role": null` **removes** the `role` member. A patch cannot set a member to the JSON literal null; a writer that needs that must send a whole `value`.

## Response

The VTA responds with `type: https://trusttasks.org/spec/vta/app-state/put/1.0#response`, whose payload names the address, the version the write took, and whether it created the record.

### Written

```json
{
  "id": "c5028eb6-7fd9-41a4-ce5b-6d80a2cf4153",
  "type": "https://trusttasks.org/spec/vta/app-state/put/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T10:01:01Z",
  "threadId": "9d2f5b83-4ca6-4e71-9b28-3a5d7f9c1e20",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/acme",
    "version": 52,
    "created": false,
    "updatedAt": "2026-08-22T10:01:01Z",
    "valueBytes": 95
  }
}
```

The version jumped from 47 to 52 because four writes to other keys in the
`openvtc` namespace took the values in between. The next conditional write to
this record supplies `expectedVersion: 52`.

### Conflict — the loser is handed the winner's view

```json
{
  "id": "d6139fc7-80ea-42b5-df6c-7e91b3da5264",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T10:01:02Z",
  "threadId": "9d2f5b83-4ca6-4e71-9b28-3a5d7f9c1e20",
  "payload": {
    "code": "vta/app-state/put:versionConflict",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/app-state/put/1.0",
      "id": "9d2f5b83-4ca6-4e71-9b28-3a5d7f9c1e20"
    },
    "message": "Record is at version 52; expected 47.",
    "retryable": false,
    "details": {
      "reason": "versionMismatch",
      "currentVersion": 52,
      "currentValue": {
        "label": "Acme Engineering",
        "joinedAt": "2026-07-02T14:10:00Z",
        "role": "owner"
      }
    }
  }
}
```

The caller now has everything it needs to merge and re-issue with
`expectedVersion: 52`. It never has to re-read, which is the point: a re-read
races the next write.

### Over the cap

```json
{
  "id": "e724a0d8-91fb-43c6-e07d-8f02c4eb6375",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T10:04:00Z",
  "threadId": "8c1e4a72-3b95-4d60-8a17-2f4c6e8b0d19",
  "payload": {
    "code": "vta/app-state/put:valueTooLarge",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/app-state/put/1.0",
      "id": "8c1e4a72-3b95-4d60-8a17-2f4c6e8b0d19"
    },
    "message": "Value is 131072 bytes; this maintainer's per-record cap is 65536.",
    "retryable": false,
    "details": {
      "limitBytes": 65536,
      "actualBytes": 131072
    }
  }
}
```

## Security & Privacy

**Not a secret store.** Producers **MUST NOT** place credentials, keys, tokens,
or other secret material in `value` or `mergePatch`. Secrets belong in
`vault/*`, where release is policy-gated, audited, and carried in sealed
envelopes; the application-state store applies no such handling and operator
tooling may display its contents. The boundary is normative here because a
boundary that is only implied erodes.

**A precondition is not a lock.** `expectedVersion` gives compare-and-set, not
mutual exclusion over a sequence of operations. An application that needs a
lease should build one from `expectedVersion: 0` plus an expiry it enforces
itself, as in the example above — and must accept that the maintainer does not
expire the lease record for it.

**Retry safety.** A `put` without `expectedVersion` is **not** safely
replayable: a replay writes twice and takes two counter values, so a peer
watching the namespace sees a change that never happened. A `put` **with**
`expectedVersion` converges — the replay fails its own precondition and one
record results. Implementations **SHOULD** nonetheless treat the task as
requiring a client-supplied idempotency key, because the class is per task
type and not per payload, and a caller reading the classification cannot see
which of the two shapes a given request took.

**Merge-patch reduces conflicts but not authority.** Two instances patching
disjoint members both succeed; that is the feature. It also means neither saw
the other's edit, so an application whose members carry a joint invariant
**MUST NOT** rely on patches to maintain it — send a whole `value` under
`expectedVersion`, or use the `atomic` mode of
[`vta/app-state/put-many`](../../put-many/1.0/spec.md).

**Size caps are a denial-of-service boundary as well as a correctness one.** A
documented cap bounds what one namespace can consume; a maintainer **SHOULD**
additionally bound the record count per namespace and surface that limit the
same way.

**Audit.** A maintainer **SHOULD** record writes with the requesting VID, the
address, the resulting version, and whether the record was created — but
**SHOULD NOT** record the value, which would copy application data into a store
with a different retention policy.

The optional `ext` member is part of the producer's signed surface; producers **MUST NOT** place data in `ext` they would not be comfortable signing.
