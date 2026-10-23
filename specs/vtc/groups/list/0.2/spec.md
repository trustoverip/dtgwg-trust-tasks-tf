---
slug: vtc/groups/list
version: '0.2'
title: VTC Groups — List
summary: List the community groups the caller may see — all of them for an administrator, the visible ones for a member — optionally only those they belong to or could join.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
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
  requirement: RECOMMENDED
  rationale: What is listed depends on who asks; an attributable caller gets a correctly scoped page.
sideEffects:
  level: none
  rationale: Reads group records; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses group names, descriptions and settings the caller may see, and their own membership of each.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/groups/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and these filters.'
  retryable: false
related:
- vtc/groups/show
- vtc/groups/join
---

## Abstract

The **VTC Groups — List** Trust Task returns the community groups the caller may see. A group's `visibility` decides who sees that it exists; its `showMembers` decides, separately, who sees its member count and list. Administrators holding `vtc.groups.manage` see every group.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Changes from 0.1

The response's **`nextCursor`** changes type from `["string", "null"]` to **`"string"`** (still at most 1024 characters). It is present when more items remain and **absent** on the last page; a `null` value now fails validation. A producer that detected the last page by `nextCursor === null` **MUST** detect it by absence instead.

This is the registry-wide pagination convention: a continuation token is an optional string whose absence marks the last page, so there is one spelling of "no more pages" instead of two.

The request's `cursor` and `limit` already followed the convention and are unchanged on the wire — same types and bounds; only their schema descriptions are reworded to the registry's paging text. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. Return only groups whose `visibility` admits the caller: `members` to every active member, `groupMembers` to the group's members and managers, `admins` to managers only. A group the caller may not see **MUST** be absent, not redacted.
3. Return `memberCount` only where `showMembers` admits the caller, and `managers` and `credentialAuthorityValidUntil` only to the group's managers.
4. Bind `nextCursor` to the caller and the filters.

## Authorization

**Authority: standing in the community, bounded by each group's visibility.** Holders of `vtc.groups.manage` — unqualified, or qualified `group:<groupId>` for that group — see what managers see. The proof, where present, identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `Group` — see [`group.schema.json`](../../_shared/0.1/group.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Groups I could join

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000037",
  "type": "https://trusttasks.org/spec/vtc/groups/list/0.2#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000036",
  "payload": {
    "joinable": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/list/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Groups I could join — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000038",
  "type": "https://trusttasks.org/spec/vtc/groups/list/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000036",
  "payload": {
    "items": [
      {
        "group": {
          "groupId": "grp_W5k2N8xRb3Qz7TpL",
          "groupDid": "did:webvh:QmGroupScid:community.example:group-dids:grp_W5k2N8xRb3Qz7TpL",
          "name": "Working Group: Identity",
          "description": "Drafts the community's identity profile.",
          "kind": "static",
          "joinPolicy": "request",
          "visibility": "members",
          "showMembers": "groupMembers",
          "issuesCredentials": true,
          "carriesAuthority": false,
          "memberCount": 14,
          "createdAt": "2026-10-01T08:00:00Z"
        },
        "myMembership": "none"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Group names, descriptions and settings, and the caller's own standing in each. No other member's identity.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

**A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
