---
slug: vtc/badges/mine/list
version: '0.1'
title: VTC Badges — Mine — List
summary: "A member's own badges: every badge they hold, hidden ones included, in their own order, with the offers waiting for their decision, their wall visibility and their kudos total."
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
  rationale: Everything answered is the signer's own — what they hold, what they hid, what they were offered and by whom. Without an attributable caller there is nothing to answer.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Not consequential, so SPEC §7.3 item 17 sets no floor; REQUIRED because a delegated portal key signs for one session (VTI-BDG-095) and the community places the document in that session by its `issuedAt`.
sideEffects:
  level: none
  rationale: Reads the signer's awards, their wall settings and the offers that name them; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses, to the signer alone, the badges they hold including those they hid, the credentials that evidence them, the offers waiting for them with each giver as the directory projects them, and their kudos total.
retention:
  class: transient
  rationale: A read.
related:
- vtc/badges/mine/accept
- vtc/badges/mine/decline
- vtc/badges/mine/hide
- vtc/badges/mine/arrange
- vtc/badges/wall/show
- vtc/badges/kudos/list
---

## Abstract

The **VTC Badges — Mine — List** Trust Task reads the signer's own badges as their portal shows them (VTI Community Badges chapter): every badge they hold, as a `MyBadge` — how many times, since when, pinned or not, hidden or not, their own position for it, and the credential that evidences it — in the order they chose; every open offer waiting for their decision, each with the badge it offers and the giver as the directory projects them to the signer; who may see their wall; and, where the community shows kudos totals, their total.

There is no `entryId`: a member reads only their own. Another member's wall is read with [`vtc/badges/wall/show`](../../../wall/show/0.1/spec.md), a badge's holders with [`vtc/badges/holders/list`](../../../holders/list/0.1/spec.md). An offer listed here is decided with [`vtc/badges/mine/accept`](../../accept/0.1/spec.md) or [`vtc/badges/mine/decline`](../../decline/0.1/spec.md); the wall is arranged with [`vtc/badges/mine/arrange`](../../arrange/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a signer who is not an active member with `permissionDenied`. While its badge settings have `enabled` false, answer as if the signer held no badge and had no offer — `badges` and `offers` empty, `kudosTotal` absent — and change nothing (VTI-BDG-004).
2. Return in `badges` one entry for every badge the signer holds — at least one accepted award — including the badges they hid, marked `hidden: true` (VTI-BDG-030), and the retired badges they hold (VTI-BDG-005); each with `count` the number of awards of it they accepted (VTI-BDG-022), `position` and `pinned` as they arranged them (VTI-BDG-032), and `credentialId` and `credentialValidUntil` where the badge carries a credential (VTI-BDG-060). The entries are in the signer's own order, lowest `position` first.
3. Return in `offers` every award in state `offered` whose `entryId` is the signer — a community badge an owner offered, or a kudos badge a member gave (VTI-BDG-020, VTI-BDG-041) — each with the badge it is for, which the signer may see for that reason alone, and `givenBy` the giver's directory fields projected exactly as [`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md) would project them for the signer; absent where the signer may see none of the giver or a rule made the award. The award's `note` is returned: it is written for the recipient.
4. Return `visibility` as the signer set it, `members` where they never did (VTI-BDG-031), and `kudosTotal` only where the settings' `showKudosTotals` is true, counted over the kudos awards the signer accepted and did not hide (VTI-BDG-043).
5. Answer the signer alone, and **MUST NOT** answer another member's badges, hidden set, offers or credentials to anyone through this task.

## Authorization

**Authority: being the member.** The task has no target member: the signer reads only what they hold and what was offered to them. The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and what is answered follows from it alone.

This is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms.

## Definitions

- `MyBadge`, `Award`, `Badge`, `WallVisibility` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- `EntryFields` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json). The payload names nothing: the signer is the subject.

