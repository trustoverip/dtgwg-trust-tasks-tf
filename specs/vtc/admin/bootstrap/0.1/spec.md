---
slug: vtc/admin/bootstrap
version: "0.1"
title: VTC Admin — Bootstrap
summary: Write the first admin to a community's ACL, consuming the setup-session token from install claim.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords: [vtc, admin, bootstrap, install]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: installer
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: The single-use setup-session token from install/claim/finish is the gate; this task is what creates the first admin, so no prior admin key exists to sign.
sideEffects:
  level: mutating
  rationale: "Writes the first admin ACL entry and records the CommunityInstalled audit envelope."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/admin/bootstrap:invalidToken
    meaning: The setup-session token is missing, malformed, expired, or already consumed.
    retryable: false
  - code: vtc/admin/bootstrap:alreadyBootstrapped
    meaning: The community already has an admin; bootstrap is single-use.
    retryable: false
---

## Abstract

The **VTC Admin — Bootstrap** Trust Task writes the first admin to the community's ACL, consuming the single-use `setupSessionToken` minted by [`vtc/install/claim/finish`](../../install/claim/finish/0.1/). It returns the `adminDid` written and the `eventId` of the persisted `CommunityInstalled` audit envelope.

## Conformance

Producer: supply `setupSessionToken`.

Consumer: verify the token (`invalidToken`); if the community already has an admin, return `alreadyBootstrapped`. Otherwise write the admin ACL entry, record the install audit envelope, and return `{ adminDid, eventId }`.

## Security & Privacy

**Single-use cold-start.** This is the one moment a community gains its first admin without a prior admin, so it is gated by the single-use setup-session token and refuses once bootstrapped (`alreadyBootstrapped`) — a second bootstrap cannot mint a second first-admin.
