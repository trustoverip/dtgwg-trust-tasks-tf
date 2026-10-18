---
slug: vtc/groups/requests/decide
version: '0.1'
title: VTC Groups — Requests — Decide
summary: A group manager approves or declines a request to join; an approval admits the member, with a membership credential issued by the group's DID where the group issues credentials.
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
  rationale: Admits a member or refuses them; must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed decision would re-admit a member who has since left.
sideEffects:
  level: mutating
  rationale: Records the decision; an approval records a membership and may issue a credential and offer room admissions.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Carries a decision and an optional reason; returns the decided request and any membership.
retention:
  class: durable
  rationale: The decision is the record of why the member was or was not admitted.
errorCodes:
- code: vtc/groups/requests/decide:notFound
  meaning: No request to a group the caller manages has this identifier.
  retryable: false
- code: vtc/groups/requests/decide:alreadyDecided
  meaning: The request is no longer pending.
  retryable: false
- code: vtc/groups/requests/decide:selfDecision
  meaning: The caller made the request; nobody decides their own.
  retryable: false
- code: vtc/groups/requests/decide:keyHolderUnavailable
  meaning: The community's VTA did not answer. The request is still pending.
  retryable: true
- code: vtc/groups/requests/decide:keyHolderRefused
  meaning: The community's VTA refused a step. Its own code and message are carried in `details`. The request is still pending.
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
- vtc/groups/requests/list
- vtc/groups/join
---

## Abstract

The **VTC Groups — Requests — Decide** Trust Task approves or declines a pending request to join a `request` group. An approval admits the member exactly as [`vtc/groups/members/add`](../../../members/add/0.1/spec.md) does, with `source: requestApproved`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` for a request to a group the caller does not manage, `alreadyDecided` for one that is not pending, and `selfDecision` where the caller is the requester.
2. On approval, re-check that the requester is still an active member and the group still takes requests, then admit them as [`vtc/groups/members/add`](../../../members/add/0.1/spec.md) Conformance items 3–5 describe, leaving the request pending and answering `keyHolderUnavailable` or `keyHolderRefused` when the credential cannot be issued.
3. On decline, record the decision. The requester sees the request `declined`, never the `reason`.

**The group's membership credential.** Where the group issues credentials (`issuesCredentials: true`), admitting a member — by [`vtc/groups/members/add`](../../../members/add/0.1/spec.md), [`vtc/groups/join`](../../../join/0.1/spec.md), an approved request ([`vtc/groups/requests/decide`](spec.md)) or a dynamic group's recomputation — includes this step, **before** the membership is recorded:

- The community has its VTA sign, **as the group's DID**, a membership credential to the member's membership DID: `type` `["VerifiableCredential", "DTGCredential", "MembershipCredential"]`, `@context` including `https://firstperson.network/credentials/dtg/v1`. The VTA signs it through the generic `keys/sign-document/0.1` Trust Task (specified separately; it is not specific to groups), so the community never holds the group's key.
- Its validity is the community's setting, not a constant of this specification. A community **SHOULD** default to 30 days, renewed when 10 days remain.
- The community offers it to the member through the existing credential-exchange offer path ([`credential-exchange/offer`](../../../../../credential-exchange/offer/0.1/spec.md)), as it offers any credential it issues.

A verifier chains the member's group credential to the group's authorization credential, issued by the community to the group's DID, and that to the community. The community renews the member's credential before it expires, through its VTA, for as long as the member stays in the group. **There is no status list per group**: a small group's list would single its members out (VTI-CRD-014). Leaving, removal, opting out of a dynamic group and departure from the community stop renewal, and the credential lapses at its expiry.

## Authorization

**Authority: `vtc.groups.manage`** for the request's group, read from the signer's access-control entry at execution time. A manager does not decide their own request. The proof establishes who decided, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupJoinRequest`, `GroupMember` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Approve a request

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000063",
  "type": "https://trusttasks.org/spec/vtc/groups/requests/decide/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000062",
  "payload": {
    "requestId": "greq_T7m2K9xVb4Tz8LpR",
    "decision": "approve"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/requests/decide/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Approve a request — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000064",
  "type": "https://trusttasks.org/spec/vtc/groups/requests/decide/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000062",
  "payload": {
    "request": {
      "requestId": "greq_T7m2K9xVb4Tz8LpR",
      "groupId": "grp_W5k2N8xRb3Qz7TpL",
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "createdAt": "2026-10-18T08:00:00Z",
      "state": "approved",
      "decidedAt": "2026-10-18T09:00:01Z",
      "decidedBy": "did:example:administrator"
    },
    "member": {
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "source": "requestApproved",
      "since": "2026-10-18T09:00:01Z",
      "state": "active",
      "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d"
    },
    "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d"
  }
}
```

## Security & Privacy

### Data carried

A request identifier, a decision and an optional reason.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for the decision.

### Consent/purpose

Deciding who takes part in a group. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