### My badges

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000301",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/list/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:00:00Z",
  "payload": {}
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/mine/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### My badges — answered: two held, one of them hidden, one offer waiting

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000302",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:00:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000301",
  "payload": {
    "badges": [
      {
        "badge": {
          "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
          "kind": "community",
          "name": "Workshop host",
          "description": "Ran a workshop for the community.",
          "criteria": "Hosted any workshop on the community calendar. A badge owner gives it after the session.",
          "image": { "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08", "mediaType": "image/png", "size": 61440 },
          "repeatable": true,
          "requestable": true,
          "listed": true,
          "shareableOutside": true,
          "groupId": "grp_Hs7Qm2Lk9Tn4Bv1X",
          "state": "active",
          "createdAt": "2026-09-01T09:00:02Z"
        },
        "count": 2,
        "since": "2026-09-14T18:00:00Z",
        "latestAt": "2026-10-19T18:30:00Z",
        "pinned": true,
        "hidden": false,
        "position": 0,
        "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d",
        "credentialValidUntil": "2028-10-19T18:30:00Z"
      },
      {
        "badge": {
          "badgeId": "bdg_Kd0sCl4mpWh1sp3r",
          "kind": "kudos",
          "name": "Clamp whisperer",
          "description": "Found the good clamps when nobody else could.",
          "image": { "sha256": "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae", "mediaType": "image/png", "size": 24576 },
          "repeatable": true,
          "listed": true,
          "state": "active",
          "createdAt": "2026-10-02T09:05:00Z",
          "createdBy": "dir_Pr1yaN4t7Rj2Qw8K"
        },
        "count": 3,
        "since": "2026-10-03T12:00:00Z",
        "latestAt": "2026-11-01T16:20:00Z",
        "pinned": false,
        "hidden": true,
        "position": 1
      }
    ],
    "offers": [
      {
        "award": {
          "awardId": "awd_H7q2Lm9Xc4Vt1Bz8",
          "badgeId": "bdg_Yr1Ab0ardTn2Qx7V",
          "entryId": "dir_Tm4sV3g4K9xQ2bLp",
          "givenBy": "dir_B3nOk4f0rRq9Wz2X",
          "note": "A year of Saturdays. Thank you.",
          "state": "offered",
          "offeredAt": "2026-11-05T20:00:00Z",
          "expiresAt": "2026-12-05T20:00:00Z"
        },
        "badge": {
          "badgeId": "bdg_Yr1Ab0ardTn2Qx7V",
          "kind": "community",
          "name": "One year aboard",
          "description": "A year in the community.",
          "image": { "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "mediaType": "image/png", "size": 30720 },
          "repeatable": false,
          "listed": true,
          "groupId": "grp_Yr1Ab0ardGr0upX2",
          "state": "active",
          "createdAt": "2026-09-01T09:10:00Z"
        },
        "givenBy": {
          "displayName": "Ben"
        }
      }
    ],
    "visibility": "members",
    "kudosTotal": 0
  }
}
```

The kudos total is 0: the signer's only kudos badge is hidden, and a hidden badge counts for nothing (VTI-BDG-043).

### Refused: the signer is not a member

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000303",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:stranger",
  "issuedAt": "2026-11-06T10:00:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000301",
  "payload": {
    "code": "permissionDenied",
    "message": "Not an active member of this community.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

Nothing in. Out, to the signer alone: the badges they hold and hid, the identifiers of the credentials that evidence them, the offers that name them with each giver's projected directory fields and note, their wall visibility and their kudos total. The response never carries a key or a credential, only a credential's identifier. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. A giver appears as the directory projects them to the signer, so this task joins nothing the directory would not; `givenBy` on the award is the giver's `entryId`, which survives their rotation and reveals no DID. A delegated portal key signs as the member for one session and is bound to that member's entry, so it names nobody else.

### Retention

**Transient.** Nothing beyond the audit record of the request. What the response lists is kept by the community for the member's own history regardless of this read (VTI-BDG-005).

### Consent/purpose

The member seeing their own badges and deciding what to do with them. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and nothing answered here widens what the member may do.
