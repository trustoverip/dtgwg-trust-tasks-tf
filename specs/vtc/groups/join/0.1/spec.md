---
slug: vtc/groups/join
version: '0.1'
title: VTC Groups — Join
summary: A member joins an open group at once, or asks to join a group that takes requests; where the group issues credentials, they are offered a membership credential issued by the group's DID.
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
  rationale: The member's own decision to join, or ask to join, must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed join would re-join a member who has since left.
sideEffects:
  level: mutating
  rationale: Records a membership or a join request; may issue a credential and offer room admissions. Recoverable with vtc/groups/leave.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: The member acts on their own membership. Returns their membership record or request.
retention:
  class: durable
  rationale: The membership or request is kept for as long as it stands.
errorCodes:
- code: vtc/groups/join:notFound
  meaning: No group the caller may see has this identifier.
  retryable: false
- code: vtc/groups/join:notJoinable
  meaning: The group is `closed`, or dynamic and the caller has not opted out of it.
  retryable: false
- code: vtc/groups/join:alreadyMember
  meaning: The caller already belongs to the group.
  retryable: false
- code: vtc/groups/join:requestPending
  meaning: The caller already has a pending request to join.
  retryable: false
- code: vtc/groups/join:policyRefused
  meaning: The community's `groups` policy refused the join.
  retryable: false
- code: vtc/groups/join:keyHolderUnavailable
  meaning: The community's VTA did not answer. Nothing was recorded.
  retryable: true
- code: vtc/groups/join:keyHolderRefused
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
- vtc/groups/leave
- vtc/groups/requests/decide
- credential-exchange/offer
---

## Abstract

The **VTC Groups — Join** Trust Task is how a member enters a group by themselves. An `open` group admits them at once, subject to the community's `groups` policy. A `request` group records a request for its managers to decide ([`vtc/groups/requests/decide`](../../requests/decide/0.1/spec.md)). A `closed` group cannot be joined. A dynamic group's membership follows its rule, so the only join there is is undoing an opt-out ([`vtc/groups/leave`](../../leave/0.1/spec.md)): the member is a member again whenever the rule admits them. A group carrying authority is always `closed` and static, so joining never confers authority.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** is the member, signing with their membership DID.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`, and answer `notFound` for a group whose `visibility` does not admit them.
2. Refuse a `closed` group, and a dynamic group the caller has not opted out of, with `notJoinable`; an existing member with `alreadyMember`; and a caller with a pending request with `requestPending`.
3. For a dynamic group the caller opted out of: remove the opt-out. Where the rule admits the caller now, admit them as a recomputation would (`source: rule`, credential first where the group issues credentials) and answer `joined`; otherwise answer `optOutWithdrawn`.
4. For an `open` group: evaluate its `groups` policy and refuse with `policyRefused` where it refuses; where the group issues credentials, issue and offer the credential **before** recording the membership; then record it and let its reconciler offer the group's room bindings.
5. For a `request` group: record a pending request and answer `requested`. `message` is shown to the group's managers only.

**The group's membership credential.** Where the group issues credentials (`issuesCredentials: true`), admitting a member — by [`vtc/groups/members/add`](../../members/add/0.1/spec.md), [`vtc/groups/join`](spec.md), an approved request ([`vtc/groups/requests/decide`](../../requests/decide/0.1/spec.md)) or a dynamic group's recomputation — includes this step, **before** the membership is recorded:

- The community has its VTA sign, **as the group's DID**, a membership credential to the member's membership DID: `type` `["VerifiableCredential", "DTGCredential", "MembershipCredential"]`, `@context` including `https://firstperson.network/credentials/dtg/v1`. The VTA signs it through the generic `keys/sign-document/0.1` Trust Task (specified separately; it is not specific to groups), so the community never holds the group's key.
- Its validity is the community's setting, not a constant of this specification. A community **SHOULD** default to 30 days, renewed when 10 days remain.
- The community offers it to the member through the existing credential-exchange offer path ([`credential-exchange/offer`](../../../../credential-exchange/offer/0.1/spec.md)), as it offers any credential it issues.

A verifier chains the member's group credential to the group's authorization credential, issued by the community to the group's DID, and that to the community. The community renews the member's credential before it expires, through its VTA, for as long as the member stays in the group. **There is no status list per group**: a small group's list would single its members out (VTI-CRD-014). Leaving, removal, opting out of a dynamic group and departure from the community stop renewal, and the credential lapses at its expiry.

## Authorization

**Authority: being the member, and the group's join policy.** The `issuer` is the member who joins; the task has no target member, so no party can use it for anyone else. The group's `joinPolicy` and the community's `groups` policy decide whether a join is admitted. The proof establishes who signed ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); here who signed is the authorization question.

## Definitions

- `JoinPolicy`, `GroupJoinRequest`, `GroupMember` — see [`group.schema.json`](../../_shared/0.1/group.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Join an open group

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000051",
  "type": "https://trusttasks.org/spec/vtc/groups/join/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000050",
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

### Undo an opt-out of a dynamic group

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000053",
  "type": "https://trusttasks.org/spec/vtc/groups/join/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000052",
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

### Ask to join a request group

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000055",
  "type": "https://trusttasks.org/spec/vtc/groups/join/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000054",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "message": "I'd like to help with the identity profile."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/join/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Join an open group — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000056",
  "type": "https://trusttasks.org/spec/vtc/groups/join/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000050",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "outcome": "joined",
    "member": {
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "source": "joined",
      "since": "2026-10-18T09:00:01Z",
      "state": "active",
      "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d",
      "credentialValidUntil": "2026-11-17T09:00:01Z"
    },
    "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d"
  }
}
```

### Undo an opt-out of a dynamic group — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000057",
  "type": "https://trusttasks.org/spec/vtc/groups/join/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000052",
  "payload": {
    "groupId": "grp_D9m4K2xVb7Tz3LpQ",
    "outcome": "joined",
    "member": {
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "source": "rule",
      "since": "2026-10-18T09:00:01Z",
      "state": "active",
      "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d",
      "credentialValidUntil": "2026-11-17T09:00:01Z"
    },
    "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d"
  }
}
```

### Ask to join a request group — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000058",
  "type": "https://trusttasks.org/spec/vtc/groups/join/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000054",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "outcome": "requested",
    "request": {
      "requestId": "greq_T7m2K9xVb4Tz8LpR",
      "groupId": "grp_W5k2N8xRb3Qz7TpL",
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "message": "I'd like to help with the identity profile.",
      "createdAt": "2026-10-18T09:00:01Z",
      "state": "pending"
    },
    "credentialId": null
  }
}
```

## Security & Privacy

### Data carried

A group identifier and an optional message to its managers. The member's own membership or request in the response.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for the membership or the request while it stands.

### Consent/purpose

The member choosing to take part in a group. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
