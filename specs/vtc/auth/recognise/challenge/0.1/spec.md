---
slug: vtc/auth/recognise/challenge
version: "0.1"
title: VTC Auth Recognise — Challenge
summary: Issue the single-use nonce a foreign community's member binds into the presentation they use to obtain a cross-community session.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - vtc
  - auth
  - recognise
  - cross-community
  - challenge
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: foreign member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: Pre-authentication. The caller has no session yet; the nonce is what they will later bind a proof to.
sideEffects:
  level: mutating
  rationale: "Persists a single-use nonce with a short expiry."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/auth/recognise/challenge:rateLimited
    meaning: Too many challenges requested from this source.
    retryable: true
---

## Abstract

The **VTC Auth Recognise — Challenge** Trust Task issues the nonce that anchors cross-community recognition. A member of *another* community requests one, binds it into a Verifiable Presentation of their foreign membership credential, and presents that to [`vtc/auth/recognise`](../../../recognise/0.1/) to obtain a session here.

The nonce is what makes the presentation fresh. Without it, a presentation captured once could be replayed indefinitely.

## Conformance

Producer: no payload members are required — the caller is unauthenticated and identifies itself only in the presentation that follows.

Consumer: issue a single-use, unpredictable nonce with a short expiry and return it with `expiresAt`. Bind it to nothing else: the caller's identity is not yet known and MUST NOT be inferred from anything they send here. Consume the nonce on first use at `recognise`, whether or not that attempt succeeds — a nonce that survives a failed presentation is replayable against a second one.

Rate-limit by source. This endpoint is pre-authentication and drives downstream credential verification.

## Security & Privacy

This task is deliberately information-free in both directions. It accepts nothing that could be used to probe for members, and returns nothing but a random value and its expiry, so `exposure.discloses` is `none`.

Single-use consumption is the load-bearing property, and it belongs on the *use* side rather than here: this task cannot know whether the presentation it enables will succeed, so the nonce must be burned by the consumer of the presentation regardless of verdict.
