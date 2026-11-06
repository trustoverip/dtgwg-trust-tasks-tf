---
slug: vtc/badges/settings/show
version: '0.1'
title: VTC Badges — Settings — Show
summary: 'Read the community''s badge settings: whether badges are on, who may make a kudos badge, the limits, whether kudos totals show, whether automatic badges run and whether members may ask for a badge.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- settings
parties:
- role: member or administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The settings are answered to members and administrators only, never to an application entry; the signer's standing decides whether they are answered at all.
sideEffects:
  level: none
  rationale: Reads the settings record; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses the community's badge settings — policy, limits and switches — never a badge, an award or a member.
retention:
  class: transient
  rationale: A read.
errorCodes: []
related:
- vtc/badges/settings/update
- vtc/badges/create
- vtc/badges/list
---

## Abstract

The **VTC Badges — Settings — Show** Trust Task reads the community's badge settings (VTI-BDG-004): whether badges are offered at all, who may make a kudos badge, the limits of VTI-BDG-040, whether a member's kudos total is shown, whether automatic badges run, and whether any community badge may be marked as one members may ask for. An administrator reads them to change them with [`vtc/badges/settings/update`](../../update/0.1/spec.md); a member's portal reads them to know whether the member may make a kudos badge with [`vtc/badges/create`](../../../create/0.1/spec.md), and within what limits, before offering to.

Every member gets every field. The settings describe the community's rules, not any member, and a portal that cannot read `kudosMakers` and `limits` cannot tell a member what they may do before they try.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`. An `application` entry is refused the same way: the settings are the community's members' to read, and an application is not a member.
2. Answer the settings in force, every field, to every caller it answers at all — a member and an administrator read the same record (VTI-BDG-004).
3. Answer them whether or not `enabled` is true. This task and [`vtc/badges/settings/update`](../../update/0.1/spec.md) are exempt from the rule that reads answer as if no badge existed while badges are off (VTI-BDG-004): the switch must be readable to be turned back on, and a portal reads `enabled` to know whether to show a badge surface at all.
4. **MUST NOT** read the settings in any authorization decision of its own (VTI-BDG-002): they decide what a member may do with badges, which confers nothing.

## Authorization

**Standing in the community: an active member or an administrator**, read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A member's read is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it.

## Definitions

- `BadgeSettings`, `KudosMakers`, `BadgeLimits` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member's portal reads the settings

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000201",
  "type": "https://trusttasks.org/spec/vtc/badges/settings/show/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T09:00:00Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:tomas#key-1",
    "created": "2026-11-06T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/settings/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The settings — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000202",
  "type": "https://trusttasks.org/spec/vtc/badges/settings/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T09:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000201",
  "payload": {
    "settings": {
      "enabled": true,
      "kudosMakers": { "policy": "roles", "roles": ["steward"], "groups": ["grp_Hs7Qm2Lk9Tn4Bv1X"] },
      "limits": { "kudosPerGiverPerDay": 10, "sameGiverSameMember": "day", "minTenureDays": 7, "kudosBadgesPerMakerPerMonth": 5, "maxPinned": 2 },
      "showKudosTotals": true,
      "automaticBadges": true,
      "membersMayRequest": true,
      "credentialValidDays": 730,
      "updatedAt": "2026-11-01T10:00:00Z"
    }
  }
}
```

### Refused: an application entry asks

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000203",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:bot",
  "issuedAt": "2026-11-06T09:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000201",
  "payload": {
    "code": "permissionDenied",
    "message": "Badge settings are read by members and administrators; this entry is an application.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

Nothing in; the community's badge settings out. No badge, award or member is named: `kudosMakers.roles` names roles and `kudosMakers.groups` names groups, never who holds or belongs to them.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The settings are one record for the whole community, the same to every caller, so reading them tells the community nothing about the reader it did not know from the signature.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

A member learning what the community lets them do with badges before they try. **A badge confers nothing.** The settings decide who may make and give badges and within what bounds; the community **MUST NOT** read them, a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002).
