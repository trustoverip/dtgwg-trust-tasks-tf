---
slug: vtc/members/make-membership
version: "0.1"
title: "VTC Members — Make-Membership"
summary: "An administrator gives an access-control entry with no member record — typically the bootstrap administrator — the member record and membership credentials a join would have given it, at the role it already holds."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - members
  - admission
  - direct-admission
  - membership
  - bootstrap
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Making an entry a membership issues a membership credential to a party that never applied, on one administrator's word. It must be attributable to that administrator after the transport has closed — by the approvers of a referred operation and by the audit trail."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Replayed after the membership it created was removed, it would be refused (a removed record is never made a membership), but a document placeable in time is what lets the community bound its duplicate-execution record at all."
sideEffects:
  level: mutating
  rationale: "Writes a member record and a revocation slot and issues a membership credential and a role credential. The access-control entry is not changed. Recoverable by removing the member (vtc/members/admin-remove), which revokes the credentials."
consequences:
  - "The subject becomes a member of the community at the role its entry already holds, holding a membership credential and a role credential the community signed."
  - "The subject's access-control entry — role, authority, resource grants, label, expiry — is unchanged."
subjectPath: /subject
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names a DID and an operator-authored reason. The response returns the membership and the two credentials issued to the subject — credentials naming the subject and the community, which the subject would receive anyway and which a presentation must be signed by the subject to use."
retention:
  class: durable
  rationale: "The operation is the community's record of how an entry holder became a member without applying, by whose word and on whose approval."
errorCodes:
  - code: vtc/members/make-membership:noEntry
    meaning: "`subject` holds no access-control entry. A DID with none is admitted with vtc/members/admit."
    retryable: false
  - code: vtc/members/make-membership:applicationEntry
    meaning: "`subject`'s entry is of the `application` community role — a non-member holding resource grants. It becomes a membership through the join flow, or with vtc/members/admit, which keeps its grants."
    retryable: false
  - code: vtc/members/make-membership:entryNotLive
    meaning: "`subject`'s entry has expired or is suspended, and authorizes nothing now."
    retryable: false
  - code: vtc/members/make-membership:alreadyMember
    meaning: "`subject` already holds a member record that has not been removed."
    retryable: false
  - code: vtc/members/make-membership:removedMember
    meaning: "`subject` was a member and was removed. Making the entry a membership would undo the removal without the decision re-admitting a former member takes; re-admit them through the join flow, or revoke the entry and admit them with vtc/members/admit."
    retryable: false
  - code: vtc/members/make-membership:deniedByPolicy
    meaning: "The community's admission policy refuses this operation."
    retryable: false
related:
  - vtc/members/admit
  - vtc/members/admin-remove
  - vtc/admin/actions/show
  - acl/grant
  - trust-task-next-step
---

## Abstract

**VTC Members — Make-Membership** — *Make this a membership* — gives an access-control entry that has no member record the membership a completed join would have given it: a member record, a revocation slot, the membership credential (VMC) and the role credential (VAC), at the community role the entry already holds. The entry itself — its role, authority, resource grants, label and expiry — is left exactly as it is.

The case that prompted it is the community's **bootstrap administrator**: the first administrator is written into the access-control list when the community is installed, never joins, and so holds authority without a membership — and cannot, for instance, sign in where only members may. Any entry granted directly with [`acl/grant`](../../../../acl/grant/0.2/spec.md) is in the same position.

A DID with no entry is admitted with [`vtc/members/admit`](../../admit/0.1/spec.md) instead.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (`issuer`) supplies `subject`, optionally `reason`, and carries a proof.

A conforming **community** (`recipient`) checks, in this order, before its policy runs:

