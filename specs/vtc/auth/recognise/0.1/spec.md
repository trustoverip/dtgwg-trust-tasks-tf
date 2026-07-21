---
slug: vtc/auth/recognise
version: "0.1"
title: VTC Auth — Recognise
summary: Mint a scoped cross-community session by presenting a foreign community's endorsement + membership credentials; maps the foreign role to a local one via policy.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - auth
  - recognise
  - cross-community
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: foreign community member
    requirement: REQUIRED
    member: issuer
  - role: community
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Sender authentication is carried by the presented credentials (the vec/vmc
    proofs), not the framework proof. A framework proof is recommended for
    channel binding but the credential proofs are the primary evidence.
sideEffects:
  level: mutating
  rationale: "Mints a cross-community session token; recoverable by letting it expire or revoking it."
consequences:
  - "Issues a scoped access token granting the caller a mapped local role, effective until expiry."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/auth/recognise:credentialInvalid
    meaning: The vec or vmc failed proof verification, was expired, or was revoked via credentialStatus.
    retryable: false
  - code: vtc/auth/recognise:issuerNotRecognised
    meaning: The foreign issuer DID is not in this community's cross_community_roles policy.
    retryable: false
  - code: vtc/auth/recognise:roleNotMapped
    meaning: The foreign role has no mapping to a local role under policy.
    retryable: false
---

## Abstract

The **VTC Auth — Recognise** Trust Task lets a member of a *foreign* Verifiable
Trust Community obtain a scoped session in *this* community by presenting two
foreign-issued credentials:

- **`vec`** — a Verifiable Endorsement Credential carrying the role grant, and
- **`vmc`** — a Verifiable Membership Credential proving membership (optionally
  with `credentialStatus` for live revocation).

Both are opaque W3C credentials here, each self-authenticating via its own
`eddsa-jcs-2022` proof. The community's `cross_community_roles` policy decides
whether the foreign issuer is recognised and which **local** role the foreign
role maps to.

The response returns a `sessionId` prefixed `xc-` (cross-community, distinct
from a local-member session) and a short-lived `accessToken` whose expiry is
clamped to the earliest of the community default and the credentials'
`validUntil`.

## Conformance

Producer: present a `vec` and a `vmc` issued by a foreign community.

Consumer: verify both credential proofs; reject on invalid/expired/revoked
(`credentialInvalid`). Resolve the foreign issuer against
`cross_community_roles` (`issuerNotRecognised` if absent). Map the foreign role
to a local role (`roleNotMapped` if unmapped). Mint an `xc-` session with the
**mapped local role** — never the raw foreign role — and an expiry clamped to
`min(local default, earliest credential validUntil)`.

## Security & Privacy

**Cross-trust boundary.** The `mappedRole` MUST come from local policy; a
foreign credential never asserts a local role directly. The `xc-` session
prefix keeps cross-community sessions auditably distinct from local ones.
Expiry clamping ensures a session cannot outlive the evidence that justified
it. `discloses: metadata` — the response reveals the foreign issuer DID and
the mapped role.
