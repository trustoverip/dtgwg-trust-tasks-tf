---
slug: vtc/members/admit
version: "0.1"
title: "VTC Members — Admit"
summary: "An administrator admits a DID as a member without a join request: the access-control entry, the member record and the membership credentials a completed join leaves behind, at a community role the administrator names, decided by the community's admission policy."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - members
  - admission
  - direct-admission
  - membership
  - onboarding
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
  rationale: "Admitting a member bypasses the community's join criteria and vetting on one administrator's word. The admission must be attributable to that administrator after the transport has closed — by the approvers of a referred admission, by the audit trail, and by a later reviewer asking how someone who never applied became a member."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed admission re-admits a member removed since it was first executed, without the decision re-admitting a former member takes."
sideEffects:
  level: mutating
  rationale: "Writes an access-control entry, a member record and a revocation slot, and issues a membership credential and a role credential. Recoverable by removing the member (vtc/members/admin-remove), which revokes the credentials."
consequences:
  - "The subject becomes a member of the community at the named role, holding a membership credential and a role credential the community signed."
  - "A role other than `member` carries the authority that role implies, within the requester's own."
subjectPath: /subject
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names a DID, a role, an optional label and an operator-authored reason. The response returns the membership and the two credentials issued to the subject — credentials naming the subject and the community, which the subject would receive anyway and which a presentation must be signed by the subject to use."
retention:
  class: durable
  rationale: "The admission is the community's record of how this member came to be one without applying, by whose word and on whose approval. A community that discarded it could not later explain a membership nobody vetted."
errorCodes:
  - code: vtc/members/admit:hasEntry
    meaning: "`subject` already holds an access-control entry of a role other than `application`. An entry with no member record is made a membership with vtc/members/make-membership, which keeps its role and authority."
    retryable: false
  - code: vtc/members/admit:alreadyMember
    meaning: "`subject` already holds a member record that has not been removed."
    retryable: false
  - code: vtc/members/admit:roleNotPermitted
    meaning: "The community does not recognise `role`, or it may not be seated by a direct admission (`admin`, `application`, or a role whose authority would confer authority-conferring capabilities), or the requester may not assign it: a role other than `member` takes the role-assignment capability and must lie within the requester's own authority."
    retryable: false
  - code: vtc/members/admit:deniedByPolicy
    meaning: "The community's admission policy refuses this admission."
    retryable: false
related:
  - vtc/members/make-membership
  - vtc/members/admin-remove
  - vtc/join-requests/decide
  - vtc/admin/actions/show
  - acl/grant
  - acl/change-role
  - trust-task-next-step
---

## Abstract

**VTC Members — Admit** lets an administrator admit a DID as a member of the community without a join request: *Add a member*. The community writes what a completed join leaves behind — an access-control entry at the community role the administrator names, a member record, a revocation slot — and issues the membership credential (VMC) and role credential (VAC) for it, which it returns in the response and delivers to the member as on a join approval.

A former member — a removed member record and no access-control entry — is re-admitted as a rejoin would re-admit them, and the response says so (`returning`). A subject holding only an `application` entry (a non-member the community records because it holds resource grants, such as a forge bridge or an external signer) is admitted onto that entry, which keeps its grants.

An access-control entry that already exists with no member record — the community's bootstrap administrator is the case — is not admitted here; [`vtc/members/make-membership`](../../make-membership/0.1/spec.md) gives it a membership at the role it already holds.

**Why a task of its own.** No existing task fits. [`acl/grant`](../../../../acl/grant/0.2/spec.md) writes an access-control entry and nothing else, and carrying "and admit them" in its `ext` would make a membership — credentials, a revocation slot, a policy decision — ride on a member an older community is free to ignore. [`vtc/members/update`](../../update/0.1/spec.md) edits a membership that exists, and the join tasks are an applicant's.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (`issuer`) supplies `subject` and `role`, and optionally `label` and `reason`, and carries a proof. A producer that has no reason to name a role sends `member`.

A conforming **community** (`recipient`) checks, in this order, before its policy runs:

