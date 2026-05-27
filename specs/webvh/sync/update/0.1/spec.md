---
slug: webvh/sync/update
version: "0.1"
title: WebVH — Sync Update
summary: A did:webvh control plane replicates a DID's current log and witness content to a registered hosting server.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [webvh, sync, replication, control-plane, hosting-server]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Control plane
    requirement: REQUIRED
  - role: Hosting server (Service role)
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: Sync messages travel between trusted infrastructure nodes already bound by Service-role authentication on the receiving server; a transport-independent proof is valuable for audit replay but not required for steady-state replication.
errorCodes:
  - code: webvh/sync/update:not_authorized
    meaning: Sender DID is not the configured control plane for the receiving server.
    retryable: false
  - code: webvh/sync/update:invalid_log
    meaning: The `logContent` failed structural validation or hash-chain verification.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: string }
related:
  - did-management/did/publish
  - did-management/did/register
  - webvh/sync/delete
  - webvh/witness/publish
---

## Abstract

The **Sync Update** Trust Task replicates the current state of a did:webvh slot from the control plane to a hosting server registered under the slot's domain. It fires whenever the slot mutates — register, publish, witness publish, change-owner — so each registered server sees the same log and witness content the control plane holds.

The message is **idempotent**: receiving the same `(mnemonic, versionCount)` pair twice is a no-op replace. Servers MUST tolerate at-least-once delivery from the control plane's durable outbox without producing duplicate side effects. Receiving an *older* `versionCount` than the server already holds is also a no-op (the server keeps its newer state and acknowledges the request).

Servers respond with a status flag confirming the apply. Failures use `trust-task-error`; the control plane retries with exponential backoff until acknowledgement.

## Status of this Document

Draft.

## Conformance

Producer (control plane) MUST emit `type: https://trusttasks.org/spec/webvh/sync/update/0.1` with `payload.mnemonic`, `payload.didId`, `payload.logContent`, `payload.versionCount`, and optionally `payload.witnessContent`. Consumer (hosting server) MUST:

1. Verify the sender DID is the server's configured control plane, else reject with `webvh/sync/update:not_authorized`.
2. Validate `logContent` parses as a did:webvh log (one JSON object per line) and that the hash chain verifies, else reject with `webvh/sync/update:invalid_log`.
3. Apply `logContent` and `witnessContent` (when present) as a single atomic batch.
4. Respond with `status: "applied"` referencing the same `mnemonic`.

## Request

```json
{ "id": "su-1", "type": "https://trusttasks.org/spec/webvh/sync/update/0.1",
  "issuer": "did:web:control.example.com", "recipient": "did:web:node1.example.com",
  "issuedAt": "2026-06-26T09:00:00Z",
  "payload": {
    "mnemonic": "alice",
    "didId": "did:webvh:abc123:did.example.com:alice",
    "logContent": "{\"versionId\":\"1-...\",...}\n{\"versionId\":\"2-...\",...}",
    "witnessContent": "{\"versionId\":\"2-...\",\"witness\":\"did:webvh:WIT1:witness.example.com\",...}",
    "versionCount": 2
  } }
```

## Response

```json
{ "id": "su-1-r", "type": "https://trusttasks.org/spec/webvh/sync/update/0.1#response",
  "threadId": "su-1", "issuer": "did:web:node1.example.com", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-26T09:00:01Z",
  "payload": { "mnemonic": "alice", "status": "applied" } }
```

## Security & Privacy

The `not_authorized` gate is load-bearing: any sender that is not the configured control plane DID MUST be refused, since a successful sync overwrites the server's local view of the slot. Servers MUST NOT accept sync updates from arbitrary callers even if the caller authenticates as Service — the gate is specifically against the configured control-plane DID, not against the role generally.

Sync messages carry the full current log; a compromised control plane can rewrite history on a slot it controls. Consumers (resolvers) that need byzantine-resistance against a compromised control plane SHOULD rely on the witness-proof chain in `webvh/witness/publish`, which provides independent attestation per version.
