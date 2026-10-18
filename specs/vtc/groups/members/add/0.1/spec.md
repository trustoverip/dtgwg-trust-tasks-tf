---
slug: vtc/groups/members/add
version: '0.1'
title: VTC Groups — Members — Add
summary: A group manager adds a community member to a static group; where the group issues credentials, the group's DID issues the member a membership credential first.
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
  rationale: Adds a member to a group, through which they may gain room admissions and, for a group carrying authority, access-control grants. Must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed addition would re-admit someone since removed.
sideEffects:
  level: mutating
  rationale: Records a membership; may issue a credential, offer room admissions and provision access-control grants. Recoverable with vtc/groups/members/remove.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Names a member by entry; returns their membership record and the identifier of the credential offered to them, never the credential.
retention:
  class: durable
  rationale: The membership record is the source of everything the group provides the member.
errorCodes:
- code: vtc/groups/members/add:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
- code: vtc/groups/members/add:entryNotFound
  meaning: No active member has this entry.
  retryable: false
- code: vtc/groups/members/add:alreadyMember
  meaning: The member already belongs to the group, or an addition is pending.
  retryable: false
- code: vtc/groups/members/add:dynamicGroup
  meaning: The group is dynamic; its membership is computed from its rule.
  retryable: false
- code: vtc/groups/members/add:exceedsRoleCeiling
  meaning: The group carries authority and a grant it confers is outside the member's role ceiling.
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
- code: vtc/groups/members/add:keyHolderUnavailable
  meaning: The community's VTA did not answer. Nothing was recorded.
  retryable: true
- code: vtc/groups/members/add:keyHolderRefused
  meaning: The community's VTA refused a step. Its own code and message are carried in `details`. Nothing was recorded.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      code:
        type: string
        maxLength: 128
      message:
        type: string
        maxLength: 500
related:
- keys/sign-document
- vtc/groups/members/remove
- vtc/groups/members/list
- credential-exchange/offer
---

## Abstract

The **VTC Groups — Members — Add** Trust Task puts a community member into a static group. What the group provides then follows: a membership credential issued by the group's DID, offers for the rooms bound to the group, and — for a group carrying authority — the grants it confers, written into the member's access-control entry with the group as their source.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not manage the group with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Refuse an entry that is not an active member with `entryNotFound`, an existing or pending member with `alreadyMember`, and a dynamic group with `dynamicGroup`.
3. Where the group issues credentials, issue and offer the member's group credential **before** recording the membership, and refuse with `keyHolderUnavailable` or `keyHolderRefused`, recording nothing, when its VTA does not.
4. Record the membership, then let its reconciler offer the member the group's room bindings and provision its grants.
5. Provision each grant as a resource grant whose source is this group, bounded by the member's role ceiling and by the holdings of the administrator who made the binding, and refuse with `exceedsRoleCeiling` where the member's ceiling does not admit one.

**The group's membership credential.** Where the group issues credentials (`issuesCredentials: true`), admitting a member — by [`vtc/groups/members/add`](spec.md), [`vtc/groups/join`](../../../join/0.1/spec.md), an approved request ([`vtc/groups/requests/decide`](../../../requests/decide/0.1/spec.md)) or a dynamic group's recomputation — includes this step, **before** the membership is recorded:

- The community has its VTA sign, **as the group's DID**, a membership credential to the member's membership DID: `type` `["VerifiableCredential", "DTGCredential", "MembershipCredential"]`, `@context` including `https://firstperson.network/credentials/dtg/v1`. The VTA signs it through the generic `keys/sign-document/0.1` Trust Task (specified separately; it is not specific to groups), so the community never holds the group's key.
- Its validity is the community's setting, not a constant of this specification. A community **SHOULD** default to 30 days, renewed when 10 days remain.
- The community offers it to the member through the existing credential-exchange offer path ([`credential-exchange/offer`](../../../../../credential-exchange/offer/0.1/spec.md)), as it offers any credential it issues.

A verifier chains the member's group credential to the group's authorization credential, issued by the community to the group's DID, and that to the community. The community renews the member's credential before it expires, through its VTA, for as long as the member stays in the group. **There is no status list per group**: a small group's list would single its members out (VTI-CRD-014). Leaving, removal, opting out of a dynamic group and departure from the community stop renewal, and the credential lapses at its expiry.

**Groups carrying authority.** Where the group has a `grants` or `role` binding, a membership change is an authority change, and the community treats it exactly as it treats the matching access-control change:

- adding a member is an authority-conferring grant: the community holds it for the consent it requires of such a grant, answering with `trust-task-next-step` naming the held action, and records the member `pending` until it executes;
- removing a member is a reduction of their authority, which the community executes on the terms it applies to any reduction, including any cooling-off.

Groups give no way round consent. Whether consent or a cooling-off applies, and whose, is the community's policy, which this specification does not declare.

## Authorization

**Authority: `vtc.groups.manage`** for the group — unqualified, or qualified `group:<groupId>` — read from the signer's access-control entry at execution time. For a group carrying authority, adding a member is also an authority-conferring grant, which the community treats as it treats any such grant; the manager's capability alone does not confer authority on someone else. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupMember`, `GroupCredentialId` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Add a member

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000004b",
  "type": "https://trusttasks.org/spec/vtc/groups/members/add/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000004a",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "entryId": "dir_Q7m2K9xVb4Tz8LpR"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/members/add/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Add a member — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000004c",
  "type": "https://trusttasks.org/spec/vtc/groups/members/add/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000004a",
  "payload": {
    "member": {
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "source": "added",
      "since": "2026-10-18T09:00:01Z",
      "state": "active",
      "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d",
      "credentialValidUntil": "2026-11-17T09:00:01Z"
    },
    "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d"
  }
}
```

## Security & Privacy

### Data carried

A group and an entry; the membership record and the identifier of the credential offered. The credential itself goes to the member alone, through the credential-exchange offer path.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for the membership; on the member's departure from the community it is removed.

### Consent/purpose

The community organising its members. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
