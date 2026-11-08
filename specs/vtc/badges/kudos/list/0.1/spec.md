---
slug: vtc/badges/kudos/list
version: '0.1'
title: VTC Badges — Kudos — List
summary: "List the kudos the signer received or gave, a page at a time, each with its badge and the other party as the directory projects them; or, for a badge administrator, every giving of one kudos badge."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- kudos
parties:
- role: member or badge administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: What is listed is the signer's own givings and receivings, or an administrator's view of everyone's; without an attributable caller there is nothing to answer.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Not consequential, so SPEC §7.3 item 17 sets no floor; REQUIRED because a delegated portal key signs for one session (VTI-BDG-095) and the community places the document in that session by its `issuedAt`.
sideEffects:
  level: none
  rationale: Reads award records; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses to a member who gave them kudos and whom they gave kudos to, with each note; to a badge administrator, every giver and recipient of one badge. Each party as the directory projects them to the caller.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/badges/kudos/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and these filters.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/kudos/give
- vtc/badges/mine/list
- vtc/badges/mine/accept
- vtc/badges/mine/decline
- vtc/badges/awards/list
---

## Abstract

The **VTC Badges — Kudos — List** Trust Task pages through kudos (VTI Community Badges chapter, VTI-BDG-042). A member lists what they **received** — each giving with its badge, its giver as the directory projects them, and the giver's note — and what they **gave**, each with its recipient. A holder of `vtc.badges.admin` lists, with `all` and a `badgeId`, every giving of that kudos badge, giver and recipient both. Every other viewer sees only the count a holder shows (VTI-BDG-030, VTI-BDG-043), and never through this task.

A kudos giving is an award ([`vtc/badges/kudos/give`](../../give/0.1/spec.md)); the open ones a member received are also among the offers of [`vtc/badges/mine/list`](../../../mine/list/0.1/spec.md), where they are decided. A community badge's awards are the owners' [`vtc/badges/awards/list`](../../../awards/list/0.1/spec.md), never here.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a signer who is not an active member with `permissionDenied`, and refuse `all` from a signer without `vtc.badges.admin` with `permissionDenied` (VTI-BDG-042). While its badge settings have `enabled` false, answer an empty page and change nothing (VTI-BDG-004).
2. For `direction` `received` — the default — list the awards of kudos badges whose `entryId` is the signer, in every state; for `given`, those whose `givenBy` is the signer. Return only awards of kudos badges: a community badge's awards are listed to its owners with [`vtc/badges/awards/list`](../../../awards/list/0.1/spec.md) and to their holder with [`vtc/badges/mine/list`](../../../mine/list/0.1/spec.md).
3. Return each item with its `badge`, which the signer may see for being a party to the award, even where it is retired or unlisted (VTI-BDG-005); and with `counterpart` the other party's directory fields — the giver for `received`, the recipient for `given` — projected exactly as [`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md) would project them for the signer, absent where the signer may see none or the party has departed (VTI-BDG-090). The award's `note` is returned: it is written for the recipient and seen by the giver (VTI-BDG-042).
4. With `all` and `badgeId`, for a holder of `vtc.badges.admin`: refuse with `notFound` a `badgeId` that names no kudos badge; otherwise list every award of it, in every state, each with `giver` and `recipient` the parties' fields projected for the signer, and `counterpart` absent (VTI-BDG-042). `direction` is ignored.
5. Narrow the list to `badgeId` and `state` where present, and return it newest first by `offeredAt`.
6. Bind `nextCursor` to the caller and the filters, and refuse with `cursorInvalid` a cursor it did not issue for them.
7. **MUST NOT** list another member's kudos to anyone but a holder of `vtc.badges.admin` using `all`: a member who is neither party to an award sees at most the recipient's count, and only where the recipient shows the badge (VTI-BDG-030, VTI-BDG-043).

## Authorization

**Authority: being a party to the kudos.** A member lists only awards they gave or received; the task has no `entryId`. **`all` takes `vtc.badges.admin`** at the community, read from the signer's access-control entry at execution time. The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); it never establishes that they may.

A member's own listing is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms. The administrator's `all` **MUST NOT** be accepted from such a key.

## Definitions

- `BadgeId`, `Award`, `AwardState`, `Badge` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- `EntryFields` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The member or badge administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### The kudos I received

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000361",
  "type": "https://trusttasks.org/spec/vtc/badges/kudos/list/0.1#request",
  "issuer": "did:example:ben",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:40:00Z",
  "payload": {
    "direction": "received",
    "limit": 20
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/kudos/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The kudos I received — answered

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000362",
  "type": "https://trusttasks.org/spec/vtc/badges/kudos/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:ben",
  "issuedAt": "2026-11-06T10:40:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000361",
  "payload": {
    "items": [
      {
        "award": {
          "awardId": "awd_K3m8Qp2Vt7Xz4Ln9",
          "badgeId": "bdg_Kd0sCl4mpWh1sp3r",
          "entryId": "dir_B3nOk4f0rRq9Wz2X",
          "givenBy": "dir_Tm4sV3g4K9xQ2bLp",
          "note": "Found the long sash clamps behind the bandsaw. Saved the glue-up.",
          "state": "offered",
          "offeredAt": "2026-11-06T10:30:01Z",
          "expiresAt": "2026-12-06T10:30:01Z"
        },
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
        "counterpart": {
          "displayName": "Tomás"
        }
      },
      {
        "award": {
          "awardId": "awd_P9r4Wn6Yc2Hs8Vq1",
          "badgeId": "bdg_Kd0sCl4mpWh1sp3r",
          "entryId": "dir_B3nOk4f0rRq9Wz2X",
          "givenBy": "dir_Pr1yaN4t7Rj2Qw8K",
          "state": "accepted",
          "offeredAt": "2026-10-28T19:00:00Z",
          "decidedAt": "2026-10-28T21:12:00Z"
        },
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
        "counterpart": {
          "displayName": "Priya"
        }
      }
    ]
  }
}
```

### Refused: `all` without `vtc.badges.admin`

```json
{
  "id": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000363",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:ben",
  "issuedAt": "2026-11-06T10:40:01Z",
  "threadId": "urn:uuid:5e7a1c2d-9b4f-4e8a-a1c3-000000000361",
  "payload": {
    "code": "permissionDenied",
    "message": "Listing every giving of a badge takes vtc.badges.admin.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

Filters in. Out: awards the signer is a party to, with each note and the other party's projected directory fields — or, to a badge administrator, every giving of one badge with both parties projected. Never a credential or a key: a kudos badge carries none. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. A counterpart appears as the directory projects them to the caller, so this task joins nothing the directory would not; `givenBy` and `entryId` on the award are entry identifiers, which survive rotation and reveal no DID. The administrator's `all` is the one view that joins givers to recipients across the community (VTI-BDG-042); a member's own list shows them only their own pairs.

### Retention

**Transient.** Nothing beyond the audit record of the request. The awards listed are kept by the community as both parties' history regardless of this read (VTI-BDG-090).

### Consent/purpose

A member seeing what they gave and received; an administrator running the community's kudos. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and a count of kudos given or received is not a standing.
