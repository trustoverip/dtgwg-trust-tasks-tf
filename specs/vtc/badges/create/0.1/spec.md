---
slug: vtc/badges/create
version: '0.1'
title: VTC Badges — Create
summary: 'Define a badge: a community badge, which creates the badge group that backs it and names its owners, or a kudos badge a member makes for other members to give.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- group
parties:
- role: badge administrator or member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: A community badge creates an identity the community answers for and an authorization credential in the community's name; a kudos badge is a picture every member may give under its maker's name. Both must be attributable to their requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed create would define a second badge and, for a community badge, mint a second DID.
sideEffects:
  level: mutating
  rationale: For a community badge, has the community's VTA mint a DID, has the community issue the group an authorization credential, and stores the badge and its group. For a kudos badge, stores the badge. Recoverable with vtc/badges/retire.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: The request carries a name, description, criteria, the digest of an uploaded picture, settings and a rule; the response is the badge record and, for a community badge, its group record — never a key or a credential.
retention:
  class: durable
  rationale: A badge persists until retired, and a retired badge is kept for its holders' own history (VTI-BDG-005).
errorCodes:
- code: vtc/badges/create:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was stored.
  retryable: false
- code: vtc/badges/create:kindNotPermitted
  meaning: The signer may not define a badge of this kind — a community badge without `vtc.badges.admin`, or a kudos badge when the community's badge settings do not admit the signer as a maker (policy, tenure).
  retryable: false
- code: vtc/badges/create:limitReached
  meaning: The signer has made as many kudos badges as the community allows in the current month. Nothing was stored.
  retryable: true
- code: vtc/badges/create:imageNotFound
  meaning: '`image.sha256` is not an image this signer uploaded with vtc/badges/images/upload that is still held and not yet taken up by another badge.'
  retryable: false
- code: vtc/badges/create:settingsInvalid
  meaning: A setting that does not fit the kind — `requestable`, `shareableOutside`, `holdersVisibleTo`, `automatic` or `owners` on a kudos badge, `repeatable` false on a kudos badge, `requestable` together with `automatic`, `requestable` while the community's `membersMayRequest` is off, or `automatic` while `automaticBadges` is off.
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
- code: vtc/badges/create:ruleInvalid
  meaning: '`automatic.rule` is malformed or reads a fact that is not verified, as vtc/groups/create refuses it.'
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
- code: vtc/badges/create:ownerNotEligible
  meaning: An entry in `owners` is not an active member, or the signer does not cover `vtc.groups.manage` for the new group and so cannot appoint anyone to it. Nothing was stored.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      entryId:
        type: string
        maxLength: 128
- code: vtc/badges/create:noKeyHolder
  meaning: The community has no link to a VTA that can hold the badge group's keys. Its host's operator configures one.
  retryable: false
- code: vtc/badges/create:keyHolderUnavailable
  meaning: The community's VTA did not answer. Nothing was stored; a DID it may have minted is named in `details.groupDid` so an operator can retire it.
  retryable: true
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      groupDid:
        type: string
        maxLength: 2048
- code: vtc/badges/create:keyHolderRefused
  meaning: The community's VTA refused a step. Its own code and message are carried in `details`. Nothing was stored.
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
- vtc/badges/show
- vtc/badges/update
- vtc/badges/retire
- vtc/badges/images/upload
- vtc/badges/awards/offer
- vtc/groups/create
- vtc/groups/managers/add
---

## Abstract

The **VTC Badges — Create** Trust Task defines a badge (VTI Community Badges chapter). A **community badge** is defined by a holder of `vtc.badges.admin`, and creating it creates the **badge group** that backs it: a community group with its own DID, held by the community's VTA, whose managers are the badge's owners, whose members are its holders, and whose DID issues each holder the badge credential (VTI-BDG-010, VTI-BDG-011, VTI-BDG-060). A **kudos badge** is made by a member the community's badge settings admit, has no group and no credential, and may be given by any active member to any other (VTI-BDG-040, VTI-BDG-041).

