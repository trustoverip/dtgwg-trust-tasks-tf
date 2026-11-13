---
slug: vtc/badges/show
version: '0.1'
title: VTC Badges — Show
summary: 'Read one badge the caller may see: the badge, how many of it the caller holds and where they stand with it, its owners or its maker, and the credential regime it carries.'
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
  rationale: Whether the badge is shown at all, and the caller's own count and standing on it, depend on who asks; an unattributable caller gets nothing.
sideEffects:
  level: none
  rationale: Reads a badge record and the caller's awards of it; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses one badge, its owners' or maker's entry identifiers, and the caller's own standing with it.
retention:
  class: transient
  rationale: A read.
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/list
- vtc/badges/holders/list
- vtc/badges/update
- vtc/groups/join
---

## Abstract

The **VTC Badges — Show** Trust Task reads one badge, as [`vtc/badges/list`](../../list/0.1/spec.md) would show it, with what a member looking at a badge wants to know: how many of it they hold and where they stand with it, who gives it — the `owners` of a community badge, whom a request for it goes to; the `maker` of a kudos badge — and what a holder gets: the `credentialRegime`, `none` for a kudos badge, `inside` for a community badge shown only within the community, `outside` for one its holders may show anywhere (VTI-BDG-041, VTI-BDG-061, VTI-BDG-062). Its holders are a listing of their own, [`vtc/badges/holders/list`](../../holders/list/0.1/spec.md), never inline (VTI-BDG-033).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. Answer `notFound` while its badge settings have `enabled` false, as if no badge existed (VTI-BDG-004).
3. Answer `notFound` alike for a badge that does not exist and one the caller may not see: a member sees an active badge that is `listed`, or that they hold, own or have an open offer of; a holder sees a retired badge they hold; a holder of `vtc.badges.admin` sees every badge (VTI-BDG-005).
4. Set `myCount` to the caller's accepted awards of the badge, 0 where none, and `myState` as [`vtc/badges/list`](../../list/0.1/spec.md) item 5 sets it. Where the caller holds any, answer them in `myAwards`, newest first, each with its `shared` where the caller shows it outside the community ([`vtc/badges/awards/share`](../../awards/share/0.1/spec.md)) — the caller's own awards only, never another member's.
5. Return `holderCount` on the badge only where its badge group's `showMembers`, or for a kudos badge the community's active membership, admits the caller, counted over holders who have not hidden it (VTI-BDG-030, VTI-BDG-033).
6. For a community badge, return `owners` — the entry identifiers of its badge group's managers — to every caller who may see the badge, so a member knows whom a request goes to (VTI-BDG-014); for a kudos badge, return `maker` — its `createdBy` — and no `owners`.
7. Derive `credentialRegime` from the badge alone: `none` for a kudos badge, `outside` for a community badge with `shareableOutside` true, `inside` otherwise.

## Authorization

**Standing in the community, bounded by the badge's visibility**, as for [`vtc/badges/list`](../../list/0.1/spec.md). The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A member's read is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it.

## Definitions

- `Badge`, `BadgeId` — see [`badge.schema.json`](../../_shared/0.1/badge.schema.json).
- `EntryId` — see the directory family's [`directory-card.schema.json`](../../../directory/_shared/0.1/directory-card.schema.json).
- **Credential regime** — what a holder of the badge is issued: nothing (`none`, a kudos badge); a short-lived credential renewed with no status, shown only within the community (`inside`, VTI-BDG-061); a long-lived credential with a status in the community's own status set, which its holders may show outside (`outside`, VTI-BDG-062).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read a badge

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000261",
  "type": "https://trusttasks.org/spec/vtc/badges/show/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:00:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-06T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Read a badge — answered

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000262",
  "type": "https://trusttasks.org/spec/vtc/badges/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T10:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000261",
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
      "holderCount": 12,
      "createdAt": "2026-11-06T09:00:02Z"
    },
    "myCount": 0,
    "myState": "requested",
    "owners": ["dir_Pr1yaNatarajan00", "dir_BenOkafor0000000"],
    "credentialRegime": "outside"
  }
}
```

### Refused: the badge is unlisted and the caller neither holds, owns nor is offered it

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000263",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T10:01:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000261",
  "payload": {
    "code": "notFound",
    "message": "No badge you may see has this identifier.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A badge identifier in; the badge, the caller's own standing with it, its owners' or maker's entry identifiers and its credential regime out. No holder: `holderCount` is a number over holders who show it, and who they are is [`vtc/badges/holders/list`](../../holders/list/0.1/spec.md), projected through the directory.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. `owners` and `maker` name members by entry identifier to every member who may see the badge, as the directory already names them; what a member may see *of* an owner is the directory's projection ([`vtc/directory/entries/show`](../../../directory/entries/show/0.1/spec.md)), not this task's. A badge's picture is the community's own re-encoding, so its digest links nothing across communities (VTI-BDG-050).

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

A member looking at a badge before asking for it, or at what holding it means. **A badge confers nothing.** The community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and a badge group is never bound to a capability or a role (VTI-BDG-012).
