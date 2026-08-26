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
    identifierScope: pairwise
  - role: Holder agent
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
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
  ingests: none
  actsAsSubject: true
  rationale: "The response carries the vp_token — the holder's own claims, disclosed. It is returned to the approver as well as sent to the verifier so that the operator holds exactly what was released. The request itself carries nothing but an opaque deferral handle, which is the asymmetry that defines the task: the highest-consequence document in the family ingests nothing and releases everything."
retention:
  class: durable
  rationale: The disclosure cannot be undone, so the record of who authorized it has to outlive the thread that carried it. A consumer keeps what was disclosed, to whom, under which stated purpose and on whose authority; without it an irreversible release of the holder's claims has no attributable decision behind it, and a later dispute cannot distinguish an approved disclosure from an unauthorized one. The deferral it acted on is retired in the same step and is not part of what is kept.
errorCodes:
  - code: credential-exchange/pending/approve:notFound
    meaning: No actionable deferral matches `id`.
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

### Data carried

The request is one opaque handle. `id` names a deferral and carries nothing else — no
claims, no query, no variation on what the verifier asked — which is why
`exposure.ingests` is `none` on the most consequential task in the family. The
asymmetry is the design: this document releases everything and accepts nothing, because
anything it accepted would be an opportunity to alter what was consented to.

The response carries the `vp_token`: the holder's own claims, selectively disclosed to a
named audience, in the same shape [`present`](../../../present/0.1/) defines. It goes to
the verifier on the original thread and comes back to the approver as well, deliberately.
The operator who authorized a disclosure should be able to see exactly what left the
wallet rather than infer it from the summary they approved against — the two can differ
if anything in the minting path is wrong, and this is the only place that divergence is
visible.

The same subset rule that governs `present` governs here: on a format without
claim-level selective disclosure the holder presents only if the whole credential is
within the consented set, and otherwise refuses. Approving a request for two claims must
never release a credential containing twelve.

### Correlation

Approving joins three things that were separate: a verifier, a set of the holder's
claims, and a moment in time. That join is the point of the task and it is also its
whole risk — the verifier learns the claims, and the audit record learns the verifier,
and both are durable.

Both parties declare `identifierScope: pairwise` because operator and agent are
components of one wallet and nothing here asks either to be recognisable elsewhere. The
identifiers that do correlate are inside the `vp_token`, chosen at issuance and at
[`present`](../../../present/0.1/), and this task cannot improve them: an approver
consenting to a disclosure is also consenting to whatever holder identifier the
credential carries, without being shown it.

The expiry rule protects a correlation property that is easy to lose. A consumer
**MUST NOT** substitute a fresh nonce to make an expired approval work, because a
presentation minted against a request the verifier has forgotten making binds the
holder's claims to an exchange nobody re-confirmed — and the resulting token is
indistinguishable, to anyone auditing later, from one the verifier had asked for.

### Retention

Two things persist and they must not be confused. The deferral is retired: on success a
consumer **MUST NOT** leave it actionable, because approving twice presents twice and
the second disclosure has no consent behind it — the operator agreed once. The consent
record persists instead, and is why `retention.class` is `durable`: what was disclosed,
to whom, under which stated purpose, and on whose authority. It is the only after-the-fact
account of a decision that is otherwise invisible once the thread closes, and deleting it
leaves an irreversible release of the holder's claims with nothing attributable behind it.

That record is itself sensitive — it is a history of what this holder has disclosed and
to whom — and inherits the protection of the wallet rather than of a log. On the far
side, the verifier's copy of the `vp_token` is retained on
[`present`](../../../present/0.1/)'s terms, bounded by the stated purpose and by nothing
this task can enforce.

### Consent/purpose

This document *is* the consent. Everything else in the family proposes, asks, or records;
this one is the act, and `proofRequirement` is REQUIRED so that the act is attributable —
a disclosure nobody can be shown to have approved is a disclosure nobody agreed to.

Its scope is exactly the request already made. The presentation is minted against the
original query, its nonce and its stated `purpose`, never against a fresh request and
never against anything the approver supplies; the payload carries an `id` and nothing
else precisely so that approval cannot become renegotiation. If the thing approved and
the thing presented could differ, the consent record would be worthless as evidence of
either.

The consent does not extend past this one disclosure. Approving is a decision about *this
verifier*, for *this purpose*, once — not a standing permission, and a consumer
**MUST NOT** treat a prior approval as pre-trust for a later query from the same
verifier. Whether a human makes the decision, and what evidence they are shown, is a
consumer policy question this specification takes no position on; it defines what the
decision binds to once it is made.
