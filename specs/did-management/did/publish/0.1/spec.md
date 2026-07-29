---
slug: did-management/did/publish
version: "0.1"
title: DID Management — Publish
summary: A DID owner uploads a signed log entry (or full log chain) for an existing reserved slot, completing the two-step reserve-then-publish flow.
status: retired
supersededBy: did-management/did/register
targetFrameworkVersion: "0.1"
category: did-management
keywords:
  - did
  - did-hosting
  - publish
  - log
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID owner
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A publish is an evidentiary record of which log content was uploaded by which owner against which slot; the maintainer SHOULD be able to demonstrate, after the fact, that the upload was authorised.
sideEffects:
  level: mutating
  rationale: Appends a new entry to the DID's log chain. Recoverable — the prior chain state is retained and a subsequent entry can supersede it — but it is a persisted, publicly-resolvable change to the identity's history.
consequences:
  - Extends the DID's public log; the new state resolves immediately.
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management/did/publish:not_owner
    meaning: The caller is not the current owner of the slot (or an admin with takeover authority).
    retryable: false
  - code: did-management/did/publish:invalid_log
    meaning: The `didData` payload failed structural or cryptographic-proof validation for the declared `method`.
    retryable: false
  - code: did-management/did/publish:host_mismatch
    meaning: The host segment embedded in the log's DID identifier does not match the slot's configured hosting domain.
    retryable: false
  - code: did-management:unknown_domain
    meaning: The submitted `domain` is not a known hosting domain on this consumer. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
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
  - did-management/did/check-name
methodExtensions:
  - method: webvh
    schema: did-management/_shared/0.1/did-method-extensions/webvh
    requirement: OPTIONAL
---

## Abstract

The **DID Management — Publish** Trust Task uploads method-specific log content for an already-reserved slot. It is the second half of the two-step flow whose first half is [`did-management/did/check-name`](../../check-name/0.1/spec.md) with `reserve: true`. The one-shot equivalent is [`did-management/did/register`](../../register/0.1/spec.md).

The task is **idempotent**: re-submitting the same `didData` against an unchanged record is a no-op. A submission with a strictly newer log entry advances `versionCount` and replaces the held content; for `did:webvh` and other append-only methods, the consumer MUST verify the new chain is a valid extension of the prior one before accepting.

## Status of this Document

**Retired.** This task is superseded by [`did-management/did/register`](../../../did/register/0.1/spec.md) — a register against an already-reserved (or existing) slot owned by the caller publishes the log as an update, which fully covers the reserved-slot flow. Consumers SHOULD NOT accept new documents of this type; the specification is retained for auditability of previously-issued documents.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**: emit a *Trust Task document* of type `https://trusttasks.org/spec/did-management/did/publish/0.1` with `payload.mnemonic`, `payload.method`, and `payload.didData`.

A conforming **consumer** **MUST**: validate per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements); verify the caller is the slot's current owner (or an admin); validate `didData` against `method`; commit the new log content and bumped record in a single atomic batch.

## Definitions

* **DID owner**, **DID hosting service**, **Slot/Path**: as in [`did-management/did/register`](../../register/0.1/spec.md).

## Request

```json
{
  "id": "11111111-2222-4333-8444-555555555555",
  "type": "https://trusttasks.org/spec/did-management/did/publish/0.1",
  "issuer": "did:key:z6MkAlice",
  "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T10:05:00Z",
  "payload": {
    "mnemonic": "alice",
    "method": "webvh",
    "didData": "{\"versionId\":\"1-...\",...}"
  },
  "proof": { "type": "DataIntegrityProof", "cryptosuite": "eddsa-rdfc-2022", "verificationMethod": "did:key:z6MkAlice#key-0", "created": "2026-06-01T10:05:00Z", "proofPurpose": "assertionMethod", "proofValue": "z..." }
}
```

## Response

A success *response* carries `type: https://trusttasks.org/spec/did-management/did/publish/0.1#response` with payload `{ record: DidRecord }` for the updated record.

```json
{
  "id": "22222222-3333-4444-8555-666666666666",
  "type": "https://trusttasks.org/spec/did-management/did/publish/0.1#response",
  "threadId": "11111111-2222-4333-8444-555555555555",
  "issuer": "did:web:did.example.com",
  "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T10:05:01Z",
  "payload": {
    "record": {
      "mnemonic": "alice", "owner": "did:key:z6MkAlice",
      "createdAt": "2026-06-01T10:00:00Z", "updatedAt": "2026-06-01T10:05:01Z",
      "versionCount": 1, "didId": "did:webvh:abc:did.example.com:alice",
      "method": "webvh", "domain": "did.example.com", "disabled": false
    }
  }
}
```

## Security & Privacy

A captured publish document is *evidence of upload*. Consumers MUST reject replays via `id` uniqueness within the configured window. The REQUIRED `proof` ensures the owner cannot repudiate the upload and intermediaries cannot tamper with `didData` in transit.
