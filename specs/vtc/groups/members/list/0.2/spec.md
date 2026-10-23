---
slug: vtc/groups/members/list
version: '0.2'
title: VTC Groups — Members — List
summary: List a group's members, a page at a time, each projected for the caller as the directory projects them — only where the group shows its members to the caller.
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
  rationale: What is listed depends on who asks.
sideEffects:
  level: none
  rationale: Reads the membership record; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses which members belong to the group and the directory fields each chose to show the caller.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/groups/members/list:notFound
  meaning: No group the caller may see has this identifier.
  retryable: false
- code: vtc/groups/members/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and group.'
  retryable: false
related:
- vtc/groups/show
- vtc/directory/entries/list
---

## Abstract

The **VTC Groups — Members — List** Trust Task pages through a group's members. A member list is never inlined into a group read: groups can be large, and who belongs is often more sensitive than that the group exists.

**Why there is no `show`.** A single membership is read through the member's directory entry ([`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md), `sharedGroups`) or their own [`vtc/groups/show`](../../../show/0.1/spec.md) (`myMembership`); a per-membership read would be a membership oracle for a pair of identifiers.

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

1. Answer `notFound` alike for a group that does not exist and one whose `visibility` does not admit the caller, and `permissionDenied` where the caller may see the group but `showMembers` does not admit them.
2. Return each member's `fields` projected exactly as [`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md) would project them for this caller — a member who chose `hidden` is still listed here to those the group shows its members to, with only the fields that projection admits.
3. Return `credentialValidUntil` only to the member themselves and to the group's managers, and `pending` members only to managers.
4. Bind `nextCursor` to the caller and the group.

## Authorization

**Authority: the group's `showMembers` setting**, which the community enforces for the caller's standing at execution time, and each member's own directory visibility for what is shown of them. The proof, where present, identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupMember` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### List a group's members

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000048",
  "type": "https://trusttasks.org/spec/vtc/groups/members/list/0.2#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000047",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/members/list/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### List a group's members — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000049",
  "type": "https://trusttasks.org/spec/vtc/groups/members/list/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000047",
  "payload": {
    "items": [
      {
        "entryId": "dir_Q7m2K9xVb4Tz8LpR",
        "source": "requestApproved",
        "since": "2026-10-05T10:00:00Z",
        "state": "active",
        "fields": {
          "displayName": "Carol"
        },
        "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Which members belong to the group, and the directory fields each chose to show the caller. A member's DID only where they made `did` visible to the caller.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

**A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
