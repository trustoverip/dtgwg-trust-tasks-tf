---
slug: credential-exchange/pending/approve
version: "0.1"
title: Credential Exchange — Pending Approve
summary: Approve a deferred presentation request, minting the presentation against the verifier's original query and nonce and returning it to the approver.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - presentation
  - consent
  - approve
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
  rationale: This is the act of consent itself, and it causes an irreversible disclosure of the holder's claims to a third party. It must be attributable to the party that authorized it — a disclosure nobody can be shown to have approved is a disclosure nobody agreed to.
sideEffects:
  level: destructive
  rationale: Discloses the holder's claims to the verifier, which cannot be undone, and retires the deferral. The record is not retained for a second decision — approval is terminal.
consequences:
  - The claims listed in the deferral's `requested` are disclosed to `verifierDid`.
  - The deferral becomes terminal and disappears from `pending/list`.
  - Disclosure is irreversible; the verifier retains whatever it received.
exposure:
  discloses: secret
  actsAsSubject: true
  rationale: The response carries the vp_token — the holder's own claims, disclosed. It is returned to the approver as well as sent to the verifier so that the operator holds exactly what was released.
errorCodes:
  - code: credential-exchange/pending/approve:notFound
    meaning: No actionable deferral matches `id`.
    retryable: false
  - code: credential-exchange/pending/approve:expired
    meaning: The deferral is past `expiresAt`; the verifier's nonce is stale and no valid presentation can be minted.
    retryable: false
  - code: credential-exchange/pending/approve:permissionDenied
    meaning: The caller is not authorized to approve disclosures for this agent.
    retryable: false
related:
  - credential-exchange/pending/list
  - credential-exchange/pending/deny
  - credential-exchange/present
---

## Abstract

The **Credential Exchange — Pending Approve** Trust Task turns a deferred [query](../../../query/0.1/) into an actual disclosure. The holder mints the presentation and delivers it to the verifier on the original thread, returning the same `vp_token` to the approver.

This is the highest-consequence task in the family. Everything else proposes, asks, or records; this one discloses.

## Approval is bound to the original request

The presentation is minted against the **original** query and its nonce — not against a fresh request, and not against anything the approver supplies. The payload carries an `id` and nothing else, and that is deliberate: approval is a yes-or-no on a request already made, never an opportunity to renegotiate it. Allowing the approver to vary the claims here would mean the thing approved and the thing presented could differ, which makes the consent record worthless as evidence.

It also explains the expiry rule. A deferral past `expiresAt` **MUST** be refused with `expired`. The verifier's nonce is what freshness is bound to; once the verifier has stopped accepting it, any presentation minted against it fails at the far end. A consumer MUST NOT silently substitute a fresh nonce to make an expired approval work — that would present against a request the verifier has forgotten making, on a holder's authority nobody re-confirmed.

## Terminal, and why the record goes

On success the deferral is retired. A consumer MUST NOT leave it actionable: approving twice would present twice, and the second disclosure has no consent behind it — the operator agreed once.

The response returns the `vp_token` to the approver rather than only sending it to the verifier. The operator who authorized the disclosure should be able to see exactly what left the wallet, not infer it from what they approved.

## Conformance

Producer: send the `id` from [`pending/list`](../../list/0.1/). Nothing else — the payload rejects unknown members.

Consumer: verify authorization; refuse an unknown or already-terminal id with `notFound` and an expired one with `expired`, **before** minting anything. Apply the same disclosure rules as [`present`](../../../present/0.1/), including the subset check on formats without claim-level selective disclosure. Deliver on the original thread, retire the deferral, and audit the approval against the approving identity.

## Security & Privacy

`sideEffects.level` is `destructive` in the sense that matters here: the effect cannot be walked back. There is no revocation of a disclosure — once the verifier holds the claims, it holds them.

`exposure.discloses` is `secret` on both legs. A consumer SHOULD record what was disclosed, to whom, under which stated purpose, and on whose authority; that record is the only after-the-fact account of a decision that is otherwise invisible once the thread closes.
