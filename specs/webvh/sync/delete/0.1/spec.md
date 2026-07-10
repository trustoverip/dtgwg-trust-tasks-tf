---
slug: webvh/sync/delete
version: "0.1"
title: WebVH — Sync Delete
summary: A did:webvh control plane replicates a DID deletion to a registered hosting server.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [webvh, sync, replication, delete, control-plane, hosting-server]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Control plane
    requirement: REQUIRED
    member: issuer
  - role: Hosting server (Service role)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Same trust model as `webvh/sync/update` — Service-role authentication binds the producer; a transport-independent proof is useful for audit but not required for steady-state replication.
sideEffects:
  level: destructive
  rationale: "Replicates a DID deletion to a hosting server; the server removes the DID's content."
consequences:
  - "The hosting server drops the DID; resolution there stops."
subjectPath: /mnemonic
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: webvh/sync/delete:not_authorized
    meaning: Sender DID is not the configured control plane for the receiving server.
    retryable: false
related:
  - did-management/did/delete
  - webvh/sync/update
---

## Abstract

The **Sync Delete** Trust Task replicates a slot deletion from the control plane to a registered hosting server. It is the deletion counterpart to [`webvh/sync/update`](../update/0.1/spec.md) and fires once per registered server immediately after the control plane's own `did/delete` completes.

The message is **idempotent**: deleting an already-deleted (or never-known) slot succeeds with `status: "deleted"`. Servers MUST tolerate at-least-once delivery from the control plane's durable outbox without producing duplicate side effects.

On the server, a successful delete removes the metadata record, log content, witness content, owner index, and any watcher-sync markers in a single atomic batch — the resolver returns 404 for the slot afterwards.

## Status of this Document

Draft.

## Conformance

Producer (control plane) MUST emit `type: https://trusttasks.org/spec/webvh/sync/delete/0.1` with `payload.mnemonic`. Consumer (hosting server) MUST:

1. Verify the sender DID is the server's configured control plane, else reject with `webvh/sync/delete:not_authorized`.
2. Remove the slot's record, log, witness content, owner index, and watcher markers in a single atomic batch; if the slot is not present, treat as a no-op.
3. Respond with `status: "deleted"` referencing the same `mnemonic`.

## Request

```json
{ "id": "sd-1", "type": "https://trusttasks.org/spec/webvh/sync/delete/0.1",
  "issuer": "did:web:control.example.com", "recipient": "did:web:node1.example.com",
  "issuedAt": "2026-06-27T09:00:00Z",
  "payload": { "mnemonic": "alice" } }
```

## Response

```json
{ "id": "sd-1-r", "type": "https://trusttasks.org/spec/webvh/sync/delete/0.1#response",
  "threadId": "sd-1", "issuer": "did:web:node1.example.com", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-27T09:00:01Z",
  "payload": { "mnemonic": "alice", "status": "deleted" } }
```

## Security & Privacy

The `not_authorized` gate is load-bearing: a successful sync-delete is destructive against the server's local copy. Servers MUST refuse sync-delete messages from any sender other than the configured control plane DID, even if the sender holds Service role generally.

A control plane that issues a sync-delete is asserting that the canonical record has been removed centrally; servers SHOULD NOT mirror the deletion to caches outside the slot's record set (e.g. external resolver caches) — that responsibility belongs to the resolver's own TTL management.
