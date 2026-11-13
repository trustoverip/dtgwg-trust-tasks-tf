---
slug: vtc/badges/claims/withdraw
version: '0.1'
title: VTC Badges — Claims — Withdraw
summary: "Withdraw a claim code: it admits no one from now on, and the claims already made stand."
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
  rationale: Withdrawing a code stops admissions to a badge; who did it must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: Marks the code withdrawn. Not reversible; a new code is issued with vtc/badges/claims/create.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: The request names a code's record; the response is that record.
retention:
  class: durable
  rationale: The withdrawn record is kept with the badge.
errorCodes:
- code: vtc/badges/claims/withdraw:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing changed.
  retryable: false
- code: vtc/badges/claims/withdraw:notFound
  meaning: No claim code of a badge the signer owns or administers has this identifier.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/claims/create
- vtc/badges/claims/list
---

## Abstract

The **VTC Badges — Claims — Withdraw** Trust Task stops a claim code admitting anyone (VTI-BDG-027): a code that leaked, an event that was cancelled. Claims already made stand — each is its member's award, taken back only with [`vtc/badges/awards/revoke`](../../../awards/revoke/0.1/spec.md). Withdrawing is idempotent: a withdrawn code is answered as it is.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every request with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` a `claimId` that names no code of a badge the signer owns or administers.
3. Mark the code `withdrawn`; from then on [`vtc/badges/claims/redeem`](../../redeem/0.1/spec.md) refuses it as it refuses an unknown code. Change no award.
4. Audit the withdrawal by reference (VTI-BDG-091) and answer the record.

## Authorization

**Authority: the badge's owner or a badge administrator — read now**, as [`vtc/badges/claims/create`](../../create/0.1/spec.md) states it. An owner's act (VTI-BDG-095): a member portal's delegated key **MUST NOT** sign it.

## Definitions

- `ClaimId`, `ClaimCode` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The issuer sends the community maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An owner withdraws a code that was posted publicly

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000631",
  "type": "https://trusttasks.org/spec/vtc/badges/claims/withdraw/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-08T09:00:00Z",
  "payload": {
    "claimId": "bcl_T4nQ8mLw2Rx9Kv5Z"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/claims/withdraw/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Withdrawn

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000632",
  "type": "https://trusttasks.org/spec/vtc/badges/claims/withdraw/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-08T09:00:01Z",
  "threadId": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000631",
  "payload": {
    "claim": {
      "claimId": "bcl_T4nQ8mLw2Rx9Kv5Z",
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "label": "Saturday joinery workshop",
      "validUntil": "2026-11-08T18:00:00Z",
      "limit": 40,
      "claims": 17,
      "state": "withdrawn",
      "createdAt": "2026-11-08T09:00:01Z",
      "createdBy": "dir_Pr1yaNatarajan00"
    }
  }
}
```

## Security & Privacy

### Data carried

A code's record identifier in; the record out.

### Correlation

Nothing names a member who claimed the code.

### Retention

**Durable.** The withdrawn record stays with the badge.

### Consent/purpose

An owner stopping a code that should admit no one more.
