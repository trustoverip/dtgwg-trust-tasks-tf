---
slug: vtc/badges/settings/update
version: '0.1'
title: VTC Badges — Settings — Update
summary: 'An administrator replaces the community''s badge settings: badges on or off, who may make a kudos badge, the limits, whether kudos totals show, whether automatic badges run and whether members may ask for a badge.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- settings
parties:
- role: community administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Changes what every member may do with badges and whether any badge is shown at all; must be attributable to its requester after the transport has closed, and is what the requester's step-up binds to.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update would revert a later one — turning badges back on, or widening who may make a kudos badge, without anyone asking.
sideEffects:
  level: mutating
  rationale: Replaces the community's badge settings record. Turning badges off hides every badge surface and keeps every record; turning automatic badges off stops recomputing dynamic badge groups. Nothing is revoked, ended or erased, and a later update restores what an earlier one hid.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Carries the settings — policy, limits and switches — and a reason; returns the settings record. Never a badge, an award or a member.
retention:
  class: durable
  rationale: The settings in force decide who may make and give badges, within what bounds, and whether badges are shown at all, until the next update.
errorCodes:
- code: vtc/badges/settings/update:settingsInvalid
  meaning: '`kudosMakers.roles` names a role the community does not define, or `kudosMakers.groups` names a group that does not exist or that backs a badge. Nothing was stored.'
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
- vtc/badges/settings/show
- vtc/badges/create
- vtc/badges/kudos/give
---

## Abstract

The **VTC Badges — Settings — Update** Trust Task replaces the community's badge settings (VTI-BDG-004). The request carries the whole record; what it does not carry takes the shared schema's default. It is a change to the community's own settings, so it takes what any such change takes — `vtc.config.admin` and a step-up of the requester's bound to this one document — and never the action list: the settings decide what members may do with badges, which confers nothing (VTI-BDG-002).

Two of the switches are shutters, not purges. Turning `enabled` off hides every badge surface and keeps every record: no badge is given and no credential issued while it is off, and nothing is revoked, ended or erased (VTI-BDG-004). Turning `automaticBadges` off stops recomputing dynamic badge groups without ending anyone's membership of one (VTI-BDG-013). Turning either back on restores what it hid.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator's console) **SHOULD** show the administrator, before the request is signed and before any step-up gesture bound to it, what turning `enabled` or `automaticBadges` off does and does not do: every surface hidden, every record kept, nothing revoked.

A conforming **consumer** (the community) **MUST**:

1. Refuse a signer without `vtc.config.admin` with `permissionDenied` (VTI-BDG-004). **MUST NOT** accept the document from a key a member authorized for `vtc/badges` in the portal (VTI-BDG-095).
2. Require a step-up of the requester's bound to this one document, as it requires of a change to any of the community's own settings (VTI-BDG-004, VTI-OPS-050). Park nothing on the action list: the settings confer nothing.
3. Refuse with `settingsInvalid` a `kudosMakers.roles` entry that is not a role the community defines, and a `kudosMakers.groups` entry that is not a group of the community's or that is a badge group — a badge group never admits anyone to anything (VTI-BDG-012), and that includes making kudos badges.
4. Replace the settings record whole with the request's `settings`, every absent optional member at the shared schema's default, and set `updatedAt`.
5. While `enabled` is false: answer every read of a badge, an award or a wall as if no badge existed, refuse every giving and every create with that task's `badgesOff`, issue no credential, and keep every badge, award, membership and credential exactly as it was — **MUST NOT** revoke a credential, end a membership, withdraw an offer or erase an award on account of the switch (VTI-BDG-004). This task and [`vtc/badges/settings/show`](../../show/0.1/spec.md) stay answerable.
6. While `automaticBadges` is false: stop recomputing every dynamic badge group and refuse a new automatic badge ([`vtc/badges/create`](../../../create/0.1/spec.md) `settingsInvalid`), and keep every membership a recomputation made (VTI-BDG-013). Resume recomputing when it is true again.
7. While `membersMayRequest` is false: refuse marking any community badge `requestable`, and treat every badge group as `closed` — a request to join one is refused as a closed group refuses it (VTI-BDG-014).
8. Audit the change by reference — the signer and the settings record's version — never with a picture or a note (VTI-BDG-091).

