---
slug: credential-exchange/pending/deny
version: "0.1"
title: Credential Exchange — Pending Deny
summary: Refuse a deferred presentation request. Nothing is presented, the deferral is retired, and the verifier learns nothing about why.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - presentation
  - consent
  - deny
  - deferred
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Holder operator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Holder agent
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: A denial is a consent decision on the same footing as an approval and retires the request permanently. It must be attributable — otherwise an unauthenticated caller could quietly clear a holder's backlog and the holder would never learn what was asked.
sideEffects:
  level: destructive
  rationale: Retires the deferral permanently. Nothing is disclosed, but the request is gone — the holder cannot reconsider it later, and the verifier would have to ask again.
consequences:
  - Nothing is disclosed to the verifier.
  - The deferral becomes terminal and disappears from `pending/list`.
  - The decision cannot be reversed; a change of mind requires the verifier to ask again.
exposure:
  discloses: none
  ingests: none
  actsAsSubject: true
  rationale: "The caller is the holder deciding about its own wallet, so the subject and the actor are the same party. Nothing is disclosed — the whole effect of the task is that a disclosure does not happen — and nothing is carried in either: the payload is an opaque deferral handle, with no reason field for anything about the holder to travel in."
retention:
  class: durable
  rationale: The deferral is destroyed, but the decision is kept. A denial is a consent record on the same footing as an approval, and a holder that retained only its approvals could not later show that a request was seen and refused rather than lost — which is the distinction that matters if a verifier claims it was never answered. What is retained is the decision and its author, not the claims that were never released.
errorCodes:
  - code: credential-exchange/pending/deny:notFound
    meaning: No actionable deferral matches `id`.
    retryable: false
related:
  - credential-exchange/pending/list
  - credential-exchange/pending/approve
---

## Abstract

The **Credential Exchange — Pending Deny** Trust Task refuses a deferred [query](../../../query/0.1/). No presentation is made and the deferral is retired.

## A denial carries no reason

The payload is an `id` and nothing else — there is no reason field, and unknown members are rejected.

That is a privacy decision, not an oversight. Anything the holder said about *why* it refused would be information the verifier did not have and is not entitled to: "I don't hold that" and "I hold it and won't show you" must remain indistinguishable. A reason string is the obvious place for that distinction to leak, so the field does not exist.

For the same reason a consumer SHOULD make a denial indistinguishable from a query that simply expired unanswered. A verifier that can tell "actively refused" from "ignored" learns something about the holder's attention, and can probe for it.

## Terminal

A denial is final. The deferral is retired and disappears from [`pending/list`](../../list/0.1/); a holder who changes their mind cannot approve it afterwards — the verifier must ask again, which re-establishes a fresh nonce and a fresh decision.

This is the conservative direction, and deliberately so. The alternative — a denial that can be reversed — leaves a refused request sitting in a state where a later mis-click discloses claims the holder has already declined to share.

## Conformance

Producer: send the `id` from `pending/list`.

Consumer: verify authorization, refuse an unknown or already-terminal id with `notFound`, retire the record, and answer with `status: "denied"`. Do not notify the verifier with anything that distinguishes a denial from silence. Audit the decision against the deciding identity — a denial is as much a consent record as an approval.

## Security & Privacy

### Data carried

Nothing, in both directions, and that is the achievement rather than an accident.
`exposure.discloses` is `none` because the entire effect of the task is that a
disclosure does not happen; `ingests` is `none` because the payload is an opaque
deferral handle and unknown members are rejected.

The absences are the design. There is **no reason field** — anything the holder said
about *why* it refused would be information the verifier did not have and is not
entitled to, and "I don't hold that" and "I hold it and won't show you" must remain
indistinguishable. A free-text reason is the obvious place for that distinction to leak,
so the member does not exist rather than being marked optional and discouraged. `status`
is a single-valued enum for the same reason one step further out: a bare string would
invite consumers to infer meaning from text a producer never promised, and a field that
can carry meaning eventually carries a reason.

### Correlation

The property this task protects is *refusal indistinguishability*, and it is a
correlation property. A verifier that can tell an active denial from a query that simply
expired unanswered learns something real — that the holder was present, saw the request,
and declined — and having learned that it can probe for it, timing one query against
another until the holder's attention and holdings can be inferred without a single
presentation. So a consumer **SHOULD** make a denial indistinguishable from silence, and
**MUST NOT** notify the verifier of anything that separates the two. The same reasoning
runs through [`query`](../../../query/0.1/), where `noMatch` and "held but not consented"
are required to look alike; a denial that leaked would defeat that rule from the other
end.

Internally the picture is inverted, and worth being honest about. The holder's own
records *do* distinguish the cases, because the consent log below keeps the decision and
its author. That asymmetry is correct — the point was never that the holder should forget,
only that the verifier should not learn — but it means the denial log is a record of what
this holder was asked and chose not to share, which is sensitive in exactly the way the
refusal was protecting.

Both parties declare `identifierScope: pairwise`: operator and agent are components of
one wallet, and a public identifier on this leg would let an observer tie
consent-administration traffic to the same party's activity elsewhere for no benefit.

### Retention

Two opposite things happen at once. The deferral is destroyed — retired, terminal, gone
from [`pending/list`](../../list/0.1/) — and cannot be reconsidered; a holder who changes
their mind must be asked again, which re-establishes a fresh nonce and a fresh decision.
That is the conservative direction deliberately: a reversible denial leaves a refused
request sitting where a later mis-click discloses claims the holder has already declined
to share.

The decision, meanwhile, is kept. `retention.class` is `durable` because a denial is as
much a consent record as an approval, and a holder that retained only its approvals could
not later show that a request was seen and refused rather than lost — the distinction
that matters if a verifier claims it was never answered. What is retained is the
decision and who made it, not the claims that were never released and, per the previous
section, not a reason that was never recorded.

### Consent/purpose

A denial is a consent decision, exercised in the negative, and this specification treats
it with the same weight as the positive one: `proofRequirement` is REQUIRED, and the
decision is audited against the deciding identity. The reason is practical rather than
symmetrical bookkeeping — without attribution, an unauthenticated caller could quietly
clear a holder's backlog and the holder would never learn what had been asked of them.
Denial-by-deletion is an attack, not a refusal.

The purpose the data was collected for does not survive the decision. The deferral was
persisted so that a decision could be made; the decision has been made, so the deferral
goes, and nothing about the request is available for any other use. A consumer
**MUST NOT** mine denied deferrals to build a profile of which verifiers the holder
declines — that is a purpose the holder never agreed to, assembled from records kept only
to enable a refusal. Whether a person makes the decision at all is, as everywhere in this
family, a consumer policy question this specification does not answer.
