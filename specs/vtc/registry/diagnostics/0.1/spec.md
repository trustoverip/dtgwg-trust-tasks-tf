---
slug: vtc/registry/diagnostics
version: "0.1"
title: VTC Registry — Diagnostics
summary: Report the registry-reconciler's health — queue depth, RTBF-batched and failed counts, oldest-pending age, and last success/failure timestamps.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - registry
  - diagnostics
  - reconciler
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only operational telemetry. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads reconciler counters and timestamps; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: "`lastError` may echo upstream registry URLs, HTTP codes, or messages, which are operationally sensitive."
errorCodes:
  - code: vtc/registry/diagnostics:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Registry — Diagnostics** Trust Task reports the health of the
community's **registry reconciler** — the background worker that syncs
community state to the trust registry (including RTBF-batched removals).

It returns:

- `registryStatus` — `active` | `degraded` reachability;
- `queueDepth` — pending + in-flight sync jobs;
- `rtbfBatchedCount` — jobs parked behind the RTBF batch window;
- `failedCount` — terminal-failure rows awaiting operator triage;
- `oldestPendingAgeSeconds` — age of the oldest dispatchable job (null when
  the queue is empty);
- `lastSuccessAt` / `lastFailureAt` / `lastError` — last outcomes.

This replaces the earlier non-conformant `health/diagnostics` naming: it is
registry-reconciler telemetry grouped under the `vtc/registry/*` namespace,
not a generic health probe.

## Conformance

Producer: send an empty request (only the framework `ext` point is allowed).

Consumer: verify the community-admin capability, then return the current
reconciler counters and timestamps. Fields describing an absent condition
(`oldestPendingAgeSeconds` on an empty queue, `lastError` with no prior
failure) are `null`.

## Security & Privacy

**Admin-class metadata** (`discloses: metadata`). `lastError` may echo
upstream registry URLs, HTTP status codes, or error messages — operationally
sensitive detail. The task sits behind the community-admin gate; do not
surface `lastError` to non-admin callers.
