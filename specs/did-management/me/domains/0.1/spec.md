---
slug: did-management/me/domains
version: "0.1"
title: DID Management — Me / Domains
summary: An authenticated caller asks a hosting service for the subset of hosting domains it is permitted to act on, plus the caller's default.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [did-hosting, domain, discovery, acl, default]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Caller (any authenticated principal)
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: The result is a caller-scoped view of public-by-domain-name configuration; the maintainer authenticates the producer via the transport-layer session and a proof becomes valuable only when the response is replayed for audit.
sideEffects:
  level: none
  rationale: "Read-only read of the domains the caller may act on."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - did-management/domain/create
  - did-management/domain/update
  - did-management/did/check-name
  - did-management/did/register
---

## Abstract

The **Me / Domains** Trust Task lets a caller discover which hosting domains they are permitted to use on a hosting service, and which of those is their *default*. The result is a caller-scoped projection of the host's domain registry — Admin and Service callers (and any caller whose ACL declares `DomainScope::All`) see every active domain; scoped callers see only the names their ACL entry lists, plus the explicit default the ACL records (or the system default when none is recorded).

This task is the canonical discovery surface for interactive tooling that needs to populate a `--domain` selector before a [`did/check-name`](../../../did/check-name/0.1/spec.md) or [`did/register`](../../../did/register/0.1/spec.md). It is intentionally a read-only projection — the caller learns *what is accessible*, not *what could be created* — and never mutates state.

## Status of this Document

Draft.

## Conformance

Caller authenticates to the hosting service and emits `type: https://trusttasks.org/spec/did-management/me/domains/0.1` with an empty payload. The consumer (hosting service) MUST:

1. Resolve the caller's ACL `DomainScope` and project the active-domain registry down to the permitted subset.
2. Return the scoped list as `payload.domains[]` — each element a `DomainEntry` per the shared schema, sorted by `name` ascending.
3. Set `payload.default` to the caller's effective default domain — the ACL-recorded default for the caller if present, else the system default, else `null`.
4. Return an empty list with `default: null` when the caller has no permitted domains; this is not an error.

## Request

```json
{ "id": "md-1", "type": "https://trusttasks.org/spec/did-management/me/domains/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:control.example.com",
  "issuedAt": "2026-06-22T09:00:00Z", "payload": {} }
```

## Response

```json
{ "id": "md-1-r", "type": "https://trusttasks.org/spec/did-management/me/domains/0.1#response",
  "threadId": "md-1", "issuer": "did:web:control.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-22T09:00:01Z",
  "payload": {
    "domains": [
      { "name": "tenant-a.example.com", "label": "Tenant A",
        "status": "active", "defaultDomain": false,
        "createdAt": "2026-06-10T09:00:00Z" },
      { "name": "did.example.com", "label": "Default tenant",
        "status": "active", "defaultDomain": true,
        "createdAt": "2026-05-01T09:00:00Z" }
    ],
    "default": "did.example.com"
  } }
```

Each element of `domains[]` is a `DomainEntry` per [`_shared/0.1/domain-entry.schema.json`](../../../_shared/0.1/domain-entry.schema.json). Consumers MAY populate operator-managed fields under `ext.<vendor>:*` (per [SPEC §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) when the host configures additional per-domain metadata; those keys are advisory and consumers SHOULD ignore unrecognized vendor namespaces.

## Security & Privacy

The response reveals which domain names the caller is permitted to use — domain *names* on a multi-tenant host can themselves be sensitive (they may reflect customer-internal naming). Consumers MUST scope the projection to the caller's ACL; an unscoped Admin or Service caller seeing every domain is the legitimate path, not a leak.

The task is idempotent and side-effect-free; a replayed request returns the current state. Consumers MAY rate-limit the endpoint to slow domain-name enumeration by a compromised caller.
