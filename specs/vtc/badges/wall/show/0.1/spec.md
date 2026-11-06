---
slug: vtc/badges/wall/show
version: '0.1'
title: VTC Badges — Wall — Show
summary: 'A member''s badge wall as a viewer sees it: the badges they hold and show, pinned first and then in their own order, with their kudos total where the community shows one — answered only where the directory shows the viewer that member''s card and the member shows their badges.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- wall
- projection
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
  rationale: A wall is shown to the viewers the directory shows the member's card to, and whether that includes the caller depends on who asks. Without a proof the caller is nobody, and nothing is answered.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: A read with no side effects; a replay discloses only what the caller was already entitled to see.
sideEffects:
  level: none
  rationale: Reads a member's badges and their wall settings; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses which badges one member holds and chose to show, their counts, and their kudos total where the community shows one, to a viewer the directory and the member admit.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/badges/wall/show:notFound
  meaning: No wall the caller may see is at this entry. Conflates an unknown entry, an entry whose card the directory does not show the caller, a member whose wall visibility is `nobody`, badges being off, and a caller who is not an active member — the same answer the directory gives for the card.
  retryable: false
related:
- vtc/badges/holders/list
- vtc/badges/mine/list
- vtc/badges/mine/arrange
- vtc/badges/mine/hide
- vtc/badges/images/get
- vtc/directory/entries/show
---

## Abstract

The **VTC Badges — Wall — Show** Trust Task reads the badges on a member's directory card: the ones they hold and have not hidden, pinned ones first and then in the order they arranged ([`vtc/badges/mine/arrange`](../../../mine/arrange/0.1/spec.md)), each with how many times they hold it and since when. Where the community shows kudos totals, the wall carries the member's total too (VTI-BDG-043).

A wall is shown to exactly the viewers the directory shows the card to, and only where the member shows their badges: their `WallVisibility` is the member's own setting, `members` by default and never wider than the card (VTI-BDG-031). A viewer the directory would not show the entry gets the same `notFound` the directory gives, so a wall is never a way to learn that an entry exists. The member always sees their own wall; their fuller view, hidden badges and all, is [`vtc/badges/mine/list`](../../../mine/list/0.1/spec.md).

**A wall is the holder's statement.** A badge the viewer could not otherwise see — one that is not `listed` and that the viewer does not hold — still appears on a member's wall when that member shows it: the wall says what the member chose to say about themselves, and the badge's own listing setting decides only who finds it in the community's badge list. The viewer may then read the badge's picture with [`vtc/badges/images/get`](../../../images/get/0.1/spec.md), which admits a badge seen on a wall.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer as if no badge existed while its badge settings have `enabled` false (VTI-BDG-004): an empty wall with no `kudosTotal` to a viewer the directory admits, so that off is a shutter and not a signal.
2. Answer only an active member — signing with their membership DID or with a key they delegated for this community under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) whose `signingScope` covers `vtc/badges`, read as its member — and only where [`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md) would show that caller the entry's card (VTI-BDG-031), and the entry's member has `WallVisibility` `members` or is the caller themselves. Refuse everyone else with `notFound`, exactly as the directory answers an entry it does not show.
3. List as `entries` every badge the member holds at least one accepted award of and has not hidden (VTI-BDG-030) — a retired badge included, since it is the holder's own history (VTI-BDG-005) — pinned entries first, then in the member's own order (VTI-BDG-032), each with `count`, `since` and `latestAt`. Show a badge the viewer could not see in the badge list, where the member shows it. Carry each badge's `holderCount` only where the viewer may see the badge's holders.
4. Carry `kudosTotal` exactly when the settings' `showKudosTotals` is true, counted over the kudos awards the member accepted and has not hidden (VTI-BDG-043); leave it absent otherwise, never zero in its place.
5. Carry `visibility` as the member set it.
6. **SHOULD** bound the rate at which one caller reads walls, as it bounds browsing the directory.

## Authorization

**Authority: the directory's projection, decided now, and then the member's own setting.** The caller's identity matters only as an input to whether the directory shows them the entry; the member's `WallVisibility` narrows from there and never widens. Nothing here grants access beyond what [`entries/show`](../../../../directory/entries/show/0.1/spec.md) would show the same caller. The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A read is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it.

## Definitions

- `WallEntry`, `WallVisibility`, `Badge` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- `EntryId` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member opens another member's card

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000801",
  "type": "https://trusttasks.org/spec/vtc/badges/wall/show/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T15:00:00Z",
  "payload": {
    "entryId": "dir_BenOkafor0000000"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-06T15:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/wall/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The wall — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000802",
  "type": "https://trusttasks.org/spec/vtc/badges/wall/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T15:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000801",
  "payload": {
    "visibility": "members",
    "kudosTotal": 14,
    "entries": [
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
          "holderCount": 2,
          "createdAt": "2026-03-01T09:00:02Z",
          "createdBy": "dir_AnnaReyes0000000"
        },
        "count": 3,
        "since": "2026-03-14T19:30:00Z",
        "latestAt": "2026-10-31T20:10:00Z",
        "pinned": true
      },
      {
        "badge": {
          "badgeId": "bdg_Kd8sPq2Vn4Xm7Rt1",
          "kind": "kudos",
          "name": "Clamp whisperer",
          "description": "Found the good clamps when nobody else could.",
          "image": { "sha256": "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae", "mediaType": "image/webp", "size": 18342 },
          "repeatable": true,
          "listed": true,
          "state": "active",
          "createdAt": "2026-11-06T09:05:01Z",
          "createdBy": "dir_TomasVega0000000"
        },
        "count": 5,
        "since": "2026-11-06T12:00:00Z",
        "latestAt": "2026-11-06T14:30:00Z",
        "pinned": false
      }
    ]
  }
}
```

### Refused: the directory does not show the caller this entry

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000803",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T15:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000801",
  "payload": {
    "code": "vtc/badges/wall/show:notFound",
    "message": "No entry dir_BenOkafor0000000.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

An entry identifier in; the badges one member holds and shows, their counts and dates, and the member's kudos total where the community shows one. No award, no giver and no note travels: a wall shows badges, not who gave them. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The entry identifier reveals nothing about the member's DID, and a viewer can only have it from a projection that showed it. A wall is answered to exactly the viewers the card is, so it joins nothing to the card that the card did not already show them; a kudos total, where shown, is a count and names no giver.

### Retention

**Transient.** Nothing beyond the audit record of the request.

### Consent/purpose

A member showing the badges they chose to show, to the people they chose to show their card to (VTI-BDG-031). Every entry on a wall is there by two choices of the member's — accepting the award and not hiding the badge — and the wall's order and pins are theirs (VTI-BDG-032). **A badge confers nothing**: the community **MUST NOT** read a wall, a badge or an award in any authorization decision of its own (VTI-BDG-002).
