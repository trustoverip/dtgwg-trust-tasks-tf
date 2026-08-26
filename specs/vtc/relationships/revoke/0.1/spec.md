---
slug: vtc/relationships/revoke
version: "0.1"
title: VTC Relationships — Revoke
summary: The issuer revokes a Verifiable Relationship Credential they previously published.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
  - revoke
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
  rationale: Only the credential's issuer may revoke it; the proof signer is checked against the stored issuer, and the revocation must be attributable.
sideEffects:
  level: mutating
  rationale: "Revokes a relationship credential; reversible by publishing a fresh one."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/relationships/revoke:notFound
    meaning: No relationship with the supplied id exists, or the proof signer is not its issuer.
    retryable: false
---

## Abstract

The **VTC Relationships — Revoke** Trust Task revokes a relationship credential `id` previously published via [`vtc/relationships/publish`](../../publish/0.1/). Only the original issuer may revoke.

## Conformance

Producer: supply `id`. Carry a proof.

Consumer: resolve the relationship; if absent, or the proof signer is not its issuer, return `notFound` (the same code for both, so a caller cannot probe for relationships it did not issue). Otherwise revoke it and return `{ id }`. Audit the revocation.

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.2, where the declaration is not yet required.*

The authorization evidence this task presupposes is being the **relationship's own issuer** — the proof signer must be the party that created the relationship being revoked.

Note the deliberate choice in how a failure is reported: a caller that is not the issuer receives `notFound`, the same code as for a relationship that does not exist. That is an anti-probing measure, not an oversight. Distinguishing the two would let a caller enumerate relationships it did not issue by observing which id returns a permission error rather than a not-found.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

**Issuer-gated.** Unknown id and not-your-relationship collapse to one `notFound`, so revoke is not an oracle over others' relationships. Only the issuer, proven by the signer match, can revoke their own attribution.