1. **The requester holds `vtc.members.manage`** — read from the signer's live access-control entry, or for a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.1/spec.md)) the entry of the identity it acts for, at execution time; an entry that has expired or is suspended holds nothing. Else `permissionDenied`.
2. **The role may be seated by a direct admission.** It **MUST** refuse with `roleNotPermitted` a role it does not recognise, `application` (which marks a non-member, never a membership), and `admin`. An administrator is admitted as a member and then promoted with [`acl/change-role`](../../../../acl/change-role/0.2/spec.md) or granted with `acl/grant`, which carry their own rules for conferring that authority.
3. **A role other than `member` is an assignment of a role**, held to the rule an invitation's role is held to. The community **MUST** refuse with `roleNotPermitted` when the requester does not hold `vtc.roles.assign`; when the entry the role would seat holds any capability, at any qualifier, that the requester's own entry does not hold; and when that entry would hold a capability that permits its holder to create or widen authority. A direct admission never confers such a capability; granting one is `acl/grant`'s, under the community's rules for authority-conferring grants.
4. **The subject is not already a member or an entry holder.** The community **MUST** refuse with `hasEntry` a subject holding an access-control entry of any role but `application`, and with `alreadyMember` a subject whose member record exists and has not been removed. A removed member record is not a refusal: the subject is admitted as a returning member.
5. `label`, when given, is 1 to 256 characters and not blank; `reason` is at most 1024 characters. The schema states both; a document that fails them is `malformedRequest`.

Then:

6. **The community's policy for direct admission decides** whether the admission happens now, is referred for another administrator's approval, or is refused (`deniedByPolicy`). The policy sees the action (`add`), the role, whether that role carries administrative authority, and whether the subject is returning. It **MUST NOT** be able to widen items 1 to 5, nor choose a different role.
7. **A referred admission** is parked, and the community answers with [`trust-task-next-step/0.1`](../../../../trust-task-next-step/0.1/spec.md) as [`vtc/admin/actions/show/0.2`](../../../admin/actions/show/0.2/spec.md) describes. An approval **MUST** count only from a holder of `vtc.members.manage` who is neither the requester nor the subject. When the approvals are complete the community executes the parked document itself, **exactly once**, and **MUST** first re-run items 1 to 6 against the community as it is then; it answers on the request's `threadId` with this task's `#response` or a `trust-task-error`.
8. **On execution** the community **MUST** write the entry at the requested role with the requested `label` (onto the existing entry for an `application` subject, keeping its resource grants), write the member record — with no publication consent, which is the member's to give — allocate a revocation slot, issue the VMC and the role VAC, and only then answer. It **MUST** audit the admission: the requester, the subject, the role, the label, whether the subject was returning, how it was allowed (`approval`), the parked action's identifier when there was one, and `reason`. It **SHOULD** deliver the credentials to the member as it does on a join approval; that delivery is best effort, and the credentials are in the response whether or not it succeeds.

## Authorization

The authority this task presupposes is the **`vtc.members.manage` capability at the community** — the standing to manage its membership — read from the signer's access-control entry (or the entry a delegated signing key acts for) at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A role beyond `member` additionally needs **`vtc.roles.assign`**, and is bounded by the requester's own entry: an admission is a delegation of whatever authority the role implies, and nobody delegates what they do not hold. No role that confers the power to create authority is ever seated this way.

**The policy decides; it does not authorize.** Admission skips the join criteria and vetting, which exist to stop exactly that, so the community decides through its own policy whether its administrators may admit alone. The policy can refuse or refer; it cannot admit someone the checks above refuse. A community whose policy refers admissions requires a second holder of `vtc.members.manage` to agree, and that approver's own signed decision — not the requester's proof — is what carries the consent.

**The model this task is designed for.** A community implementing administrator step-up asks the requester for a re-authentication bound to this one document before it executes an admission at a role carrying administrative authority, and before it parks a referred admission; it answers with the refusal its own step-up mechanism defines, and the same document is sent again once the step is done. A community with no other administrator to approve, running in a single-administrator mode, may waive a referral on the requester's own bound re-authentication, and records the admission as `waived`. Whether a step-up or consent is required, and for which admissions, is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- **Community role** — the role a membership holds (`member`, `moderator`, `issuer`, `admin`, `custom:<name>`), in the community's wire form, as [`vtc/_shared/0.1/member.schema.json`](../../../_shared/0.1/member.schema.json) carries it.
- **`application` entry** — an access-control entry of the `application` community role: a non-member the community records because it holds resource grants. It is never a membership.
- **Returning member** — a subject with a removed member record and no access-control entry.
- `AdmittedMembership`, `Approval` — see [`vtc/_shared/0.1/admission.schema.json`](../../../_shared/0.1/admission.schema.json).