The picture comes first: the signer uploads it with [`vtc/badges/images/upload`](../../images/upload/0.1/spec.md) and names its digest here. A badge is created with no holders; it is given with [`vtc/badges/awards/offer`](../../awards/offer/0.1/spec.md), or with [`vtc/badges/kudos/give`](../../kudos/give/0.1/spec.md), or, for an automatic badge, by its rule.

## The steps, and their order

For a community badge the community, on a valid request:

1. Creates the badge group exactly as [`vtc/groups/create`](../../../groups/create/0.1/spec.md) does — has its VTA mint the group's DID from the `vtc-group` template, then issues the group the community's authorization credential — with `kind` `static` (or `dynamic` with `automatic.rule`), `joinPolicy` `request` where `requestable` is true and `closed` otherwise, `visibility` `members`, `showMembers` from `holdersVisibleTo`, `issuesCredentials` true, and the badge's name as the group's name.
2. **Then** stores the badge and the group, each referencing the other (`groupId` on the badge, `badgeId` on the group), and appoints each entry in `owners` a manager of the group as [`vtc/groups/managers/add`](../../../groups/managers/add/0.1/spec.md) appoints one.
3. Where the community publishes to a trust registry, projects the badge group as an issuer the community recognises (VTI-BDG-080), after the badge is stored and through a projector that is the one retry owner.

Every remote step comes before the one local write. A failure in step 1 leaves no badge and no group behind; at most a DID the VTA minted and nobody uses, which `keyHolderUnavailable` names.

For a kudos badge the community stores the badge. Nothing is minted and nothing is issued.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every create with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. For a community badge, refuse a signer without `vtc.badges.admin` with `kindNotPermitted` (VTI-BDG-003). For a kudos badge, refuse with `kindNotPermitted` a signer who is not an active member the settings' `kudosMakers` admit, or whose membership is younger than `limits.minTenureDays`; and refuse with `limitReached` a signer who has made `limits.kudosBadgesPerMakerPerMonth` kudos badges this month.
3. Refuse with `imageNotFound` an `image.sha256` that is not an image this signer uploaded with [`vtc/badges/images/upload`](../../images/upload/0.1/spec.md), still held and not taken up by another badge. A digest seen on someone else's badge is never a way to put their picture on your own.
4. Refuse with `settingsInvalid` a kudos badge carrying `requestable`, `shareableOutside`, `holdersVisibleTo`, `automatic` or `owners`, or `repeatable` false; a community badge carrying both `requestable` and `automatic`; `requestable` while the settings' `membersMayRequest` is false; and `automatic` while `automaticBadges` is false. Refuse an `automatic.rule` that [`vtc/groups/create`](../../../groups/create/0.1/spec.md) would refuse with `ruleInvalid`, except that a badge's rule **MAY** read the counts `kudosReceived`, `ideasSelected` and `electionTickets` with `atLeast` (VTI-BDG-015), each counted from the community's own records whenever the rule is evaluated.
5. For a community badge, refuse with `ownerNotEligible` an entry in `owners` that is not an active member, and a signer who does not cover `vtc.groups.manage` for the new group ([`vtc/groups/managers/add`](../../../groups/managers/add/0.1/spec.md) item 4), before anything is minted.
6. For a community badge, perform the steps above in order, store the badge and its group only after they succeed, and answer `noKeyHolder`, `keyHolderUnavailable` or `keyHolderRefused` as [`vtc/groups/create`](../../../groups/create/0.1/spec.md) does. The badge group **MUST NOT** carry authority: the community refuses every `grants` or `role` binding to it (VTI-BDG-012).
7. Record the badge `active` with no holders. A kudos badge is `repeatable`; a community badge's `repeatable` is as requested.
8. Audit the creation by reference — the badge, the group and the signer — and **MUST NOT** return or log a key or a credential.

## Authorization

