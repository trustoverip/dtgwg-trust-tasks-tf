---
slug: vta/app-state/get
version: "1.0"
title: VTA Application State — Get
summary: An application reads one of its versioned state records from a VTA by its context, namespace and key, receiving the value together with the version a later conditional write will need.
status: draft
targetFrameworkVersion: "0.4"
category: data-exchange
keywords:
  - vta
  - application-state
  - versioned
  - namespace
  - read
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
  rationale: A read makes no durable change and the maintainer has already authenticated the caller on the transport that carried the request. A proof lets a maintainer attribute the read to a specific application key on transports with no prior handshake, and a maintainer MAY require one as a policy choice.
sideEffects:
  level: none
  rationale: "Reads one record; nothing at the maintainer changes."
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
  - code: vta/app-state/get:notFound
    meaning: No record exists at `(contextId, namespace, key)`. Also returned when a tombstone exists but `includeDeleted` was not set.
    retryable: false
related:
  - vta/app-state/put
  - vta/app-state/list
  - vta/app-state/delete
  - vta/app-state/get-many
  - vta/app-state/put-many
  - vta/memory/put
---

## Abstract

**VTA Application State — Get** reads one record from the VTA's
application-state store: versioned, namespaced JSON that an application owns
and the VTA stores without interpreting.

The store exists because applications built on a VTA have had nowhere to keep
versioned metadata. It is deliberately a third store, beside the two that
already exist and are wrong for this in ways worth naming. Agent memory
(`vta/memory/*`) is closest and the most dangerous: *"forget everything"* is a
reasonable thing to ask an agent, and it must not take an account's community
memberships with it. The secrets vault (`vault/upsert`) is shaped as a password
manager, and application records are not site credentials. The credential vault
is right for verifiable credentials — application metadata is *about*
credentials rather than being one. Three stores, three jobs: secrets,
verifiable credentials, application state.

