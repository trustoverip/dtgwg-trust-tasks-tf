---
slug: vtc/members/role/set
version: "0.1"
title: "VTC Members — Role Set"
summary: "An administrator moves a subject's community role — member, moderator, issuer, admin or a custom role — with a compare-and-swap on the current role, carrying the administrative authority the role implies and re-issuing the role credential."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - members
  - community role
  - promote
  - demote
  - role credential
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
  rationale: "A community role move can make a member an administrator or take that standing away, and re-issues the credential the subject presents its role with. It must be attributable to one administrator after the transport has closed — by the administrators asked to approve a parked move, by the subject a demotion is reported to, and by the audit trail."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A role move overwrites the entry's role rather than incrementing it, so a stale copy applied out of order reinstates a role an administrator has already moved the subject off. The compare-and-swap on `fromRole` refuses most such copies, and the issue time bounds the window SPEC §7.2 item 11 must hold a duplicate for."
sideEffects:
  level: mutating
  rationale: "Moves the subject's community role, moves the administrative authority with it where that authority is the one the role implies, and issues a role credential at the new role. Recoverable by moving the role back, which issues another credential."
consequences:
  - "The subject holds the new community role, and — where its administrative authority was the one its previous role implied — the authority the new role implies, at its next authorization decision."
  - "A move away from an administrative role is a privilege reduction effective at once, not at the expiry of a session or token."
  - "The subject is issued a role credential at the new role. The credential it held before is not revoked by this task and stays verifiable until it expires."
subjectPath: /subject
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names a DID, two roles and an operator-authored reason. The response returns the subject's access-control entry and the role credential issued to it — a credential naming the subject and the community, which the subject receives anyway and which a presentation must be signed by the subject to use."
retention:
  class: durable
  rationale: "The chain of role moves for a subject is how the community, and the subject, reconstruct how administrative standing was acquired and withdrawn, on whose word and on whose approval."
errorCodes:
  - code: vtc/members/role/set:notFound
    meaning: "`subject` holds no access-control entry at the community, or none the requester can reach."
    retryable: false
  - code: vtc/members/role/set:stateMismatch
    meaning: "The subject's current community role is not `fromRole`; the move was based on stale state. Re-read the member and send the move from the role it holds."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [currentRole]
      properties:
        currentRole: { type: string, minLength: 1, maxLength: 128 }
  - code: vtc/members/role/set:roleNotRecognized
    meaning: "`fromRole` or `toRole` is not a community role this community recognises."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [offendingRole]
      properties:
        offendingRole: { type: string, minLength: 1, maxLength: 128 }
  - code: vtc/members/role/set:roleNotAssignable
    meaning: "`toRole` or the subject's current role is `application`, which marks a non-member the community records for its resource grants rather than a role a member holds. A non-member is made a member with vtc/members/admit."
    retryable: false
  - code: vtc/members/role/set:delegationExceedsGranter
    meaning: "Under the new role the entry would hold administrative authority — a capability, or a capability to approve — that the requester does not hold itself."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [axes]
      properties:
        axes:
          type: array
          minItems: 1
          items:
            type: string
            enum: [capabilities, approveCapabilities]
  - code: vtc/members/role/set:lastRoleAssigner
    meaning: "The move would end the subject's role-assignment capability and leave no other live holder of it, or too few to meet the community's consent threshold for authority-conferring grants. Make another administrator, or lower the threshold, first."
    retryable: false
  - code: vtc/members/role/set:deniedByPolicy
    meaning: "The community's role-change policy refuses this move."
    retryable: false
related:
  - vtc/members/admit
  - vtc/members/make-membership
  - vtc/members/update
  - vtc/members/authority-reduced-notice
  - vtc/admin/actions/show
  - acl/change-role
  - auth/step-up/approve-request
  - trust-task-next-step
---

## Abstract

