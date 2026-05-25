---
slug: did-management/did/register
version: "0.1"
title: DID Management — Register
summary: A DID owner asks a hosting service to atomically claim a path and publish a signed DID log under it in one step, replacing the two-step reserve-then-publish flow for the common case.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords:
  - did
  - did-hosting
  - register
  - publish
  - atomic
  - webvh
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID owner
    requirement: REQUIRED
  - role: DID hosting service
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: A register is an evidentiary record of who claimed which path on which host; transport-independent integrity is valuable for audit but not strictly required when an authenticated transport already binds the producer's identity.
errorCodes:
  - code: did-management/did/register:path_taken
    meaning: The requested path is already reserved by a different owner and `force` was not set (or the caller lacks authority to force-replace).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        path: { type: string }
  - code: did-management/did/register:invalid_log
    meaning: The `didData` payload failed structural or cryptographic-proof validation for the declared `method`.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: string }
  - code: did-management/did/register:host_mismatch
    meaning: The host segment embedded in the log's DID identifier does not match this hosting service or any configured hosting domain.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        embeddedHost: { type: string }
        configuredHosts: { type: array, items: { type: string } }
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
  - did-management/did/check-name
  - did-management/did/publish
  - did-management/did/info
---

## Abstract

The **DID Management — Register** Trust Task is the atomic claim-and-publish path for a hosted DID. The *DID owner* submits the desired `path`, the signed log content, the DID `method`, and an optional `domain`; the *hosting service* validates the log, checks the path is available (or the caller has authority to replace it), and commits the slot and the log in a single batch.

The two-step alternative is [`did-management/did/check-name`](../../check-name/0.1/spec.md) (with `reserve: true`) followed by [`did-management/did/publish`](../../publish/0.1/spec.md). Both are valid; `register` is preferred when the caller has the signed log ready at submit time — it avoids the resolvability gap where a reserved-but-not-yet-published path returns 404 to any in-flight resolver.

The task is **idempotent for the slot's owner**: re-submitting the same `path` + signed log against an unchanged record is a no-op. A second call from the same owner with a *new* log entry is an update — `versionCount` advances, the new log replaces the old, and the response carries the new state.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the DID owner) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/did-management/did/register/0.1`, with itself as `issuer` and the hosting service's VID as `recipient`.
2. Populate `payload.path` with the desired path, `payload.method` with the DID method identifier, and `payload.didData` with the method-specific log content (a JSONL string for `webvh`; the latest DID document for `web`).
3. Set `payload.force: true` ONLY when intentionally replacing a slot the producer does not currently own (admin takeover); the hosting service rejects forced replacement unless the caller has administrative authority on the slot's hosting domain.

A conforming **consumer** (the hosting service) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Validate `payload.didData` against the declared `payload.method` — for `webvh`, the consumer walks the log chain and verifies each entry's signature against `parameters.updateKeys`; for `web`, the consumer verifies the document parses as a DID document and embeds an `id` consistent with this host.
3. Extract the host segment from the embedded DID identifier and refuse the register when it does not match the hosting service's configured hosting domains (`did-management/did/register:host_mismatch`).
4. Resolve the hosting domain to record on the slot: explicit `payload.domain` → caller's ACL default → system default. Persist the resolved value on the new record so subsequent reads carry it.
5. On a path collision with `force === false`, respond with `did-management/did/register:path_taken`.
6. On acceptance, commit the slot, log content, and owner-index entry in a single atomic batch — a resolver MUST see either the prior state or the new state, never an intermediate.

## Definitions

* **DID owner.** The party submitting the register; identified by `issuer`. Becomes the owner of the slot on a successful first-time register.
* **DID hosting service.** The party that holds the record store and serves resolutions; identified by `recipient`.
* **Path.** Local identifier under which the DID is hosted (e.g. `alice`, `tenant/staff/alice`, `.well-known` for the root slot). Compared by exact string equality.
* **Hosting domain.** The DNS hostname under which the DID resolves (e.g. `did.example.com`). The host segment of the embedded DID identifier MUST match an active hosting domain on the consumer.

## Request

A *request* document carries `type: https://trusttasks.org/spec/did-management/did/register/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### A new `did:webvh` is registered atomically

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/did-management/did/register/0.1",
  "issuer": "did:key:z6MkAlice",
  "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T10:00:00Z",
  "payload": {
    "path": "alice",
    "method": "webvh",
    "domain": "did.example.com",
    "didData": "{\"versionId\":\"1-...\",\"versionTime\":\"2026-06-01T09:59:50Z\",\"parameters\":{...},\"state\":{...},\"proof\":[...]}"
  }
}
```

### Admin force-replaces an existing slot

```json
{
  "id": "8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/did-management/did/register/0.1",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-01T11:00:00Z",
  "payload": {
    "path": "abandoned-slot",
    "method": "webvh",
    "didData": "{\"versionId\":\"1-...\",...}",
    "force": true
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/did-management/did/register/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The payload is `{ record: DidRecord }` carrying the canonical record the hosting service now holds.

Failures use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful first-time register

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/did-management/did/register/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:did.example.com",
  "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-01T10:00:01Z",
  "payload": {
    "record": {
      "mnemonic": "alice",
      "owner": "did:key:z6MkAlice",
      "createdAt": "2026-06-01T10:00:01Z",
      "updatedAt": "2026-06-01T10:00:01Z",
      "versionCount": 1,
      "didId": "did:webvh:abc123:did.example.com:alice",
      "method": "webvh",
      "domain": "did.example.com",
      "disabled": false
    }
  }
}
```

## Security & Privacy

A register is the *evidence of claim*: an attacker who can replay a captured register from a victim's account against an empty slot can squat the victim's preferred path. Consumers MUST bind register attempts to an authenticated session (or require a `proof` and reject replays via `id` uniqueness within the configured window).

`force: true` is a privileged operation — the only legitimate path through it is admin takeover of an abandoned slot. Consumers MUST refuse `force: true` from any caller that lacks administrative authority on the slot's current hosting domain.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the payload; producers MUST NOT place data in `ext` they would not be comfortable signing.
