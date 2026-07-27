---
slug: credential-exchange/pending/list
version: "0.1"
title: Credential Exchange — Pending List
summary: List the presentation requests this holder deferred for consent, showing who asked, why, and exactly which claims answering would disclose.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - presentation
  - consent
  - deferred
  - pending
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
  rationale: The response enumerates which credentials the wallet holds and what a verifier asked of them. That is the wallet's contents by another name, so the caller must be authenticated and attributable — transport authentication alone is not enough for a surface whose output is an inventory.
sideEffects:
  level: none
  rationale: Reads the deferral backlog; decides nothing.
exposure:
  discloses: metadata
  actsAsSubject: true
  rationale: The caller is the holder asking about its own wallet, so nothing leaves the holder's control. What is returned is nonetheless sensitive — which credentials are held and who has asked for them — which is why the caller is authenticated rather than this being an open read.
errorCodes:
  - code: credential-exchange/pending/list:permissionDenied
    meaning: The caller is not authorized to see this agent's deferral backlog.
    retryable: false
related:
  - credential-exchange/pending/approve
  - credential-exchange/pending/deny
  - credential-exchange/query
---

## Abstract

When a verifier the holder has not pre-trusted sends a [query](../../../query/0.1/), the holder does not answer and does not refuse — it **defers**, persisting the request and telling the verifier consent is required. The **Credential Exchange — Pending List** Trust Task is how the holder's operator sees that backlog.

It takes no parameters. The caller's authenticated identity is the scope: an agent answers for its own wallet, and there is nothing else to select.

## What is shown, and what is not

Each entry carries who asked (`verifierDid`), why (`purpose`, carried through from the query), when it was recorded and when it goes stale, and — the substance — `requested`: every held credential the query would disclose, resolved against what the wallet actually holds, down to the claims.

That last part is the whole point. An approver is not consenting to a query in the abstract; they are consenting to **specific claims of specific credentials leaving the wallet**. A list that showed only the verifier and a purpose string would be asking for a decision without its inputs.

What is *not* shown is the original DCQL query. A consumer retains it so an approval can re-present byte-faithfully against the verifier's original nonce, but it is machinery rather than a decision input, and putting it here would invite an operator to read the query instead of the resolved disclosure — which is the same mistake in a different place.

## Only actionable entries

A consumer MUST omit deferrals that are already terminal or past `expiresAt`. Both are unactionable: approval of an expired deferral necessarily fails, because the presentation would be bound to a nonce the verifier has stopped accepting.

Listing them anyway would offer an approver a decision guaranteed not to take effect — and worse, one whose failure looks like a bug rather than a rule. An empty array means nothing is awaiting a decision.

## Conformance

Producer: send an empty payload.

Consumer: verify the caller's authorization and scope the result to this agent's own deferrals. Return only `Pending`, unexpired records. Resolve `requested` against the wallet as it stands *now*, not as it stood when the query arrived — a credential deleted in the meantime cannot be disclosed and MUST NOT be listed as though it could.

## Security & Privacy

`exposure.discloses` is `metadata` and `actsAsSubject` is true: the holder is asking about itself, and nothing crosses a trust boundary. The information is still sensitive — the response is an inventory of held credentials plus a record of who has been asking — so this task is authenticated, and a consumer SHOULD audit reads of it.
