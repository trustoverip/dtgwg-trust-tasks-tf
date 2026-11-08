---
slug: vtc/badges/promote
version: '0.1'
title: VTC Badges — Promote
summary: 'Turn a kudos badge into a community badge: its badge group is created and its owners named, and every current holder is offered the badge so that each accepts before anything is issued or recorded; the kudos awards stay as history.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- group
parties:
- role: badge administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Creates an identity the community answers for and an authorization credential in the community's name, and offers a credential-backed badge to every holder; must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed promote would mint a second DID for a badge that already has a group.
sideEffects:
  level: mutating
  rationale: Has the community's VTA mint a DID, has the community issue the group an authorization credential, rewrites the badge's kind and settings, and offers the badge to every holder. Recoverable with vtc/badges/retire; the badge does not become a kudos badge again.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Carries a badge identifier, settings, owners and a reason; returns the badge and its group record, never a key or a credential.
retention:
  class: durable
  rationale: The badge, its group and the offers persist; the kudos awards already accepted are kept as the holders' history.
errorCodes:
- code: vtc/badges/promote:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was stored.
  retryable: false
- code: vtc/badges/promote:kindMismatch
  meaning: The badge is already a community badge.
  retryable: false
- code: vtc/badges/promote:badgeRetired
  meaning: The badge is retired; a retired badge is kept for its holders' history and is not promoted (VTI-BDG-005).
  retryable: false
- code: vtc/badges/promote:settingsInvalid
  meaning: '`requestable` while the community''s `membersMayRequest` is off.'
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
- code: vtc/badges/promote:ownerNotEligible
  meaning: An entry in `owners` is not an active member, or the signer does not cover `vtc.groups.manage` for the new group and so cannot appoint anyone to it. Nothing was stored.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      entryId:
        type: string
        maxLength: 128
- code: vtc/badges/promote:noKeyHolder
  meaning: The community has no link to a VTA that can hold the badge group's keys. Its host's operator configures one.
  retryable: false
- code: vtc/badges/promote:keyHolderUnavailable
  meaning: The community's VTA did not answer. Nothing was stored and the badge is still a kudos badge; a DID the VTA may have minted is named in `details.groupDid` so an operator can retire it.
  retryable: true
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      groupDid:
        type: string
        maxLength: 2048
- code: vtc/badges/promote:keyHolderRefused
  meaning: The community's VTA refused a step. Its own code and message are carried in `details`. Nothing was stored and the badge is still a kudos badge.
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
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/create
- vtc/badges/show
- vtc/badges/awards/offer
- vtc/badges/mine/accept
- vtc/groups/create
- vtc/groups/managers/add
---

## Abstract

The **VTC Badges — Promote** Trust Task turns a kudos badge into a community badge (VTI-BDG-044). A kudos badge that members have made their own — given often, worn widely — can become one the community stands behind: a holder of `vtc.badges.admin` names its settings and its owners, the community creates the badge group that backs it exactly as [`vtc/badges/create`](../../create/0.1/spec.md) creates one, and every member who holds the badge is **offered** it as [`vtc/badges/awards/offer`](../../awards/offer/0.1/spec.md) offers one, so that each accepts with [`vtc/badges/mine/accept`](../../mine/accept/0.1/spec.md) before anything is issued or recorded in the group (VTI-BDG-020). Nobody is admitted and nothing is issued without their acceptance.

The kudos awards already accepted stay as the holders' history and keep counting on the wall. The badge's maker becomes an owner only if named in `owners`; making a badge never made anyone its administrator.

## The steps, and their order

The community, on a valid request:

1. Creates the badge group as [`vtc/badges/create`](../../create/0.1/spec.md) items 5 and 6 do — checks the owners, has its VTA mint the group's DID, issues the group the community's authorization credential — with `kind` `static`, `joinPolicy` `request` where `requestable` is true and `closed` otherwise, `visibility` `members`, `showMembers` from `holdersVisibleTo`, `issuesCredentials` true, and the badge's name as the group's name.
2. **Then** sets the badge's `kind` to `community`, its `groupId`, `requestable`, `shareableOutside` and `holdersVisibleTo`, stores the badge and the group each referencing the other, and appoints each entry in `owners` a manager of the group as [`vtc/groups/managers/add`](../../../groups/managers/add/0.1/spec.md) appoints one. `repeatable` stays true: a kudos badge always was.
3. Offers the badge to every current holder — each member with at least one accepted award of it — one offer each, made by the signer, naming the number of awards they hold, so that the credential issued on acceptance evidences them. Each holder accepts or declines as any offer is accepted or declined; nothing is issued or recorded in the group until they do (VTI-BDG-020, VTI-BDG-021, VTI-BDG-044).
4. Where the community publishes to a trust registry, projects the badge group as an issuer the community recognises (VTI-BDG-080), after the badge is stored and through the projector that is the one retry owner.

