---
slug: vault/get
version: "0.2"
title: Vault — Get
summary: A vault consumer fetches the metadata view of a single vault entry by id; secret material is never returned.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vault
  - credentials
  - get
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
  rationale: Read-only by id, usually session-authenticated. Recommended on non-session-bound transports so the maintainer can attribute the request to a specific consumer key.
errorCodes:
  - code: vault/get:notFound
    meaning: No entry with this id exists in any context the consumer can read.
    retryable: false
  - code: vault/get:permissionDenied
    meaning: The entry exists but the consumer lacks VaultRead on its context.
    retryable: false
---

## Abstract

The **Vault — Get** Trust Task returns the metadata view of one entry. Consumers use it to refresh a single row after a `sync/event` notification, or to fetch detail for a UI panel after a list result.

Like `vault/list/0.1`, this task **never returns secret material**. Use `vault/release/0.1` to obtain secret bytes.

## Conformance

A conforming **producer** **MUST** populate `payload.id` with a non-empty entry identifier. A conforming **consumer** **MUST** authorise the requesting consumer against the context the entry belongs to, return `vault/get:notFound` when the entry is absent OR the consumer lacks visibility into its context (so existence cannot be probed by enumeration), and return the entry in the metadata-only view per the shared `VaultEntry` schema.

Maintainers MAY return `vault/get:permissionDenied` instead of `vault/get:notFound` only when the consumer can already prove existence via another channel (e.g. the entry id appeared in a prior list response with redacted fields). The default is to conflate the two to deny enumeration.

## Payload

`payload.id` (REQUIRED) — the entry id.

`payload.ext` (optional).

## Response

`payload.entry` — the VaultEntry in metadata-only view.

`payload.redactedFields` (optional) — names of fields the maintainer redacted.

## Security & Privacy

**Enumeration resistance.** Conflating `notFound` and `permissionDenied` is the default for a reason: a consumer that can distinguish them can probe id space to map who has what. Maintainers operating in lower-trust environments (e.g. publicly-listed vaults) MUST keep them conflated.

Other guidance: see the Security & Privacy section of `vault/list/0.1`. The same secret-leakage, timing-data, and audit considerations apply here, scoped to a single entry.
