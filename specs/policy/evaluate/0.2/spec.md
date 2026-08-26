---
slug: policy/evaluate
version: "0.2"
wireCompatibleWith: "0.1"
title: Policy — Evaluate
summary: Dry-run a policy decision against a synthetic PolicyInput; returns the decision plus a trace of which modules matched and which rules fired.
status: draft
targetFrameworkVersion: "0.5"
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
---

## Abstract

The **Policy — Evaluate** Trust Task is the policy-editor's preview button: given a synthetic input, return the decision the live evaluator would make. Optional `candidateModule` lets admins preview a pending upsert without persisting it.

Returns the decision, the list of matched policy ids (in evaluation order), and an optional execution trace.

## Conformance

Producer: populate `input` with a representative PolicyInput. Optionally supply `candidateModule` for pre-save preview. Consumer: load existing policies; if `candidateModule` is supplied, parse and layer at `candidatePriority`; evaluate; return decision plus matchedPolicies.

## Security & Privacy

**No persistent state.** Evaluate never modifies the live policy set, even with `candidateModule`. The candidate is layered for this single call only.

**Trace verbosity.** Traces can reveal implementation detail of the maintainer's Rego rules; consumers should treat them as security-relevant.

**Free text.** Two members of the response are free text and are now bounded at
1024 characters each. `decision.explanation` is the evaluator's own account of
why a policy decided as it did; `trace[]` is the Rego evaluator's line-by-line
trace, returned only when `includeTrace` was set. Both are authored by the
maintainer that signs the response, so both are trusted to the same degree as
the decision itself — but both describe the maintainer's *internal* rules, which
is why the note above calls traces security-relevant. Their reader is the human
debugging a policy, and neither is intended for a machine: a consumer MUST NOT
parse either to recover a decision that `decision` did not state. Neither is
retained by this task — evaluation persists nothing — so anything kept is kept
by whatever the caller logs, and a caller that logs a trace has copied the
maintainer's rule structure into its own retention.

