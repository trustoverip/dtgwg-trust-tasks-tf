---
slug: vtc/badges/awards/offer
version: '0.1'
title: VTC Badges — Awards — Offer
summary: 'A badge owner offers a community badge to members, one at a time or a whole group at once: each offer is an award waiting for its recipient, and nothing is admitted or issued until they accept.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- award
- offer
parties:
- role: badge owner
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: An offer is made in the owner's name and, once accepted, becomes a credential in the badge group's name naming the owner as its giver. The giver must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed offer of a repeatable badge would offer it again.
sideEffects:
  level: mutating
  rationale: Records one award per recipient in state `offered`. Nothing is admitted to any group and no credential is issued; an offer is withdrawn with vtc/badges/awards/withdraw or declined by its recipient.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: The request names a badge, recipients and a note; the response is the award records and which recipients were skipped and why.
retention:
  class: durable
  rationale: An award is kept in every state as part of the badge's and the holder's history (VTI-BDG-005).
errorCodes:
- code: vtc/badges/awards/offer:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was recorded.
  retryable: false
- code: vtc/badges/awards/offer:badgeRetired
  meaning: The badge is retired and is never given again (VTI-BDG-005).
  retryable: false
- code: vtc/badges/awards/offer:kindMismatch
  meaning: The badge is a kudos badge. Kudos are given by any member with vtc/badges/kudos/give, not offered by an owner.
  retryable: false
- code: vtc/badges/awards/offer:automaticBadge
  meaning: The badge is automatic — its badge group is dynamic — and is given by its rule, never by hand (VTI-BDG-013).
  retryable: false
- code: vtc/badges/awards/offer:groupNotVisible
  meaning: '`group` names a group the signer may not see, or whose member list the signer may not see. Nothing was recorded.'
  retryable: false
- code: vtc/badges/awards/offer:nothingToOffer
  meaning: Every recipient was skipped. The reasons are in `details.skipped`, shaped as the response's `skipped`.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      skipped:
        type: array
        maxItems: 2000
        items:
          type: object
          additionalProperties: false
          properties:
            entryId:
              type: string
              maxLength: 128
            reason:
              type: string
              maxLength: 64
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/create
- vtc/badges/awards/withdraw
- vtc/badges/awards/revoke
- vtc/badges/awards/list
- vtc/badges/mine/accept
- vtc/badges/mine/decline
- vtc/badges/kudos/give
- vtc/groups/join
---

## Abstract

The **VTC Badges — Awards — Offer** Trust Task is how a community badge is given: a **badge owner** — a manager of the badge's group — names the members, or a group of members, and the community records one **award** per recipient in state `offered`. The award is a record of the badge's own, not a membership: nothing is admitted to the badge group, no credential is issued and nothing appears on any card until the recipient accepts it with [`vtc/badges/mine/accept`](../../../mine/accept/0.1/spec.md) (VTI-BDG-020, VTI-BDG-021). A recipient who declines ([`vtc/badges/mine/decline`](../../../mine/decline/0.1/spec.md)) leaves no trace in the group, and the owner may withdraw an offer before it is answered ([`vtc/badges/awards/withdraw`](../../withdraw/0.1/spec.md)).

