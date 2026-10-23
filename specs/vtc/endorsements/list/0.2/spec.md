---
slug: vtc/endorsements/list
version: "0.2"
title: VTC Endorsements — List
summary: Page through the statement credentials the community issued through vtc/endorsements/issue, optionally filtered by predicate or subject; live and revoked rows both surface.
status: draft
targetFrameworkVersion: "0.5.0"
category: credentials
keywords:
  - vtc
  - endorsements
  - credentials
  - list
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community issuer
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only enumeration. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads endorsement rows; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsements/list:invalidCursor
    meaning: The supplied `cursor` is malformed or has expired.
    retryable: false
---

## Abstract

The **VTC Endorsements — List** Trust Task pages through the community's issued
[`Endorsement`](../../../_shared/0.1/endorsement.schema.json) rows. Filters:
`typeUri` (a registered predicate IRI, matched against the statement's
`credentialSubject.predicate`), `subjectDid`, and `includeRevoked`. A row's
`typeUri` says what it records: a registered predicate IRI for a DTG Verifiable
Statement Credential (`StatementCredential`) issued by
[`vtc/endorsements/issue`](../../issue/0.1/spec.md) — the community's own
identity checks among them, under
`https://registry.trustoverip.org/dtg/vsc/vetted/1` — or `role:vetter` for a
vetter role credential (a Verifiable Authority Credential) issued by
[`vtc/vetting/vetters/grant`](../../../vetting/vetters/grant/0.1/spec.md). The
`typeUri` filter accepts either.

Cursor pagination follows the same convention as the rest of the `vtc/*`
families: an opaque `cursor`, a `limit` clamped to `1..=200` (default 50), and
a `nextCursor` that is absent on the last page.

## Changes from 0.1

The response's **`nextCursor`** changes type from `["string", "null"]` to **`"string"`**. It is present when more items remain and **absent** on the last page; a `null` value now fails validation. A producer that detected the last page by `nextCursor === null` **MUST** detect it by absence instead.

This is the registry-wide pagination convention: a continuation token is an optional string whose absence marks the last page, so there is one spelling of "no more pages" instead of two.

The Abstract's paging paragraph, which said `nextCursor` is `null` on the last page, now says it is absent.

The request's `cursor` and `limit` already followed the convention and are unchanged on the wire — same types and bounds; only their schema descriptions are reworded to the registry's paging text. `totalEstimate` is unchanged. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

Producer: send an empty payload for the first page; pass the previous page's
`nextCursor` as `cursor` to continue.

Consumer: verify the community-admin **or** issuer capability against the live
ACL. Apply the supplied filters, clamp `limit`, and return the page. By default
**both live and revoked rows surface** — consumers filter on `revokedAt`; set
`includeRevoked: false` to omit revoked rows server-side. Reject a malformed or
expired `cursor` with `invalidCursor` rather than silently restarting from the
first page.

## Security & Privacy

### Data carried

The request carries optional filters (`typeUri`, `subjectDid`,
`includeRevoked`) and paging members. The response carries
[`Endorsement`](../../../_shared/0.1/endorsement.schema.json) rows — each with
its identifier, kind (`typeUri`), subject DID, claim, the issued credential, a
status-list index and, when revoked, `revokedAt` — plus an optional
`totalEstimate`. By default both live and revoked rows surface. The
specification imposes no further minimisation rule.

### Correlation

**Admin/issuer-class metadata** (`discloses: metadata`). Enumeration reveals
every subject the community has endorsed and under which types, so the
capability gate is load-bearing — this task must not be exposed to ordinary
members.

A row joins a subject DID to the predicate it was endorsed under and, through
`revokedAt`, to when that endorsement was withdrawn. `threadId` correlates
request and response, and `nextCursor`/`cursor` link the pages of one
traversal; a responder MUST NOT accept a cursor it did not issue.

### Retention

The task reads endorsement rows and persists nothing. The specification
imposes no retention rule on the producer's copy of the page.

### Consent/purpose

The purpose is for a community administrator or issuer to review the
statement and role credentials the community has issued. The specification
states no further purpose limitation.
