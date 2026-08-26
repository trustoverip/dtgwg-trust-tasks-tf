---
slug: vta/app-state/put-many
version: "1.0"
title: VTA Application State — Put Many
summary: An application flushes up to 64 state writes to a VTA in one round trip, each carrying its own version precondition, applied either independently so one conflict does not block the rest or atomically for records sharing an invariant.
status: draft
targetFrameworkVersion: "0.4"
category: data-exchange
keywords:
  - vta
  - application-state
  - batch
  - write-behind
  - optimistic-concurrency
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
  rationale: A batch write mutates durable state that an account's recoverability depends on, and the maintainer audits it. Attribution must survive the transport that carried the request, so that an audit record read later names the application key that wrote rather than the session it arrived on.
sideEffects:
  level: mutating
  rationale: "Creates or replaces up to 64 records. Recoverable — prior values are overwritten but addresses remain, and a conditional write cannot clobber a version it did not see."
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
  - code: vta/app-state/put-many:duplicateKey
    meaning: Two writes in the batch name the same key. Refused rather than serialised, because their relative order is undefined and any choice the maintainer made would be arbitrary.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        keys:
          type: array
          items:
            type: string
  - code: vta/app-state/put-many:atomicBatchRejected
    meaning: An `atomic` batch was not applied because at least one write failed its precondition or its size check. Nothing was written. The details carry the per-record outcomes, so the caller learns which writes failed and which were merely skipped.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["results"]
      properties:
        results:
          type: array
          items:
            type: object
  - code: vta/app-state/put-many:batchTooLarge
    meaning: The batch's aggregate size exceeds what the maintainer accepts in one request, independently of whether any single value is within the per-record cap. The caller must split the batch.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        limitBytes:
          type: integer
          minimum: 0
        actualBytes:
          type: integer
          minimum: 0
related:
  - vta/app-state/get
  - vta/app-state/put
  - vta/app-state/list
  - vta/app-state/delete
  - vta/app-state/get-many
---

## Abstract

**VTA Application State — Put Many** applies several writes to one namespace in
a single *Trust Task*, each carrying its own `expectedVersion`. A write-behind
flush or a post-reconnect reconciliation becomes one round trip instead of N.

The interesting decision is not the batching but the **default mode**.

`independent` — the default — applies each write on its own merits. One
conflicted record does not block the other nine, and the response reports what
happened to each. This is what a flush of unrelated edits actually wants: the
records have no relationship to each other beyond having been queued at the
same time, and holding nine good writes hostage to one stale one is a loss with
no corresponding safety gain.

`atomic` is available for records that carry a joint invariant, and it applies
all or none.

An atomic *default* would let one stale record silently wedge an entire flush,
and — worse — the caller could not distinguish a wedged flush from a slow one
without inspecting per-record state it does not have. The default is chosen
against that failure, not for convenience.

