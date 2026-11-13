---
slug: vtc/badges/mine/hide
version: '0.1'
title: VTC Badges — Mine — Hide
summary: "Hide a badge the signer holds: off their wall, out of the badge's holder list and its count, out of their kudos total — while every award, the badge-group membership and the credential stand unchanged."
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
  rationale: What shows on a member's card is the member's choice about their own card; hiding a badge must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed hide would undo a show the member made since.
sideEffects:
  level: mutating
  rationale: Marks one badge hidden for the signer, and unpins it. Reversed with vtc/badges/mine/show.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: The member acts on the showing of their own badge; the response is that badge as they hold it.
retention:
  class: durable
  rationale: The hidden mark is a setting of the member's, kept until they show the badge again (VTI-BDG-030).
errorCodes:
- code: vtc/badges/mine/hide:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was changed.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/mine/show
- vtc/badges/mine/list
- vtc/badges/mine/arrange
- vtc/badges/wall/show
- vtc/badges/holders/list
---

## Abstract

The **VTC Badges — Mine — Hide** Trust Task takes a badge off the signer's card without giving it up (VTI Community Badges chapter, VTI-BDG-030). A hidden badge is left out of the signer's wall ([`vtc/badges/wall/show`](../../../wall/show/0.1/spec.md)), out of the badge's holder list and its count ([`vtc/badges/holders/list`](../../../holders/list/0.1/spec.md)), and out of the signer's kudos total (VTI-BDG-043). Its awards, the signer's membership of the badge group and the credential that evidences it are untouched: hiding is the member's choice about their own card, not a change to what the community recorded. The badge's owners and holders of `vtc.badges.admin` see the holder marked `hidden: true`.

A badge is hidden as a whole, every award of it at once. [`vtc/badges/mine/show`](../../show/0.1/spec.md) reverses it. Both are idempotent.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every hide with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` a `badgeId` the signer does not hold — no accepted award of it. A badge that does not exist, one the signer may not see, and one only offered to them and not yet accepted answer alike. A retired badge the signer holds may be hidden (VTI-BDG-005).
3. Mark the badge hidden for this holder, and from then on leave the signer out of the badge's holder list shown to members and out of its `holderCount`, leave the badge off the signer's wall, and leave its awards out of the signer's kudos total (VTI-BDG-030, VTI-BDG-033, VTI-BDG-043). The mark is the holder's and covers every award of the badge they hold.
4. **MUST NOT** change any award, end or alter the signer's membership of the badge group, or revoke, renew differently or otherwise touch any credential (VTI-BDG-030).
5. Unpin the badge where it was pinned (VTI-BDG-032): a hidden badge is never pinned. Showing it again does not pin it.
6. Show the badge's owners and holders of `vtc.badges.admin` that the holder has hidden it, as `hidden: true` wherever the holder is listed to them, and show it to nobody else (VTI-BDG-030).
7. Treat the task as idempotent: hiding a badge the signer already hid changes nothing and answers the badge as it stands.
8. Audit the change by reference — the badge and the signer — never with a picture or a note (VTI-BDG-091).

## Authorization

**Authority: holding the badge.** The task has no target member: the signer changes only what shows of their own badge. Hiding confers and removes nothing (VTI-BDG-002). The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms.

## Definitions

- `BadgeId`, `MyBadge` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Hide a kudos badge

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000321",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/hide/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:10:00Z",
  "payload": {
    "badgeId": "bdg_Kd0sCl4mpWh1sp3r"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/mine/hide/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Hide a kudos badge — answered

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000322",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/hide/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:10:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000321",
  "payload": {
    "badge": {
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
  }
}
```

### Refused: the signer does not hold the badge

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000323",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:10:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000321",
  "payload": {
    "code": "notFound",
    "message": "You hold no badge with this identifier.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A badge identifier in; the badge as the signer holds it out, with the identifier of the credential that evidences it where there is one — never the credential or a key. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. That the signer hid the badge is seen by its owners and administrators, who already knew they held it, and by nobody else; a member reading the holder list cannot tell a hidden holder from a non-holder (VTI-BDG-030). The credential is unchanged, so no verifier outside sees a hide happen.

### Retention

**Durable.** The hidden mark is a setting of the member's and stands until they show the badge again; it does not change how long the awards or the credential are kept (VTI-BDG-030). The audit record names the badge and the signer (VTI-BDG-091).

### Consent/purpose

The member's choice about their own card. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002); hiding one changes no authority, and the community **MUST NOT** treat a hide as a decline, a departure or a request to revoke.
