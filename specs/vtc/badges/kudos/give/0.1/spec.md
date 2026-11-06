---
slug: vtc/badges/kudos/give
version: '0.1'
title: VTC Badges — Kudos — Give
summary: "Give a kudos badge to another member: records an award offered to them, with the giver and a note, within the community's limits. The recipient accepts or declines it; nothing is issued."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- kudos
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
  rationale: A kudos badge is given in the giver's name and counted against the giver's limits; the giving must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed giving would record a second award and spend the giver's daily allowance again.
sideEffects:
  level: mutating
  rationale: Records one award in state `offered`, counted against the giver's limits. Closed by the recipient's decision, the giver's withdrawal or expiry; never recalled once accepted except by an owner's or administrator's revocation.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: The request carries a note the giver writes about the recipient, shown to the recipient, the giver and badge administrators; the response is the award record. No credential and no key.
retention:
  class: durable
  rationale: An award persists as both members' history; a departed giver's givings stand by entry reference so the recipient's count holds (VTI-BDG-090).
errorCodes:
- code: vtc/badges/kudos/give:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was recorded.
  retryable: false
- code: vtc/badges/kudos/give:kindMismatch
  meaning: '`badgeId` names a community badge. Those are given by the badge''s owners with vtc/badges/awards/offer, never here.'
  retryable: false
- code: vtc/badges/kudos/give:selfGiving
  meaning: '`to` is the signer''s own entry (VTI-BDG-040). Nothing was recorded.'
  retryable: false
- code: vtc/badges/kudos/give:notEligible
  meaning: The signer's membership is younger than the community's `limits.minTenureDays` (VTI-BDG-040). `details.eligibleFrom` says when they may give.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      eligibleFrom:
        type: string
        format: date-time
- code: vtc/badges/kudos/give:limitReached
  meaning: 'A limit of VTI-BDG-040 is reached: `details.limit` is `kudosPerGiverPerDay` (the signer has given today as many kudos as the community allows) or `sameGiverSameMember` (the signer gave `to` a kudos badge within the bound). `details.until` says when the bound lifts. Nothing was recorded.'
  retryable: true
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - limit
    properties:
      limit:
        type: string
        enum:
        - kudosPerGiverPerDay
        - sameGiverSameMember
      until:
        type: string
        format: date-time
related:
- vtc/badges/kudos/list
- vtc/badges/mine/accept
- vtc/badges/mine/decline
- vtc/badges/awards/offer
- vtc/badges/create
- vtc/badges/list
---

## Abstract

The **VTC Badges — Kudos — Give** Trust Task is one member thanking another (VTI Community Badges chapter, VTI-BDG-040, VTI-BDG-041). Any active member gives any kudos badge the community lists — made with [`vtc/badges/create`](../../../create/0.1/spec.md), found with [`vtc/badges/list`](../../../list/0.1/spec.md) — to any other active member, never to themselves, with a note, within the limits the community states: so many a day, the same pair not too often, and not before the giver has been a member for a while.

Each giving is an award of its own, recorded `offered` with `givenBy` the signer. The recipient is told in their portal and accepts it with [`vtc/badges/mine/accept`](../../../mine/accept/0.1/spec.md) or declines it with [`vtc/badges/mine/decline`](../../../mine/decline/0.1/spec.md) (VTI-BDG-020, VTI-BDG-021). Nothing is issued: a kudos badge carries no credential and has no badge group. A community badge is given by its owners with [`vtc/badges/awards/offer`](../../../awards/offer/0.1/spec.md), not here. What was given and received is listed with [`vtc/badges/kudos/list`](../../list/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every giving with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004). Refuse a signer who is not an active member with `permissionDenied`.
2. Refuse with `notFound` a `badgeId` that is not an active badge the signer may see — listed, or held by them — and a `to` that is not an active member whose card the directory shows the signer (VTI-DIR-020). A retired badge, an unlisted badge the signer does not hold, a departed member and an entry that does not exist answer alike.
3. Refuse with `kindMismatch` a badge whose `kind` is `community` (VTI-BDG-041): its owners give it with [`vtc/badges/awards/offer`](../../../awards/offer/0.1/spec.md).
4. Refuse with `selfGiving` a `to` that is the signer's own entry (VTI-BDG-040).
5. Refuse with `notEligible` a signer whose membership is younger than the settings' `limits.minTenureDays` — 7 where the community states none — naming in `details.eligibleFrom` when they may give (VTI-BDG-040).
6. Refuse with `limitReached` a signer who has given today as many kudos as `limits.kudosPerGiverPerDay` allows (`details.limit` `kudosPerGiverPerDay`), or who gave `to` any kudos badge within `limits.sameGiverSameMember` (`details.limit` `sameGiverSameMember`) — 10 a day and once a day where the community states none (VTI-BDG-040). Every giving the signer made counts, whatever its later state: a declined or withdrawn giving is not an allowance returned. `details.until` **SHOULD** name when the bound lifts.
7. Record an award in state `offered` for `badgeId` to `to`, with `givenBy` the signer, `offeredAt` now, `note` as sent, and `expiresAt` where the community expires offers (`limits.offerExpiresAfterDays`); and **MUST NOT** admit anyone to any group, issue anything, or show the badge on the recipient's card until they accept (VTI-BDG-020, VTI-BDG-041). The recipient is told in their portal and finds the offer in [`vtc/badges/mine/list`](../../../mine/list/0.1/spec.md).
8. Answer the award. The `note` is shown to the recipient, the giver and holders of `vtc.badges.admin`, never to other members (VTI-BDG-042).
9. Audit the giving by reference — the award, the badge, the giver and the recipient — and **MUST NOT** record the note in the audit trail (VTI-BDG-091).

