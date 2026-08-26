---
slug: vta/app-state/delete
version: "1.0"
title: VTA Application State — Delete
summary: An application deletes one of its state records from a VTA, leaving a versioned tombstone so that peers syncing incrementally learn of the deletion instead of resurrecting the record.
status: draft
targetFrameworkVersion: "0.4"
category: data-exchange
keywords:
  - vta
  - application-state
  - tombstone
  - delete
  - convergence
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
  rationale: A delete removes durable state that an account's recoverability may depend on, and after the tombstone's retention window expires it is unrecoverable. Attribution must survive the transport, so an audit record read later names the application key that deleted rather than the session it arrived on.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Deleting application state destroys it. Replayed after the key was written again it destroys the new value, and the document names the key but not the revision.
sideEffects:
  level: destructive
  rationale: "Removes the record's value. A tombstone preserves the fact of the deletion for the retention window, but the value is gone immediately and the tombstone itself is reaped afterwards — after which nothing records that the record ever existed."
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
  - code: vta/app-state/delete:versionConflict
    meaning: The `expectedVersion` precondition failed. The details carry the maintainer's current version and value, so the caller can see the edit it was about to discard.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["reason"]
      properties:
        reason:
          type: string
          enum: ["versionMismatch", "recordAbsent", "createOnlyNotApplicable"]
        currentVersion:
          type: integer
          minimum: 1
        currentValue: {}
        currentDeleted:
          type: boolean
related:
  - vta/app-state/get
  - vta/app-state/put
  - vta/app-state/list
  - vta/app-state/get-many
  - vta/app-state/put-many
---

## Abstract

**VTA Application State — Delete** removes one record from the VTA's
application-state store and leaves a **tombstone** in its place: a versioned
marker, taking the namespace's next counter value, recording that the record
was deleted.

The tombstone is the whole reason this task is specified rather than assumed.
A consumer pulling changes since a watermark learns about every create and
every update from the records themselves; a deletion leaves nothing behind to
be pulled. Without a tombstone that consumer never learns the record is gone,
resurrects it on its next full rebuild, and disagrees with every peer that saw
the delete live. Incremental sync does not converge without this, and adding it
after the fact silently invalidates every watermark already in the field.

Deleting an address that holds nothing is a **success**. That is deliberate:
it is what makes the task converge under replay, since a second delete finds a
tombstone and changes nothing further.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the application) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/app-state/delete/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`, `payload.namespace` and `payload.key`.
3. Include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).
4. **MUST NOT** supply `expectedVersion: 0`.

A conforming producer deleting on the strength of something it read **SHOULD** supply `expectedVersion`, so the delete cannot discard an edit made between the read and the delete.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Refuse a caller that lacks write access to application state in `contextId` with the framework's standard `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)). A maintainer whose authorization model can distinguish "no such context" from "not permitted to reach it" **MAY** answer the former with `vta/app-state:contextNotFound`; one whose ACL enumerates the contexts a caller may act in cannot, and answers `permissionDenied` to both.
3. Refuse `expectedVersion: 0` with `vta/app-state/delete:versionConflict` and `reason: "createOnlyNotApplicable"` — a create-only precondition on a delete is never satisfiable and never intended.
4. Evaluate a positive `expectedVersion` against the live record, refusing with `reason: "versionMismatch"` when it differs and `reason: "recordAbsent"` when no live record exists, and populating `currentVersion` / `currentValue` / `currentDeleted` in the details so the caller sees what it was about to discard.
5. Replace a live record with a tombstone taking the namespace's next counter value, discard the value, and return `existed: true` with the tombstone's `version` and `deletedAt`.
6. Return `existed: false` and the existing tombstone's `version` when the address already holds one, changing nothing — a repeated delete **MUST NOT** take a new counter value, because that would present a change to every watching consumer where none occurred.
7. Return `existed: false` with no `version` when the address holds nothing at all, writing **no** tombstone. Nothing ever existed there for a consumer to have learned about, so there is nothing to converge.
8. Retain tombstones for a documented window — RECOMMENDED at least **30 days** — and, once it reaps them, refuse a [`vta/app-state/list`](../../list/1.0/spec.md) whose `sinceVersion` predates the oldest it still holds. A maintainer that reaps tombstones without that check hands consumers a feed that silently omits deletions, which is worse than not offering one.

## Authorization

Authority is **write access to application state in the named context**, held
on the maintainer's ACL — the same authority
[`vta/app-state/put`](../../put/1.0/spec.md) requires, and the same context
boundary that gates keys, vault entries and policy. This task adds no scope of
its own, and 1.0 grants nothing at namespace granularity: an application with
write access to a context can delete records in every namespace within it.

This is the point at which that coarseness has teeth. A shared context means a
misbehaving or compromised application can delete another application's
records, and only the tombstone retention window stands between that and
permanent loss. A maintainer hosting mutually distrusting applications **MUST**
put them in separate contexts rather than separate namespaces.