A record is addressed by the triple `(contextId, namespace, key)`. Every
response carries the record's `version`, which is the value a later
[`vta/app-state/put`](../../put/1.0/spec.md) supplies as `expectedVersion` to
make its write conditional on nothing having changed in between.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the application) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/app-state/get/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`, `payload.namespace` and `payload.key`.
3. Treat the returned `version` as opaque and monotonic — see [Definitions](#definitions). It **MUST NOT** assume the version increases by one between two writes to the same record.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements). Where a `proof` is present, verify it.
2. Refuse a caller that lacks read access to application state in `contextId` with the framework's standard `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)), and **MUST NOT** degrade an unreachable context to an empty result — a consumer that mistypes a context id must not be told its records are gone. A maintainer whose authorization model can distinguish "no such context" from "not permitted to reach it" **MAY** answer the former with `vta/app-state:contextNotFound`; one whose ACL enumerates the contexts a caller may act in cannot, and answers `permissionDenied` to both.
4. Return the record at `(contextId, namespace, key)` with its `value`, `version` and timestamps.
5. Refuse with `vta/app-state/get:notFound` when no live record exists at the address, and when a tombstone exists but `includeDeleted` was not set.
6. Return a tombstone as a record with `deleted: true` and no `value` when `includeDeleted` is set.
7. **MUST NOT** interpret, validate, migrate, or rewrite `value` in any way. The store is schema-agnostic by construction; see [Security & Privacy](#security--privacy).

## Definitions

* **Application.** The party that owns the namespace and its records; identified by `issuer`.
* **Context.** The VTA context the record is scoped to — the isolation boundary keys, DIDs, vault entries and policy also belong to.
* **Namespace.** An opaque partition within a context, scoping one application's records so several tools can share a context without colliding. Defined in [`_shared/0.1/app-state-record.schema.json`](../../../_shared/0.1/app-state-record.schema.json).
* **Version.** A value of the namespace's monotonic write counter. The counter is maintained per `(contextId, namespace)`, **not** per record: every write in a namespace takes the counter's next value, and a record's `version` is the value its most recent write took. One number therefore serves as both the optimistic-concurrency token and the incremental-sync watermark, which a per-record counter could not — two records' per-record counters are not comparable to each other, so no single number could mean "everything changed after this point". A record's version can consequently jump by any amount between two writes.
* **Tombstone.** A versioned marker left by [`vta/app-state/delete`](../../delete/1.0/spec.md) recording that a record was deleted, so that a consumer syncing incrementally learns of the deletion rather than resurrecting the record on its next rebuild.

## Request

### Read a record

```json
{
  "id": "f1b0c4d2-7a35-4e18-9c60-2d4e6f8a1b30",
  "type": "https://trusttasks.org/spec/vta/app-state/get/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T09:00:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/acme"
  }
}
```

### Ask whether an address was deleted or never existed

```json
{
  "id": "0c8e2a44-91d7-4f60-8b13-5e7a9c2d4f61",
  "type": "https://trusttasks.org/spec/vta/app-state/get/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T09:00:05Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "key": "community/defunct",
    "includeDeleted": true
  }
}
```

## Response

The VTA responds with `type: https://trusttasks.org/spec/vta/app-state/get/1.0#response`, whose payload is `{ record }`. An absent record is not a success; it is a `trust-task-error` carrying `vta/app-state/get:notFound`.

### The record, with the version a conditional write will need

```json
{
  "id": "3d5f7091-2b46-4a8c-91e5-7f0a2c4e6d83",
  "type": "https://trusttasks.org/spec/vta/app-state/get/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T09:00:01Z",
  "threadId": "f1b0c4d2-7a35-4e18-9c60-2d4e6f8a1b30",
  "payload": {
    "record": {
      "contextId": "personal",
      "namespace": "openvtc",
      "key": "community/acme",
      "version": 47,
      "deleted": false,
      "value": {
        "label": "Acme Engineering",
        "joinedAt": "2026-07-02T14:10:00Z",
        "role": "member"
      },
      "valueBytes": 96,
      "createdAt": "2026-07-02T14:10:00Z",
      "updatedAt": "2026-08-19T11:42:00Z"
    }
  }
}
```

Note that the record's `version` is 47 although the record has been written
twice. The intervening values were taken by writes to its neighbours in the
`openvtc` namespace. This is the property a consumer must not model as a
per-record edit count.

### A tombstone, returned because the caller asked for one

```json
{
  "id": "5a7c9013-4d68-4c0e-b307-91a2c4e6f085",
  "type": "https://trusttasks.org/spec/vta/app-state/get/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T09:00:06Z",
  "threadId": "0c8e2a44-91d7-4f60-8b13-5e7a9c2d4f61",
  "payload": {
    "record": {
      "contextId": "personal",
      "namespace": "openvtc",
      "key": "community/defunct",
      "version": 44,
      "deleted": true,
      "createdAt": "2026-06-11T08:00:00Z",
      "updatedAt": "2026-08-18T16:05:00Z",
      "deletedAt": "2026-08-18T16:05:00Z"
    }
  }
}
```

### The address holds nothing

```json
{
  "id": "7b9e1235-6f80-4e2a-8519-b3c4e6f80a27",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T09:00:07Z",
  "threadId": "0c8e2a44-91d7-4f60-8b13-5e7a9c2d4f61",
  "payload": {
    "code": "vta/app-state/get:notFound",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/app-state/get/1.0",
      "id": "0c8e2a44-91d7-4f60-8b13-5e7a9c2d4f61"
    },
    "message": "No record at (personal, openvtc, community/defunct).",
    "retryable": false
  }
}
```

## Security & Privacy

**Not a secret store, and the boundary has to be written down.** Application
state is stored as the application supplies it: the maintainer does not encrypt
it beyond whatever it applies to its keyspaces generally, and operator tooling
may display it. Secret material belongs in `vault/*`, where release is
policy-gated, audited, and carried in sealed envelopes. A boundary that is only
implied by the existence of a vault next door erodes; this one is normative.
Producers **MUST NOT** place credentials, keys, tokens, or other secret
material in `value`.

**Schema-agnosticism is a security property, not only an engineering one.** A
maintainer that parsed `value` would be running a parser over attacker-shaped
input on behalf of whichever application wrote it, and would acquire opinions
that block that application's next release. It stores bytes and returns them.

**Reads disclose application data, and enumeration discloses shape.** A caller
that can read a namespace learns its contents; a caller that can
[`list`](../../list/1.0/spec.md) it learns the existence, count and change
timing of every record even without values. A maintainer granting read access
to a third-party application **SHOULD** scope the grant to a context, and
**SHOULD** consider whether a future per-namespace grant is the boundary it
actually wants — the address supports one, and nothing in this version grants
one.

**Replay is benign but not free.** This task is read-only and idempotent, so a
replayed request changes nothing. A maintainer **SHOULD** nevertheless apply
the framework's `issuedAt` freshness check, to bound the window in which a
captured read can be repeated against a record whose contents have since become
more sensitive.

**Audit.** A maintainer **SHOULD** record reads with the requesting VID, the
context and namespace, and whether the record was found — but **SHOULD NOT**
record the value, which would duplicate the data into a store with a different
retention policy.

The optional `ext` member is part of the producer's signed surface; producers **MUST NOT** place data in `ext` they would not be comfortable signing.
