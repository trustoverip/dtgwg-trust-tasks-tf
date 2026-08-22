---
slug: vta/app-state/get-many
version: "1.0"
title: VTA Application State — Get Many
summary: An application reads up to 256 of its state records from one namespace in a single round trip, with every requested key accounted for as found, missing, or deferred to a follow-up request.
status: draft
targetFrameworkVersion: "0.4"
category: data-exchange
keywords:
  - vta
  - application-state
  - batch
  - read
  - reconnect
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
  rationale: A batch read makes no durable change and the maintainer has already authenticated the caller on the transport that carried the request. A proof lets a maintainer attribute the read to a specific application key on transports with no prior handshake.
sideEffects:
  level: none
  rationale: "Reads records; nothing at the maintainer changes."
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
  - code: vta/app-state/get-many:duplicateKey
    meaning: The `keys` array contains the same key more than once. Refused rather than deduplicated, because a caller that sent a duplicate did not mean to.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        keys:
          type: array
          items:
            type: string
related:
  - vta/app-state/get
  - vta/app-state/put
  - vta/app-state/list
  - vta/app-state/delete
  - vta/app-state/put-many
---

## Abstract

**VTA Application State — Get Many** reads several records from one namespace
in a single *Trust Task*.

The motivation is latency and nothing else. A cold start or a reconnect
rebuilds N records, and issuing N [`vta/app-state/get`](../../get/1.0/spec.md)
requests over a DIDComm or TSP round trip each is the difference between a
usable reconnect and an unusable one. Where the keys are known in advance this
is the right shape; where they are not,
[`vta/app-state/list`](../../list/1.0/spec.md) with `includeValues` covers the
same ground by prefix.

Every requested key is accounted for in the response — in `records`, in
`missing`, or in `deferred`. A caller never has to diff its request against the
response to discover what happened to a key, which is the failure mode of a
batch API that returns only what it found.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the application) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/app-state/get-many/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`, `payload.namespace` and a `payload.keys` array of 1–256 distinct keys.
3. Re-request the keys named in `deferred`, if any, rather than treating them as absent.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements). Where a `proof` is present, verify it.
2. Refuse a caller that lacks read access to application state in `contextId` with the framework's standard `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)). A maintainer whose authorization model can distinguish "no such context" from "not permitted to reach it" **MAY** answer the former with `vta/app-state:contextNotFound`; one whose ACL enumerates the contexts a caller may act in cannot, and answers `permissionDenied` to both.
3. Refuse a `keys` array containing duplicates with `vta/app-state/get-many:duplicateKey`, naming the offending keys in the details.
4. Return the found records in `records`, each with its `value`, **in the order the keys were requested** — so a caller can zip the two lists rather than index by key.
5. Name in `missing` every requested key holding no record, and every key holding only a tombstone unless `includeDeleted` was set.
6. Name in `deferred` every requested key it did not evaluate because the response reached its size budget, and **MUST NOT** silently omit such a key from all three lists.
7. Ensure the union of `records`, `missing` and `deferred` is exactly the requested `keys` set. This is the property that makes the response self-accounting, and a consumer that cannot maintain it **MUST** fail the request rather than return an ambiguous one.
8. **MUST NOT** interpret, validate, migrate, or rewrite any stored value.

A conforming consumer **SHOULD** evaluate keys in request order when deferring, so that a caller re-requesting `deferred` makes forward progress rather than receiving an arbitrary subset each time.

## Definitions

* **Deferred key.** A requested key the maintainer chose not to evaluate because the response would have exceeded its size budget. Not an error and not an absence — a caller re-requests it. The concept exists because the per-record cap multiplied by the key ceiling exceeds any reasonable response limit, so a maintainer must be able to return a partial batch without either lying about the remainder or refusing the whole request.
* **Namespace, key, version, tombstone.** As defined in [`vta/app-state/get`](../../get/1.0/spec.md) and the [shared record schema](../../../_shared/0.1/app-state-record.schema.json).

