---
slug: vtc/badges/update
version: '0.1'
title: VTC Badges — Update
summary: 'Change a badge''s name, description, criteria, picture, listing and, for a community badge, whether members may ask for it and who sees its holders. Its kind and credential regime are fixed at creation.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- group
parties:
- role: badge owner, maker or administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Changes what every member sees of a badge and, for a community badge, its group's join policy and who sees its members; must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update would revert a later one.
sideEffects:
  level: mutating
  rationale: Replaces the named members of the badge record and, for a community badge, its group's name, join policy or member-list visibility. Recoverable by a later update. Issues and revokes nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Carries changed badge text, a picture's digest and settings; returns the badge record.
retention:
  class: durable
  rationale: The badge record in force is what members see and what a verifier reading the badge live is shown.
errorCodes:
- code: vtc/badges/update:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was stored.
  retryable: false
- code: vtc/badges/update:badgeRetired
  meaning: The badge is retired; a retired badge is kept for its holders' history and is not changed (VTI-BDG-005).
  retryable: false
- code: vtc/badges/update:imageNotFound
  meaning: '`image.sha256` is not an image this signer uploaded with vtc/badges/images/upload that is still held and not yet taken up by another badge.'
  retryable: false
- code: vtc/badges/update:settingsInvalid
  meaning: A member that is fixed at creation — `kind`, `repeatable`, `shareableOutside`, `automatic` — or one that does not fit the kind — `requestable` or `holdersVisibleTo` on a kudos badge, `requestable` on an automatic badge, or `requestable` while the community's `membersMayRequest` is off.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      field:
        type: string
        maxLength: 128
      reason:
        type: string
        maxLength: 500
related:
- vtc/badges/show
- vtc/badges/create
- vtc/badges/retire
- vtc/badges/images/upload
- vtc/groups/update
---

## Abstract

The **VTC Badges — Update** Trust Task changes a badge. Only the members sent change: its `name`, `description`, `criteria`, `image` (a digest the signer uploaded with [`vtc/badges/images/upload`](../../images/upload/0.1/spec.md), as [`vtc/badges/create`](../../create/0.1/spec.md) takes one), whether it is `listed` and, for a community badge, whether members may ask for it (`requestable`, which moves its badge group's `joinPolicy` between `request` and `closed`) and who sees its holders (`holdersVisibleTo`, the group's `showMembers`).

What a badge *is* does not change. Its `kind`, whether it is `repeatable`, whether its holders may show it outside (`shareableOutside`) and whether it is `automatic` are fixed when it is created, because each of them fixes the credential regime of every award already accepted (VTI-BDG-061, VTI-BDG-062). A badge whose regime must change is retired and made again.

Changing a community badge's name, picture or criteria re-issues no credential. A badge credential names its Achievement by a stable identifier the community serves, and a verifier reads the badge as it now is from there; the holders' credentials stand as they are (VTI-BDG-060).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every update with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Answer `notFound` for a badge the caller may not see — one that does not exist, an unlisted or retired one they neither hold, own nor administer — and `permissionDenied` for one they may see but may not change: a community badge by anyone but a holder of `vtc.badges.admin` or an owner; a kudos badge by anyone but its maker (`createdBy`) or a holder of `vtc.badges.admin` (VTI-BDG-003).
3. Refuse a retired badge with `badgeRetired` (VTI-BDG-005).
4. Refuse with `imageNotFound` an `image.sha256` that is not an image this signer uploaded with [`vtc/badges/images/upload`](../../images/upload/0.1/spec.md), still held and not taken up by another badge, as [`vtc/badges/create`](../../create/0.1/spec.md) item 3 refuses one.
5. Refuse with `settingsInvalid` `requestable` or `holdersVisibleTo` on a kudos badge, `requestable` on an automatic badge, and `requestable` true while the settings' `membersMayRequest` is false. The schema admits no `kind`, `repeatable`, `shareableOutside` or `automatic`; a community that receives one through an extension or a later version refuses it with `settingsInvalid` too — they are fixed at creation.
6. For a community badge, apply the change to its badge group as [`vtc/groups/update`](../../../groups/update/0.1/spec.md) applies one: the badge's `name` is the group's name, `requestable` true sets the group's `joinPolicy` to `request` and false to `closed` (VTI-BDG-014), and `holdersVisibleTo` sets its `showMembers`. A narrowing of `holdersVisibleTo` applies at once.
7. Issue, re-issue and revoke nothing. A credential names its Achievement by a stable identifier and a verifier reads the badge live, so a new name, picture or criteria reaches every verifier without touching any holder's credential (VTI-BDG-060).
8. Record `updatedAt`, and audit the change by reference — the badge, the members changed and the signer — never with the picture or the text (VTI-BDG-091).

