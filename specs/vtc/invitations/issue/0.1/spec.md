---
slug: vtc/invitations/issue
version: "0.1"
title: VTC Invitations — Issue
summary: Issue a single-use Invitation Credential (VIC) admitting a named DID to a Verifiable Trust Community, optionally granting a role and bounded by an expiry.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - invitation
  - vic
  - join
  - credential
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: inviter
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Mints a credential that admits its bearer's subject to the community. The issuer is recorded against it and must be attributable.
sideEffects:
  level: mutating
  rationale: "Issues a credential, records it in the invitation registry, and allocates its revocation-list slot."
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: Returns the signed Invitation Credential itself, which is the bearer artifact that admits its subject.
errorCodes:
  - code: vtc/invitations/issue:permissionDenied
    meaning: The consumer lacks the inviter capability (admin, moderator, or issuer).
    retryable: false
  - code: vtc/invitations/issue:unknownRole
    meaning: The requested `role` is not one this community defines.
    retryable: false
---

## Abstract

The **VTC Invitations — Issue** Trust Task mints an Invitation Credential (VIC) for `subjectDid`. The VIC is the artifact a prospective member presents to join: it is single-use, optionally role-granting, and revocable through a published status list.

The signed credential is returned **once**, in `vic`. [`vtc/invitations/list`](../../list/0.1/) enumerates the registry afterwards but never re-discloses credential material; [`vtc/invitations/revoke`](../../revoke/0.1/) withdraws one.

Split from the former `invitations/issue` mount, which also served the listing — see `list` for why.

## Conformance

Producer: supply `subjectDid`. `validityDays` bounds the credential; `role` names the role granted on redemption.

Consumer: verify the inviter capability (admin, moderator, or issuer — this is deliberately wider than community-admin). Reject a `role` the community does not define with `unknownRole` rather than issuing a credential that cannot be redeemed. Allocate a revocation-list slot at issuance so the credential is revocable from the moment it exists, and record the issuing DID. Return the signed VIC once.

## Security & Privacy

**The VIC is a bearer artifact**, so `exposure.discloses` is `secret`: whoever holds the returned credential can present it, regardless of who it names as subject. Single-use redemption and the revocation slot are the two controls that bound the damage from a leaked one, which is why the slot is allocated at issuance rather than lazily on first revoke.

Returning the credential exactly once — and never again from `list` — keeps the exposure window to the issuing response.