Every remote step comes before the one local write. A failure in step 1 leaves the badge a kudos badge with no group; at most a DID the VTA minted and nobody uses, which `keyHolderUnavailable` names.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every promote with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse a signer without `vtc.badges.admin` with `permissionDenied` (VTI-BDG-044), and answer `notFound` for a badge the caller may not see.
3. Refuse a community badge with `kindMismatch` and a retired badge with `badgeRetired`.
4. Refuse with `settingsInvalid` `requestable` true while the settings' `membersMayRequest` is false, and refuse with `ownerNotEligible` an entry in `owners` that is not an active member, and a signer who does not cover `vtc.groups.manage` for the new group ([`vtc/groups/managers/add`](../../../groups/managers/add/0.1/spec.md) item 4), before anything is minted.
5. Perform the steps above in order, store the badge's new kind and its group only after the group exists, and answer `noKeyHolder`, `keyHolderUnavailable` or `keyHolderRefused` as [`vtc/groups/create`](../../../groups/create/0.1/spec.md) does, with the badge still a kudos badge. The badge group **MUST NOT** carry authority (VTI-BDG-012).
6. Offer the badge to every current holder, one offer each, and **MUST NOT** admit any of them to the badge group, issue any credential or record anything in the group before they accept (VTI-BDG-020, VTI-BDG-044). A holder who declines keeps their kudos awards and their count; the badge is shown on their wall as it was.
7. Keep every kudos award as it is: accepted awards keep counting on the holder's wall and in their kudos total; open offers of the kudos badge stay open, and are accepted or declined as they were given (VTI-BDG-041).
8. Make the maker an owner only where `owners` names them. Record `updatedAt`, and audit the promotion by reference — the badge, the group, the owners, the number of holders offered and the signer (VTI-BDG-091).

## Authorization

**Authority: `vtc.badges.admin`** at the community, read from the signer's access-control entry at execution time (VTI-BDG-044). Appointing owners also takes the signer's cover of `vtc.groups.manage` for the new group, as any manager appointment does. The badge's maker has no say: a kudos badge is the community's once made, and its maker may change or retire it (VTI-BDG-003) but not promote it. Promoting a badge confers nothing on anyone (VTI-BDG-002). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an administrator's act, never a member's: a key the member authorized for `vtc/badges` in the portal **MUST NOT** be accepted (VTI-BDG-095).

## Definitions

- `Badge`, `BadgeId` — see [`badge.schema.json`](../../_shared/0.1/badge.schema.json).
- `Group`, `GroupAudience` — see the groups family's [`group.schema.json`](../../../groups/_shared/0.1/group.schema.json).
- `EntryId` — see the directory family's [`directory-card.schema.json`](../../../directory/_shared/0.1/directory-card.schema.json).
- **Current holder** — a member with at least one accepted award of the badge, hidden or not.

## Request

The badge administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Promote a kudos badge the community has come to stand behind

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000241",
  "type": "https://trusttasks.org/spec/vtc/badges/promote/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T09:40:00Z",
  "payload": {
    "badgeId": "bdg_Kd3mR9vXs2Wq8LpN",
    "requestable": false,
    "shareableOutside": false,
    "holdersVisibleTo": "members",
    "owners": ["dir_Pr1yaNatarajan00", "dir_TomasVrba0000000"],
    "reason": "Given four hundred times this year; the community should stand behind it."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-11-06T09:40:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/promote/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The badge and its new group — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000242",
  "type": "https://trusttasks.org/spec/vtc/badges/promote/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-06T09:40:03Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000241",
  "payload": {
    "badge": {
      "badgeId": "bdg_Kd3mR9vXs2Wq8LpN",
      "kind": "community",
      "name": "Clamp whisperer",
      "description": "Found the good clamps when nobody else could.",
      "image": { "sha256": "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae", "mediaType": "image/png", "size": 48120 },
      "repeatable": true,
      "requestable": false,
      "listed": true,
      "shareableOutside": false,
      "groupId": "grp_Qm4Vx8Tk2Rn7Hs1L",
      "state": "active",
      "holderCount": 0,
      "createdAt": "2026-11-06T09:05:02Z",
      "createdBy": "dir_TomasVrba0000000",
      "updatedAt": "2026-11-06T09:40:03Z"
    },
    "group": {
      "groupId": "grp_Qm4Vx8Tk2Rn7Hs1L",
      "groupDid": "did:webvh:QmGroupScid:community.example:group-dids:grp_Qm4Vx8Tk2Rn7Hs1L",
      "name": "Clamp whisperer",
      "kind": "static",
      "joinPolicy": "closed",
      "visibility": "members",
      "showMembers": "members",
      "issuesCredentials": true,
      "carriesAuthority": false,
      "badgeId": "bdg_Kd3mR9vXs2Wq8LpN",
      "memberCount": 0,
      "managers": ["dir_Pr1yaNatarajan00", "dir_TomasVrba0000000"],
      "credentialAuthorityValidUntil": "2027-11-06T09:40:02Z",
      "createdAt": "2026-11-06T09:40:03Z"
    },
    "holdersOffered": 37
  }
}
```

### Refused: the badge is already a community badge

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000243",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-06T09:40:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000241",
  "payload": {
    "code": "vtc/badges/promote:kindMismatch",
    "message": "bdg_Kd3mR9vXs2Wq8LpN is a community badge; change it with vtc/badges/update.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A badge identifier, settings, owners and a reason in; the badge record, its group record and the number of holders offered out. The group's keys are created inside the community's VTA and never leave it. The offers reach the holders through their own list ([`vtc/badges/mine/accept`](../../mine/accept/0.1/spec.md)), never through this response: the signer learns how many were offered, not who. A producer **MUST NOT** place personal data in `reason` or `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The badge group's DID is minted for that badge alone and its DID document carries no name or description (VTI-DIR-082). A holder who accepts receives a credential binding their membership DID to the badge group, which is what they accept it for; one who declines is bound to nothing. Where the badge may be shown outside, its picture and description become public (VTI-BDG-051); its holders do not (VTI-BDG-071).

### Retention

**Durable.** The badge, its group and the offers persist; the kudos awards already accepted are kept as the holders' history and keep counting. A badge, once promoted, is a community badge until retired (VTI-BDG-005).

### Consent/purpose

The community taking a badge its members made into its own hands, with each holder's consent to wearing it under the community's name. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and the new badge group is never bound to a capability or a role (VTI-BDG-012).
