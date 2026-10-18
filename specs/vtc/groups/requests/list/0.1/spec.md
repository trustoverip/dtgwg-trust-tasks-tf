---
slug: vtc/groups/requests/list
version: '0.1'
title: VTC Groups — Requests — List
summary: A group manager lists requests to join the groups they manage, pending by default.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: group manager
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: What is listed depends on which groups the caller manages.
sideEffects:
  level: none
  rationale: Reads join requests; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses which members asked to join, and their messages.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/groups/requests/list:notFound
  meaning: '`groupId` names no group the caller may manage.'
  retryable: false
- code: vtc/groups/requests/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and these filters.'
  retryable: false
related:
- vtc/groups/requests/decide
- vtc/groups/join
---

## Abstract

The **VTC Groups — Requests — List** Trust Task returns the join requests for the groups the caller manages, for one group or all of them.

**Why there is no `show`.** Requests are only ever read to be decided, and [`vtc/groups/requests/decide`](../../decide/0.1/spec.md) addresses one by identifier and returns it; a separate read of one request would add nothing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Return only requests for groups the caller manages, and `notFound` for a `groupId` they do not manage.
2. Project each requester's `fields` as [`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md) would for the caller.
3. Bind `nextCursor` to the caller and the filters.

## Authorization

**Authority: `vtc.groups.manage`** for each group listed — unqualified, or qualified `group:<groupId>` — read from the signer's access-control entry at execution time ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupJoinRequest` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Pending requests for my group

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000060",
  "type": "https://trusttasks.org/spec/vtc/groups/requests/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000005f",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/requests/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Pending requests for my group — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000061",
  "type": "https://trusttasks.org/spec/vtc/groups/requests/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000005f",
  "payload": {
    "items": [
      {
        "request": {
          "requestId": "greq_T7m2K9xVb4Tz8LpR",
          "groupId": "grp_W5k2N8xRb3Qz7TpL",
          "entryId": "dir_Q7m2K9xVb4Tz8LpR",
          "message": "I'd like to help with the identity profile.",
          "createdAt": "2026-10-18T08:00:00Z",
          "state": "pending"
        },
        "fields": {
          "displayName": "Carol"
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Join requests and their messages, and each requester's directory fields as projected for the caller.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

**A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
