---
slug: device/heartbeat
version: "0.2"
title: Device — Heartbeat
summary: Periodic check-in from a Companion or Service that refreshes lastSeenAt, carries optional state digests, and lets the maintainer deliver queued operations (queued wipes especially).
status: draft
targetFrameworkVersion: "0.2"
category: identity
keywords:
  - device
  - heartbeat
  - keepalive
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: device
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Heartbeat is frequent and low-stakes; transport-level auth is sufficient. Recommended for non-session-bound transports for attribution.
errorCodes:
  - code: device/heartbeat:notRegistered
    meaning: The issuer's DID has no DeviceBinding. The consumer SHOULD complete device/register.
    retryable: false
---

## Abstract

The **Device — Heartbeat** Trust Task is the periodic check-in every Companion and Service makes against the maintainer. It refreshes `lastSeenAt`, gives the maintainer a chance to push queued operations (notably **queued wipes** for targets that were offline at the moment a wipe was issued), and lets the maintainer hint when a sync is due.

Recommended cadence: every 5 minutes for online Companions, every 60 minutes for Services.

## Conformance

A conforming **producer** populates optional state digests if available. A conforming **consumer** updates `lastSeenAt`, returns any queued operations from the maintainer's outbox (the consumer MUST execute them in order before continuing), and computes a `syncHint`.

The consumer MUST execute returned `wipe` operations before any other op (including any subsequent heartbeat).

## Payload

Optional: `platform` update, `vaultSeq` baseline.

## Response

`serverTime`, `queuedOperations`, `syncHint`.

## Security & Privacy

**Queued-wipe authentication.** The wipe document inside `queuedOperations[].task` is independently signed by the maintainer's admin key; the consumer verifies the proof on the inner document before executing. The heartbeat envelope itself does not need REQUIRED proof — it's the inner operations that matter.

**Drift detection.** Consumers MAY use `serverTime` to detect local clock drift. A drift greater than ~5 minutes is a sign of something wrong (manipulated client clock, NTP failure, attempt to backdate); consumers SHOULD log and SHOULD refresh time before security-sensitive operations.

**Audit volume.** Heartbeats are high-volume; maintainers MAY sample audit records, recording only state transitions (lastSeenAt gap > N hours, sync-hint changes, queued-op delivery).
