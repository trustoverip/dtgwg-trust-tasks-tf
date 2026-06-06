---
slug: vault/delete
version: "0.1"
title: Vault — Delete
summary: A vault consumer tombstones a vault entry; the maintainer keeps the tombstone for a grace period so late-syncing consumers wipe their caches before garbage collection.
status: draft
targetFrameworkVersion: "0.1"
category: credentials
keywords:
  - vault
  - credentials
  - delete
  - tombstone
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Delete is destructive and silently propagates to every Companion cache via sync. The producer's identity MUST be verifiable so the maintainer can attribute the deletion to a specific consumer in the audit log.
errorCodes:
  - code: vault/delete:not_found
    meaning: No entry with this id exists in the consumer's visible scope (conflates "absent" and "permission denied" — see Security).
    retryable: false
  - code: vault/delete:permission_denied
    meaning: Returned only when the consumer can already prove existence (e.g. via a prior list); maintainers operating enumeration-resistant modes use `not_found` instead.
    retryable: false
  - code: vault/delete:version_conflict
    meaning: An `expectedVersion` was supplied and does not match the current version.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        currentVersion: { type: "integer", minimum: 0 }
---

## Abstract

The **Vault — Delete** Trust Task tombstones a vault entry. The maintainer **MUST** preserve the tombstone for a configured grace period (RECOMMENDED 30 days) so late-syncing consumers can observe the deletion via `vault/sync/0.1` and wipe their local caches before the tombstone is garbage-collected.

## Conformance

A conforming **producer** **MUST** populate `id`; **SHOULD** populate `expectedVersion`; **MUST** carry a `proof`.

A conforming **consumer** (the vault maintainer) **MUST**:

1. Verify proof and `VaultWrite` capability on the entry's context.
2. If `expectedVersion` is supplied and does not match → `vault/delete:version_conflict` with `details.currentVersion`.
3. Atomically: set a tombstone marker on the entry with `deletedAt = now`, `graceUntil = now + maintainer-policy-grace`. The secret material MUST be zeroised immediately (cleartext erased from storage). The metadata view MAY be retained until `graceUntil` so sync clients can observe the deletion identity.
4. Emit a `sync/event/0.1` of kind `vault.deleted` to every consumer with VaultRead on the entry's context, carrying `{ id, deletedAt, graceUntil }`.
5. After `graceUntil`, the maintainer MAY purge the tombstone entirely. Consumers that connect after purge see the entry as absent (indistinguishable from "never existed").
6. Treat a delete-of-an-already-deleted entry as idempotent if the tombstone still exists; return success. If the tombstone has been purged, return `not_found`.

## Payload

`payload.id` (REQUIRED).

`payload.expectedVersion` (SHOULD).

`payload.reason` (optional) — recorded in audit log.

## Response

`payload.id`, `payload.deletedAt`, `payload.graceUntil`.

## Security & Privacy

**Enumeration resistance.** As with `vault/get`, conflate `not_found` and `permission_denied` by default. A consumer cannot probe whether an entry exists by trying to delete it.

**Immediate secret zeroisation.** The metadata tombstone may linger; the secret bytes MUST NOT. A maintainer that retains zeroised-secret records past `deletedAt` violates this spec.

**Audit.** Every deletion is logged with `{ who, when, id, reason? }`. The reason field is part of the producer's signed surface.

**Replay.** Retries with the same `id` are idempotent within the grace period.
