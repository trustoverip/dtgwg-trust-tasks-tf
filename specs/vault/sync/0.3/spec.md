---
slug: vault/sync
version: "0.3"
title: Vault — Sync
summary: A vault consumer requests an incremental delta of vault, ACL, and policy events since a known seq baseline; pairs with sync/event push notifications to keep local caches converged with the maintainer.
status: draft
targetFrameworkVersion: "0.5.0"
category: credentials
keywords:
  - vault
  - sync
  - delta
  - multi-device
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
  requirement: RECOMMENDED
  rationale: Read-only delta query. Recommended on non-session-bound transports for the same attribution reason as vault/list.
sideEffects:
  level: none
  rationale: "Read-only incremental delta of vault/ACL/policy events since a baseline."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vault/sync:seqTooOld
    meaning: The supplied `sinceSeq` is older than the maintainer's retained event horizon; the consumer cannot catch up incrementally and MUST resync from scratch (omit `sinceSeq`). This happens when a consumer has been offline longer than the maintainer's event-retention window.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        oldestRetainedSeq: { type: "integer", minimum: 0 }
---

## Abstract

The **Vault — Sync** Trust Task is the incremental-delta companion to the `sync/event/0.1` server-push notification: a long-lived consumer subscribes to push events while connected; when reconnecting after offline, it calls `vault/sync` to catch up on missed events.

Events are strictly ordered by `seq` within a context. The consumer applies them in order to keep its local cache convergent with the maintainer's authoritative state.

The wire format intentionally returns lightweight events (full `VaultEntry` on upsert, only ids on delete, summary discriminators for ACL and policy changes). Consumers fetch full state for ACL / policy via the dedicated tasks (`acl/show`, `policy/list`).

## Changes from 0.2

* Request **`pageSize`** → **`limit`**, following the registry-wide pagination convention's name for the page bound. Same type and bounds (integer, 1–5000); the maintainer still chooses the default and MAY return fewer.

Nothing else moves: `sinceSeq`, `newSinceSeq` and `truncated` are unchanged. This task is a position-keyed change feed rather than a page of a collection, and a feed whose position (`newSinceSeq`) is always returned keeps `truncated` deliberately — there is no cursor whose presence could stand in for it. 0.2's `wireCompatibleWith: "0.1"` declaration is not carried forward, since this version is not wire-identical to `0.1`.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains published.

## Conformance

A conforming **producer** **MUST**:

1. Omit `sinceSeq` on first sync to receive a full snapshot (events for every visible entry, plus the current ACL and policy state).
2. Persist `newSinceSeq` from each response; supply it as `sinceSeq` on the next call.
3. Loop on `truncated: true` — call again with the new baseline until `truncated: false`.

A conforming **consumer** (the vault maintainer) **MUST**:

1. Filter the event stream by the requesting consumer's ACL: an event referencing an entry the consumer cannot read MUST be omitted (NOT replaced with a redacted stub). The consumer never learns of events outside its scope.
2. Order events strictly by `seq` within each context. Across contexts, events MAY interleave by occurrence time; consumers apply per-context in order.
3. Return `seqTooOld` with `details.oldestRetainedSeq` when the consumer's `sinceSeq` is older than the maintainer's retention window. The consumer recovers by resyncing from scratch.
4. Garbage-collect events older than the retention window. The window MUST be at least as long as the maintainer's longest documented vault-delete `graceUntil` — otherwise an offline consumer could miss a deletion entirely.

## Payload

`payload.contextId` (optional) — narrow to one context.

`payload.sinceSeq` (optional) — observed baseline; omit for full snapshot.

`payload.limit` (optional, 1–5000) — the most events to return; the maintainer chooses a default and MAY return fewer.

## Response

`payload.events` — ordered list.

`payload.newSinceSeq` — next baseline.

`payload.truncated` — more available.

## Examples

### First-time sync (full snapshot)

```json
{
  "id": "vsync-1234",
  "type": "https://trusttasks.org/spec/vault/sync/0.3",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T12:00:00Z",
  "payload": { "limit": 1000 }
}
```

### Catch-up after offline

```json
{
  "id": "vsync-2345",
  "type": "https://trusttasks.org/spec/vault/sync/0.3",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T12:05:00Z",
  "payload": { "sinceSeq": 4217, "limit": 500 }
}
```

## Security & Privacy

**Replay.** This task is read-only; replay is benign.

### Data carried

The request carries an optional `contextId`, the `sinceSeq` baseline and a page bound. The response carries ordered `SyncEvent`s — a full metadata-view `VaultEntry` on upsert, only ids on delete, and summary discriminators for ACL and policy changes — together with `newSinceSeq` and `truncated`. No secret material is carried; the entries carry the same metadata `vault/list` returns. The spec imposes no further rule on what a producer puts in `ext`.

### Correlation

**Strict ACL filtering.** The maintainer never leaks events for entries outside the consumer's scope — not even existence by gap in the seq sequence. Per-consumer seq sequences (the consumer's `sinceSeq` is local to that consumer) prevent gap-based inference.

Entry ids are stable across events, so a consumer can follow one entry's history through successive syncs, and the `sinceSeq` a consumer sends reveals to the maintainer how long it has been offline.

### Retention

**Retention window.** Setting retention shorter than the longest delete grace period creates a correctness bug: an offline consumer could miss the delete and re-apply a stale entry on reconnect. The retention SHOULD be at least 30 days (default delete grace) and is RECOMMENDED to be 90 days.

**Audit.** Sync calls are typically high-volume; maintainers MAY sample audit records rather than log every call. RECOMMENDED to log first-sync (no `sinceSeq`) and `seqTooOld` errors fully.

### Consent/purpose

The purpose is to keep a consumer's local cache convergent with the maintainer's authoritative state after time offline. The spec states no further limit on reuse of the events received.