**VTC Members — Role Set** moves a subject from one community role to another:
`member` to `admin`, `admin` to `member`, `member` to `moderator`, or to and
from a community's custom roles. The community role is the role a membership
holds — the one its role credential states — and at a Verifiable Trust
Community it also implies an administrative role: `admin` implies
`community-admin`, `moderator` implies `moderator`, `issuer` implies
`credential-officer`, and every other community role implies none.

The task is **state-checked**. The producer names the role the subject holds
(`fromRole`) and the one to move it to (`toRole`), and a mismatch is
`stateMismatch`, so a race against another administrator surfaces as a refusal
rather than a silent overwrite.

The move carries what a community role carries:

- **Administrative authority.** Where the entry's administrative authority is
  exactly the one its current community role implies, that authority follows
  the role: promoting a member to `admin` makes it a `community-admin`, and
  moving an `admin` to `member` withdraws it. Where the entry holds
  administrative authority stated on its own — granted, narrowed or qualified
  with [`acl/grant/0.2`](../../../../../acl/grant/0.2/spec.md) or
  [`acl/update/0.2`](../../../../../acl/update/0.2/spec.md) — that authority is
  kept, and only the community role moves.
- **The role credential.** A subject with a member record is issued a role
  credential at its new role, returned in the response and delivered to it.

**Why a task of its own.** A community role is the community's notion, not the
access-control list's. [`acl/change-role/0.2`](../../../../../acl/change-role/0.2/spec.md)
moves an entry's administrative role, a ceiling over capabilities, which is the
same operation at any maintainer. Moving a community role also means re-issuing
a membership's role credential and applying the community's own role-change
policy, and neither belongs in a generic access-control task. Version 0.1 of
`acl/change-role` also moved the community role. That move is made here now,
and `acl/change-role` is the generic access-control operation only.
[`vtc/members/update/0.1`](../../../update/0.1/spec.md) also accepts a `role`, for
any role but `admin`. This task can move any assignable role, `admin` included,
and adds the compare-and-swap.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (`issuer`) supplies `subject`, `fromRole` — the role it read from the community, never one it assumes — and `toRole`, optionally `reason`, and carries a proof.

A conforming **community** (`recipient`) checks, in this order, before its policy runs:

1. **The requester holds `vtc.roles.assign`** — read from the signer's live access-control entry, or for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.2/spec.md)) the entry of the identity it acts for, at execution time; an entry that has expired or is suspended holds nothing. Else `permissionDenied`.
2. **The subject is not the requester.** Nobody moves their own community role, in either direction: a promotion would be self-elevation, and a demotion of oneself is still a change to an administrator's standing that another administrator should make. `permissionDenied`, naming this task as what another administrator would send.
3. **The subject holds an entry.** Else `notFound`, which the community **MUST** also answer for an entry the requester may not see, whether or not it exists.
4. **The roles are assignable.** Either role not recognised is `roleNotRecognized`, naming it. `application` as `toRole`, or as the role the entry holds, is `roleNotAssignable`.
5. **The entry holds `fromRole`.** Else `stateMismatch`, with the role it holds in `details.currentRole`.
6. **The requester covers the entry.** Its own entry **MUST** hold every capability, at every qualifier, that the subject's entry holds, else `permissionDenied`: moving a role re-grants the entry, and nobody re-grants authority they could not have granted.

Then:

