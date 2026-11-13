---
slug: vtc/badges/mine/decline
version: '0.1'
title: VTC Badges — Mine — Decline
summary: 'Decline a badge offered to the signer: the offer is closed, nothing is recorded in any group and nothing is issued; the same badge may be offered again later.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
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
  rationale: A badge on a card is a statement the member makes in their own name; declining one is theirs alone to decide and must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed decline would close an offer made afresh after the member had changed their mind.
sideEffects:
  level: mutating
  rationale: Closes one offer to the signer as `declined`. Recoverable only by a new offer of the same badge.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: The member decides on an award that names them; the response is that award, closed.
retention:
  class: durable
  rationale: The declined award is kept as the member's own history and so that the badge's owners see the offer closed (VTI-BDG-021).
errorCodes:
- code: vtc/badges/mine/decline:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was changed.
  retryable: false
- code: vtc/badges/mine/decline:alreadyDecided
  meaning: The award names the signer but is no longer `offered` — it was accepted, declined, withdrawn, expired or revoked. Its state is in `details.state`. Nothing was changed.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      state:
        type: string
        maxLength: 32
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/mine/accept
- vtc/badges/mine/list
- vtc/badges/awards/offer
- vtc/badges/awards/withdraw
- vtc/badges/kudos/give
---

## Abstract

The **VTC Badges — Mine — Decline** Trust Task is the member's "no" to a badge (VTI Community Badges chapter). An award is an offer until its recipient decides (VTI-BDG-020): an owner's offer of a community badge made with [`vtc/badges/awards/offer`](../../../awards/offer/0.1/spec.md), or a member's giving of a kudos badge made with [`vtc/badges/kudos/give`](../../../kudos/give/0.1/spec.md). The signer finds their open offers in [`vtc/badges/mine/list`](../../list/0.1/spec.md) and takes one with [`vtc/badges/mine/accept`](../../accept/0.1/spec.md) or declines it here.

Declining records the award `declined` and nothing else: no membership in the badge group, no credential, nothing on the wall (VTI-BDG-021). The badge's owners — or, for a kudos badge, its giver — see the offer closed; nobody else does. A declined badge may be offered to the same member again.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every decline with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` an `awardId` that does not name an award whose `entryId` is the signer. An award that does not exist and an award made to another member answer alike: the task says nothing about awards that are not the signer's.
3. Refuse with `alreadyDecided` an award of the signer's whose state is not `offered`, naming its state in `details.state`. An offer the giver withdrew or that expired before the decline arrived is `alreadyDecided`, not `notFound`.
4. Record the award `declined` with `decidedAt`, and **MUST** record nothing in the badge group, issue nothing, and change nothing on the signer's wall (VTI-BDG-021). A kudos giving declined is closed the same way: the giving stands as a record, counted against its giver's limits, and the recipient's count does not move (VTI-BDG-041).
5. Show the closed offer to the badge's owners and to holders of `vtc.badges.admin` — and, for a kudos badge, to its giver in their own list of kudos given (VTI-BDG-042) — and to nobody else (VTI-BDG-021).
6. **MAY** accept a later offer of the same badge to the same member (VTI-BDG-021): a decline is not a block, and a badge that is not repeatable may still be offered again after a decline, since nothing was held.
7. Audit the decline by reference — the award, the badge and the signer — and **MUST NOT** record the award's note in the audit trail (VTI-BDG-091).

## Authorization

**Authority: being the award's recipient.** The task has no target member: the signer closes only an offer that names them. Declining confers and removes nothing: no authority of the signer's or anyone's changes (VTI-BDG-002). The proof establishes who decided ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms.

## Definitions

- `AwardId`, `Award`, `AwardState` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Decline an offered badge

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000311",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/decline/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:05:00Z",
  "payload": {
    "awardId": "awd_H7q2Lm9Xc4Vt1Bz8"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/mine/decline/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Decline an offered badge — answered

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000312",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/decline/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:05:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000311",
  "payload": {
    "award": {
      "awardId": "awd_H7q2Lm9Xc4Vt1Bz8",
      "badgeId": "bdg_Yr1Ab0ardTn2Qx7V",
      "entryId": "dir_Tm4sV3g4K9xQ2bLp",
      "givenBy": "dir_B3nOk4f0rRq9Wz2X",
      "note": "A year of Saturdays. Thank you.",
      "state": "declined",
      "offeredAt": "2026-11-05T20:00:00Z",
      "decidedAt": "2026-11-06T10:05:01Z"
    }
  }
}
```

### Refused: the offer was already withdrawn

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000313",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:05:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000311",
  "payload": {
    "code": "vtc/badges/mine/decline:alreadyDecided",
    "message": "The offer was withdrawn by its giver on 2026-11-06.",
    "retryable": false,
    "details": { "state": "withdrawn" }
  }
}
```

## Security & Privacy

### Data carried

An award identifier in; the closed award out, with the giver's `entryId` and the note they wrote for the signer. Nothing is issued and no credential or key appears. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The award identifier is random and names one offer to one member; it links the request to that offer and to nothing else. Who declined what is seen by the badge's owners and the kudos giver, who already knew whom they offered it to, and by nobody else (VTI-BDG-021).

### Retention

**Durable.** The declined award is kept with the member's other awards, for their own history and so that the offer reads as closed to those who made it; retiring the badge keeps it still (VTI-BDG-005). Erasure removes the note its giver wrote (VTI-BDG-090). The audit record names the award, the badge and the signer, never the note (VTI-BDG-091).

### Consent/purpose

The member's own decision about what is said of them on their card: nobody holds a badge they did not accept (VTI-BDG-020). **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002); declining one changes no authority.
