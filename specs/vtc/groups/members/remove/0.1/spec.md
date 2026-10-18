---
slug: vtc/groups/members/remove
version: '0.1'
title: VTC Groups — Members — Remove
summary: 'A group manager removes a member from a static group: renewal of their group credential stops, and room admissions and grants that only this group provided are withdrawn.'
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
  rationale: Withdraws what the group provided a member; must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed removal would remove someone since re-added.
sideEffects:
  level: mutating
  rationale: Ends a membership and withdraws what only it provided. Recoverable by adding the member again.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: Names a member by entry; returns the outcome.
retention:
  class: durable
  rationale: The removal is why the member lost what the group provided.
errorCodes:
- code: vtc/groups/members/remove:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
- code: vtc/groups/members/remove:notInGroup
  meaning: The entry does not belong to the group.
  retryable: false
- code: vtc/groups/members/remove:dynamicGroup
  meaning: The group is dynamic; its membership is computed from its rule.
  retryable: false
related:
- vtc/groups/members/add
- vtc/groups/leave
---

## Abstract

The **VTC Groups — Members — Remove** Trust Task takes a member out of a static group. Everything the group provided them goes with the membership, unless another source still provides it: a room admission another binding or a direct admission covers stays, and a grant they hold directly or through another group stays.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not manage the group with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Refuse an entry not in the group with `notInGroup`, and a dynamic group with `dynamicGroup`.
3. Stop renewing the member's group credential. It **MUST NOT** maintain a status list for the group; the credential lapses at its expiry.
4. Remove the member from the rooms bound to the group, and remove the grants whose **only** source is this group, through its reconciler. A removal from a room happens when the last source of the admission goes.
5. Not be blocked by its VTA being unreachable: the membership ends and its access-control grants are withdrawn at once; only room removals wait for the reconciler.

**Groups carrying authority.** Where the group has a `grants` or `role` binding, a membership change is an authority change, and the community treats it exactly as it treats the matching access-control change:

- adding a member is an authority-conferring grant: the community holds it for the consent it requires of such a grant, answering with `trust-task-next-step` naming the held action, and records the member `pending` until it executes;
- removing a member is a reduction of their authority, which the community executes on the terms it applies to any reduction, including any cooling-off.

Groups give no way round consent. Whether consent or a cooling-off applies, and whose, is the community's policy, which this specification does not declare.

## Authorization

**Authority: `vtc.groups.manage`** for the group, read from the signer's access-control entry at execution time. For a group carrying authority the removal is a reduction of the member's authority, executed on the community's terms for reductions. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **Source** — why a member holds a room admission or a grant: directly, or through a named group. A thing is withdrawn when its last source goes.

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Remove a member

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000004e",
  "type": "https://trusttasks.org/spec/vtc/groups/members/remove/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000004d",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "reason": "Left the working group."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/members/remove/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Remove a member — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000004f",
  "type": "https://trusttasks.org/spec/vtc/groups/members/remove/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000004d",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "state": "removed",
    "credentialValidUntil": "2026-11-17T09:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

A group, an entry and a reason. A producer **SHOULD NOT** put in `reason` anything about the member that the community does not need to record.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for the record of the removal.

### Consent/purpose

The community organising its members. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
