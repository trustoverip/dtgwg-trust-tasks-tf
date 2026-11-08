---
slug: vtc/badges/claims/list
version: '0.1'
title: VTC Badges — Claims — List
summary: "A badge's claim codes, as its owners and administrators see them — label, time, limit and claims — a page at a time, never a code."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- event
parties:
- role: badge owner or administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The list is answered only to a badge's owners and administrators, which depends on who asks.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: A read with no side effects; a replay discloses only what the caller was already entitled to see.
sideEffects:
  level: none
  rationale: Reads claim code records; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses each code's label, time, limit, claims and issuer — never a code.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/badges/claims/list:notFound
  meaning: No badge the caller owns or administers has this identifier. Conflates an unknown badge, one the caller may not see, badges being off, and a badge the caller may see but does not own.
  retryable: false
- code: vtc/badges/claims/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and badge.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/claims/create
- vtc/badges/claims/withdraw
- vtc/badges/awards/list
---

## Abstract

The **VTC Badges — Claims — List** Trust Task pages through a badge's claim codes (VTI-BDG-027), newest first, for its owners and administrators: what each is for, until when, its limit, how many have claimed it and whether it was withdrawn. It never answers a code — the community keeps only digests. The awards a code admitted are listed with every other award by [`vtc/badges/awards/list`](../../../awards/list/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse with `notFound` a `badgeId` that names no badge the caller owns or administers, and every request while badges are off.
2. Refuse with `cursorInvalid` a `cursor` it did not issue to this caller for this badge.
3. Answer the badge's claim code records, newest `createdAt` first, at most `limit` (default 50), with `nextCursor` where more remain. Never a code.

## Authorization

**Authority: the badge's owner or a badge administrator — read now**, as [`vtc/badges/claims/create`](../../create/0.1/spec.md) states it.

## Definitions

- `BadgeId`, `ClaimCode` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The caller sends the community maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An owner reads a badge's codes

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000621",
  "type": "https://trusttasks.org/spec/vtc/badges/claims/list/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-08T09:00:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
    "limit": 50
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-08T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/claims/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Answered

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000622",
  "type": "https://trusttasks.org/spec/vtc/badges/claims/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-08T09:00:01Z",
  "threadId": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000621",
  "payload": {
    "items": [
      {
        "claimId": "bcl_T4nQ8mLw2Rx9Kv5Z",
        "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
        "label": "Saturday joinery workshop",
        "validUntil": "2026-11-08T18:00:00Z",
        "limit": 40,
        "claims": 17,
        "state": "active",
        "createdAt": "2026-11-08T09:00:01Z",
        "createdBy": "dir_Pr1yaNatarajan00"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

A badge identifier in; code records out — labels, times, limits, counts and issuers' entries.

### Correlation

Nothing here names a member who claimed a code; the awards do, in the owners' own view.

### Retention

**Transient.** A read.

### Consent/purpose

The owners running their badge's codes.
