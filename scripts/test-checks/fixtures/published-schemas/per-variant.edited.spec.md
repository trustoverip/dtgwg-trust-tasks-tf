---
slug: fixture/per-variant
version: "0.1"
title: "Fixture — per-variant policy, edited in place"
summary: "Every policy flag the old regex reader could not see, moved (#1134)."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
parties:
  - role: Caller
    requirement: RECOMMENDED
    member: issuer
  - role: Service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    Fixture.
issuedAtRequirement:
  request: RECOMMENDED
  response: REQUIRED
  rationale: >-
    Fixture.
errorCodes:
  - code: fixture/per-variant:notFound
    description: Fixture.
sideEffects:
  level: none
---

# Fixture
