---
slug: vtc/badges/awards/withdraw
version: '0.1'
title: VTC Badges — Awards — Withdraw
summary: Withdraw an open offer of a badge before its recipient decides — by a badge's owner or a badge administrator, or by the giver of a kudos award.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- award
parties:
- role: badge owner, administrator or kudos giver
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: An offer is closed in the name of the badge's owners, or of the member who gave it; the closing must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed withdraw is refused as already decided, but bounding it keeps a captured document from closing an offer made again later.
sideEffects:
  level: mutating
  rationale: Closes one offer as `withdrawn`. Nothing is recorded in any group and nothing is revoked. Recoverable by offering the badge again.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: The request names an award and a reason; the response is the award record, closed.
retention:
  class: durable
  rationale: A withdrawn award is kept, closed, for the badge's owners and administrators to see what was offered and taken back.
errorCodes:
- code: vtc/badges/awards/withdraw:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing changed.
  retryable: false
- code: vtc/badges/awards/withdraw:notFound
  meaning: No award the signer may withdraw has this identifier. Conflates an unknown award, an award of a badge the signer may not see, and an award that is not theirs to withdraw — not an owner's or administrator's, nor, for a kudos award, its giver's.
  retryable: false
- code: vtc/badges/awards/withdraw:alreadyDecided
  meaning: The award is not `offered` — the recipient accepted or declined it, it expired, or it was already withdrawn. Its state is in `details.state`. Nothing changed.
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
- vtc/badges/awards/offer
- vtc/badges/awards/revoke
- vtc/badges/awards/list
- vtc/badges/kudos/give
- vtc/badges/mine/decline
---

## Abstract

The **VTC Badges — Awards — Withdraw** Trust Task closes an offer of a badge before its recipient has answered it. An award is an offer until accepted (VTI-BDG-020): a badge's owner may withdraw one they or another owner made with [`vtc/badges/awards/offer`](../../offer/0.1/spec.md), a holder of `vtc.badges.admin` may withdraw any, and the giver of a kudos award ([`vtc/badges/kudos/give`](../../../kudos/give/0.1/spec.md)) may withdraw their own giving.

Withdrawing records nothing in any group and revokes nothing: there was nothing to revoke. The award is kept as `withdrawn`, shown as closed to the badge's owners and administrators, and the recipient's portal stops showing the offer. The badge may be offered to the same member again. An award the recipient has already accepted is taken back with [`vtc/badges/awards/revoke`](../../revoke/0.1/spec.md), which is a different act with different consequences.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every withdraw with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` an `awardId` that names no award, an award of a badge the signer may not see, and an award the signer may not withdraw. The signer may withdraw an award of a community badge when they own it — manage its badge group, by `vtc.groups.manage` unqualified or qualified `group:<groupId>` — or hold `vtc.badges.admin`; and an award of a kudos badge when they hold `vtc.badges.admin` or are its `givenBy`. The three cases answer alike, so a withdraw does not tell a stranger that an award exists.
3. Refuse with `alreadyDecided` an award whose `state` is not `offered`, naming its state in `details.state`.
4. Set the award `withdrawn` with `decidedAt` now, and record nothing in any group and issue nothing (VTI-BDG-020, VTI-BDG-021). A withdrawn offer **MUST NOT** appear in the recipient's own list ([`vtc/badges/mine/list`](../../../mine/list/0.1/spec.md)) and **MUST** be shown as closed to the badge's owners and to holders of `vtc.badges.admin` ([`vtc/badges/awards/list`](../../list/0.1/spec.md)), and to the giver of a kudos award, and to nobody else.
5. Accept a later offer of the same badge to the same member, as it would after a decline.
6. Record `reason` with the change and audit the withdraw by reference — the award, the badge, the recipient's entry and the signer, never a note or a picture (VTI-BDG-091).

## Authorization

**Authority: ownership of the badge, or administration of badges, or authorship of the giving — read now.** An owner withdraws because the badge's owners answer for whom it is offered to, and ownership is the badge group's management (VTI-BDG-011), read from the signer's access-control entry at execution time; a holder of `vtc.badges.admin` withdraws because they run the community's badges; the giver of a kudos award withdraws because the giving was their own. Withdrawing an offer confers nothing and removes nothing: no membership and no credential existed yet (VTI-BDG-002, VTI-BDG-020). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A kudos giver's withdraw of their own giving is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) **MAY** sign it. An owner's or administrator's withdraw **MUST NOT** be accepted from such a key.

## Definitions

- `AwardId`, `Award`, `AwardState` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The badge owner, administrator or kudos giver (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An owner withdraws an offer made to the wrong member

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000401",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/withdraw/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T11:00:00Z",
  "payload": {
    "awardId": "awd_K3mT9pQx2Lw8Rv5N",
    "reason": "Offered to the wrong Sam; the host was Sam Ortiz."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-06T11:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/awards/withdraw/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Withdrawn

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000402",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/withdraw/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T11:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000401",
  "payload": {
    "award": {
      "awardId": "awd_K3mT9pQx2Lw8Rv5N",
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "entryId": "dir_SamNguyen0000000",
      "givenBy": "dir_Pr1yaNatarajan00",
      "note": "For the Saturday joinery workshop.",
      "state": "withdrawn",
      "offeredAt": "2026-11-05T18:00:00Z",
      "decidedAt": "2026-11-06T11:00:01Z"
    }
  }
}
```

### Refused: the recipient already accepted

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000403",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T11:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000401",
  "payload": {
    "code": "vtc/badges/awards/withdraw:alreadyDecided",
    "message": "The award was accepted on 2026-11-06T08:12:40Z; take it back with vtc/badges/awards/revoke.",
    "retryable": false,
    "details": { "state": "accepted" }
  }
}
```

## Security & Privacy

### Data carried

An award identifier and a reason in; the award record out, which names the recipient and the giver by entry and carries the giver's note. A producer **MUST NOT** place personal data about the recipient in `reason` or `ext`: the reason is kept with the change for the badge's owners and administrators, not shown to the recipient.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. An award identifier is random and names one giving only; a withdrawn award is seen by the badge's owners, administrators and the kudos giver, who already knew of the offer, and by nobody else.

### Retention

**Durable.** The award is kept as `withdrawn`, with its `decidedAt` and the reason, for as long as the badge's awards are kept (VTI-BDG-005). The giver's note is erased with its author (VTI-BDG-090).

### Consent/purpose

The badge's owners, or a kudos giver, correcting an offer before anyone acted on it. An offer is a statement the giver makes; withdrawing it is theirs to do until the recipient has made it their own by accepting (VTI-BDG-020). **A badge confers nothing**: the community **MUST NOT** read a badge or an award in any authorization decision of its own (VTI-BDG-002).
