---
slug: vtc/groups/leave
version: '0.1'
title: VTC Groups — Leave
summary: 'A member leaves a static group, opts out of a dynamic one, or withdraws a pending request to join: renewal of their group credential stops, and what only the group provided is withdrawn.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
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
  rationale: The member's own decision to leave must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed leave would remove a member who has since re-joined.
sideEffects:
  level: mutating
  rationale: Ends the caller's own membership or request. Recoverable by joining again where the group allows.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: The member acts on their own membership.
retention:
  class: durable
  rationale: The departure is why the member lost what the group provided.
errorCodes:
- code: vtc/groups/leave:notFound
  meaning: No group the caller may see has this identifier.
  retryable: false
- code: vtc/groups/leave:notInGroup
  meaning: The group is static and the caller neither belongs to it nor has a pending request to join it.
  retryable: false
- code: vtc/groups/leave:alreadyOptedOut
  meaning: The caller has already opted out of this dynamic group.
  retryable: false
related:
- vtc/groups/join
- vtc/groups/members/remove
---

## Abstract

The **VTC Groups — Leave** Trust Task is how a member leaves a group they joined, or withdraws a request to join one. Leaving is always allowed for a static group. For a group carrying authority it is a reduction of the member's own authority, which the community executes on its terms for reductions.

**Opting out of a dynamic group.** A dynamic group's members are computed from its rule, but a member may still decline to be in it: leaving a dynamic group records an **opt-out**, an exclusion the rule cannot override. An opted-out member is not a member — their group credential stops renewing, and room bindings treat them as gone — until they undo it with [`vtc/groups/join`](../../join/0.1/spec.md). Dynamic groups never confer authority, so an opt-out is always allowed.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`; answer `notFound` for a group the caller may not see, `notInGroup` for a static group in which they have neither a membership nor a pending request, and `alreadyOptedOut` for a dynamic group they have already opted out of.
2. Withdraw a pending request (`requestWithdrawn`), or end the membership of a static group (`left`) as [`vtc/groups/members/remove`](../../members/remove/0.1/spec.md) Conformance items 3–5 describe.
3. For a dynamic group, record an opt-out for the caller's entry and end their membership as item 2 does (`optedOut`). Every later recomputation **MUST** exclude an opted-out entry, whatever the rule says, until the member joins again. An opt-out also counts when the caller is not admitted by the rule now, so a member can decline a group before the rule reaches them.
4. **MUST NOT** refuse a leave or an opt-out for any reason other than those in item 1.

## Authorization

**Authority: being the member.** The caller leaves only their own membership; the task has no target member. The proof establishes who signed ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupMember` — see [`group.schema.json`](../../_shared/0.1/group.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Leave a group

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000005a",
  "type": "https://trusttasks.org/spec/vtc/groups/leave/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000059",
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

### Opt out of a dynamic group

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000005c",
  "type": "https://trusttasks.org/spec/vtc/groups/leave/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000005b",
  "payload": {
    "groupId": "grp_D9m4K2xVb7Tz3LpQ"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/leave/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Leave a group — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000005d",
  "type": "https://trusttasks.org/spec/vtc/groups/leave/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000059",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "state": "left",
    "credentialValidUntil": "2026-11-17T09:00:01Z"
  }
}
```

### Opt out of a dynamic group — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000005e",
  "type": "https://trusttasks.org/spec/vtc/groups/leave/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000005b",
  "payload": {
    "groupId": "grp_D9m4K2xVb7Tz3LpQ",
    "state": "optedOut",
    "credentialValidUntil": "2026-11-17T09:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

A group identifier.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for the record of the departure.

### Consent/purpose

The member choosing to stop taking part in a group. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
