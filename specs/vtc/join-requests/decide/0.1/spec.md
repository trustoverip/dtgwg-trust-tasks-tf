---
slug: vtc/join-requests/decide
version: "0.1"
title: VTC Join-Requests — Decide
summary: An administrator decides a pending join request — approving admits the applicant as a member, rejecting refuses them — optionally recording a reason.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - community
  - decide
  - approve
  - reject
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
  rationale: Admitting or refusing an applicant changes (or forecloses) community membership; the decision MUST be attributable to the operator who made it.
sideEffects:
  level: mutating
  rationale: "Resolves the pending request: `approved` admits the applicant as a member; `rejected` refuses them, recoverable only by the applicant re-applying."
consequences:
  - "`approved` admits the applicant to the community as a member."
  - "`rejected` refuses the applicant; they are not admitted."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/join-requests/decide:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/join-requests/decide:notFound
    meaning: No join request with the supplied id exists.
    retryable: false
  - code: vtc/join-requests/decide:notPending
    meaning: The request is not in the pending state, so it cannot be decided.
    retryable: false
related:
  - vtc/join-requests/list
  - vtc/join-requests/show
  - vtc/join-requests/submit
---

## Abstract

The **VTC Join-Requests — Decide** Trust Task resolves a pending join request `id` with a single `decision`: `approved` admits the applicant as a member; `rejected` refuses them, with an optional operator `reason`.

This task supersedes the [`vtc/join-requests/approve`](../../approve/0.1/) / [`vtc/join-requests/reject`](../../reject/0.1/) pair. The two payloads were near-identical (`{id}` vs `{id, reason?}`), shared the same admin gate, the same pending-state lifecycle check, and the same proof posture; the decision is one enum field, not two tasks. This mirrors the established enum-variant pattern (`provision/integration`, `auth/passkey/login/start`'s `purpose`).

## Conformance

Producer: supply `id` and `decision`; optionally `reason` (recorded whichever way the decision goes, but chiefly useful with `rejected`). Carry a proof.

Consumer: verify the community-admin capability (`permissionDenied` otherwise). Resolve the request (`notFound` if absent); if it is not `pending`, return `notPending` — a request already decided, withdrawn, or deferred cannot be re-decided through this task. Otherwise apply the decision: on `approved`, admit the applicant and set status to `approved`; on `rejected`, set status to `rejected` and record the `reason`. Return `{ requestId, status }` echoing the decision. Audit the decision with the operator identity and reason.

## Security & Privacy

**Membership decision, so attributable.** Both outcomes are proof-REQUIRED and audited: admission changes community membership; refusal forecloses it and is recoverable only by re-application. Deployments that require a second approver gate this behind the community's confirm flow at the enforcement point, orthogonal to this payload.

Because one task now carries both outcomes, a consumer's per-task gating cannot distinguish "may approve" from "may reject"; deployments needing that split enforce it on the payload `decision` at the policy layer, not the task URI.
