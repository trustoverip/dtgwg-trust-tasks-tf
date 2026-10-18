---
slug: vtc/groups/update
version: '0.1'
title: VTC Groups — Update
summary: A group manager changes a group's name, description, rule, join policy, visibility, member-list visibility or whether it issues credentials.
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
  requirement: REQUIRED
  rationale: Changes who may see or join a group, or which members a dynamic group computes; must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update would revert a later one.
sideEffects:
  level: mutating
  rationale: Replaces the named members of the group record; a rule change recomputes membership. Recoverable by a later update.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Carries changed group settings; returns the group record.
retention:
  class: durable
  rationale: The group record in force decides who sees and joins the group.
errorCodes:
- code: vtc/groups/update:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
- code: vtc/groups/update:ruleInvalid
  meaning: The rule is malformed, reads an unverified fact, or was sent for a static group.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      field:
        type: string
        maxLength: 128
      reason:
        type: string
        maxLength: 500
- code: vtc/groups/update:settingsInvalid
  meaning: '`joinPolicy` on a dynamic group, or `showMembers` wider than `visibility`.'
  retryable: false
- code: vtc/groups/update:authorityGroupLocked
  meaning: The group carries authority and must stay `closed`.
  retryable: false
related:
- vtc/groups/show
- vtc/groups/create
---

## Abstract

The **VTC Groups — Update** Trust Task changes a group's settings. Only the members sent change; `kind` is fixed for the group's life. A changed rule recomputes a dynamic group's membership, adding and removing members as any recomputation does.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not manage the group with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Apply the rule and settings checks of [`vtc/groups/create`](../../create/0.1/spec.md) to the result, refusing with `ruleInvalid` or `settingsInvalid`.
3. Refuse any `joinPolicy` other than `closed` on a group carrying authority with `authorityGroupLocked`.
4. Apply a narrowing of `visibility` or `showMembers` at once.
5. Where `issuesCredentials` becomes true, issue the group's authorization credential before recording the change, and offer credentials to existing members; where it becomes false, stop renewing members' credentials.
6. Recompute a dynamic group's membership after a rule change, excluding every entry that has opted out ([`vtc/groups/leave`](../../leave/0.1/spec.md)), with each addition and removal following the rules of [`vtc/groups/members/add`](../../members/add/0.1/spec.md) and [`vtc/groups/members/remove`](../../members/remove/0.1/spec.md).

## Authorization

**Authority: `vtc.groups.manage`** for the group — unqualified, or qualified `group:<groupId>` — read from the signer's access-control entry at execution time, under the community's `groups` policy purpose. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `Group` — see [`group.schema.json`](../../_shared/0.1/group.schema.json).

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Open the group to everyone

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000042",
  "type": "https://trusttasks.org/spec/vtc/groups/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000041",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "joinPolicy": "open",
    "reason": "Drafting is open to all members now."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Open the group to everyone — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000043",
  "type": "https://trusttasks.org/spec/vtc/groups/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000041",
  "payload": {
    "group": {
      "groupId": "grp_W5k2N8xRb3Qz7TpL",
      "groupDid": "did:webvh:QmGroupScid:community.example:group-dids:grp_W5k2N8xRb3Qz7TpL",
      "name": "Working Group: Identity",
      "description": "Drafts the community's identity profile.",
      "kind": "static",
      "joinPolicy": "open",
      "visibility": "members",
      "showMembers": "groupMembers",
      "issuesCredentials": true,
      "carriesAuthority": false,
      "memberCount": 14,
      "createdAt": "2026-10-01T08:00:00Z",
      "updatedAt": "2026-10-18T09:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

Changed group settings. A producer **MUST NOT** place personal data in `name`, `description`, `reason` or `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for the record in force.

### Consent/purpose

The community organising its members. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
