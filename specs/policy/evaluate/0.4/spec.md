---
slug: policy/evaluate
version: "0.4"
title: Policy — Evaluate
summary: Dry-run a policy decision against a synthetic PolicyInput; returns the decision plus a trace of which modules matched and which rules fired.
status: draft
targetFrameworkVersion: "0.5.0"
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
related:
  - policy/upsert
---

## Abstract

The **Policy — Evaluate** Trust Task is the policy-editor's preview button: given a synthetic input, return the decision the live evaluator would make. Optional `candidateModule` lets admins preview a pending upsert without persisting it.

Returns the decision, the list of matched policy ids (in evaluation order), and an optional execution trace.

## Changes from 0.3

- Re-pinned to [`policy/_shared/0.4`](../../_shared/0.4/policy.schema.json), in which a `requireConsent` decision must carry `minApprovals` and `excludeRequester`. 0.3 defaulted them to `1` and `false`, so an evaluation trace could show a consent decision whose quorum and self-approval rule nobody had chosen ([SPEC §6.6](/SPEC.md#66-shared-schema-components) rule 5).
- No other change: the request, the trace and the error codes are as in 0.3.

Breaking on the wire for a `requireConsent` decision that omitted either member, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.3` remains published.

## Changes from 0.2

Version 0.3 tracks `policy/_shared/0.3`, which **generalises `PolicyInput`** from the vault-flow triad (`proxyLogin` / `release` / `stepUpResponse`) to any Trust Task. The evaluated `request` now carries the task's `typeUri`, its `subject`, and the two authoritative SPEC §7.3 classifications — `sideEffects` (item 13) and `exposure` (item 14) — that a maintainer derives from the compiled handler. `PolicyDecision` gains the **`requireConsent`** disposition, so a policy can demand a signed consent decision from a named approver set (optionally excluding the requesting device) before a task executes. The dry-run surface is otherwise unchanged.

## Conformance

Producer: populate `input` with a representative PolicyInput. Optionally supply `candidateModule` for pre-save preview. Consumer: load existing policies; if `candidateModule` is supplied, parse and layer at `candidatePriority`; evaluate; return decision plus matchedPolicies.

A maintainer **MUST** populate `input.request.sideEffects` and `input.request.exposure` from the authoritative classification of the handler the task would invoke, not from any client-supplied or registry-declared value (SPEC §7.3 items 13–14).

## Security & Privacy

### Data carried

The request carries a synthetic `PolicyInput`: a task's Type URI, subject, side-effect and exposure classes, and the consumer's DID, device and network class. It may also carry a candidate Rego module. The response carries the resulting `PolicyDecision`, which for a `requireConsent` decision names an approver set, its quorum and whether the requester may self-approve, and, when `includeTrace` was set, the evaluator's trace. No secret is carried in either direction.

### Correlation

The `PolicyInput` describes a hypothetical request, but a caller that copies a real consumer's DID or device identifier into it ties the dry-run to that consumer in the maintainer's logs. Callers SHOULD use synthetic identifiers unless the point of the dry-run is that consumer's policy.

### Retention

Evaluation persists nothing, as stated below. What is kept is what the caller logs.

### Consent/purpose

A dry-run is a debugging aid for the maintainer's own operators; it acts for no subject and grants nothing. A `requireConsent` decision returned here demands no consent, since no task executes.

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