## Authorization

**Community badge: `vtc.badges.admin`, or ownership** — `vtc.groups.manage` for the badge group, unqualified or qualified `group:<groupId>` — read from the signer's access-control entry at execution time (VTI-BDG-003). **Kudos badge: its maker** (the badge's `createdBy`) **or `vtc.badges.admin`**. A holder of `vtc.badges.admin` administers every badge without being an owner of any. Changing a badge confers nothing on anyone (VTI-BDG-002). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an owner's, a maker's or an administrator's act, never a holder's: a key the member authorized for `vtc/badges` in the portal **MUST NOT** be accepted (VTI-BDG-095).

## Definitions

- `Badge`, `BadgeId`, `BadgeName`, `BadgeDescription`, `BadgeCriteria` — see [`badge.schema.json`](../../_shared/0.1/badge.schema.json).
- `GroupAudience` — see the groups family's [`group.schema.json`](../../../groups/_shared/0.1/group.schema.json).

## Request

The badge owner, maker or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Stop taking requests for a badge and narrow who sees its holders

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000221",
  "type": "https://trusttasks.org/spec/vtc/badges/update/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T09:20:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
    "criteria": "Hosted a workshop on the community calendar this year. A badge owner gives it after the session.",
    "requestable": false,
    "holdersVisibleTo": "groupMembers",
    "reason": "Owners give it after each session now; nobody needs to ask."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-06T09:20:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The badge — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000222",
  "type": "https://trusttasks.org/spec/vtc/badges/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T09:20:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000221",
  "payload": {
    "badge": {
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "kind": "community",
      "name": "Workshop host",
      "description": "Ran a workshop for the community.",
      "criteria": "Hosted a workshop on the community calendar this year. A badge owner gives it after the session.",
      "image": { "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08", "mediaType": "image/png", "size": 61440 },
      "repeatable": true,
      "requestable": false,
      "listed": true,
      "shareableOutside": true,
      "groupId": "grp_Hs7Qm2Lk9Tn4Bv1X",
      "state": "active",
      "holderCount": 12,
      "createdAt": "2026-11-06T09:00:02Z",
      "createdBy": "dir_AnnaReyes0000000",
      "updatedAt": "2026-11-06T09:20:01Z"
    }
  }
}
```

### Refused: the badge was retired yesterday

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000223",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T09:20:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000221",
  "payload": {
    "code": "vtc/badges/update:badgeRetired",
    "message": "Workshop host was retired on 2026-11-05; a retired badge is kept as it was for its holders.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

Changed text, a digest and settings in; the badge record out. A producer **MUST NOT** place personal data in `name`, `description`, `criteria`, `reason` or `ext`: a badge's name, description and criteria are shown to every member who may see the badge, written into the Achievement its credential names and, where it may be shown outside, served to the world (VTI-BDG-051).

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. A badge's picture is the community's own re-encoding, so its digest links nothing across communities (VTI-BDG-050). Where the badge may be shown outside, a change to its picture or description is public at once; its holders are not (VTI-BDG-071).

### Retention

**Durable.** The badge record in force replaces the previous one; the community keeps the record of each change and who made it. No credential is issued, re-issued or revoked by an update.

### Consent/purpose

Owners and makers keeping a badge's name, picture and rules current. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and a badge group is never bound to a capability or a role (VTI-BDG-012).
