---
slug: vtc/groups/show
version: '0.1'
title: VTC Groups — Show
summary: 'Read one community group the caller may see: its DID, name, kind, policies and, where permitted, its member count and managers.'
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
  rationale: What is shown depends on who asks.
sideEffects:
  level: none
  rationale: Reads a group record; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses one group's settings and the caller's own standing in it.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/groups/show:notFound
  meaning: No group the caller may see has this identifier. A group that exists and is not visible to the caller answers the same way.
  retryable: false
related:
- vtc/groups/list
- vtc/groups/members/list
- vtc/groups/bindings/list
---

## Abstract

The **VTC Groups — Show** Trust Task reads one group, as [`vtc/groups/list`](../../list/0.1/spec.md) would show it, with its DID (`groupDid`) — the identity that issues its members' membership credentials.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. Answer `notFound` alike for a group that does not exist and one whose `visibility` does not admit the caller.
3. Apply the field rules of [`vtc/groups/list`](../../list/0.1/spec.md) Conformance item 3.

## Authorization

**Authority: standing in the community, bounded by the group's visibility**, as for [`vtc/groups/list`](../../list/0.1/spec.md).

## Definitions

- `Group` — see [`group.schema.json`](../../_shared/0.1/group.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read a group

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000003a",
  "type": "https://trusttasks.org/spec/vtc/groups/show/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000039",
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Read a group — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000003b",
  "type": "https://trusttasks.org/spec/vtc/groups/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000039",
  "payload": {
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
    "myMembership": "requested"
  }
}
```

## Security & Privacy

### Data carried

One group's settings and the caller's standing in it.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

**A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