## Request

### Rebuild a known working set

```json
{
  "id": "c4e60728-963d-48cf-d193-b7d8f114960a",
  "type": "https://trusttasks.org/spec/vta/app-state/get-many/1.0",
  "issuer": "did:key:z6MkOpenVtcClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-22T13:00:00Z",
  "payload": {
    "contextId": "personal",
    "namespace": "openvtc",
    "keys": [
      "community/acme",
      "community/borealis",
      "community/defunct",
      "profile/labels"
    ]
  }
}
```

## Response

The VTA responds with `type: https://trusttasks.org/spec/vta/app-state/get-many/1.0#response`, whose payload is `{ records, missing, deferred? }`.

### Two found, one deleted, one never written

```json
{
  "id": "d5f71839-a74e-49d0-e2a4-c8e90225a71b",
  "type": "https://trusttasks.org/spec/vta/app-state/get-many/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T13:00:01Z",
  "threadId": "c4e60728-963d-48cf-d193-b7d8f114960a",
  "payload": {
    "records": [
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
      },
      {
        "contextId": "personal",
        "namespace": "openvtc",
        "key": "community/borealis",
        "version": 31,
        "deleted": false,
        "value": {
          "label": "Borealis Collective",
          "joinedAt": "2026-05-19T07:20:00Z",
          "role": "member"
        },
        "valueBytes": 92,
        "createdAt": "2026-05-19T07:20:00Z",
        "updatedAt": "2026-08-01T09:14:00Z"
      }
    ],
    "missing": ["community/defunct", "profile/labels"]
  }
}
```

`community/defunct` holds a tombstone and `profile/labels` was never written.
Both are reported as missing, because the request did not set `includeDeleted`
and the caller asked only whether the records are there.

### The batch did not fit

```json
{
  "id": "e6082940-b85f-4ae1-f3b5-d9fa1336b82c",
  "type": "https://trusttasks.org/spec/vta/app-state/get-many/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOpenVtcClient",
  "issuedAt": "2026-08-22T13:01:01Z",
  "threadId": "c4e60728-963d-48cf-d193-b7d8f114960a",
  "payload": {
    "records": [
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
    "missing": [],
    "deferred": ["community/borealis", "community/defunct", "profile/labels"]
  }
}
```

The caller re-requests exactly the three deferred keys. Nothing is lost and
nothing is guessed at — which is the alternative a maintainer would otherwise
force on it by refusing the whole batch and leaving it to bisect a workable
size.

## Security & Privacy

**A batch read is an enumeration the caller already knew how to spell.** It
discloses no more than the same keys read one at a time, but it does so at a
rate that makes bulk exfiltration cheap. A maintainer **SHOULD** rate-limit
this task by requesting VID and namespace, and **SHOULD** treat a caller
issuing large batches against keys it did not itself write as a signal worth
auditing.

**`missing` leaks existence, and that is deliberate.** Telling a caller which
of its keys are absent is the point of the task, but it also means a caller can
probe for keys it did not write, one batch of 256 at a time. Where a namespace
is shared between mutually distrusting applications this is a real disclosure —
and the answer is separate contexts, since 1.0 specifies no namespace-level
grant. See [`vta/app-state/put`](../../put/1.0/spec.md#authorization).

**Not a secret store.** Values are returned as stored, with no sealing and no
release policy. Secret material belongs in `vault/*`.

**Replay is benign.** The task is read-only and idempotent. A maintainer
**SHOULD** still apply the framework's `issuedAt` freshness check.

**Audit.** A maintainer **SHOULD** record the requesting VID, the context,
namespace and the count of keys requested, found, missing and deferred —
**not** the key names, which would copy the application's addressing scheme
into the audit store, and not the values.

The optional `ext` member is part of the producer's signed surface; producers **MUST NOT** place data in `ext` they would not be comfortable signing.
