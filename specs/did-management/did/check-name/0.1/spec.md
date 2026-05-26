---
slug: did-management/did/check-name
version: "0.1"
title: DID Management — Check Name
summary: A prospective DID owner asks a hosting service whether a given path is available, and optionally — in the same round-trip — reserves it.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords:
  - did
  - did-hosting
  - check-name
  - availability
  - reserve
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Prospective DID owner
    requirement: REQUIRED
  - role: DID hosting service
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: An availability probe is short-lived and consumed over an authenticated transport; a reservation outcome may be retained but the reservation's evidentiary record is the subsequent register/publish, not this check.
errorCodes:
  - code: did-management/did/check-name:invalid_path
    meaning: The submitted `path` violates the host's path grammar (length bounds, character set, reserved roots).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        path: { type: string }
        reason: { type: string }
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain on this consumer. See [the category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        domain: { type: string }
        activeDomains:
          type: array
          items: { type: string }
related:
  - did-management/did/register
  - did-management/did/publish
---

## Abstract

The **DID Management — Check Name** Trust Task lets a prospective owner ask the hosting service two questions in one round-trip:

1. *Is this path available?* — always answered.
2. *And if so, reserve it for me now.* — answered when the request sets `reserve: true`.

The two modes share one task so a client that wants the cheap availability probe and one that wants the atomic "check, then claim" don't need two different specs. When `reserve: false` (the default), the task is **read-only** and never mutates state. When `reserve: true` and the path is available, the consumer atomically commits a reservation owned by the caller and returns the resulting `DidRecord` — the caller's next step is [`did-management/did/publish`](../../publish/0.1/spec.md) to upload the signed log.

If the path is not available, the response carries `available: false` and (when `reserve: true`) the consumer does not mutate state — no `record` is returned.

A reservation does not block forever; the consumer's retention policy MAY garbage-collect reservations that haven't been followed by a publish within a configured grace period. The grace period and any reservation-decay semantics are policy concerns, opaque to this spec.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/did-management/did/check-name/0.1`.
2. Populate `payload.path` with the path to test.
3. Set `payload.reserve: true` ONLY when the producer also intends to claim the slot in the same call; in that case `payload.domain` MAY be set to override the consumer's default domain resolution.

A conforming **consumer** (the hosting service) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Apply the same path-grammar checks it would on `did-management/did/register` (lengths, character set, reserved roots).
3. Determine availability against the current record store; when the path is taken by a soft-deleted record, treat as not available unless the host's recovery policy permits resurrection.
4. When `reserve: true` AND the path is available:
   - Resolve the hosting domain via explicit `payload.domain` → caller's ACL default → system default.
   - Atomically commit a reservation (no log content yet, `versionCount: 0`) owned by the caller and return `available: true, reserved: true, record: <DidRecord>`.
5. When `reserve: false` OR the path is not available, return `available: <bool>, reserved: false` and DO NOT mutate state.

## Definitions

* **Prospective DID owner.** The party probing or reserving; identified by `issuer`.
* **DID hosting service.** The party that holds the record store; identified by `recipient`.
* **Path.** Local identifier as defined in [`did-management/did/register`](../../register/0.1/spec.md).
* **Reservation.** A `DidRecord` with `versionCount: 0` — the slot is owned but no log has been published. Subsequent calls to `did-management/did/publish` from the same owner advance `versionCount` to ≥1.

## Request

### Pure availability probe

```json
{
  "id": "1a2b3c4d-5e6f-4789-abcd-ef0123456789",
  "type": "https://trusttasks.org/spec/did-management/did/check-name/0.1",
  "issuer": "did:key:z6MkAlice",
  "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T10:00:00Z",
  "payload": {
    "path": "alice",
    "reserve": false
  }
}
```

`reserve` defaults to `false`; the field is shown explicitly so the example matches the byte-for-byte wire form a conformant serializer emits.

### Check-and-reserve in one round-trip

```json
{
  "id": "2b3c4d5e-6f78-4901-bcde-f01234567890",
  "type": "https://trusttasks.org/spec/did-management/did/check-name/0.1",
  "issuer": "did:key:z6MkAlice",
  "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T10:00:00Z",
  "payload": {
    "path": "alice",
    "reserve": true,
    "domain": "did.example.com"
  }
}
```

## Response

A *response* carries `type: https://trusttasks.org/spec/did-management/did/check-name/0.1#response`. The payload always carries `available` and `reserved`; `record` is present iff `reserved === true`.

### Available, not reserved

```json
{
  "id": "3c4d5e6f-7890-4123-cdef-012345678901",
  "type": "https://trusttasks.org/spec/did-management/did/check-name/0.1#response",
  "threadId": "1a2b3c4d-5e6f-4789-abcd-ef0123456789",
  "issuer": "did:web:did.example.com",
  "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T10:00:01Z",
  "payload": {
    "available": true,
    "reserved": false
  }
}
```

### Available, reserved in one round-trip

```json
{
  "id": "4d5e6f78-9012-4234-def0-123456789012",
  "type": "https://trusttasks.org/spec/did-management/did/check-name/0.1#response",
  "threadId": "2b3c4d5e-6f78-4901-bcde-f01234567890",
  "issuer": "did:web:did.example.com",
  "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T10:00:01Z",
  "payload": {
    "available": true,
    "reserved": true,
    "record": {
      "mnemonic": "alice",
      "owner": "did:key:z6MkAlice",
      "createdAt": "2026-06-01T10:00:01Z",
      "updatedAt": "2026-06-01T10:00:01Z",
      "versionCount": 0,
      "domain": "did.example.com",
      "disabled": false
    }
  }
}
```

### Not available

```json
{
  "id": "5e6f7890-1234-4345-ef01-234567890123",
  "type": "https://trusttasks.org/spec/did-management/did/check-name/0.1#response",
  "threadId": "...",
  "issuer": "did:web:did.example.com",
  "recipient": "did:key:z6MkBob",
  "issuedAt": "2026-06-01T10:05:00Z",
  "payload": {
    "available": false,
    "reserved": false
  }
}
```

## Security & Privacy

A `reserve: true` request is functionally a slot claim — consumers MUST bind it to an authenticated session OR require a `proof` so an attacker who can read a victim's availability probe cannot upgrade it to a reservation under their own DID.

Path-availability is generally not privacy-sensitive on hosting domains intended for public DIDs (resolvers will be able to enumerate paths anyway), but consumers on private hosting domains MAY rate-limit or audit availability probes to slow path enumeration.
