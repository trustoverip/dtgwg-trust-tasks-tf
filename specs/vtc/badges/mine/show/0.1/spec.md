---
slug: vtc/badges/mine/show
version: '0.1'
title: VTC Badges — Mine — Show
summary: "Show a badge the signer had hidden: back on their wall, back in the badge's holder list and its count, back in their kudos total. The reverse of vtc/badges/mine/hide."
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
  rationale: What shows on a member's card is the member's choice about their own card; showing a badge again must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed show would undo a hide the member made since.
sideEffects:
  level: mutating
  rationale: Clears the hidden mark on one badge for the signer. Reversed with vtc/badges/mine/hide.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: The member acts on the showing of their own badge; the response is that badge as they hold it.
retention:
  class: durable
  rationale: The showing is the member's setting, kept until they hide the badge again (VTI-BDG-030).
errorCodes:
- code: vtc/badges/mine/show:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was changed.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/mine/hide
- vtc/badges/mine/list
- vtc/badges/mine/arrange
- vtc/badges/wall/show
- vtc/badges/holders/list
---

## Abstract

The **VTC Badges — Mine — Show** Trust Task puts a badge the signer hid back on their card (VTI Community Badges chapter, VTI-BDG-030). It is the reverse of [`vtc/badges/mine/hide`](../../hide/0.1/spec.md): the badge returns to the signer's wall ([`vtc/badges/wall/show`](../../../wall/show/0.1/spec.md)), to the badge's holder list and its count ([`vtc/badges/holders/list`](../../../holders/list/0.1/spec.md)), and to the signer's kudos total (VTI-BDG-043). Nothing else moves: the awards, the badge-group membership and the credential were never touched by the hide and are not touched by the show.

Showing does not pin: a badge unpinned by its hide is pinned again only with [`vtc/badges/mine/arrange`](../../arrange/0.1/spec.md). Both hide and show are idempotent.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every show with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` a `badgeId` the signer does not hold — no accepted award of it. A badge that does not exist, one the signer may not see, and one only offered to them and not yet accepted answer alike.
3. Clear the hidden mark for this holder, and from then on list the signer among the badge's holders to members the badge shows them to, count them in its `holderCount`, show the badge on the signer's wall subject to their wall visibility (VTI-BDG-031), and count its awards in the signer's kudos total (VTI-BDG-030, VTI-BDG-033, VTI-BDG-043). A retired badge shown again stays off every card and listing to members who do not hold it (VTI-BDG-005).
4. **MUST NOT** change any award, the signer's membership of the badge group, or any credential (VTI-BDG-030), and **MUST NOT** pin the badge: `pinned` stays as it was after the hide, which is false (VTI-BDG-032).
5. Treat the task as idempotent: showing a badge that is not hidden changes nothing and answers the badge as it stands.
6. Audit the change by reference — the badge and the signer — never with a picture or a note (VTI-BDG-091).

## Authorization

**Authority: holding the badge.** The task has no target member: the signer changes only what shows of their own badge. Showing confers nothing (VTI-BDG-002). The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms.

## Definitions

- `BadgeId`, `MyBadge` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Show a hidden badge again

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000331",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/show/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:15:00Z",
  "payload": {
    "badgeId": "bdg_Kd0sCl4mpWh1sp3r"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/mine/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Show a hidden badge again — answered

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000332",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:15:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000331",
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
      "hidden": false,
      "position": 1
    }
  }
}
```

### Refused: badges are off

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000333",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T10:15:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000331",
  "payload": {
    "code": "vtc/badges/mine/show:badgesOff",
    "message": "This community has badges turned off.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A badge identifier in; the badge as the signer holds it out, with the identifier of the credential that evidences it where there is one — never the credential or a key. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Showing a badge again makes the signer visible as its holder to the members the badge shows its holders to, as they were before the hide, and to the viewers their wall visibility admits (VTI-BDG-031); it discloses nothing the holder did not choose to show.

### Retention

**Durable.** The showing is the member's own setting and stands until they hide the badge again; it does not change how long the awards or the credential are kept (VTI-BDG-030). The audit record names the badge and the signer (VTI-BDG-091).

### Consent/purpose

The member's choice about their own card. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002); showing one changes no authority.
