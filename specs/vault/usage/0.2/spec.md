---
slug: vault/usage
version: "0.2"
wireCompatibleWith: "0.1"
title: Vault — Usage
summary: A vault consumer queries the maintainer's audit log of recent credential uses (proxy-logins, releases), filtered by entry, context, consumer, kind, and time range.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vault
  - audit
  - usage
  - activity
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
  rationale: Read-only audit query.
sideEffects:
  level: none
  rationale: "Read-only query of the credential-use audit log."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vault/usage:permissionDenied
    meaning: The consumer lacks visibility into the requested scope.
    retryable: false
  - code: vault/usage:filterConflict
    meaning: Invalid filter combination.
    retryable: false
---

## Abstract

The **Vault — Usage** Trust Task is the read side of the maintainer's vault-use audit log. It drives UIs like "recent activity", "what has agent X done on my behalf", and "how many times has my GitHub credential been used this month".

## Conformance

A conforming **producer** **MAY** populate any combination of filters. A conforming **consumer** **MUST** authorise the requesting consumer and return records only within its visibility scope — at minimum:

- A Companion with broad scope (admin, policy-admin) sees all records.
- A Companion with limited scope sees its own records plus records on entries in contexts it has VaultRead for.
- A Service consumer sees only its own records — never another consumer's.

Records are returned in `occurredAt` descending order by default. Pagination is opaque-cursor based.

## Payload

`entryId`, `contextId`, `byConsumer`, `since`, `until`, `kindFilter`, `pageSize`, `cursor` — all optional, AND-combined.

## Response

`uses` — list of `UsageRecord`. `truncated`, `cursor` for pagination.

## Security & Privacy

**Visibility scoping.** A Service consumer querying for `byConsumer` of a *different* DID MUST be denied (`permissionDenied`). Only admin-class Companions can audit across consumers.

**No secret leakage.** Usage records carry IDs and decision metadata, never secret material.

**Retention.** The maintainer's retention policy determines how far back records are available. RECOMMENDED 12 months for human deployments, shorter for AI-agent-heavy deployments (storage bound).

**Replay.** Read-only; replay is benign.
