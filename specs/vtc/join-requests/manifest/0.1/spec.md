---
slug: vtc/join-requests/manifest
version: "0.1"
title: VTC Join-Requests — Manifest
summary: Discover a community's join criteria — the presentation-definitions an applicant must satisfy — before submitting.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - discovery
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Join criteria are pre-membership discovery information; a proof is not required to read them, though it is recommended so a community can rate-limit or attribute discovery.
sideEffects:
  level: none
  rationale: "Reads the community's published join criteria; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
---

## Abstract

The **VTC Join-Requests — Manifest** Trust Task returns a community's join criteria so a prospective applicant knows what to present. Each entry names a presentation-definition (and an optional human description) the applicant must satisfy; the applicant then submits via [`vtc/join-requests/submit`](../submit/0.1/). This is pre-submit discovery — it precedes membership.

## Conformance

Producer: send with no parameters.

Consumer: return the `communityDid` and the list of `criteria`, each with an `id` and a `presentationDefinition`. A community MAY tailor the criteria it returns to the caller, but the default is its public join policy.

## Security & Privacy

**Pre-membership discovery.** The criteria describe what the community asks of applicants — not member data — so this is `metadata` and requires no proof to read. A community that wants to rate-limit or attribute discovery MAY require a proof; the framework recommends but does not mandate one.