## Request

The administrator sends the community the DID to admit and the role. The payload is described by [`payload.schema.json`](payload.schema.json).

### Add a member

```json
{
  "id": "urn:uuid:5b0f6a2e-0c1d-4b8e-9a51-2f7d3c4e8a10",
  "type": "https://trusttasks.org/spec/vtc/members/admit/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-10-16T09:00:00Z",
  "threadId": "urn:uuid:5b0f6a2e-0c1d-4b8e-9a51-2f7d3c4e8a10",
  "payload": {
    "subject": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "role": "member",
    "label": "Dana (founding contributor)",
    "reason": "Founding contributor, admitted before the join flow opened."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-16T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7cDExampleProofValue"
  }
}
```

## Response

The community answers with the membership as admitted and the credentials issued for it — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json), which is `AdmittedMembership`. A referred admission is first answered with `trust-task-next-step`, and this response follows on the same `threadId` when it executes. A refusal is a `trust-task-error`.

### Admitted at once

```json
{
  "id": "urn:uuid:9e8d7c6b-5a49-4382-8170-6f5e4d3c2b1a",
  "type": "https://trusttasks.org/spec/vtc/members/admit/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-16T09:00:01Z",
  "threadId": "urn:uuid:5b0f6a2e-0c1d-4b8e-9a51-2f7d3c4e8a10",
  "payload": {
    "subject": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "role": "member",
    "label": "Dana (founding contributor)",
    "status": "admitted",
    "returning": false,
    "approval": "policyAllowed",
    "statusListIndex": 42,
    "vmc": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "DTGCredential", "MembershipCredential"],
      "issuer": "did:webvh:QmCommunity:community.example",
      "credentialSubject": { "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK" }
    },
    "roleVac": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "DTGCredential", "AuthorityCredential"],
      "issuer": "did:webvh:QmCommunity:community.example",
      "credentialSubject": {
        "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "authority": { "scope": "did:webvh:QmCommunity:community.example", "actions": ["role:member"] }
      }
    }
  }
}
```

The credentials are abbreviated here; the community returns them whole, proofs included.

### Refused: the subject already holds an entry

```json
{
  "id": "urn:uuid:1a2b3c4d-5e6f-4a7b-8c9d-0e1f2a3b4c5d",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-16T09:00:01Z",
  "threadId": "urn:uuid:5b0f6a2e-0c1d-4b8e-9a51-2f7d3c4e8a10",
  "payload": {
    "code": "vtc/members/admit:hasEntry",
    "message": "did:key:z6Mkha… already has an access-control entry; if it has no membership, make it one with vtc/members/make-membership.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request is a DID, a role, an optional label of at most 256 characters and an optional reason of at most 1024. `label` lands on the access-control entry, where administrators read it; `reason` lands in the audit trail and is shown to the administrators asked to approve a referred admission. Neither is shown to the subject. A producer **MUST NOT** put a third party into either — a referee, another applicant — and **SHOULD** keep `label` to what an administrator needs to recognise the entry.

The response returns the membership and the two credentials issued for it. They name the subject and the community and are the subject's to hold; the administrator receives them because the community already holds them, and a presentation of either must be signed by the subject to be of use.

### Correlation

The community maintainer is addressed by its public DID, the identifier members, administrators and verifiers already know it by. The administrator declares `identifierScope: pairwise`: accountability for an admission belongs inside the community, where the audit trail must name the same administrator over time, and nothing here needs anyone outside to recognise them.

An admission links the subject's DID to the community durably — in the member record, the credentials and the revocation slot — exactly as a join does. A returning admission links the new membership to the removed record of the old one, and `returning` says so to the requester; that link is what lets the community see that it is re-admitting someone it removed.

### Retention

Durable. The entry, the member record and the credentials last as long as the membership; the audit row of the admission — who admitted whom, at what role, on whose approval, and why — outlasts it, because an admission that skipped vetting is the record a later reviewer most needs. A community **SHOULD** keep it on the same schedule as the record of a join decision.

### Consent/purpose

The purpose is admission: an administrator vouches for a member the community did not vet. The `reason` is collected for the community's own accountability and for the approvers of a referred admission, and **SHOULD NOT** be reused for anything else. The member's consent to having their membership published is not given here: the member record starts without it, and it is the member's to give. Whether an admission needs a second administrator, or a step-up, is the community's policy; this specification describes the model and does not require a gate ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
