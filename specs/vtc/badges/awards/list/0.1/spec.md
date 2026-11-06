---
slug: vtc/badges/awards/list
version: '0.1'
title: VTC Badges — Awards — List
summary: 'The owners'' view of a badge: every award of it, open, accepted, declined, withdrawn, expired or revoked, a page at a time, each recipient projected as the directory projects them and marked where they hid the badge.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- award
parties:
- role: badge owner or administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The list is answered only to a badge's owners and to badge administrators, and which badges a caller owns depends on who asks. Without a proof the caller is nobody, and nothing is answered.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: A read with no side effects; a replay discloses only what the caller was already entitled to see.
sideEffects:
  level: none
  rationale: Reads award records; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses to a badge's owners whom it was offered to and who holds it, including holders who hid it, with the directory fields each recipient shows the caller.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/badges/awards/list:notFound
  meaning: No badge the caller owns or administers has this identifier. Conflates an unknown badge, one the caller may not see, badges being off, and a badge the caller may see but does not own.
  retryable: false
- code: vtc/badges/awards/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller, badge and filters.'
  retryable: false
related:
- vtc/badges/awards/offer
- vtc/badges/awards/withdraw
- vtc/badges/awards/revoke
- vtc/badges/holders/list
- vtc/badges/kudos/list
- vtc/groups/members/list
---

## Abstract

The **VTC Badges — Awards — List** Trust Task is how a badge's owners run it. It pages through every award of one badge — offers waiting on their recipients, accepted awards, and the declined, withdrawn, expired and revoked ones — newest first, filtered by state or by recipient where asked. Each item carries the award, whether its recipient has hidden the badge, and the recipient as the directory projects them to the caller.

This is the view VTI-BDG-030 reserves for owners and administrators: a holder who hides a badge is left out of its wall, its holder list and its count, but the owners who gave it still see whom they gave it to, marked `hidden`. For a kudos badge, which has no owners, only a holder of `vtc.badges.admin` may call this, and sees every giving (VTI-BDG-042). Members see a badge's holders with [`vtc/badges/holders/list`](../../../holders/list/0.1/spec.md), and their own kudos with [`vtc/badges/kudos/list`](../../../kudos/list/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer as if no badge existed while its badge settings have `enabled` false (VTI-BDG-004).
2. Answer only an owner of the badge — a member who manages its badge group, by `vtc.groups.manage` unqualified or qualified `group:<groupId>` — or a holder of `vtc.badges.admin`, and refuse everyone else with `notFound`, whether or not they may see the badge. A kudos badge has no owners: only a holder of `vtc.badges.admin` is answered for one.
3. List every award of the badge in every state, newest `offeredAt` first, narrowed to `state` and to `entryId` where the request names them. For a kudos badge, list every giving (VTI-BDG-042).
4. Mark each item `hidden` true exactly where its recipient has hidden the badge (VTI-BDG-030), and `false` otherwise.
5. Return each recipient's `fields` projected exactly as [`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md) would project them for this caller; a recipient the caller may see nothing of appears with `fields` absent and their `entryId` on the award. Never a DID.
6. Bind `nextCursor` to the caller, the badge and the filters, refusing any other cursor with `cursorInvalid`.
7. Audit the read by reference — the caller and the badge (VTI-BDG-091).

## Authorization

**Authority: ownership of the badge, or `vtc.badges.admin`, read now.** An owner is shown the awards because the badge's owners answer for whom it is given to, and ownership is the badge group's management (VTI-BDG-011), read from the caller's access-control entry at execution time; a holder of `vtc.badges.admin` because they run the community's badges. The projection of each recipient is the directory's, decided for this caller. The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

An owners' read **MUST NOT** be accepted from a key a member delegated to a portal (VTI-BDG-095): the signer is resolved as the member or administrator themselves.

## Definitions

- `BadgeId`, `Award`, `AwardState` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- `EntryId`, `EntryFields` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The badge owner or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An owner lists the open offers of a badge

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000601",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/list/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T13:00:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
    "state": "offered",
    "limit": 50
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-06T13:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/awards/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The open offers — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000602",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T13:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000601",
  "payload": {
    "items": [
      {
        "award": {
          "awardId": "awd_K3mT9pQx2Lw8Rv5N",
          "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
          "entryId": "dir_SamNguyen0000000",
          "givenBy": "dir_Pr1yaNatarajan00",
          "note": "For the Saturday joinery workshop.",
          "state": "offered",
          "offeredAt": "2026-11-05T18:00:00Z",
          "expiresAt": "2026-12-05T18:00:00Z"
        },
        "hidden": false,
        "fields": {
          "displayName": "Sam N."
        }
      }
    ]
  }
}
```

### Refused: a member who may see the badge but does not own it

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000603",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T13:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000601",
  "payload": {
    "code": "vtc/badges/awards/list:notFound",
    "message": "No badge bdg_W5k2N8xRb3Qz7TpL.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A badge identifier and filters in; the badge's awards out — whom it was offered to, who holds it, who declined, who hid it — with the giver's note and the directory fields each recipient shows the caller. This is the most a badge discloses about its holders, and it goes to its owners and administrators only. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. A recipient's DID appears only where they made the directory's `did` field visible to the caller. An `entryId` filter lets an owner ask whether one member was ever offered the badge; that is what owners know anyway, and the answer is as the badge's own record has it.

### Retention

**Transient.** Nothing beyond the audit record of the request, which names the caller and the badge.

### Consent/purpose

The badge's owners running the badge: seeing which offers stand, whom they gave it to, and what was declined or taken back (VTI-BDG-030). A holder's hiding is shown to them because owners answer for the badge, and to nobody else. **A badge confers nothing**: the community **MUST NOT** read an award in any authorization decision of its own (VTI-BDG-002).
