---
slug: vtc/members/personhood/assert
version: "0.1"
title: VTC Members Personhood — Assert
summary: A member asserts personhood by presenting a Verifiable Presentation that satisfies the community's personhood policy; re-issues their credentials with the personhood flag set.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - personhood
  - assert
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
  rationale: The carried Verifiable Presentation is the personhood evidence, gated by its own proof over the challenge; a framework proof adds session attribution but is not the gate.
sideEffects:
  level: mutating
  rationale: "Sets the member's personhood flag and re-issues their credentials; reversible via personhood/revoke."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: true
  rationale: "The asserting member is the subject — they present a Verifiable Presentation held as themselves, exercising their own authority over their own personhood state."
errorCodes:
  - code: vtc/members/personhood/assert:notFound
    meaning: No member with the supplied did exists.
    retryable: false
  - code: vtc/members/personhood/assert:challengeExpired
    meaning: The presentation's challenge is unknown or expired. Open a new personhood/challenge.
    retryable: false
  - code: vtc/members/personhood/assert:presentationInvalid
    meaning: The Verifiable Presentation failed verification, its holder did not match did, or it did not satisfy the community's active personhood policy.
    retryable: false
---

## Abstract

The **VTC Members Personhood — Assert** Trust Task asserts that the member `did` is a person. The member submits a W3C Verifiable Presentation whose `holder` is `did`, whose `proof.challenge` is the `challengeId` from a prior [`vtc/members/personhood/challenge`](../../challenge/0.1/), and which carries at least one credential satisfying the community's active personhood policy. On success the community sets the personhood flag and re-issues the member's membership (`vmc`) and role (`roleVec`) credentials.

## Conformance

Producer: supply `did` and the `presentation`. The presentation's holder MUST equal `did` and its `proof.challenge` MUST be the paired `challengeId`.

Consumer: resolve the member (`notFound` if absent). Verify the presentation; if the challenge is unknown/expired return `challengeExpired`, and if verification fails, the holder mismatches, or the community's personhood policy is not satisfied, return `presentationInvalid` and change nothing. Otherwise set personhood, re-issue the credentials, and return `{ did, personhood: true, vmc, roleVec }`. Audit the assertion.

## Security & Privacy

**The presentation is the gate.** Personhood is proven by the VP over the single-use challenge, not by the framework proof — so `proofRequirement` is RECOMMENDED (for session attribution) rather than the primary control. The holder-match and challenge binding together stop one member asserting personhood on another's behalf or replaying an old presentation.
