---
slug: did-management/server/stats-sync
version: "0.1"
title: DID Management — Server Stats Sync
summary: A hosting server pushes a delta of its resolve/update counters to the control plane since the last sync.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, server, stats, sync]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Hosting server (Service role)
    requirement: REQUIRED
  - role: Control plane
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: Stats are operational, not adjudicative; transport authentication suffices for most uses, but a proof becomes valuable when stats feed billing.
errorCodes:
  - code: did-management/server/stats-sync:forbidden
    meaning: Caller does not hold the `Service` role on this control plane.
    retryable: false
related: [did-management/server/register]
---

## Abstract

The **Server Stats Sync** Trust Task is a one-way push from a hosting server to the control plane. The payload carries per-mnemonic increments since the last sync (`resolves`, `updates`) and optionally the underlying time-series bucket deltas at 5-minute resolution. Service-role only — accepting stats from a non-Service caller would let any authenticated DID skew the control plane's view of fleet activity.

## Status of this Document

Draft.

## Conformance

The producer (Service-role caller) emits `type: https://trusttasks.org/spec/did-management/server/stats-sync/0.1` with `payload.instanceId` and `payload.perMnemonic[]`. Bucket-level data is optional. The consumer verifies Service-role, merges the deltas into its aggregate counters, and replies with how many records it accepted.

## Request

```json
{ "id": "ss-1", "type": "https://trusttasks.org/spec/did-management/server/stats-sync/0.1",
  "issuer": "did:web:node1.example.com", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-22T09:00:00Z",
  "payload": { "instanceId": "did_web_node1_example_com",
    "perMnemonic": [ { "mnemonic": "alice", "resolves": 42, "updates": 0 } ],
    "buckets": [ { "mnemonic": "alice", "epoch": 1718956800, "r": 42, "u": 0 } ] } }
```

## Response

```json
{ "id": "ss-1-r", "type": "https://trusttasks.org/spec/did-management/server/stats-sync/0.1#response",
  "threadId": "ss-1", "issuer": "did:web:control.example.com", "recipient": "did:web:node1.example.com",
  "issuedAt": "2026-06-22T09:00:01Z",
  "payload": { "instanceId": "did_web_node1_example_com", "accepted": 1 } }
```

## Security & Privacy

Service-role gating is critical. Sync messages are diagnostic; consumers MAY clamp absurd deltas (e.g. claims of more updates than the slot's `versionCount` supports).
