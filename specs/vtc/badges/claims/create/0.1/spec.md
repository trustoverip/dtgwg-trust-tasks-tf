---
slug: vtc/badges/claims/create
version: '0.1'
title: VTC Badges — Claims — Create
summary: "Issue a claim code for a community badge — for handing out at an event: the community generates the code, answers it once and keeps only its digest."
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
  rationale: A claim code admits members to a badge; who issued it is the giver of every award it admits, and must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: Records a claim code (its digest, label, time and limit). Reversed with vtc/badges/claims/withdraw.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: The request names a badge and the code's label, time and limit; the response is the code's record and the code itself, once — a bearer secret the issuer hands out.
retention:
  class: durable
  rationale: The code's record (never the code) is kept while the badge is, for its owners' view of who it admitted.
errorCodes:
- code: vtc/badges/claims/create:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was recorded.
  retryable: false
- code: vtc/badges/claims/create:notFound
  meaning: No badge the signer owns or administers has this identifier. Conflates an unknown badge, one the signer may not see, and one they may see but do not own.
  retryable: false
- code: vtc/badges/claims/create:notClaimable
  meaning: The badge is the signer's to run, but no code may admit anyone to it — a kudos badge, given by members one by one, or a retired badge. `details.reason` says which.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      reason:
        type: string
        enum:
        - kudos
        - retired
- code: vtc/badges/claims/create:settingsInvalid
  meaning: '`validUntil` is not in the future or is more than 90 days ahead. Nothing was recorded.'
  retryable: false
related:
- vtc/badges/claims/list
- vtc/badges/claims/withdraw
- vtc/badges/claims/redeem
- vtc/badges/awards/offer
---

## Abstract

The **VTC Badges — Claims — Create** Trust Task issues a **claim code** for a community badge (VTI Community Badges chapter, VTI-BDG-025): a secret an event's host hands out — on a screen, on a card — that any member who was there redeems with [`vtc/badges/claims/redeem`](../../redeem/0.1/spec.md) to hold the badge. The issuer states what the code is for (`label`), when it stops admitting anyone (`validUntil`) and how many claims it admits (`limit`). The community generates the code, answers it in this response only, and keeps nothing of it but a digest.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every request with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` a `badgeId` that names no badge the signer owns — a manager of its badge group — or administers (`vtc.badges.admin`).
3. Refuse with `notClaimable` a kudos badge (`details.reason` `kudos`) and a retired one (`retired`).
4. Refuse with `settingsInvalid` a `validUntil` that is not after now, or more than 90 days after it.
5. Generate the code from at least 128 random bits, never derived from anything, and keep only a digest of it, the label, the time, the limit, a count of claims at 0, the issuer's entry and when. Answer the record and the code. A replay of the request **MUST NOT** issue a second code.
6. Audit the issue by reference — the badge and the code's record — never the code (VTI-BDG-091).

## Authorization

**Authority: the badge's owner or a badge administrator — read now.** The signer must manage the badge's group (`vtc.groups.manage` qualified by the group) or hold `vtc.badges.admin`, read at execution time. This is an owner's act (VTI-BDG-095): a member portal's delegated key **MUST NOT** sign it.

## Definitions

- `BadgeId`, `ClaimCode`, `ClaimSecret` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The issuer sends the community maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A host issues a code for a workshop

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000611",
  "type": "https://trusttasks.org/spec/vtc/badges/claims/create/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-08T09:00:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
    "label": "Saturday joinery workshop",
    "validUntil": "2026-11-08T18:00:00Z",
    "limit": 40
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/claims/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Issued: the code, this once

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000612",
  "type": "https://trusttasks.org/spec/vtc/badges/claims/create/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-08T09:00:01Z",
  "threadId": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000611",
  "payload": {
    "claim": {
      "claimId": "bcl_T4nQ8mLw2Rx9Kv5Z",
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "label": "Saturday joinery workshop",
      "validUntil": "2026-11-08T18:00:00Z",
      "limit": 40,
      "claims": 0,
      "state": "active",
      "createdAt": "2026-11-08T09:00:01Z",
      "createdBy": "dir_Pr1yaNatarajan00"
    },
    "code": "Qm7xV2kR9pLw4Nt8Hs3Fz6Bd1Yc5Je0Ua"
  }
}
```

## Security & Privacy

### Data carried

A badge identifier, a label, a time and a limit in; the code's record and the code out. The code is a bearer secret: whoever holds it may claim the badge until its time or its limit. A producer **SHOULD** show it to the issuer with that warning and **MUST NOT** keep it.

### Correlation

The community maintainer's identifier scope is public. The code names nothing and is derived from nothing, so codes of two events share nothing a reader could walk.

### Retention

**Durable.** The record — never the code — is kept with the badge for its owners' view of who it admitted.

### Consent/purpose

An event's host letting those who came claim the badge for coming. The code confers nothing (VTI-BDG-002); every claim is its redeemer's own act.
