---
slug: vtc/badges/claims/redeem
version: '0.1'
title: VTC Badges — Claims — Redeem
summary: "Redeem a claim code: the member who was there is admitted to the badge — the redemption is their consent — with an award whose giver is the code's issuer."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- event
- portal
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The redemption is the member's consent to hold the badge (VTI-BDG-026), and is recorded against them; it must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: Records an accepted award, issues the badge credential and admits the member to the badge group; counts the claim. Reversed by vtc/groups/leave (declining the badge) or an owner's take-back.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: The request carries the code; the response is the award it recorded. The badge credential goes to the member's wallet as every badge credential does.
retention:
  class: durable
  rationale: The award is kept as every award is; the code is not kept.
errorCodes:
- code: vtc/badges/claims/redeem:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was recorded.
  retryable: false
- code: vtc/badges/claims/redeem:codeInvalid
  meaning: The code admits no one — unknown, past its time, out of claims, withdrawn, or for a retired badge. One code for all of them, so a guess learns nothing.
  retryable: false
- code: vtc/badges/claims/redeem:alreadyClaimed
  meaning: The signer has already redeemed this code. Nothing was recorded.
  retryable: false
- code: vtc/badges/claims/redeem:alreadyHeld
  meaning: The badge may be held once, and the signer already holds it. Nothing was recorded and the claim was not counted.
  retryable: false
- code: vtc/badges/claims/redeem:keyHolderUnavailable
  meaning: The community's VTA did not answer the signing request for the badge credential. Nothing was recorded and the claim was not counted; redeem again.
  retryable: true
- code: vtc/badges/claims/redeem:keyHolderRefused
  meaning: The community's VTA refused to sign the badge credential. Its own code and message are carried in `details`. Nothing was recorded.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      code:
        type: string
        maxLength: 128
      message:
        type: string
        maxLength: 500
related:
- vtc/badges/claims/create
- vtc/badges/mine/list
- vtc/badges/mine/accept
- vtc/groups/leave
---

## Abstract

The **VTC Badges — Claims — Redeem** Trust Task is how a member who was at an event claims its badge with the code its host handed out (VTI-BDG-026). The redemption is the member's own act and their consent to hold the badge, so it admits them as an approved request does (VTI-BDG-024): the community issues the badge credential as the badge's group — with the new award among its evidence — then records the award, accepted, with the code's issuer as its giver, and the membership. A member who later does not want the badge leaves its group, which declines it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every request with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `codeInvalid` a code whose digest names no code, or one that is withdrawn, past `validUntil`, has `claims` equal to its `limit`, or whose badge is retired — alike, and without saying which.
3. Refuse with `alreadyClaimed` a second redemption of the same code by the same member, and with `alreadyHeld` a badge that is not repeatable which the signer already holds.
4. Otherwise, issue the badge credential exactly as [`vtc/badges/mine/accept`](../../../mine/accept/0.1/spec.md) does — credential first, as the badge's group, with every accepted award of the badge to the signer and this one as evidence — then record the award `accepted`, its giver the code's issuer, admit the signer to the badge group, count the claim, and record that the signer redeemed the code. Where the key holder does not answer or refuses, record nothing and count nothing (`keyHolderUnavailable`, `keyHolderRefused`).
5. Never count beyond `limit`: two redemptions racing for the last claim admit one.
6. Audit the claim by reference — the badge, the award, the code's record and the member's entry — never the code (VTI-BDG-091).

## Authorization

**Authority: an active member holding the code.** The signer must be an active member of the community; the code is the issuer's permission, carried by whoever received it. This is a holder's act (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it.

## Definitions

- `ClaimSecret`, `Award` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The member sends the community maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member claims the workshop badge

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000641",
  "type": "https://trusttasks.org/spec/vtc/badges/claims/redeem/0.1#request",
  "issuer": "did:example:sam",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-08T09:00:00Z",
  "payload": {
    "code": "Qm7xV2kR9pLw4Nt8Hs3Fz6Bd1Yc5Je0Ua"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:sam#key-1",
    "created": "2026-11-08T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/claims/redeem/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Claimed

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000642",
  "type": "https://trusttasks.org/spec/vtc/badges/claims/redeem/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:sam",
  "issuedAt": "2026-11-08T09:00:01Z",
  "threadId": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000641",
  "payload": {
    "award": {
      "awardId": "awd_C7mT9pQx2Lw8Rv5N",
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "entryId": "dir_SamNguyen0000000",
      "givenBy": "dir_Pr1yaNatarajan00",
      "state": "accepted",
      "offeredAt": "2026-11-08T11:30:00Z",
      "decidedAt": "2026-11-08T11:30:00Z",
      "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-000000000901"
    }
  }
}
```

## Security & Privacy

### Data carried

A code in; the award out. The code is a bearer secret; a producer **MUST NOT** keep it after the response.

### Correlation

The award names the member by their directory entry and the code's issuer as its giver, as any award does. The badge on the member's card shows they hold it, where they choose to show it (VTI-BDG-030).

### Retention

**Durable.** The award is kept as every award is; the community keeps no code, only its digest, and the members who redeemed each code, to refuse a second claim.

### Consent/purpose

A member claiming the badge for something they did — being at the event. The badge confers nothing (VTI-BDG-002).
