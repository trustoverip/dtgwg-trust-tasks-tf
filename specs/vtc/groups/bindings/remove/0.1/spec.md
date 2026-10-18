---
slug: vtc/groups/bindings/remove
version: '0.1'
title: VTC Groups — Bindings — Remove
summary: 'Remove a group''s binding: members lose the room admission or grants it provided, unless another source still provides them.'
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
  rationale: Withdraws what a binding gave every member; must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed removal would remove a binding since re-added.
sideEffects:
  level: mutating
  rationale: Removes a binding; the reconciler withdraws what only it provided. Recoverable by binding again.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: Names a binding; returns its state.
retention:
  class: durable
  rationale: The removal is why members lost what the binding provided.
errorCodes:
- code: vtc/groups/bindings/remove:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
- code: vtc/groups/bindings/remove:bindingNotFound
  meaning: The group has no binding with this identifier.
  retryable: false
related:
- vtc/groups/bindings/add
- vtc/groups/bindings/list
---

## Abstract

The **VTC Groups — Bindings — Remove** Trust Task withdraws a binding. For each member, what the binding provided goes only when it was its last source: a room admission another binding or a direct admission covers stays, and a grant held directly or through another group stays.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not hold `vtc.groups.manage` for the group as [`vtc/groups/bindings/add`](../../add/0.1/spec.md) item 1 does, and refuse an unknown binding with `bindingNotFound`.
2. Withdraw, through its reconciler, each member's room admission or grant whose only source is this binding, and delete the binding once that is done.
3. Treat removing a grants or role binding as a reduction of each affected member's authority, on the terms it applies to any reduction.
4. Once a group has no grants or role binding, stop treating it as carrying authority.

## Authorization

**Authority: `vtc.groups.manage`** for the group, and for a room binding `vtc.rooms.manage` for the room, read from the signer's access-control entry at execution time ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupBinding` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Remove a room binding

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000006e",
  "type": "https://trusttasks.org/spec/vtc/groups/bindings/remove/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000006d",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "bindingId": "gb_P2k8W5qYc6Vd1MsK",
    "reason": "The working group moved to a new room."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/bindings/remove/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Remove a room binding — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000006f",
  "type": "https://trusttasks.org/spec/vtc/groups/bindings/remove/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000006d",
  "payload": {
    "bindingId": "gb_P2k8W5qYc6Vd1MsK",
    "state": "removing"
  }
}
```

## Security & Privacy

### Data carried

A group, a binding identifier and a reason.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for the record of the removal.

### Consent/purpose

Withdrawing what a group provided its members. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
