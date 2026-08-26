---
slug: vtc/join-requests/show
version: "0.1"
title: VTC Join-Requests — Show
summary: Fetch one join request by id, including the applicant's presentation and the recorded policy verdict.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - community
  - show
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
  requirement: RECOMMENDED
  rationale: Read-only fetch of one join request. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads one join request; persists nothing."
subjectPath: /id
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The request carries only `id`, an identifier the community itself minted, so nothing personal travels inbound. The asymmetry is the point: the response hands back the whole `JoinRequest`, including the applicant's `vp` and its `vpClaims` projection."
retention:
  class: transient
  rationale: The community persists nothing from this request — it resolves an id and answers. The declaration reaches only the request document; what an administrator does with the presentation the response hands them is beyond where this specification can speak, and Security & Privacy → Retention says so.
errorCodes:
  - code: vtc/join-requests/show:notFound
    meaning: No join request with the supplied id exists.
    retryable: false
---

## Abstract

The **VTC Join-Requests — Show** Trust Task returns one join request by `id` — the [`JoinRequest`](../../../_shared/0.1/join-request.schema.json), including the applicant's `vp` and, once decided, the `policyDecision`. The read-one companion to [`vtc/join-requests/list`](../../list/0.1/).

## Conformance

Producer: supply `id`.

Consumer: verify the community-admin capability. Resolve the request; if none, return `notFound`. Return the full `JoinRequest`.

## Security & Privacy

### Data carried

The request is one member — `id` — and the interesting half is entirely in the
response. It returns the whole
[`JoinRequest`](../../../_shared/0.1/join-request.schema.json): `applicantDid`,
the applicant's submitted `vp`, the `vpClaims` projection extracted from it, the
recorded `policyDecision`, and the `decision` block with its `code`, `reason`
and `decidedAt`. That is every claim the applicant disclosed at
[`submit`](../../submit/0.2/spec.md), rendered twice — once as the presentation
and once in the canonical form the policy engine reads.

`exposure.discloses` is declared `metadata` because the record is descriptive
data about a request rather than released credential material, but the
classification understates what a reader sees: the presentation inside `vp` may
carry any claim the community's join policy asked for. Deployments **SHOULD**
gate this read at least as tightly as they gate the join decision itself, and
**SHOULD** treat a `show` as an auditable event — reading one applicant's
disclosed claims is an act with a subject, and a community that logs decisions
but not reads has an oversight blind spot exactly where the personal data is.

There is nothing here for a producer to minimise: `id` is required and nothing
else is offered. Minimisation on this task is a *consumer* choice, and the only
one available is answering a narrower projection than the full row — which the
schema permits nothing of, and which is worth knowing when reading `metadata`
as reassurance.

### Correlation

The response is a join key by construction. `applicantDid` sits beside the
credential material in `vp`, so a single `show` ties a community identity to
whatever issuer DIDs, credential identifiers and subject identifiers the
presentation carries — and those travel with the applicant to every other
community they apply to. An applicant who used a per-community DID, as
[`submit`](../../submit/0.2/spec.md) recommends, has protected their *identifier*
and not their *credentials*.

The administrator declares `identifierScope: pairwise`. A community administrator
acts under an identity that is meaningful inside this community — it is what the
audit trail attributes the read to — and nothing in this task asks a third party
to recognise it. Keeping it community-scoped means an operator's pattern of reads
here cannot be joined to their activity in another community by identifier alone.

`submittedAt` and `decision.decidedAt` are not decorative either: together with
the `status` transitions they reconstruct how long a community deliberated over a
particular applicant, which is information about the applicant as much as about
the community.

### Retention

The request itself is transient — the community resolves an `id`, answers, and
keeps nothing new. The retention that matters was already incurred at
[`submit`](../../submit/0.2/spec.md), which is `durable`: this task is the
interface through which that durable record stays readable, including for
applicants who were refused.

What this task *adds* is copies. Each `show` puts the presentation and its
`vpClaims` into an administrator's client — a console, a log, a support ticket —
where the community's own retention policy may not reach and this specification
certainly does not. A consumer **SHOULD** treat the response as material subject
to the same disposal policy as the stored row, and **SHOULD NOT** persist it in a
second store simply because a read made it convenient to.

### Consent/purpose

The purpose is adjudication and the accountability that follows it: an
administrator reads the evidence in order to decide the request, or to explain a
decision already taken. That is the basis on which the applicant's material moves
here, and it is a narrow one — the applicant disclosed to a community's join
policy, not to each of its administrators individually.

A community that uses `show` to browse the claims of applicants it has already
refused, to enrich a member directory, or to answer a question unrelated to the
request's own lifecycle has moved outside that purpose without any member of the
payload changing. Nothing in the schema can detect it; the constraint is
organisational.

Whether a read requires a second operator, a case reference, or a step-up is a
consumer policy question, and per
[SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification
takes no position on it.
