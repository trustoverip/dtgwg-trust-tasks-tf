---
slug: vtc/badges/mine/arrange
version: '0.1'
title: VTC Badges — Mine — Arrange
summary: "Arrange the signer's wall: the order of their badges, which of them are pinned beside their name, and who may see the wall. Each member absent from the request is unchanged."
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
  rationale: The order of a member's badges, which are pinned and who sees them are the member's own choice (VTI-BDG-032); the choice must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed arrange would put back an order, a pin set or a visibility the member has since changed.
sideEffects:
  level: mutating
  rationale: Replaces the signer's wall order, pinned set or wall visibility, whichever the request names. Recoverable by arranging again.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: The member acts on the showing of their own badges; the response is their badges as they now stand.
retention:
  class: durable
  rationale: The order, the pinned set and the visibility are settings of the member's, kept until they arrange again (VTI-BDG-031, VTI-BDG-032).
errorCodes:
- code: vtc/badges/mine/arrange:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was changed.
  retryable: false
- code: vtc/badges/mine/arrange:badgeNotHeld
  meaning: An identifier in `order` or `pinned` is not a badge the signer holds. The first such is named in `details.badgeId`. Nothing was changed.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      badgeId:
        type: string
        maxLength: 128
- code: vtc/badges/mine/arrange:tooManyPinned
  meaning: '`pinned` names more badges than the community allows, `limits.maxPinned`, carried in `details.limit`. Nothing was changed.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      limit:
        type: integer
        minimum: 0
- code: vtc/badges/mine/arrange:badgeHidden
  meaning: '`pinned` names a badge the signer has hidden, in `details.badgeId`; a hidden badge is never pinned. Show it first with vtc/badges/mine/show. Nothing was changed.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      badgeId:
        type: string
        maxLength: 128
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/mine/list
- vtc/badges/mine/hide
- vtc/badges/mine/show
- vtc/badges/wall/show
- vtc/badges/settings/show
---

## Abstract

The **VTC Badges — Mine — Arrange** Trust Task sets how the signer's badges appear on their card (VTI Community Badges chapter, VTI-BDG-031, VTI-BDG-032): `order`, the wall order of every badge they hold; `pinned`, which of them stand beside their name, at most as many as the community's `limits.maxPinned` allows; and `visibility`, who may see the wall. A request carries any of the three; a member it leaves out is unchanged.

What the signer holds is read with [`vtc/badges/mine/list`](../../list/0.1/spec.md); what others see of it is [`vtc/badges/wall/show`](../../../wall/show/0.1/spec.md). Whether a badge shows at all is [`vtc/badges/mine/hide`](../../hide/0.1/spec.md) and [`vtc/badges/mine/show`](../../show/0.1/spec.md): a hidden badge keeps its place in the order but cannot be pinned. The limit is read from the community's badge settings with [`vtc/badges/settings/show`](../../../settings/show/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every arrange with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004). Refuse a signer who is not an active member with `permissionDenied`.
2. Refuse with `badgeNotHeld` an identifier in `order` or in `pinned` that is not a badge the signer holds — no accepted award of it — naming the first such in `details.badgeId`. A badge that does not exist, one the signer may not see, and one only offered to them answer alike.
3. Where `order` is present, give each badge it lists the position of its place in the list, lowest first (VTI-BDG-032), and place every badge the signer holds that it omits after the listed ones, in their previous relative order. `order` is never a way to drop a badge: hidden badges keep a position too, and are simply not shown.
4. Where `pinned` is present, refuse with `tooManyPinned` a list longer than the settings' `limits.maxPinned` — 2 where the community states none — carrying the limit in `details.limit` (VTI-BDG-032); refuse with `badgeHidden` a listed badge the signer has hidden, naming it in `details.badgeId` (VTI-BDG-030); and otherwise replace the signer's pinned set with the list. An empty list unpins every badge.
5. Where `visibility` is present, set who may see the signer's wall (VTI-BDG-031). `members` never shows the wall to a viewer to whom the directory does not show the signer's card; `nobody` shows it to the signer alone. Hidden badges stay off the wall under either.
6. Leave every member the request does not carry unchanged, and apply the ones it carries together: a refusal of any leaves all unchanged.
7. Answer `badges` as [`vtc/badges/mine/list`](../../list/0.1/spec.md) would — every badge the signer holds, hidden ones included, in the new order with the new `pinned` marks — and `visibility` as it now stands.
8. Audit the change by reference — the signer and the badges pinned — never with a picture or a note (VTI-BDG-091).

## Authorization

**Authority: being the member.** The task has no target member: the signer arranges only their own wall. Arranging confers nothing (VTI-BDG-002). The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms.

## Definitions

- `BadgeId`, `MyBadge`, `WallVisibility`, `BadgeLimits` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Put the kudos badge first and pin both

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000341",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/arrange/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:20:00Z",
  "payload": {
    "order": ["bdg_Kd0sCl4mpWh1sp3r", "bdg_W5k2N8xRb3Qz7TpL"],
    "pinned": ["bdg_Kd0sCl4mpWh1sp3r", "bdg_W5k2N8xRb3Qz7TpL"]
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/mine/arrange/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Put the kudos badge first and pin both — answered

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000342",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/arrange/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:20:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000341",
  "payload": {
    "badges": [
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
        "pinned": true,
        "hidden": false,
        "position": 0
      },
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
        "position": 1,
        "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d",
        "credentialValidUntil": "2028-10-19T18:30:00Z"
      }
    ],
    "visibility": "members"
  }
}
```

### Refused: three pinned where the community allows two

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000343",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:20:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000341",
  "payload": {
    "code": "vtc/badges/mine/arrange:tooManyPinned",
    "message": "This community lets a member pin 2 badges.",
    "retryable": false,
    "details": { "limit": 2 }
  }
}
```

## Security & Privacy

### Data carried

Badge identifiers and a visibility in; the signer's badges as they now stand out, with the identifiers of the credentials that evidence them — never a credential or a key. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The order and the pins are seen by whoever sees the wall, which the signer's `visibility` and the directory's own visibility bound (VTI-BDG-031); the identifiers in the request name badges the signer already holds and nothing else.

### Retention

**Durable.** The order, the pinned set and the visibility are the member's own settings and stand until they arrange again; they change nothing about how long awards or credentials are kept. The audit record names the signer and the badges pinned (VTI-BDG-091).

### Consent/purpose

The member's choice about their own card: what shows, in what order, to whom (VTI-BDG-031, VTI-BDG-032). **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002); a pin is a place beside a name, not a rank.
