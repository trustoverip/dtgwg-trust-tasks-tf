---
slug: fixture/per-variant
version: "0.1"
title: "Fixture — per-variant policy, as released"
summary: "The per-variant proofRequirement and issuedAtRequirement forms (#1134)."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: Service
    requirement: RECOMMENDED
    member: recipient
proofRequirement:
  request: RECOMMENDED
  response: REQUIRED
  rationale: >-
    Fixture.
issuedAtRequirement:
  request: RECOMMENDED
  rationale: >-
    Fixture.
errorCodes:
  - code: fixture/per-variant:notFound
    description: Fixture.
sideEffects:
  level: none
---

# Fixture
