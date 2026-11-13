---
slug: vtc/badges/holders/list
version: '0.1'
title: VTC Badges — Holders — List
summary: Who holds a badge, a page at a time, each holder projected as the directory projects them to the caller, with hidden holders left out and the count stated over those shown.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- holders
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
  rationale: Holders are shown to active members the badge's holder visibility admits, and what is shown of each holder depends on who asks. Without a proof the caller is nobody, and nothing is answered.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: A read with no side effects; a replay discloses only what the caller was already entitled to see.
sideEffects:
  level: none
  rationale: Reads the badge group's membership, or a kudos badge's accepted awards; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses which members hold the badge and show it, with the directory fields each chose to show the caller.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/badges/holders/list:notFound
  meaning: No badge whose holders the caller may see has this identifier. Conflates an unknown badge, one the caller may not see, badges being off, and a badge the caller may see whose holder visibility does not admit them.
  retryable: false
- code: vtc/badges/holders/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and badge.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/show
- vtc/badges/wall/show
- vtc/badges/awards/list
- vtc/badges/mine/hide
- vtc/groups/members/list
- vtc/directory/entries/show
---

## Abstract

The **VTC Badges — Holders — List** Trust Task is the list behind a badge: click a badge and see who else has it. It pages through the members who hold the badge and show it, each as the directory projects them to the caller, with how many times they hold it and since when. A holder who hid the badge is left out, and `total` counts only the holders shown (VTI-BDG-030, VTI-BDG-033).

Who may see a badge's holders is the badge's own setting. For a community badge it is its badge group's `showMembers` — the badge's holders are the group's members (VTI-BDG-011) — so this task answers exactly whom [`vtc/groups/members/list`](../../../../groups/members/list/0.2/spec.md) on that group would answer, less the holders who hid the badge. For a kudos badge, every active member may see its holders. A holder list is never inlined into a read of the badge: [`vtc/badges/show`](../../../show/0.1/spec.md) carries `holderCount` only (VTI-BDG-033).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer as if no badge existed while its badge settings have `enabled` false (VTI-BDG-004).
2. Answer only an active member — signing with their membership DID or with a key they delegated for this community under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) whose `signingScope` covers `vtc/badges`, read as its member — who may see the badge (an active badge that is `listed`, or that they hold, own or have an open offer of; any badge to a holder of `vtc.badges.admin`; a retired badge to a holder of it) **and** whom the badge's holder visibility admits: for a community badge, the badge group's `showMembers`, read for the caller as [`vtc/groups/members/list`](../../../../groups/members/list/0.2/spec.md) reads it; for a kudos badge, every active member (VTI-BDG-033). Refuse everyone else with `notFound`, alike, so the refusal does not tell a caller that the badge exists.
3. List each member who holds at least one accepted award of the badge and has not hidden it (VTI-BDG-030): their `entryId`, `count` of accepted awards, `since` the first was accepted and `latestAt` the last was; newest `latestAt` first. Leave out members whose every award is revoked, and members who departed.
4. Return each holder's `fields` projected exactly as [`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md) would project them for this caller; a holder the caller may see nothing of appears with `fields` absent. Never a DID (VTI-BDG-033).
5. State `total` as the number of holders the list shows — those who have not hidden the badge — and the same number as the badge's `holderCount` for this caller.
6. Bind `nextCursor` to the caller and the badge, refusing any other cursor with `cursorInvalid`.
7. **SHOULD** bound the rate at which one caller lists holders, as it bounds browsing the directory.

## Authorization

**Authority: the badge's holder visibility, decided now for the caller's standing**, and each holder's own directory visibility for what is shown of them. A community badge's `holdersVisibleTo` is its badge group's `showMembers`, so nothing here shows a member list the group would not. The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A read is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it.

## Definitions

- `BadgeId` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- `EntryId`, `EntryFields` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member looks at who else holds a badge

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000701",
  "type": "https://trusttasks.org/spec/vtc/badges/holders/list/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T14:00:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
    "limit": 50
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-06T14:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/holders/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The holders — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000702",
  "type": "https://trusttasks.org/spec/vtc/badges/holders/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T14:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000701",
  "payload": {
    "total": 2,
    "items": [
      {
        "entryId": "dir_BenOkafor0000000",
        "count": 3,
        "since": "2026-03-14T19:30:00Z",
        "latestAt": "2026-10-31T20:10:00Z",
        "fields": {
          "displayName": "Ben",
          "avatar": { "sha256": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b", "mediaType": "image/webp", "size": 18342 }
        }
      },
      {
        "entryId": "dir_SamNguyen0000000",
        "count": 1,
        "since": "2026-09-02T17:45:00Z",
        "latestAt": "2026-09-02T17:45:00Z"
      }
    ]
  }
}
```

The second holder shows the caller nothing of their card; the caller sees that a member holds the badge, and no more.

### Refused: the badge shows its holders to holders only

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000703",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T14:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000701",
  "payload": {
    "code": "vtc/badges/holders/list:notFound",
    "message": "No badge bdg_W5k2N8xRb3Qz7TpL.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A badge identifier in; which members hold the badge and show it, each with their count, their dates and the directory fields they chose to show the caller. A member's DID only where they made `did` visible to the caller. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Holding a badge is a statement the holder chose to make by accepting and not hiding it (VTI-BDG-020, VTI-BDG-030); the list says it to the audience the badge's holder visibility admits, which for a community badge is the badge group's own `showMembers`. Nothing here reaches outside the community: the public artefacts of a badge its holders may show outside never list holders (VTI-BDG-071).

### Retention

**Transient.** Nothing beyond the audit record of the request.

### Consent/purpose

Members seeing who else holds a badge, which is what a badge on a card invites. A holder who would rather not be on the list hides the badge ([`vtc/badges/mine/hide`](../../../mine/hide/0.1/spec.md)) and leaves it, and its count, at once. **A badge confers nothing**: the community **MUST NOT** read the holder list, or the group it is drawn from, in any authorization decision of its own (VTI-BDG-002, VTI-BDG-012).
