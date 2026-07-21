---
slug: vtc/members/personhood/challenge
version: "0.1"
title: VTC Members Personhood — Challenge
summary: Open a personhood-assertion ceremony for a member — return the single-use challenge to embed in the assertion presentation.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - personhood
  - challenge
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
  requirement: RECOMMENDED
  rationale: Opening the ceremony only issues a challenge; the binding evidence is the Verifiable Presentation submitted to the assert task.
sideEffects:
  level: none
  rationale: "Issues a single-use personhood challenge; the assertion is a separate task."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/members/personhood/challenge:permissionDenied
    meaning: The consumer lacks the capability to assert personhood for this member.
    retryable: false
  - code: vtc/members/personhood/challenge:notFound
    meaning: No member with the supplied did exists.
    retryable: false
---

## Abstract

The **VTC Members Personhood — Challenge** Trust Task begins a personhood assertion for the member `did`. It returns a single-use `challengeId` and an `expiresAt`; the member embeds the `challengeId` as the `proof.challenge` of the Verifiable Presentation they submit to [`vtc/members/personhood/assert`](../assert/0.1/). The challenge anchors replay resistance for the assertion.

## Conformance

Producer: supply `did`.

Consumer: resolve the member; if none, return `notFound`. Mint a single-use, TTL'd `challengeId` bound to the member, and return it with its expiry.

## Security & Privacy

**Challenge only.** This task proves nothing on its own — personhood is demonstrated by the presentation at the assert step. The challenge is single-use and expires, so an assertion cannot replay an old ceremony.
