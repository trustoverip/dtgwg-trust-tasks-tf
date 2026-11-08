---
slug: vtc/badges/list
version: '0.1'
title: VTC Badges — List
summary: 'List the badges the caller may see — every listed active badge and the ones they hold, own or are offered; every badge for an administrator — with how many of each they hold and where they stand with it.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- group
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
  rationale: What is listed, and the caller's own count and standing on each item, depend on who asks; an unattributable caller gets no page.
sideEffects:
  level: none
  rationale: Reads badge and award records; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses badge names, descriptions, pictures' digests and settings the caller may see, and the caller's own standing with each. No other member.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/badges/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and these filters.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/show
- vtc/badges/mine/list
- vtc/badges/create
- vtc/groups/list
---

## Abstract

The **VTC Badges — List** Trust Task returns the badges the caller may see, a page at a time, each with the caller's own count of it and where they stand with it: `holder`, `offered`, `requested` or `none`. A member sees every active badge that is `listed`, and besides those the ones they hold, own, or have an open offer of. A holder of `vtc.badges.admin` sees every badge, and alone may ask for the unlisted and the retired. The caller's own badges, with what only a holder sees of them, are [`vtc/badges/mine/list`](../../mine/list/0.1/spec.md); the holders of one badge are [`vtc/badges/holders/list`](../../holders/list/0.1/spec.md).

Badge groups are left out of [`vtc/groups/list`](../../../groups/list/0.2/spec.md) unless asked for (VTI-BDG-012); this task is how a member finds a badge.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. Answer an empty page while its badge settings have `enabled` false, as if no badge existed (VTI-BDG-004).
3. Return to a member only active badges they may see: a `listed` one, or one they hold, own or have an open offer of. A badge the caller may not see **MUST** be absent, not redacted. Return to a holder of `vtc.badges.admin` every active listed badge, and every unlisted one where `includeUnlisted` is true and every retired one where `includeRetired` is true; refuse either filter from anyone else with `permissionDenied`. A retired badge is otherwise never listed (VTI-BDG-005); its holders find it in [`vtc/badges/mine/list`](../../mine/list/0.1/spec.md).
4. Apply `kind` and `mine` as filters over that set, `mine` keeping only badges the caller holds — at least one accepted award.
5. On each item set `myCount` to the caller's accepted awards of the badge, 0 where none, and `myState` to `holder` where it is above 0, else `offered` where the caller has an open offer of it, else `requested` where the caller has an undecided request to join its badge group, else `none`.
6. Return `holderCount` on a badge only where its badge group's `showMembers`, or for a kudos badge the community's active membership, admits the caller, counted over holders who have not hidden it (VTI-BDG-030, VTI-BDG-033).
7. Order by `name`, then `badgeId`, and bind `nextCursor` to the caller and the filters.

## Authorization

**Standing in the community, bounded by each badge's visibility**: an active member sees what a member sees; a holder of `vtc.badges.admin` sees every badge; an owner — `vtc.groups.manage` for the badge group, unqualified or qualified `group:<groupId>` — sees the badges they own whether or not listed. The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A member's read is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it. `includeUnlisted` and `includeRetired` are an administrator's filters, and a document signed with such a key that carries either is refused `permissionDenied` as any member's would be.

## Definitions

- `Badge`, `BadgeKind` — see [`badge.schema.json`](../../_shared/0.1/badge.schema.json).
- **Holder** — a member with at least one accepted award of the badge, hidden or not.

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### The community badges a member could earn

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000251",
  "type": "https://trusttasks.org/spec/vtc/badges/list/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T09:50:00Z",
  "payload": {
    "kind": "community",
    "limit": 50
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-06T09:50:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The community badges — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000252",
  "type": "https://trusttasks.org/spec/vtc/badges/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T09:50:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000251",
  "payload": {
    "items": [
      {
        "badge": {
          "badgeId": "bdg_Nf2Rk7Lp9Xs3Qw5T",
          "kind": "community",
          "name": "One year aboard",
          "description": "A year in the community.",
          "criteria": "Twelve months of membership. Given by the community itself.",
          "image": { "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "mediaType": "image/png", "size": 52200 },
          "repeatable": false,
          "requestable": false,
          "listed": true,
          "shareableOutside": false,
          "groupId": "grp_Lk8Tn2Vx4Rm9Qs7H",
          "state": "active",
          "holderCount": 213,
          "createdAt": "2026-03-01T08:00:00Z"
        },
        "myCount": 1,
        "myState": "holder"
      },
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
          "holderCount": 12,
          "createdAt": "2026-11-06T09:00:02Z"
        },
        "myCount": 0,
        "myState": "requested"
      }
    ]
  }
}
```

### Refused: a member asks for the unlisted badges

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000253",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T09:51:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000254",
  "payload": {
    "code": "permissionDenied",
    "message": "includeUnlisted is for badge administrators; the badges you hold or own are listed to you without it.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

Filters in; badge records and the caller's own standing with each out. No other member's identity: a badge's `holderCount` is a number over holders who show it, and who they are is [`vtc/badges/holders/list`](../../holders/list/0.1/spec.md), projected through the directory.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. A badge's picture is the community's own re-encoding, so its digest links nothing across communities (VTI-BDG-050). The page is scoped to the caller: an unlisted badge appears to its holders, its owners and the offered alone, so its presence on a page tells the caller only what they already are to it.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

A member finding what the community gives and where they stand with each. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and a badge group is never bound to a capability or a role (VTI-BDG-012).
