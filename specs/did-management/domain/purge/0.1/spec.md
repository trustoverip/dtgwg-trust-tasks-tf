---
slug: did-management/domain/purge
version: "0.1"
title: DID Management — Domain Purge
summary: An administrator force-removes a disabled hosting domain, bypassing the standard grace period and optionally fanning a purge directive out to every server instance currently serving the domain.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, purge, admin, destructive]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Purge is irreversible and may delete every DID hosted under the domain. The maintainer MUST retain a signed authorisation.
sideEffects:
  level: destructive
  rationale: "Force-removes a disabled domain, bypassing the grace period and optionally purging it from every serving instance."
consequences:
  - "Removes the domain past the recovery grace period and can purge it from all serving instances."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: did-management:unknown_domain
    meaning: The submitted `name` does not match a known hosting domain. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
  - code: did-management/domain/purge:not_disabled
    meaning: The domain is still `active`; disable it first before purging.
    retryable: false
  - code: did-management/domain/purge:is_default
    meaning: The domain is the current system default; cannot purge until the default is moved.
    retryable: false
related: [did-management/domain/disable, did-management/domain/set-default]
---

## Abstract

The **Domain Purge** Trust Task removes a disabled hosting domain from the host's record set immediately, skipping the cooling-off `purgeAt` window. When `payload.purgeServers: true`, the consumer fans a `domain.purge/1.0` DIDComm message out to every registry entry whose `servedDomains` includes the name, so fleet-wide cleanup happens in the same admin action. Per-server fanout is best-effort: an unreachable instance is logged and skipped, and the local delete proceeds regardless.

## Status of this Document

Draft.

## Conformance

Admin caller emits `type: https://trusttasks.org/spec/did-management/domain/purge/0.1` with `payload.name` and optional `payload.purgeServers`. Consumer rejects if the domain is not disabled or is the current default; otherwise removes the domain entry, cancels any pending purge timer, and (when fanout is requested) records each instance's queue status.

## Authorization

*Stated in anticipation of [SPEC §7.3](../../../../../SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.1, where the declaration is not yet required.*

The authorization evidence this task presupposes is **administrator standing on this consumer**. This task is `destructive` — it removes a domain entry — so the check matters more here than on its siblings, not less.

The state preconditions Conformance names — that the domain is disabled, and is not the current default — are safety interlocks, not authorization. They stop an administrator purging something still in use; they say nothing about whether the caller is an administrator. A caller that satisfies both and lacks standing is refused with `permissionDenied`.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](../../../../../SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Request

```json
{ "id": "dp-1", "type": "https://trusttasks.org/spec/did-management/domain/purge/0.1",
  "issuer": "did:key:z6MkAdmin", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-15T09:00:00Z",
  "payload": { "name": "tenant-a.example.com", "purgeServers": true } }
```

## Response

```json
{ "id": "dp-1-r", "type": "https://trusttasks.org/spec/did-management/domain/purge/0.1#response",
  "threadId": "dp-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-06-15T09:00:01Z",
  "payload": { "name": "tenant-a.example.com", "purgedAt": "2026-06-15T09:00:01Z",
    "fanout": [
      { "instanceId": "did_web_node1_example", "status": "queued" },
      { "instanceId": "did_web_node2_example", "status": "failed" }
    ] } }
```

## Security & Privacy

Destructive. Admin caller MUST be aware that captured DIDs on the domain become unresolvable immediately. The signed document and the per-instance fanout statuses together form the audit trail.
