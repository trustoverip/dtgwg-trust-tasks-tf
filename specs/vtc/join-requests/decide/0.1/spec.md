---
slug: vtc/join-requests/decide
version: "0.1"
title: VTC Join-Requests — Decide
summary: An administrator decides a pending join request — approving admits the applicant as a member, rejecting refuses them — optionally recording a reason.
status: draft
targetFrameworkVersion: "0.5"
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
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Admitting or refusing an applicant changes (or forecloses) community membership; the decision MUST be attributable to the operator who made it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The decision admits or refuses a specific applicant. Replayed against a later request from the same applicant, it decides that one too, on reasoning the deciding member never applied to it.
sideEffects:
  level: mutating
  rationale: "Resolves the pending request: `approved` admits the applicant as a member; `rejected` refuses them, recoverable only by the applicant re-applying."
consequences:
  - "`approved` admits the applicant to the community as a member."
  - "`rejected` refuses the applicant; they are not admitted."
subjectPath: /id
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "Inbound the request carries only `id`, the enum `decision`, and an operator-authored `reason` — prose about the decision rather than data about the applicant, and bounded at 1024 characters."
retention:
  class: durable
  rationale: The decision resolves the request permanently — `approved` admits, `rejected` forecloses and is recoverable only by re-application — and the recorded `reason` becomes the community's account of why, relayed to the applicant through `vtc/join-requests/status`. A community that discarded it could not later explain a refusal to the party it refused.
errorCodes:
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


An admitting decision returns the credentials it issued — `vmc`, and `roleVec`
where a role was granted — rather than only saying that it admitted.

The alternative is a second round trip whose only purpose is to collect
something the community already held when it decided: the applicant polls,
learns they were admitted, and asks again for the credential. Both members are
absent on a refusal, where there is nothing to deliver.
## Conformance

Producer: supply `id` and `decision`; optionally `reason` (recorded whichever way the decision goes, but chiefly useful with `rejected`). Carry a proof.

Consumer: verify the community-admin capability (`permissionDenied` otherwise). Resolve the request (`notFound` if absent); if it is not `pending`, return `notPending` — a request already decided, withdrawn, or deferred cannot be re-decided through this task. Otherwise apply the decision: on `approved`, admit the applicant and set status to `approved`; on `rejected`, set status to `rejected` and record the `reason`. Return `{ requestId, status }` echoing the decision. Audit the decision with the operator identity and reason.

## Security & Privacy

### Data carried

The request is `id`, a two-value `decision`, and an optional `reason` capped at
1024 characters. Nothing about the applicant travels inbound — the community
already holds their presentation from
[`submit`](../../submit/0.2/spec.md), and this task names a request rather than
re-describing a person.

`reason` is the member that needs a rule, because it is operator-authored free
text that does not stay inside the community. It is recorded in the audit trail
and relayed verbatim to the applicant by
[`status`](../../status/0.1/spec.md), so an administrator writing it is writing
for two audiences at once and will usually only be thinking of one. A producer
**MUST NOT** put a third party into it — a member who objected, another applicant
whose case set the precedent, a referee who was consulted — because none of those
people are party to this exchange and the applicant is the one who will read it.
A producer **SHOULD** also keep internal deliberation out of it: `reason` is the
community's account of a refusal, not a transcript of how the community reached
one, and the 1024-character bound is a hint that it was designed for the former.

The response returns `requestId` and `status`, and on an admitting decision the
credentials that decision issued — `vmc`, and `roleVec` where a role was granted —
inline rather than by a second round trip. Both are absent on a refusal, where
there is nothing to deliver.

### Correlation

This task is where a community's records about one party fuse. Before it, the
applicant is a row with a presentation on it; after an approval they are a member
with a credential naming them, and the two are joined by `id` forever. That link
is what makes the community's own membership auditable, and it is also why an
approval is not recoverable from the applicant's side by any later action.

A refusal correlates too, and less visibly. It writes a `decision` block with a
`code`, a `reason` and a `decidedAt` onto a row that already carries the
applicant's claims, and that row remains readable through
[`show`](../../show/0.1/spec.md) and enumerable through
[`list`](../../list/0.1/spec.md). The refused party is thereafter, within this
community, a durably identified applicant with a recorded reason attached.

The administrator declares `identifierScope: pairwise`. Accountability for a
membership decision belongs inside the community that took it: the audit trail
needs to name the operator consistently over time, and nothing in this task needs
a third party to recognise that operator. A community-scoped operator identity
means the pattern of who admits and who refuses here cannot be joined by
identifier to the same person's decisions elsewhere.

### Retention

Durable on both paths, and asymmetrically consequential. `approved` mints a
membership credential that outlives this exchange and is designed to be presented
elsewhere. `rejected` writes a refusal that, per the consequence declared in the
front matter, is recoverable only by the applicant re-applying — meaning the
community retains both the claims the applicant disclosed and its own record of
having turned them down.

The `reason` is the part of that record with the longest reach, because it is the
only free prose in the family and the only thing an operator writes in their own
words. It persists in the audit trail and in the `decision` block whether or not
the applicant ever polls for it.

Nothing here bounds any of that. A community **SHOULD** decide, and publish,
whether a refusal's supporting material is kept on the same schedule as an
admission's — an admission's evidence justifies a live entitlement, while a
refusal's justifies a decision already fully taken, and treating them alike is a
choice rather than a default.

### Consent/purpose

The purpose is adjudication: an authorised operator resolves an application the
applicant themselves opened. The applicant's consent to the community holding
their claims was given for exactly this, which makes this task the one place in
the family where the material is used precisely as it was offered.

Both outcomes are proof-REQUIRED and audited, and that is a purpose statement as
much as a security one — a decision that changes or forecloses someone's
membership should be attributable to the person who made it. Because one task now
carries both outcomes, a consumer's per-task gating cannot distinguish "may
approve" from "may reject"; deployments needing that split enforce it on the
payload `decision` at the policy layer rather than on the task URI, and
deployments requiring a second approver gate this behind their own confirm flow
at the enforcement point. Both are consumer policy: per
[SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification
describes the decision and does not require a gate on it.
