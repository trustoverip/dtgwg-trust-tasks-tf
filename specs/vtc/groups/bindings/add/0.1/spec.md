---
slug: vtc/groups/bindings/add
version: '0.1'
title: VTC Groups — Bindings — Add
summary: Bind a group to a community room, so its members are offered admission, or to a grant set or role's capabilities, which the community provisions into members' access-control entries with the group as their source.
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
  rationale: A binding admits every member of a group to a room, or confers capabilities on every member; it must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed binding would re-confer what was since removed.
sideEffects:
  level: mutating
  rationale: Records a binding; the community's reconciler then offers room admissions or provisions grants to every member. Recoverable with vtc/groups/bindings/remove.
consequences:
- A room binding offers every member of the group admission to the room.
- 'A grants or role binding writes resource grants onto every member''s access-control entry, and makes the group carry authority: it must be static and closed for as long as the binding exists.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Carries a room or capability references; returns the binding.
retention:
  class: durable
  rationale: The binding is the source of what every member receives through it.
errorCodes:
- code: vtc/groups/bindings/add:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
- code: vtc/groups/bindings/add:roomNotFound
  meaning: The community owns no room with this identifier, or the caller does not hold `vtc.rooms.manage` for it.
  retryable: false
- code: vtc/groups/bindings/add:groupNotEligible
  meaning: A grants or role binding was sent for a dynamic group or one whose `joinPolicy` is not `closed`.
  retryable: false
- code: vtc/groups/bindings/add:unknownCapability
  meaning: A capability is not in the community's capability registry.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - capabilities
    properties:
      capabilities:
        type: array
        maxItems: 32
        items:
          type: string
          maxLength: 128
- code: vtc/groups/bindings/add:unknownRole
  meaning: No role of this name is defined.
  retryable: false
- code: vtc/groups/bindings/add:exceedsBinderAuthority
  meaning: A capability is not held, at a qualifier at least as wide, by the administrators whose authority makes the binding.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - capabilities
    properties:
      capabilities:
        type: array
        maxItems: 32
        items:
          type: string
          maxLength: 128
- code: vtc/groups/bindings/add:bindingExists
  meaning: An equal binding already exists.
  retryable: false
- code: vtc/groups/bindings/add:tooManyBindings
  meaning: The group already has 64 bindings.
  retryable: false
- code: vtc/groups/bindings/add:kindNotServed
  meaning: The community serves room bindings only. A grants or role binding is not recorded; nothing was stored.
  retryable: false
related:
- vtc/groups/bindings/list
- vtc/groups/bindings/remove
- vtc/rooms/community/members/add
- acl/grant
- vtc/roles/define
---

## Abstract

The **VTC Groups — Bindings — Add** Trust Task links a group to what its members should receive.

- A **room** binding links a room the community owns. Joining the group offers the member admission through the existing room offer path; the member still accepts with their own key package, so nobody is placed in an encrypted room without their own key. Leaving the group removes them, unless another source still covers them.
- A **grants** binding, or a **role** binding by reference to a defined role's capability ceiling, confers capabilities. The community writes each grant onto every member's access-control entry as a resource grant whose source is the group, and removes it when they leave. The group itself never answers an authorization question.

A binding never assigns a role: an entry's administrative role and its community role remain direct assignments. Groups confer capabilities within a ceiling, never the ceiling itself.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not hold `vtc.groups.manage` for the group with `notFound` when they may not see it, and `permissionDenied` when they may.
2. For a room binding, refuse with `roomNotFound` a room the community does not own or for which the caller lacks `vtc.rooms.manage`.
3. For a grants or role binding, refuse a dynamic group, or a static group whose `joinPolicy` is not `closed`, with `groupNotEligible`; refuse an unknown capability with `unknownCapability` and an unknown role with `unknownRole`; and refuse with `exceedsBinderAuthority` any capability the binding administrators do not themselves hold at a qualifier at least as wide (VTI-ACL-071).
4. Record the binding, then let its reconciler — the one owner of the retries — offer admissions or provision grants to each member, idempotently, with capped backoff. A room admission is recorded only after the community's VTA has answered. Each grant is bounded by the member's role ceiling and by the binding administrators' holdings.
5. Keep a group with a grants or role binding `static` and `closed` for as long as the binding exists.
6. Renew each admission a room binding made, for as long as the member stays in the group, before it lapses: fresh membership and authority credentials with the binding's `actions`, valid `validFor` from issue, which the member collects as a renewal from [`vtc/rooms/offers/list`](../../../../rooms/offers/list/0.3/spec.md) at `0.3` or later.
7. A community that serves room bindings only **MAY** refuse a grants or role binding with `kindNotServed` once item 1 has passed. It **MUST NOT** record a binding it does not provision.
8. **MUST NOT** read a group record, or any group credential, to answer whether an entry may do something. Every authorization decision reads the access-control entry, where the provisioned grants carry the group as their source.

**Bindings that confer authority are held, not executed at once.** A community **SHOULD** hold a grants or role binding, and a room binding whose `actions` include `curate`, for the consent it requires of an authority-conferring grant, answering with `trust-task-next-step` naming the held action, and record the binding `pending` until it executes. A room binding of `read` and `write` only **MAY** execute at once. Whether consent applies, and whose, is the community's policy, which this specification does not declare.

## Authorization

**Authority: `vtc.groups.manage` for the group, plus the authority the binding confers.** A room binding also needs `vtc.rooms.manage` for the room. A grants or role binding is bounded by the binding administrators' own holdings, so a group cannot be used to manufacture a capability nobody behind it held; each member's grant is further bounded by their own role ceiling. Read from the signers' access-control entries at execution time. The proof identifies the requester; it is not the authorization ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `BindingSpec`, `GroupBinding` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).
- **Binding administrators** — the requester and, where the community executes the binding after approval, the approvers whose authority it relies on.

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Bind the working group's room

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000069",
  "type": "https://trusttasks.org/spec/vtc/groups/bindings/add/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000068",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "binding": {
      "kind": "room",
      "roomId": "did:webvh:QmRoom:community.example:rooms:wg-identity",
      "actions": [
        "read",
        "write"
      ],
      "validFor": "P365D"
    },
    "reason": "The working group's drafting room."
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

### Bind a grant set

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000006b",
  "type": "https://trusttasks.org/spec/vtc/groups/bindings/add/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000006a",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "binding": {
      "kind": "grants",
      "grants": [
        {
          "capability": "vtc.invitations.manage"
        }
      ]
    },
    "reason": "Working group members may invite."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/bindings/add/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Bind the working group's room — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000006c",
  "type": "https://trusttasks.org/spec/vtc/groups/bindings/add/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000068",
  "payload": {
    "binding": {
      "bindingId": "gb_P2k8W5qYc6Vd1MsK",
      "binding": {
        "kind": "room",
        "roomId": "did:webvh:QmRoom:community.example:rooms:wg-identity",
        "actions": [
          "read",
          "write"
        ],
        "validFor": "P365D"
      },
      "state": "active",
      "createdAt": "2026-10-18T09:00:01Z",
      "createdBy": "did:example:administrator"
    }
  }
}
```

## Security & Privacy

### Data carried

A room identifier and actions, or capability references, and a reason.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for as long as the binding exists.

### Consent/purpose

Provisioning, through the community's own records, what a group's members should receive. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