## Authorization

**Authority: `vtc.config.admin`** at the community, read from the signer's access-control entry at execution time, and a step-up bound to the request, as for any change to the community's own settings (VTI-BDG-004, VTI-OPS-050). Changing the settings confers nothing on anyone and so is never parked for another administrator's consent (VTI-BDG-002). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an administrator's act, never a member's: a key the member authorized for `vtc/badges` in the portal **MUST NOT** be accepted (VTI-BDG-095).

## Definitions

- `BadgeSettings`, `KudosMakers`, `BadgeLimits` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json). The request's `settings` is `BadgeSettings` without `updatedAt`, which the community sets.
- `GroupId` — see the groups family's [`group.schema.json`](../../../../groups/_shared/0.1/group.schema.json).

## Request

The community administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Let stewards and the welcome team make kudos badges

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000211",
  "type": "https://trusttasks.org/spec/vtc/badges/settings/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T09:10:00Z",
  "payload": {
    "settings": {
      "enabled": true,
      "kudosMakers": { "policy": "roles", "roles": ["steward"], "groups": ["grp_Hs7Qm2Lk9Tn4Bv1X"] },
      "limits": { "kudosPerGiverPerDay": 10, "sameGiverSameMember": "day", "minTenureDays": 7, "kudosBadgesPerMakerPerMonth": 5, "maxPinned": 2 },
      "showKudosTotals": true,
      "automaticBadges": true,
      "membersMayRequest": true,
      "credentialValidDays": 730
    },
    "reason": "Stewards and the welcome team asked to make their own thank-you badges."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-11-06T09:10:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/settings/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The settings — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000212",
  "type": "https://trusttasks.org/spec/vtc/badges/settings/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-06T09:10:02Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000211",
  "payload": {
    "settings": {
      "enabled": true,
      "kudosMakers": { "policy": "roles", "roles": ["steward"], "groups": ["grp_Hs7Qm2Lk9Tn4Bv1X"] },
      "limits": { "kudosPerGiverPerDay": 10, "sameGiverSameMember": "day", "minTenureDays": 7, "kudosBadgesPerMakerPerMonth": 5, "maxPinned": 2 },
      "showKudosTotals": true,
      "automaticBadges": true,
      "membersMayRequest": true,
      "credentialValidDays": 730,
      "updatedAt": "2026-11-06T09:10:02Z"
    }
  }
}
```

### Refused: a kudos-maker group that is a badge group

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000213",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-06T09:10:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000211",
  "payload": {
    "code": "vtc/badges/settings/update:settingsInvalid",
    "message": "grp_Hs7Qm2Lk9Tn4Bv1X backs the badge 'Workshop host'; a badge group admits nobody to anything.",
    "retryable": false,
    "details": { "field": "kudosMakers.groups", "reason": "badge group" }
  }
}
```

## Security & Privacy

### Data carried

The settings and a reason in; the settings record out. No badge, award or member is named: `kudosMakers.roles` names roles and `kudosMakers.groups` names groups, never who holds or belongs to them. A producer **MUST NOT** place personal data in `reason` or `ext`: the reason is kept with the audit record for the life of the community.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The settings are one record for the whole community and name no member.

### Retention

**Durable.** The settings in force decide who may make and give badges and whether any badge is shown, until the next update; the community keeps the record of each change and who made it. Turning badges off keeps every badge, award, membership and credential as it was (VTI-BDG-004).

### Consent/purpose

The community deciding how its members recognise one another. **A badge confers nothing.** The settings decide who may make and give badges and within what bounds; the community **MUST NOT** read them, a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), which is why changing them is never parked on the action list.