## Authorization

**Authority: being an active member within the community's limits.** No capability is read: every active member may give, bounded only by the settings' `limits` and the signer's tenure (VTI-BDG-040). Giving confers nothing on the recipient or the giver (VTI-BDG-002). The proof establishes who gave, which is what the limits are counted against ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms.

## Definitions

- `BadgeId`, `Award`, `AwardNote`, `BadgeLimits` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- `EntryId` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Thank a member with a kudos badge

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000351",
  "type": "https://trusttasks.org/spec/vtc/badges/kudos/give/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:30:00Z",
  "payload": {
    "badgeId": "bdg_Kd0sCl4mpWh1sp3r",
    "to": "dir_B3nOk4f0rRq9Wz2X",
    "note": "Found the long sash clamps behind the bandsaw. Saved the glue-up."
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/kudos/give/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Thank a member with a kudos badge — answered

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000352",
  "type": "https://trusttasks.org/spec/vtc/badges/kudos/give/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:30:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000351",
  "payload": {
    "award": {
      "awardId": "awd_K3m8Qp2Vt7Xz4Ln9",
      "badgeId": "bdg_Kd0sCl4mpWh1sp3r",
      "entryId": "dir_B3nOk4f0rRq9Wz2X",
      "givenBy": "dir_Tm4sV3g4K9xQ2bLp",
      "note": "Found the long sash clamps behind the bandsaw. Saved the glue-up.",
      "state": "offered",
      "offeredAt": "2026-11-06T10:30:01Z",
      "expiresAt": "2026-12-06T10:30:01Z"
    }
  }
}
```

### Refused: the same pair, twice in a day

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000353",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:30:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000351",
  "payload": {
    "code": "vtc/badges/kudos/give:limitReached",
    "message": "You already gave this member a kudos badge today; try again tomorrow.",
    "retryable": true,
    "details": { "limit": "sameGiverSameMember", "until": "2026-11-07T00:00:00Z" }
  }
}
```

## Security & Privacy

### Data carried

A badge identifier, the recipient's entry identifier and a note in; the award record out. The note is personal data about the recipient written by the giver: a producer **MUST NOT** put in `note` anything the recipient would not want shown to badge administrators, and **MUST NOT** place personal data in `ext`. No credential and no key: a kudos badge carries none (VTI-BDG-041).

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Who gave what to whom is seen by the two of them and by holders of `vtc.badges.admin` (VTI-BDG-042); every other member sees at most the recipient's count, and only where the recipient shows the badge (VTI-BDG-030, VTI-BDG-043). The limits are counted per giver, so the pattern of a member's givings is held by the community and shown to nobody but administrators.

### Retention

**Durable.** The award persists as both members' history and as the recipient's count; a giver's departure leaves it standing by entry reference, and their erasure removes the note (VTI-BDG-090). Retiring the badge keeps its awards (VTI-BDG-005). The audit record names the award, the badge, the giver and the recipient, never the note (VTI-BDG-091).

### Consent/purpose

One member recognising another, as the community's badge settings allow; the recipient's acceptance is their consent to show it (VTI-BDG-020). **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and a kudos count is a count, not a standing.
