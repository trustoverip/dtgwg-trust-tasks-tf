---
slug: policy/evaluate
version: "0.3"
title: Policy — Evaluate
summary: Dry-run a policy decision against a synthetic PolicyInput; returns the decision plus a trace of which modules matched and which rules fired.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - policy
  - dry-run
  - rego
  - debug
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only dry-run, no state change. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Dry-run of a policy decision; explicitly persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: policy/evaluate:permissionDenied
    meaning: The consumer lacks PolicyAdmin capability.
    retryable: false
  - code: policy/evaluate:candidateRegoInvalid
    meaning: The supplied `candidateModule` failed parsing.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        line: { type: "integer", minimum: 1 }
        column: { type: "integer", minimum: 1 }
        message: { type: "string" }
related:
  - policy/upsert
---

## Abstract

The **Policy — Evaluate** Trust Task is the policy-editor's preview button: given a synthetic input, return the decision the live evaluator would make. Optional `candidateModule` lets admins preview a pending upsert without persisting it.

Returns the decision, the list of matched policy ids (in evaluation order), and an optional execution trace.

## Changes from 0.2

Version 0.3 tracks `policy/_shared/0.3`, which **generalises `PolicyInput`** from the vault-flow triad (`proxyLogin` / `release` / `stepUpResponse`) to any Trust Task. The evaluated `request` now carries the task's `typeUri`, its `subject`, and the two authoritative SPEC §7.3 classifications — `sideEffects` (item 13) and `exposure` (item 14) — that a maintainer derives from the compiled handler. `PolicyDecision` gains the **`requireConsent`** disposition, so a policy can demand a signed consent decision from a named approver set (optionally excluding the requesting device) before a task executes. The dry-run surface is otherwise unchanged.

## Conformance

Producer: populate `input` with a representative PolicyInput. Optionally supply `candidateModule` for pre-save preview. Consumer: load existing policies; if `candidateModule` is supplied, parse and layer at `candidatePriority`; evaluate; return decision plus matchedPolicies.

A maintainer **MUST** populate `input.request.sideEffects` and `input.request.exposure` from the authoritative classification of the handler the task would invoke, not from any client-supplied or registry-declared value (SPEC §7.3 items 13–14).

## Security & Privacy

**No persistent state.** Evaluate never modifies the live policy set, even with `candidateModule`. The candidate is layered for this single call only.

**Trace verbosity.** Traces can reveal implementation detail of the maintainer's Rego rules; consumers should treat them as security-relevant.