The required `proof` establishes *who authored the deletion*, so it can be
attributed in the audit record. It is not the authorization — a correctly
signed request from a caller without write access is refused, and the access
check happens after the signature is settled
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Request

### Delete a record

```json
{
  "id": "7f91b2d6-41f8-437a-8c4e-6283accf4157",
  "type": "https://trusttasks.org/spec/vta/app-state/delete/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T12:00:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/defunct"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-22T12:00:00Z",
    "verificationMethod": "did:key:z6MkOpenVtcClient#z6MkOpenVtcClient",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

### Delete only if nothing changed since the read

```json
{
  "id": "80a2c3e7-52f9-448b-9d5f-7394bdd05268",
  "type": "https://trusttasks.org/spec/vta/app-state/delete/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T12:01:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "lease/reconcile",
    "expectedVersion": 53
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-22T12:01:00Z",
    "verificationMethod": "did:key:z6MkOpenVtcClient#z6MkOpenVtcClient",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

This is how a lease is released safely: the holder deletes only the lease it
still holds, so a holder whose lease has already been taken over by another
instance cannot delete the successor's.

## Response

The VTA responds with `type: https://trusttasks.org/spec/vta/app-state/delete/1.0#response`.

### Deleted, tombstone written

```json
{
  "id": "91b3d4f8-630a-459c-ae60-84a5cee16379",
  "type": "https://trusttasks.org/spec/vta/app-state/delete/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T12:00:01Z",
  "threadId": "7f91b2d6-41f8-437a-8c4e-6283accf4157",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/defunct",
    "existed": true,
    "version": 54,
    "deletedAt": "2026-08-22T12:00:01Z"
  }
}
```

Version 54 is what a consumer syncing from an earlier watermark will receive as
a tombstone, and is why it will drop its own copy rather than keep it.

### Already gone — still a success

A replay of the same delete returns the tombstone that already exists, with no
new version. A consumer watching the namespace sees nothing, because nothing
happened.

```json
{
  "id": "a2c4e509-741b-46ad-bf71-95b6dff2748a",
  "type": "https://trusttasks.org/spec/vta/app-state/delete/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T12:00:09Z",
  "threadId": "7f91b2d6-41f8-437a-8c4e-6283accf4157",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/defunct",
    "existed": false,
    "version": 54,
    "deletedAt": "2026-08-22T12:00:01Z"
  }
}
```

### The record moved under the caller

```json
{
  "id": "b3d5f61a-852c-47be-c082-a6c7e003859b",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T12:01:01Z",
  "threadId": "80a2c3e7-52f9-448b-9d5f-7394bdd05268",
  "payload": {
    "code": "vta/app-state/delete:versionConflict",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/app-state/delete/1.0",
      "id": "80a2c3e7-52f9-448b-9d5f-7394bdd05268"
    },
    "message": "Record is at version 56; expected 53.",
    "retryable": false,
    "details": {
      "reason": "versionMismatch",
      "currentVersion": 56,
      "currentValue": {
        "holder": "did:key:z6MkOtherInstance",
        "expiresAt": "2026-08-22T12:06:00Z"
      }
    }
  }
}
```

The caller can see from the details that another instance now holds the lease,
which is exactly the outcome the precondition existed to prevent it from
overwriting.

## Security & Privacy

**Deletion is not erasure, and then it is.** For the retention window the key
name, the deletion time, and the fact that a record existed remain visible to
every caller that can list the namespace; only the value goes immediately.
After the window the tombstone is reaped and nothing records that the record
existed at all. An application deleting for privacy reasons **SHOULD**
understand both halves: the metadata outlives the value, and the audit trail of
the deletion outlives the tombstone.

**Retention is a correctness parameter, not only a housekeeping one.** Too
short and a consumer that was offline over a weekend resumes from a watermark
whose deletions have been reaped — which the maintainer must detect and refuse,
rather than serve an incomplete feed. Too long and deletes are not real. 30
days is the recommended starting point, matching the vault's grace window, and
is worth revisiting on evidence rather than on taste.

**Delete is the sharp end of coarse authorization.** With namespace-level
grants unspecified in 1.0, every application writing to a context can delete
every other's records there. Separate contexts, not separate namespaces, are
the isolation boundary — see [Authorization](#authorization).

**Retry safety.** Delete converges under replay: a second attempt finds a
tombstone, returns `existed: false`, and takes no new version. This is why
`existed: false` is a success rather than an error — an implementation that
returned an error for it would make its own callers unable to retry safely.

**Audit.** A maintainer **SHOULD** record deletions with the requesting VID,
the address, and the tombstone version — and **SHOULD** retain that record
beyond the tombstone's own retention window, since after reaping it is the only
remaining evidence that the record existed.

The optional `ext` member is part of the producer's signed surface; producers **MUST NOT** place data in `ext` they would not be comfortable signing.
