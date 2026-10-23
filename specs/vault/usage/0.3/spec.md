---
slug: vault/usage
version: "0.3"
title: Vault — Usage
summary: A vault consumer queries the maintainer's audit log of recent credential uses (proxy-logins, releases), filtered by entry, context, consumer, kind, and time range.
status: draft
targetFrameworkVersion: "0.5.0"
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
  - code: vault/usage:filterConflict
    meaning: Invalid filter combination.
    retryable: false
---

## Abstract

The **Vault — Usage** Trust Task is the read side of the maintainer's vault-use audit log. It drives UIs like "recent activity", "what has agent X done on my behalf", and "how many times has my GitHub credential been used this month".

## Changes from 0.2

Paging moves to the registry-wide pagination convention — request `cursor` + `limit`, response `nextCursor` absent on the last page — so every list task pages the same way:

* Request **`pageSize`** → **`limit`**. Same type and bounds (integer, 1–1000).
* Response **`cursor`** → **`nextCursor`**. Same opaque string, sent back as the next request's `cursor`.
* Response **`truncated`** is **removed**, and `uses` is now the only required response member. The presence of `nextCursor` says more matching records remain; its absence marks the last page.

The request's `cursor` member keeps its name and meaning. `since` is unchanged on the wire (an RFC 3339 `date-time`); its description now states what 0.2 left implicit, per the registry's `since` convention: it is an inclusive lower bound on `occurredAt`. Nothing else in the payload moves. 0.2's `wireCompatibleWith: "0.1"` declaration is not carried forward, since this version is not wire-identical to `0.1`.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains published.

## Conformance

A conforming **producer** **MAY** populate any combination of filters. A conforming **consumer** **MUST** authorise the requesting consumer and return records only within its visibility scope — at minimum:

- A Companion with broad scope (admin, policy-admin) sees all records.
- A Companion with limited scope sees its own records plus records on entries in contexts it has VaultRead for.
- A Service consumer sees only its own records — never another consumer's.

Records are returned in `occurredAt` descending order by default. Pagination is opaque-cursor based: the consumer honours `limit`, returns a `nextCursor` iff more matching records remain, and **MUST NOT** accept a `cursor` it did not issue.

## Payload

`entryId`, `contextId`, `byConsumer`, `since`, `until`, `kindFilter`, `cursor`, `limit` — all optional, AND-combined. `since` keeps only uses whose `occurredAt` is at or after that instant (inclusive).

## Response

`uses` — list of `UsageRecord`. `nextCursor` — present when more matching records remain; absent on the last page.

## Security & Privacy

**Replay.** Read-only; replay is benign.

### Data carried

The request carries optional filters, including `byConsumer` (a consumer DID) and a time range. The response carries `UsageRecord`s: the entry and context ids, the initiating `consumerDid`, `kind`, `outcome`, `occurredAt`, and optionally `deviceId`, `sessionId`, `policyDecisionId` and `errorCode` — a record of who used which credential, from which device, and when.

**No secret leakage.** Usage records carry IDs and decision metadata, never secret material.

The spec imposes no further rule on what a producer puts in `ext`.

### Correlation

The response joins consumer DIDs and device ids to credential entries and timestamps, which is a behavioural timeline of each consumer; that is inherent to an audit read.

**Visibility scoping.** A Service consumer querying for `byConsumer` of a *different* DID MUST be denied (`permissionDenied`). Only admin-class Companions can audit across consumers.

### Retention

**Retention.** The maintainer's retention policy determines how far back records are available. RECOMMENDED 12 months for human deployments, shorter for AI-agent-heavy deployments (storage bound).

The spec imposes no rule on how long a consumer keeps the records it receives.

### Consent/purpose

The purpose is audit and activity display — "recent activity", "what has agent X done on my behalf", how often a credential has been used. The spec states no further limit on reuse.
