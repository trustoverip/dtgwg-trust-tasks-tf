---
slug: sync/event
version: "0.1"
title: Sync — Event
summary: One-way push notification from the maintainer to a subscribing consumer, carrying a single vault/ACL/policy change event. Pairs with vault/sync/0.1 for offline catch-up.
status: draft
targetFrameworkVersion: "0.1"
category: data-exchange
keywords:
  - sync
  - event
  - push
  - notification
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault maintainer
    requirement: REQUIRED
    member: issuer
  - role: vault consumer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A push event causes the consumer to mutate cache state (apply an upsert, wipe a tombstone, surface a device-disabled warning). The maintainer's authority MUST be verifiable so the consumer cannot be tricked by a mediator or man-in-the-middle into mutating cache against a spoofed event.
sideEffects:
  level: mutating
  rationale: "Delivers a single change event that the subscriber applies to its replicated cache."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
---

## Abstract

The **Sync — Event** Trust Task is the push-notification half of the vault sync model. The maintainer issues one of these to every subscribing consumer whenever a `vault.upserted`, `vault.deleted`, `acl.changed`, or `policy.changed` event occurs. The consumer verifies and applies; there is no response.

For offline catch-up — when a consumer connects after having missed events — it calls `vault/sync/0.1` to receive the same events in batched form.

The two tasks reuse the same `SyncEvent` shared schema so consumers can share code paths between push and pull.

## Conformance

A conforming **producer** (the maintainer) **MUST**:

1. Emit one `sync/event/0.1` document per event, with the event's `seq` monotonically increasing per consumer.
2. Filter by ACL: a consumer receives only events for resources it can read. An entry the consumer cannot see MUST NOT appear in a push event.
3. Carry a `proof`.
4. Retry delivery on failure (mediator unavailability, consumer offline). When the consumer reconnects after extended offline, the maintainer MAY stop retrying individual events and rely on the consumer's next `vault/sync` to catch up — the seq values let the consumer detect gaps.

A conforming **consumer** **MUST**:

1. Verify the proof. A push event whose proof does not verify MUST be ignored (defends against mediator-injected fake events).
2. Apply events in `seq` order per context. If a gap is detected (e.g. seq 7 arrives but the consumer's baseline is 5), the consumer SHOULD call `vault/sync` immediately to fill the gap.
3. Apply idempotently — receiving the same `seq` twice MUST be a no-op.
4. NOT respond. There is no `#response` form for this task.

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.1, where the declaration is not yet required.*

The authorization evidence this task presupposes is the **maintainer's own authority over the resources it reports**, exercised in the opposite direction from most tasks in this registry: the maintainer is the producer here, and the consumer is the party being told about its own data.

Two consequences the consumer relies on. The maintainer filters by ACL before sending, so an event names only resources this consumer may read — an authorization decision taken at composition time rather than at receipt. And the consumer verifies the `proof` before acting on an event, which is what stops a mediator injecting one; that check establishes the event came from the maintainer, not that the maintainer was entitled to send it.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Payload

`payload.event` — one of `VaultUpsertedEvent`, `VaultDeletedEvent`, `AclChangedEvent`, `PolicyChangedEvent`.

## Examples

### Vault upsert push

```json
{
  "id": "syncev-1234",
  "type": "https://trusttasks.org/spec/sync/event/0.1",
  "issuer": "did:web:vta.example",
  "recipient": "did:peer:2.Ez6LSc…",
  "issuedAt": "2026-05-26T15:00:00Z",
  "payload": {
    "event": {
      "kind": "vault.upserted",
      "seq": 4218,
      "occurredAt": "2026-05-26T15:00:00Z",
      "entry": {
        "id": "vault_01HZX2QY…",
        "contextId": "ctx_personal",
        "targets": [{ "kind": "web-origin", "origin": "https://github.com" }],
        "label": "Personal GitHub",
        "secretKind": "passkey",
        "createdAt": "2026-02-14T09:30:00Z",
        "updatedAt": "2026-05-26T15:00:00Z",
        "version": 8
      }
    }
  },
  "proof": { "…": "…" }
}
```

### Vault delete push

```json
{
  "id": "syncev-2345",
  "type": "https://trusttasks.org/spec/sync/event/0.1",
  "issuer": "did:web:vta.example",
  "recipient": "did:peer:2.Ez6LSc…",
  "issuedAt": "2026-05-26T15:01:00Z",
  "payload": {
    "event": {
      "kind": "vault.deleted",
      "seq": 4219,
      "occurredAt": "2026-05-26T15:01:00Z",
      "id": "vault_01HZX2R0…",
      "contextId": "ctx_personal",
      "graceUntil": "2026-06-25T15:01:00Z"
    }
  },
  "proof": { "…": "…" }
}
```

## Security & Privacy

**Proof verification.** A consumer MUST verify the proof before applying. This is the primary defence against a mediator-injected fake event silently introducing a malicious credential into the consumer's cache.

**Gap detection.** Lost events show up as seq gaps; the consumer's `vault/sync` call closes them. The maintainer's event-retention window MUST be at least as long as the longest plausible offline interval (RECOMMENDED 30+ days).

**No reply.** The lack of a response form is intentional — push events are fire-and-forget. The maintainer detects acknowledgement implicitly via the consumer's subsequent `vault/sync(sinceSeq = highestApplied)` call.

**Replay.** Idempotency at the consumer side: applying the same `seq` twice is a no-op. The maintainer MAY retry on transient transport failures without coordination.