**Community badge: `vtc.badges.admin`** at the community, read from the signer's access-control entry at execution time (VTI-BDG-003). Appointing owners also takes the signer's cover of `vtc.groups.manage` for the new group, as any manager appointment does. **Kudos badge: an active member the community's `kudosMakers` setting admits**, within its tenure and monthly limits. Creating a badge confers nothing on anyone: a badge is never read by an authorization decision (VTI-BDG-002). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A kudos badge's create is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it. A community badge's create **MUST NOT** be accepted from such a key.

## Definitions

- `Badge`, `BadgeKind`, `BadgeSettings` — see [`badge.schema.json`](../../_shared/0.1/badge.schema.json).
- `Group`, `DynamicRule`, `GroupAudience` — see the groups family's [`group.schema.json`](../../../groups/_shared/0.1/group.schema.json).

## Request

The badge administrator or member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A community badge members may ask for, with two owners

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000101",
  "type": "https://trusttasks.org/spec/vtc/badges/create/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T09:00:00Z",
  "payload": {
    "kind": "community",
    "name": "Workshop host",
    "description": "Ran a workshop for the community.",
    "criteria": "Hosted any workshop on the community calendar. A badge owner gives it after the session.",
    "image": { "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08" },
    "repeatable": true,
    "requestable": true,
    "listed": true,
    "shareableOutside": true,
    "holdersVisibleTo": "members",
    "owners": ["dir_Pr1yaNatarajan00", "dir_BenOkafor0000000"],
    "reason": "Workshops are the heart of the community; the hosts should wear it."
  }
}
```

### A kudos badge a member makes

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000102",
  "type": "https://trusttasks.org/spec/vtc/badges/create/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T09:05:00Z",
  "payload": {
    "kind": "kudos",
    "name": "Clamp whisperer",
    "description": "Found the good clamps when nobody else could.",
    "image": { "sha256": "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae" },
    "reason": "The Saturday glue-up."
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The community badge — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000103",
  "type": "https://trusttasks.org/spec/vtc/badges/create/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-06T09:00:02Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000101",
  "payload": {
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
      "holderCount": 0,
      "createdAt": "2026-11-06T09:00:02Z",
      "createdBy": "dir_AnnaReyes0000000"
    },
    "group": {
      "groupId": "grp_Hs7Qm2Lk9Tn4Bv1X",
      "groupDid": "did:webvh:QmGroupScid:community.example:group-dids:grp_Hs7Qm2Lk9Tn4Bv1X",
      "name": "Workshop host",
      "kind": "static",
      "joinPolicy": "request",
      "visibility": "members",
      "showMembers": "members",
      "issuesCredentials": true,
      "carriesAuthority": false,
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "memberCount": 0,
      "managers": ["dir_Pr1yaNatarajan00", "dir_BenOkafor0000000"],
      "credentialAuthorityValidUntil": "2027-11-06T09:00:01Z",
      "createdAt": "2026-11-06T09:00:02Z"
    }
  }
}
```

### Refused: a kudos badge from a member too new to make one

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000104",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T09:05:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000102",
  "payload": {
    "code": "vtc/badges/create:kindNotPermitted",
    "message": "Members make kudos badges after 7 days; you joined 3 days ago.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A name, description, criteria, a digest, settings, a rule and owners in; the badge record and its group out. The group's keys are created inside the community's VTA and never leave it. A producer **MUST NOT** place personal data in `name`, `description`, `criteria`, `reason` or `ext`: a badge's name is shown to every member who may see the badge and, where it may be shown outside, to the world.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The badge group's DID is minted for that badge alone and its DID document carries no name or description (VTI-DIR-082). A badge's picture is the community's own re-encoding, so its digest links nothing across communities. Where the badge may be shown outside, its picture and description are public (VTI-BDG-051); its holders are not.

### Retention

**Durable.** A badge persists until retired, and a retired badge is kept for its holders' own history (VTI-BDG-005).

### Consent/purpose

The community organising its members' recognition of one another. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and a badge group is never bound to a capability or a role (VTI-BDG-012).