7. **A move to the role already held** — `fromRole` equal to `toRole` — changes nothing. The community answers with `changed: false` and the entry, writes nothing, issues no credential, and asks for no step-up or approval. Setting a role is a state, so a repeated request for it succeeds ([SPEC §7.2](/SPEC.md#72-consumer-requirements), "A target already in the requested state").
8. **The resulting entry.** The community computes the entry as it will stand: the community role set to `toRole`; the administrative authority set to the one `toRole` implies when the entry's administrative authority is exactly the one `fromRole` implies, and otherwise kept unchanged. Every other member of the entry is kept.
9. **Bounded by the requester.** The resulting entry's effective capabilities and capabilities to approve **MUST** lie within the requester's own, else `delegationExceedsGranter`, naming the axes that do not.
10. **The last role assigner.** A move that ends the subject's `vtc.roles.assign` **MUST** be refused with `lastRoleAssigner` when no other live holder of it would remain, or when fewer would remain than the community's consent threshold for authority-conferring grants requires. A community that could no longer grant or consent to authority could not repair itself.
11. **The community's role-change policy decides** whether the move happens now, is referred for other administrators' approval, or is refused (`deniedByPolicy`). The policy sees the requester's role, the subject's current role, `toRole`, and whether the move confers or withdraws administrative authority. It **MUST NOT** be able to widen items 1 to 10, nor choose a different role.

When the community holds the move:

12. **Re-authentication bound to this document.** A community that asks the requester for a step-up before it executes a move asks for one bound to a digest of this document's `type` and whole `payload`, as [`auth/step-up/approve-request/0.5`](../../../../../auth/step-up/approve-request/0.5/spec.md) defines a bound step-up and delivers it inline. It **MUST NOT** elevate a session to do so. A gesture recorded for one move **MUST NOT** authorize any other, and **MUST** be spent by the move it authorizes, before anything is written.
13. **A parked move.** A move the community parks for other administrators' approval is answered with [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md) as [`vtc/admin/actions/show/0.3`](../../../../admin/actions/show/0.3/spec.md) describes. An approval **MUST** count only from a holder of the capability the move confers or withdraws who is neither the requester nor the subject. When the approvals are complete the community executes the parked document itself, **exactly once**, and **MUST** first re-run items 1 to 11 against the community as it is then. It answers on the request's `threadId` with this task's `#response` or a `trust-task-error`.

On execution:

14. **Serialised.** The community **MUST** serialise the check of item 5 with the write, so that two concurrent moves of one subject cannot both pass it, and **MUST** serialise every move that confers `vtc.roles.assign` with every other, so that the check of item 10 and the count it relies on cannot interleave.
15. **Written and applied at once.** It **MUST** write the entry of item 8, setting `updatedAt` and `updatedBy`, and **MUST** apply it at the subject's next authorization decision. A move that withdraws administrative authority **MUST NOT** wait for a session, token or cached decision to expire, and **MUST** re-evaluate the entries whose `delegatedBy` is the subject.
16. **The role credential.** Where the subject has a member record, the community **MUST** issue a role credential at `toRole`, record it as the membership's role credential, and return it as `roleVac`. It **SHOULD** deliver it to the subject as it does on a join approval; that delivery is best effort, and the credential is in the response whether or not it succeeds. A subject with an entry and no member record has no role credential, and none is issued.
17. **A demoted administrator is told.** A move that withdraws administrative authority from the subject **MUST** be followed by [`vtc/members/authority-reduced-notice/0.1`](../../../authority-reduced-notice/0.1/spec.md) to the subject, saying whether any other administrator agreed.
18. **Audited.** The community **MUST** audit the move: the requester, the subject, `fromRole`, `toRole`, the administrative authority before and after, how it was allowed (`approval`), the parked action's identifier when there was one, and `reason`. A withdrawal of administrative authority that no other administrator agreed to **SHOULD** be audited at the community's highest severity.

## Authorization

The authority this task presupposes is the **`vtc.roles.assign` capability at the community**, held over everything the subject's entry holds — the standing to say which role a member holds — read from the signer's access-control entry (or the entry a delegated signing key acts for) at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A move that confers administrative authority is a delegation of it, and is further bounded by the requester's own entry (item 9): nobody delegates what they do not hold. `stateMismatch` is a concurrency failure, not a permission one.

**The policy decides; it does not authorize.** The community's role-change policy can refuse or refer a move; it cannot allow one the checks above refuse, and it cannot choose a different role.

**The model this task is designed for.** A community implementing administrator step-up asks the requester for a re-authentication bound to this one document (item 12) before it executes a move that widens administrative authority — a promotion to `admin` in particular — and before it parks one. A community requiring consent for authority-conferring grants parks a move that confers such a capability (`vtc.roles.assign` among them) until other holders of it agree (item 13). A move that withdraws another administrator's authority is, at such a community, held for the requester's bound re-authentication and the consent of a party other than both the requester and the subject; where no such party exists, it proceeds, is reported to the subject (item 17) and is audited at the highest severity. In the Verifiable Trust Infrastructure these are VTI-OPS-050/051, VTI-APV-018 and VTI-APV-019. Whether a step-up or consent is required, and for which moves, is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

The approvers' own signed decisions, not the requester's proof, carry their consent. The requester's re-authentication is evidence that the requester took this decision, never that anyone else agreed to it.

## Definitions

- **Community role** — the role a membership holds (`member`, `moderator`, `issuer`, `admin`, `custom:<name>`), in the community's wire form, as [`vtc/_shared/0.1/member.schema.json`](../../../../_shared/0.1/member.schema.json) carries it. In an `AclEntry` 0.2 it travels in `ext["org.openvtc"].communityRole`; `entry.role` is the administrative role.
- **Implied administrative authority** — the administrative role a community role implies, at its full ceiling: `admin` → `community-admin`, `moderator` → `moderator`, `issuer` → `credential-officer`; any other community role, none.
- **`application` entry** — an access-control entry of the `application` community role: a non-member the community records because it holds resource grants. It is never a membership, and no role moves onto or off it here.
- **Role credential** (`roleVac`) — a Verifiable Authority Credential the community issues to a member, whose `credentialSubject.authority` is `{ scope: <community DID>, actions: ["role:<name>"] }`.
- `AclEntry` — see [`acl/_shared/0.2/acl-entry.schema.json`](../../../../../acl/_shared/0.2/acl-entry.schema.json). `Approval` — see [`vtc/_shared/0.1/admission.schema.json`](../../../../_shared/0.1/admission.schema.json).

## Request

The administrator sends the community the subject, the role it holds and the role to move it to. The payload is described by [`payload.schema.json`](payload.schema.json).

### Promote a member to admin

```json
{
  "id": "urn:uuid:7d1e2f30-4a5b-4c6d-8e9f-0a1b2c3d4e5f",
  "type": "https://trusttasks.org/spec/vtc/members/role/set/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-10-30T09:00:00Z",
  "threadId": "urn:uuid:7d1e2f30-4a5b-4c6d-8e9f-0a1b2c3d4e5f",
  "payload": {
    "subject": "did:web:bob.example",
    "fromRole": "member",
    "toRole": "admin",
    "reason": "Takes over moderation of the events programme."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-30T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z5RoleSetExampleProofValue"
  }
}
```

## Response

The community answers with the subject's community role as it now stands, the entry that holds it and, for a member, the role credential issued at the new role — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A held move is first answered as items 12 and 13 describe, and this response follows when it executes. A refusal is a `trust-task-error`.

### Promoted

The promotion above, once Carol's bound re-authentication was recorded and another administrator approved it. Bob's administrative authority was the one `member` implies (none), so it follows the role: he is now a `community-admin`.

```json
{
  "id": "urn:uuid:8e2f3041-5b6c-4d7e-9f0a-1b2c3d4e5f60",
  "type": "https://trusttasks.org/spec/vtc/members/role/set/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-30T09:20:00Z",
  "threadId": "urn:uuid:7d1e2f30-4a5b-4c6d-8e9f-0a1b2c3d4e5f",
  "payload": {
    "subject": "did:web:bob.example",
    "role": "admin",
    "previousRole": "member",
    "changed": true,
    "approval": "approved",
    "entry": {
      "subject": "did:web:bob.example",
      "role": "community-admin",
      "act": { "scope": "all" },
      "keys": { "scope": "none" },
      "capabilities": { "scope": "ceiling" },
      "approve": { "scope": "all" },
      "approveCapabilities": { "scope": "ceiling" },
      "delegatedBy": "did:web:carol.example",
      "createdAt": "2026-06-01T00:00:00Z",
      "createdBy": "did:web:carol.example",
      "updatedAt": "2026-10-30T09:20:00Z",
      "updatedBy": "did:web:carol.example",
      "ext": { "org.openvtc": { "communityRole": "admin" } }
    },
    "roleVac": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "DTGCredential", "AuthorityCredential"],
      "issuer": "did:webvh:QmCommunity:community.example",
      "validUntil": "2026-11-29T09:20:00Z",
      "credentialSubject": {
        "id": "did:web:bob.example",
        "authority": { "scope": "did:webvh:QmCommunity:community.example", "actions": ["role:admin"] }
      }
    }
  }
}
```

The credential is abbreviated here; the community returns it whole, proof included.

### Refused: the role moved underneath

```json
{
  "id": "urn:uuid:9f304152-6c7d-4e8f-a01b-2c3d4e5f6071",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-30T09:00:01Z",
  "threadId": "urn:uuid:7d1e2f30-4a5b-4c6d-8e9f-0a1b2c3d4e5f",
  "payload": {
    "code": "vtc/members/role/set:stateMismatch",
    "message": "did:web:bob.example holds the role moderator, not member.",
    "retryable": true,
    "details": { "currentRole": "moderator" }
  }
}
```

The administrator re-reads the member and, if the move still makes sense, sends it from `moderator`.

## Security & Privacy

**The previous role credential outlives the move.** A role credential states the role a member held when it was issued, and this task does not revoke the one it replaces. A member demoted from `admin` can therefore still present a credential saying `role:admin` until that credential's `validUntil`. A community **SHOULD** issue role credentials short-lived for that reason, and a relying party that needs a member's *current* role **SHOULD** treat a role credential as evidence of the role at issuance only. Authority at the community itself never rests on the credential: it is read from the entry at each decision (item 15).

**Promotion is the high-value target.** A move to `admin` confers the power to grant authority. That is why the requester must already hold that power over everything the subject holds (items 1, 6 and 9), why the move can be held for a re-authentication bound to this one document rather than to a session (item 12) — a session elevation would let a process holding the requester's signing key spend it on moves the requester never saw — and why a community can require other holders' consent (item 13).

**A demotion is not deferred.** Withdrawn authority applies at the subject's next authorization decision, and the subject's delegations are re-evaluated (item 15). A demotion that waited for a session to expire would leave a window in which the administrator being removed still acts as one.

### Data carried

The request carries a DID, two community roles and an optional reason of at most 1024 characters. `reason` lands in the audit trail and is shown to the administrators asked to approve a parked move, and — for a demotion — in the notice the subject receives (item 17). A producer **MUST NOT** put a third party into it, and **SHOULD** write it for the subject as well as the approvers.

The response returns the subject's access-control entry and, for a member, the role credential issued to it. Both are the community's records about the subject, returned to an administrator entitled to change them. The credential is the subject's to hold, and a presentation of it must be signed by the subject to be of use.

### Correlation

The community maintainer is addressed by its public DID, the identifier members, administrators and verifiers already know it by. The administrator declares `identifierScope: pairwise`: the audit trail must name the same administrator over time, and nothing here needs anyone outside the community to recognise them.

Each move links the subject's DID to a role at the community, durably, in the entry, the role credential and the audit trail. The role credential is presented by the subject at its own discretion. A relying party that sees two of them can tell the subject's role changed between their issuance.

### Retention

Durable. The entry lasts as long as the subject's standing. The audit row of each move — who moved whom, between which roles, with what administrative authority before and after, on whose approval and why — outlasts it, because the chain of moves is how the community and the subject reconstruct how administrative standing was acquired and withdrawn. A community **SHOULD** keep it on the same schedule as its other records of granted and withdrawn authority.

### Consent/purpose

The purpose is the community's governance of who holds which role. The `reason` is collected for the community's accountability, for the approvers of a parked move and for a demoted subject, and **SHOULD NOT** be reused for anything else. The subject's consent is not sought: a role is the community's decision about its own membership. A demoted administrator is told (item 17), so that the reduction is never silent. Whether a move needs a second administrator, or a step-up, is the community's policy; this specification describes the model and does not require a gate ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
