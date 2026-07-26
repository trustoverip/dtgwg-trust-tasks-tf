---
slug: vtc/members/vmc
version: "0.1"
title: VTC Members — Deliver VMC
summary: A member delivers their issued Membership Credential to the community, which stores it and returns a receipt naming the stored credential.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - members
  - vmc
  - membership
  - reciprocal
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The community must bind the delivered credential to the member who sent it before storing it as their half of the membership pair.
sideEffects:
  level: mutating
  rationale: "Stores the member-issued credential against the membership record."
exposure:
  discloses: metadata
  actsAsSubject: true
  rationale: The member is the subject of the act — they are delivering a credential they themselves issued, about their own membership.
errorCodes:
  - code: vtc/members/vmc:subjectMismatch
    meaning: The credential's `credentialSubject.id` is not this community's DID.
    retryable: false
  - code: vtc/members/vmc:notAMember
    meaning: The sender is not an active member of this community.
    retryable: false
  - code: vtc/members/vmc:invalidCredential
    meaning: The credential does not verify, or is not a MembershipCredential.
    retryable: false
---

## Abstract

The **VTC Members — Deliver VMC** Trust Task carries a member-issued Membership Credential **to** the community — the member → community half of the reciprocal membership pair, usually in answer to [`vtc/members/request-vmc`](../../request-vmc/0.1/) but valid unsolicited.

The credential's type tag is `MembershipCredential`, the canonical DTG / W3C tag, not `VerifiableMembershipCredential`. The `#response` receipt names the stored credential so the member knows which artifact the community now holds.

## Conformance

Producer (the member): send the signed credential in `vc`. Its `credentialSubject.id` MUST be the community's DID.

Consumer (the community): bind the credential to the **proven sender** — the delivering member — and reject a sender who is not an active member with `notAMember`. Verify the credential and reject a `credentialSubject.id` that is not this community's DID with `subjectMismatch`: a credential about some other community is not this member's half of the pair, however well-formed. Store it against the membership record and return a receipt naming `vmcId`.

Delivery is idempotent. Re-delivering the same credential MUST succeed and MUST NOT create a second stored copy.

## Security & Privacy

The subject check is what stops a member satisfying their reciprocal obligation with a credential about someone else. The sender binding is what stops one member delivering on another's behalf; both are required because the credential is self-issued and therefore says only what its issuer chose to say.

`actsAsSubject` is `true`: the member is acting on their own membership, which is why the task is available to an ordinary member rather than gated on an administrative capability.
