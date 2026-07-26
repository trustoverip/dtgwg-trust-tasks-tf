---
slug: vtc/admin/invites/create
version: "0.1"
title: VTC Admin Invites — Create
summary: Mint a one-shot install URL and claim code that lets a named DID enrol an administrator passkey for a Verifiable Trust Community.
status: draft
targetFrameworkVersion: "0.2"
category: access-control
keywords:
  - vtc
  - admin
  - invite
  - passkey
  - enrolment
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Mints a credential-bearing URL that grants administrator access. The caller must be attributable.
sideEffects:
  level: mutating
  rationale: "Persists an invite and, when the target DID has no entry, creates its ACL row."
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: Returns a claim code and install URL that are, until redeemed or expired, sufficient to enrol an administrator passkey.
errorCodes:
  - code: vtc/admin/invites/create:ttlTooLong
    meaning: The requested `ttlSeconds` exceeds the 24-hour maximum.
    retryable: false
  - code: vtc/admin/invites/create:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Admin Invites — Create** Trust Task mints a single-use install URL for `did`, letting that identity enrol an administrator passkey. It returns the URL, a `claimCode`, and the invite's `jti` — the handle [`vtc/admin/invites/revoke`](../../revoke/0.1/) acts on.

If the target DID has no ACL entry, one is created; `aclEntryCreated` reports whether that happened, so an operator can tell "invited an existing admin" from "granted a new one".

Split from the former `admin/invites/manage` mount — see [`vtc/admin/invites/list`](../../list/0.1/) for why.

## Conformance

Producer: supply the target `did`. `ttlSeconds` is optional and MUST NOT exceed 86400; `label` is an operator-facing note.

Consumer: verify the community-admin capability. Clamp or reject a `ttlSeconds` over the 24-hour maximum with `ttlTooLong` rather than silently issuing a longer-lived credential. Return the claim code and install URL **once** — they are not retrievable afterwards, and `list` MUST NOT disclose them.

## Security & Privacy

**The response is the credential.** Until it is redeemed or expires, whoever holds the `installUrl` and `claimCode` can enrol an administrator passkey for the named DID — which is why `exposure.discloses` is `secret` and why the 24-hour ceiling is a hard bound rather than a default.

That the values are returned exactly once, and never again by `list`, is deliberate: it keeps the window of exposure to the single response rather than to every subsequent read of the invite registry.