1. **The requester holds `vtc.members.manage`** — read from the signer's live access-control entry, or for a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.1/spec.md)) the entry of the identity it acts for, at execution time; an entry that has expired or is suspended holds nothing. Else `permissionDenied`.
2. **The subject holds a live entry that is not an `application` entry.** No entry is `noEntry`; an `application` entry is `applicationEntry` (it is not a membership to make, and joins through the join flow or `vtc/members/admit`); an expired or suspended entry is `entryNotLive`.
3. **The subject holds no member record.** A record that has not been removed is `alreadyMember`. A removed record is `removedMember`: the community **MUST NOT** overwrite or revive it here, because re-admitting a former member is a decision of its own — the join flow's, or `vtc/members/admit`'s once the entry is gone — taken under that task's rules.
4. `reason` is at most 1024 characters; a document that fails the schema is `malformedRequest`.

Then:

5. **The community's policy for direct admission decides** whether the operation happens now, is referred for another administrator's approval, or is refused (`deniedByPolicy`). The policy sees the action (`make-membership`), the entry's role, and whether that role carries administrative authority. It **MUST NOT** be able to widen items 1 to 4, nor choose a role.
6. **A referred operation** is parked, and the community answers with [`trust-task-next-step/0.1`](../../../../trust-task-next-step/0.1/spec.md) as [`vtc/admin/actions/show/0.2`](../../../admin/actions/show/0.2/spec.md) describes. An approval **MUST** count only from a holder of `vtc.members.manage` who is neither the requester nor the subject. When the approvals are complete the community executes the parked document itself, **exactly once**, and **MUST** first re-run items 1 to 5 against the community as it is then; it answers on the request's `threadId` with this task's `#response` or a `trust-task-error`.
7. **On execution** the community **MUST NOT** change the subject's access-control entry. It **MUST** write the member record at the entry's role — with no publication consent, which is the member's to give — allocate a revocation slot, issue the VMC and the role VAC, and only then answer. It **MUST** audit the operation: the requester, the subject, the role, how it was allowed (`approval`), the parked action's identifier when there was one, and `reason`. It **SHOULD** deliver the credentials to the member as it does on a join approval; that delivery is best effort, and the credentials are in the response whether or not it succeeds.

## Authorization

The authority this task presupposes is the **`vtc.members.manage` capability at the community**, read from the signer's access-control entry (or the entry a delegated signing key acts for) at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

**It grants no authority.** The subject's entry already holds whatever authority it holds, and this task leaves the entry alone: it adds a membership — a record and two credentials — beside it. That is why no role-assignment capability is needed, and why an entry of any role, `admin` included, can be made a membership: the role credential issued attests a role the community already granted, by whatever rules governed that grant.

**The policy decides; it does not authorize.** The operation skips the join criteria and vetting, so the community decides through its own policy whether its administrators may do it alone. The policy can refuse or refer; it cannot admit what the checks above refuse. A community whose policy refers the operation requires a second holder of `vtc.members.manage` to agree, and that approver's own signed decision is what carries the consent.

**The model this task is designed for.** A community implementing administrator step-up asks the requester for a re-authentication bound to this one document before it parks a referred operation, answering with the refusal its own step-up mechanism defines, and the same document is sent again once the step is done. Because the operation grants no authority, such a community need not ask for one when its policy admits at once. A community with no other administrator to approve, running in a single-administrator mode, may waive a referral on the requester's own bound re-authentication, and records the operation as `waived`. Whether a step-up or consent is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- **`application` entry** — an access-control entry of the `application` community role: a non-member the community records because it holds resource grants. It is never a membership.
- **Live entry** — an access-control entry that has not expired and is not suspended.
- `AdmittedMembership`, `Approval` — see [`vtc/_shared/0.1/admission.schema.json`](../../../_shared/0.1/admission.schema.json).

## Request

The administrator names the DID whose entry is to become a membership. The payload is described by [`payload.schema.json`](payload.schema.json).

### Make the bootstrap administrator a member

