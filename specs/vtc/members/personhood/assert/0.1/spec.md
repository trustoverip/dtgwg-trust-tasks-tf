---
slug: vtc/members/personhood/assert
version: "0.1"
title: VTC Members Personhood — Assert
summary: A member asserts personhood by presenting a Verifiable Presentation that satisfies the community's personhood policy; re-issues their credentials with the personhood flag set.
status: draft
targetFrameworkVersion: "0.5"
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
  requirement: REQUIRED
  rationale: The carried Verifiable Presentation remains the personhood evidence, gated by its own proof over the challenge — the framework proof is not that gate and does not replace it. It is required because execution acts with the subject's authority to set a personhood flag and re-issue their credentials, so the request must be attributable to the party that made it. Replay of a captured VP under a different envelope, and repudiation of the assertion afterwards, are the threats addressed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The assertion is made under the member's own authority and becomes the basis of how the community treats them. A replayed assertion re-asserts personhood at a moment the member did not choose and may no longer stand behind.
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

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.2, where the declaration is not yet required.*

The authorization evidence this task presupposes is the **presentation**, whose holder MUST equal the `did` being asserted and whose `proof.challenge` MUST be the paired `challengeId`.

Both halves are load-bearing and neither substitutes for the other. Holder equality establishes that the assertion is about the party making it; the challenge binding establishes that this presentation was made for this exchange, and is what stops one captured and replayed into another. A consumer that checks only the first accepts replays; one that checks only the second accepts an assertion made on someone else's behalf.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

**The presentation is the gate.** Personhood is proven by the VP over the single-use challenge, not by the framework proof — so `proofRequirement` is RECOMMENDED (for session attribution) rather than the primary control. The holder-match and challenge binding together stop one member asserting personhood on another's behalf or replaying an old presentation.