Per-record results are what make either mode usable. A caller receives one
outcome per write, in request order, and a conflicted write's outcome carries
the maintainer's **current version and current value** — the same property
[`vta/app-state/put`](../../put/1.0/spec.md) has, for the same reason: a bare
rejection obliges a re-read, and the re-read races the next write.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the application) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/app-state/put-many/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`, `payload.namespace`, and a `payload.writes` array of 1–64 writes with distinct keys, each supplying exactly one of `value` or `mergePatch`.
3. Include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).
4. Set `mode: "atomic"` **only** where the batch's records carry a joint invariant. A producer that sets it out of caution converts every independent conflict into a total failure.
5. **MUST NOT** place secret material in any `value` or `mergePatch`.

A conforming producer **MUST NOT** assume that the writes in an `independent` batch took contiguous version numbers, or that their relative order in `writes` determined the order of the versions they took.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Refuse a caller that lacks write access to application state in `contextId` with the framework's standard `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)). A maintainer whose authorization model can distinguish "no such context" from "not permitted to reach it" **MAY** answer the former with `vta/app-state:contextNotFound`; one whose ACL enumerates the contexts a caller may act in cannot, and answers `permissionDenied` to both.
3. Refuse a batch containing two writes to the same key with `vta/app-state/put-many:duplicateKey`, naming them in the details.
4. Refuse a batch whose aggregate size exceeds its request limit with `vta/app-state/put-many:batchTooLarge`, carrying both the limit and the actual size — a limit the caller cannot see is a limit it cannot plan around.
5. Evaluate each write's `expectedVersion`, `mergePatch` and per-record size cap exactly as [`vta/app-state/put`](../../put/1.0/spec.md) specifies for a single write.
6. In **`independent`** mode, apply every write that passes its own checks, and return a `#response` carrying one result per write in request order. A batch in which some or all writes conflicted is a **success**: the task did what it promised.
7. In **`atomic`** mode, apply the batch only if every write passes. If any fails, write **nothing** and return `vta/app-state/put-many:atomicBatchRejected`, whose details carry one result per write — `skipped` for those not attempted — so the caller learns which write is actually blocking it rather than only that one did.
8. Populate `currentVersion`, `currentValue` and `currentDeleted` on every `conflict` outcome, on the same normative terms as `vta/app-state/put`.
9. Assign each applied write the next value of the `(contextId, namespace)` counter, and **MUST NOT** reuse or skip counter values — a consumer syncing from a watermark relies on every intervening value having been taken by exactly one write it will be told about.
10. **MUST NOT** interpret, validate, migrate, or rewrite any stored value beyond applying a supplied `mergePatch`.

## Authorization