A badge one member may hold more than once (`repeatable`) may be offered to a holder again; each giving is an award of its own and the holder's count is the number they accepted (VTI-BDG-022). Kudos badges are not offered here: any member gives them with [`vtc/badges/kudos/give`](../../../kudos/give/0.1/spec.md). An automatic badge is given by its rule, never by hand.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse with `badgesOff` while its badge settings have `enabled` false.
2. Answer `notFound` for a badge the signer may not see, and `permissionDenied` for a badge the signer may see but does not own — one whose badge group the signer does not manage (`vtc.groups.manage` unqualified, or qualified to that group). Refuse a kudos badge with `kindMismatch`, a retired badge with `badgeRetired`, and an automatic badge with `automaticBadge`.
3. Resolve the recipients: the entries in `to`, or every active member of `group` whose member list the signer may see (`groupNotVisible` otherwise), resolved **once, now**. Skip, never refuse, a recipient who is not an active member the directory shows the signer (`notActiveMember`), who already has an open offer of this badge (`alreadyOffered`), who holds the badge when it is not `repeatable` (`alreadyHolder`), or who has a pending request for it (`requestPending`: the owners decide the request with [`vtc/groups/requests/decide`](../../../../groups/requests/decide/0.1/spec.md) instead). Refuse with `nothingToOffer` only when every recipient was skipped.
4. Record one award per remaining recipient: state `offered`, `givenBy` the signer's entry, the `note`, `offeredAt` now, and `expiresAt` where the settings' `limits.offerExpiresAfterDays` is set. It **MUST NOT** admit anyone to the badge group, issue or renew any credential, or change any card (VTI-BDG-020).
5. Let each recipient learn of the offer in their portal ([`vtc/badges/mine/list`](../../../mine/list/0.1/spec.md)); a community **MAY** also notify them through the member push it uses for a credential offer, and **MUST NOT** treat an undelivered notification as a failed offer.
6. Expire an unanswered offer at `expiresAt` (state `expired`), recording nothing else.
7. Audit the offers by reference — the badge, the signer and each recipient's entry — never the note.

## Authorization

**Authority: management of the badge group** — `vtc.groups.manage` at the community, unqualified or qualified `group:<the badge's groupId>`, read from the signer's access-control entry at execution time (VTI-BDG-011). An owner's appointment is a delegated grant ([`vtc/groups/managers/add`](../../../../groups/managers/add/0.1/spec.md)); every `community-admin` holds the capability unqualified and may give any badge. `vtc.badges.admin` alone does not make an owner. An offer confers nothing: a badge is never read by an authorization decision (VTI-BDG-002). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

An offer is the owner's own act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-BDG-095).

## Definitions

- `Badge`, `Award`, `AwardState`, `AwardNote` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- **Owner** — a member who manages the badge's group.

## Request

The badge owner (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Two members who hosted a workshop

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000201",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/offer/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T18:00:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
    "to": ["dir_NoorElSayed00000", "dir_LenaWirth0000000"],
    "note": "Resin pour workshop, 25 October 2026"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/awards/offer/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### One offered, one skipped

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000202",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/offer/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T18:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000201",
  "payload": {
    "awards": [
      {
        "awardId": "awd_Qz7TpLW5k2N8xRb3",
        "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
        "entryId": "dir_NoorElSayed00000",
        "givenBy": "dir_Pr1yaNatarajan00",
        "note": "Resin pour workshop, 25 October 2026",
        "state": "offered",
        "offeredAt": "2026-11-06T18:00:01Z",
        "expiresAt": "2026-12-06T18:00:01Z"
      }
    ],
    "skipped": [
      { "entryId": "dir_LenaWirth0000000", "reason": "alreadyOffered" }
    ]
  }
}
```

### Refused: a kudos badge

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000203",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T18:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000204",
  "payload": {
    "code": "vtc/badges/awards/offer:kindMismatch",
    "message": "Clamp whisperer is a kudos badge: give it with vtc/badges/kudos/give.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A badge, recipients and a note in; award records and skip reasons out. The note is shown to its recipient, the badge's owners and administrators, and written as the evidence narrative of the credential the recipient's acceptance issues: a producer **MUST NOT** put in it anything the recipient should not carry in a credential they may present.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. A skip reason tells an owner that a member already holds, was offered, or asked for the badge; owners see that about their own badge in any case ([`vtc/badges/awards/list`](../../list/0.1/spec.md)).

### Retention

**Durable.** Awards are kept in every state.

### Consent/purpose

Recognising a member for something they did. **The recipient's consent is the acceptance** (VTI-BDG-020): the badge goes on their card, and a credential into their wallet, only when they say so. A badge confers nothing, and the community **MUST NOT** read an award in any authorization decision of its own (VTI-BDG-002).
