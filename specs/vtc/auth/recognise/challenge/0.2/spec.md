---
slug: vtc/auth/recognise/challenge
version: "0.2"
title: VTC Auth Recognise — Challenge
summary: Issue the single-use nonce a foreign community's member binds into the presentation they use to obtain a cross-community session.
status: draft
targetFrameworkVersion: "0.5.0"
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A challenge is only meaningful while the verifier is still waiting on it. Carrying its issue time is what allows a challenge to be aged out rather than answered indefinitely.
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

The **VTC Auth Recognise — Challenge** Trust Task issues the nonce that anchors cross-community recognition. A member of *another* community requests one, binds it into a Verifiable Presentation of their foreign membership credential, and presents that to [`vtc/auth/recognise`](../../../recognise/0.3/) to obtain a session here.

The nonce is what makes the presentation fresh. Without it, a presentation captured once could be replayed indefinitely.

## Changes from 0.1

- The response's **`expiresAt`** changes type from an integer Unix timestamp
  (seconds) to an RFC 3339 date-time (`type: "string"`, `format: "date-time"`).
  Its name and meaning are unchanged: the instant after which the nonce is
  refused. `1792747800` in 0.1 is `"2026-10-23T09:30:00Z"` here.
- The presentation the nonce is bound into is now presented to
  [`vtc/auth/recognise/0.3`](../../../recognise/0.3/), which makes the same
  change to its `accessExpiresAt`.

The reason is the registry-wide convention that an instant on the wire is an
RFC 3339 string whose member name ends in `At`. The request payload is
unchanged.

Breaking on the wire, released as a `MINOR` increment under
[§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains
published.

## Conformance

Producer: no payload members are required — the caller is unauthenticated and identifies itself only in the presentation that follows.

Consumer: issue a single-use, unpredictable nonce with a short expiry and return it with `expiresAt`. Bind it to nothing else: the caller's identity is not yet known and MUST NOT be inferred from anything they send here. Consume the nonce on first use at `recognise`, whether or not that attempt succeeds — a nonce that survives a failed presentation is replayable against a second one.

Rate-limit by source. This endpoint is pre-authentication and drives downstream credential verification.

## Security & Privacy

### Data carried

This task is deliberately information-free in both directions. It accepts nothing that could be used to probe for members, and returns nothing but a random value and its expiry, so `exposure.discloses` is `none`.

### Correlation

The nonce is bound to nothing else at issue — the caller's identity is not yet known here — and it joins this exchange to the single `vtc/auth/recognise` presentation that commits to it. `threadId` joins the request to its response; the transport source the consumer rate-limits by is visible to it but carried in no payload member. This specification imposes no further rule on correlation.

### Retention

The nonce is persisted with a short expiry and refused after `expiresAt`. Single-use consumption is the load-bearing property, and it belongs on the *use* side rather than here: this task cannot know whether the presentation it enables will succeed, so the nonce must be burned by the consumer of the presentation regardless of verdict. This specification sets no further retention rule.

### Consent/purpose

The nonce exists only to make a cross-community presentation fresh. This specification places no further limit on its use.
