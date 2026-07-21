---
slug: vtc/website/rollback
version: "0.1"
title: VTC Website — Rollback
summary: Roll a managed-mode website back to a past deploy generation by flipping the current pointer.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords: [vtc, website, rollback]
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
  rationale: Rollback changes what the live site serves; it MUST be attributable.
sideEffects:
  level: mutating
  rationale: "Flips the current pointer to a past generation; reversible by rolling forward again."
subjectPath: /generation
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/website/rollback:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/website/rollback:notManaged
    meaning: The website is in live mode, which has no generations to roll back to. Managed mode only.
    retryable: false
  - code: vtc/website/rollback:generationNotFound
    meaning: No generation with the supplied label exists.
    retryable: false
---

## Abstract

The **VTC Website — Rollback** Trust Task makes a past deploy `generation` current, via an atomic pointer swap. Managed mode only; rolling back to the already-current generation is a no-op.

## Conformance

Producer: supply `generation`. Carry a proof.

Consumer: verify the community-admin capability. In live mode return `notManaged`; if the generation does not exist return `generationNotFound`. Otherwise flip `current` to it atomically and return `{ generation, current: true, noop }` — `noop: true` when it was already current. Audit the rollback.

## Security & Privacy

**Changes the live site.** A rollback alters what visitors see, so it is proof-REQUIRED and audited. The swap is atomic so a partial rollback cannot leave the site inconsistent.