```json
{
  "id": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-9a0b1c2d3e4f",
  "type": "https://trusttasks.org/spec/vtc/members/make-membership/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-10-16T09:10:00Z",
  "threadId": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-9a0b1c2d3e4f",
  "payload": {
    "subject": "did:web:carol.example",
    "reason": "Bootstrap administrator; needs a membership to sign in to the member portal."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-16T09:10:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7cDExampleProofValue"
  }
}
```

## Response

The community answers with the membership as admitted and the credentials issued for it — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json), which is `AdmittedMembership`. `label` is absent, since no entry is written, and `returning` is always `false`, since a removed record is refused. A referred operation is first answered with `trust-task-next-step`, and this response follows on the same `threadId` when it executes. A refusal is a `trust-task-error`.

### Made a membership, after another administrator's approval

```json
{
  "id": "urn:uuid:2f3e4d5c-6b7a-4980-a1b2-c3d4e5f6a7b8",
  "type": "https://trusttasks.org/spec/vtc/members/make-membership/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-16T11:42:00Z",
  "threadId": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-9a0b1c2d3e4f",
  "payload": {
    "subject": "did:web:carol.example",
    "role": "admin",
    "status": "admitted",
    "returning": false,
    "approval": "approved",
    "statusListIndex": 7,
    "vmc": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "DTGCredential", "MembershipCredential"],
      "issuer": "did:webvh:QmCommunity:community.example",
      "credentialSubject": { "id": "did:web:carol.example" }
    },
    "roleVac": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "DTGCredential", "AuthorityCredential"],
      "issuer": "did:webvh:QmCommunity:community.example",
      "credentialSubject": {
        "id": "did:web:carol.example",
        "authority": { "scope": "did:webvh:QmCommunity:community.example", "actions": ["role:admin"] }
      }
    }
  }
}
```

The credentials are abbreviated here; the community returns them whole, proofs included.

### Refused: a removed member

A request naming `did:web:dave.example`, a former member whose entry was kept when he was removed.

```json
{
  "id": "urn:uuid:3a4b5c6d-7e8f-4091-a2b3-c4d5e6f7a8b9",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-16T09:20:01Z",
  "threadId": "urn:uuid:4b5c6d7e-8f90-4a1b-b2c3-d4e5f6a7b8c9",
  "payload": {
    "code": "vtc/members/make-membership:removedMember",
    "message": "did:web:dave.example was a member and was removed; re-admit them through the join flow, or revoke the entry and admit them with vtc/members/admit.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request is a DID and an optional reason of at most 1024 characters. `reason` lands in the audit trail and is shown to the administrators asked to approve a referred operation; it is not shown to the subject. A producer **MUST NOT** put a third party into it.

The response returns the membership and the two credentials issued for it. They name the subject and the community and are the subject's to hold; the administrator receives them because the community already holds them, and a presentation of either must be signed by the subject to be of use. When the requester makes its own entry a membership, as a bootstrap administrator does, they are the requester's own.

### Correlation

The community maintainer is addressed by its public DID, the identifier members, administrators and verifiers already know it by. The administrator declares `identifierScope: pairwise`: the audit trail must name the same administrator over time, and nothing here needs anyone outside the community to recognise them.

The operation links an identifier the community already holds authority for to a membership credential. A role credential naming an administrative role, presented elsewhere, tells a verifier that its holder administers this community — which an entry without a membership did not disclose to anyone outside it. A subject who would rather not carry that should hold its administrative authority and its membership under different identifiers.

### Retention

Durable. The member record and the credentials last as long as the membership; the audit row — who made whose entry a membership, on whose approval, and why — outlasts it.

### Consent/purpose

The purpose is to give an entry holder the membership it lacks. The `reason` is collected for the community's own accountability and for the approvers of a referred operation, and **SHOULD NOT** be reused for anything else. The member's consent to having their membership published is not given here; it is the member's to give. Whether the operation needs a second administrator, or a step-up, is the community's policy; this specification describes the model and does not require a gate ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