Authority is **write access to application state in the named context**, held
on the maintainer's ACL, and identical to what
[`vta/app-state/put`](../../put/1.0/spec.md#authorization) requires. Batching
grants nothing extra: a caller that could not write one of these records
individually cannot write it here, and the check is per write rather than per
batch.

`atomic` mode does not confer a transactional guarantee beyond the batch. It
means the 64 writes land together or not at all; it does not isolate them from
a concurrent writer, and a record read during the batch may already reflect it.
An application needing more than that needs `expectedVersion` on every write —
which `atomic` composes with, and which is where the real guarantee comes from.

The required `proof` establishes *who authored the writes*, so they can be
attributed in the audit record. It is not the authorization — a correctly
signed batch from a caller without write access is refused, and the access
check happens after the signature is settled
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Request

### A write-behind flush of unrelated edits

Three records queued while offline, each conditional on the version the client
last saw. `independent` is what this wants: if the middle one has moved on, the
other two should still land.

```json
{
  "id": "f7193a51-c960-4bf2-a4c6-ea0b2447c93d",
  "type": "https://trusttasks.org/spec/vta/app-state/put-many/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T14:00:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "mode": "independent",
    "writes": [
      {
        "key": "community/acme",
        "expectedVersion": 52,
        "mergePatch": { "role": "owner" }
      },
      {
        "key": "community/borealis",
        "expectedVersion": 31,
        "mergePatch": { "label": "Borealis Collective (archived)" }
      },
      {
        "key": "profile/labels",
        "expectedVersion": 0,
        "value": { "colours": { "acme": "blue", "borealis": "green" } }
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-22T14:00:00Z",
    "verificationMethod": "did:key:z6MkOpenVtcClient#z6MkOpenVtcClient",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

### Records that must move together

A membership record and the index that points at it are only consistent as a
pair, so this batch is `atomic`.

```json
{
  "id": "082a4b62-da71-4c03-b5d7-fb1c3558da4e",
  "type": "https://trusttasks.org/spec/vta/app-state/put-many/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T14:05:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "mode": "atomic",
    "writes": [
      {
        "key": "community/cyprus",
        "expectedVersion": 0,
        "value": { "label": "Cyprus Working Group", "joinedAt": "2026-08-22T14:05:00Z", "role": "member" }
      },
      {
        "key": "index/communities",
        "expectedVersion": 58,
        "value": { "ids": ["acme", "borealis", "cyprus"] }
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-22T14:05:00Z",
    "verificationMethod": "did:key:z6MkOpenVtcClient#z6MkOpenVtcClient",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

The VTA responds with `type: https://trusttasks.org/spec/vta/app-state/put-many/1.0#response`, whose payload is `{ mode, results, highWatermark? }`. A rejected `atomic` batch is a `trust-task-error` instead.

### Independent — two applied, one conflicted

```json
{
  "id": "193b5c73-eb82-4d14-c6e8-0c2d4669eb5f",
  "type": "https://trusttasks.org/spec/vta/app-state/put-many/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T14:00:01Z",
  "threadId": "f7193a51-c960-4bf2-a4c6-ea0b2447c93d",
  "payload": {
    "mode": "independent",
    "results": [
      {
        "key": "community/acme",
        "outcome": "written",
        "version": 59,
        "created": false
      },
      {
        "key": "community/borealis",
        "outcome": "conflict",
        "currentVersion": 57,
        "currentValue": {
          "label": "Borealis Collective",
          "joinedAt": "2026-05-19T07:20:00Z",
          "role": "admin"
        }
      },
      {
        "key": "profile/labels",
        "outcome": "written",
        "version": 60,
        "created": true
      }
    ],
    "highWatermark": 60
  }
}
```

Two writes landed and one did not, which is the outcome the mode was chosen
for. The caller can merge against `currentValue` and re-issue just the one
write with `expectedVersion: 57`, without a re-read and without having lost the
other two.

### Atomic — nothing applied, and which write blocked it

```json
{
  "id": "2a4c6d84-fc93-4e25-d7f9-1d3e577afc60",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T14:05:01Z",
  "threadId": "082a4b62-da71-4c03-b5d7-fb1c3558da4e",
  "payload": {
    "code": "vta/app-state/put-many:atomicBatchRejected",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/app-state/put-many/1.0",
      "id": "082a4b62-da71-4c03-b5d7-fb1c3558da4e"
    },
    "message": "index/communities is at version 61; expected 58. Nothing was written.",
    "retryable": false,
    "details": {
      "results": [
        {
          "key": "community/cyprus",
          "outcome": "skipped"
        },
        {
          "key": "index/communities",
          "outcome": "conflict",
          "currentVersion": 61,
          "currentValue": { "ids": ["acme", "borealis", "delphi"] }
        }
      ]
    }
  }
}
```

`skipped` rather than `written` for the first record is the part that matters:
the caller knows the membership record was not created, so a retry does not
need `expectedVersion: 0` changed to a version it never got.

## Security & Privacy

**Not a secret store.** Producers **MUST NOT** place credentials, keys, tokens,
or other secret material in any `value` or `mergePatch`. Secrets belong in
`vault/*`, where release is policy-gated and audited.

**Batching amplifies a compromised writer.** One authorized request can replace
64 records. A maintainer **SHOULD** rate-limit this task by requesting VID and
namespace, and **SHOULD** audit a batch as 64 write events rather than one, so
that an incident review sees the records touched rather than a single line.

**Retry safety.** A batch is **not** safely replayable unless every write in it
carries `expectedVersion`: a replayed unconditional write applies twice and
takes two counter values, presenting a change to every watching consumer that
never happened. Partial application in `independent` mode makes a blind replay
worse still, because the second attempt's conflicts differ from the first's.
Implementations **SHOULD** require a client-supplied idempotency key for this
task unconditionally — the classification is per task type and not per payload,
so a caller reading it cannot see which shape a given batch took.

**Atomic is not isolation.** `atomic` guarantees all-or-nothing application, not
serialisability against a concurrent writer. A record involved in the batch may
be read by another caller mid-flight, and the batch's own writes are not hidden
from a concurrent reader once applied. Applications that need more must express
it with `expectedVersion` on every write, which is where the actual guarantee
lives.

**Counter values are a covert channel of sorts.** The gaps between the versions
a caller's own writes took reveal how much other traffic the namespace saw in
between. Where two applications share a context this discloses each other's
write volume. It is not fixable without giving up the single-counter design that
makes incremental sync work, and is noted here so it is a known property rather
than a discovered one.

The optional `ext` member is part of the producer's signed surface; producers **MUST NOT** place data in `ext` they would not be comfortable signing.
