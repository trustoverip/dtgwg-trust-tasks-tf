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
  - role: Holder agent
    requirement: REQUIRED
    member: recipient
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
  actsAsSubject: true
  rationale: The caller is the holder deciding about its own wallet, so the subject and the actor are the same party. Nothing is disclosed — the whole effect of the task is that a disclosure does not happen.
errorCodes:
  - code: credential-exchange/pending/deny:notFound
    meaning: No actionable deferral matches `id`.
    retryable: false
  - code: credential-exchange/pending/deny:permissionDenied
    meaning: The caller is not authorized to decide disclosures for this agent.
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

`exposure.discloses` is `none`: that is the entire purpose of the task.

The `status` member is a single-valued enum rather than a free string. A denial has exactly one outcome, and leaving the field open would invite consumers to infer meaning from text a producer never promised — including, eventually, a reason.
