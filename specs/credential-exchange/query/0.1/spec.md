---
slug: credential-exchange/query
version: "0.1"
title: Credential Exchange — Query
summary: Verifier to holder — a DCQL query with a freshness nonce and a mandatory stated purpose, which the holder answers, defers for consent, or refuses.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - presentation
  - oid4vp
  - dcql
  - query
  - purpose-binding
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Verifier
    requirement: REQUIRED
    member: issuer
  - role: Holder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: An authcrypted transport proves which DID is asking, and the verifier's identity is the input the holder's consent decision turns on. A document proof is permitted where the envelope's authentication does not survive relaying.
sideEffects:
  level: mutating
  rationale: A query the holder does not auto-consent to is PERSISTED as a deferral awaiting an out-of-band decision. Asking therefore leaves durable state at the holder even when nothing is presented — which is also why an unbounded query rate is a storage concern, not merely a nuisance.
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: credential-exchange/query:consentRequired
    meaning: The holder deferred the query for an out-of-band decision. Not a failure — the verifier should expect a later presentation on the same thread, or nothing.
    retryable: true
  - code: credential-exchange/query:noMatch
    meaning: The holder holds no credential satisfying the query.
    retryable: false
related:
  - credential-exchange/present
  - credential-exchange/pending/list
---

## Abstract

The **Credential Exchange — Query** Trust Task is a verifier asking a holder to present something. The body is an [OID4VP](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html) DCQL query, a freshness `nonce`, and a `purpose`.

## `purpose` is mandatory, and that is a design decision

`purpose` is REQUIRED and MUST be non-empty. A verifier cannot ask for a credential without stating why.

This is not decoration. The holder's decision — automatic or human — is a consent decision, and consent to an unstated use is not consent. Making `purpose` optional would make the well-behaved verifier's query indistinguishable from the one that declines to say, exactly when the difference matters. A consumer MUST surface the stated purpose to whoever decides, and MUST bind it to any consent record it keeps, so that a later disclosure can be audited against the reason given for it.

## No wallet enumeration

A holder gathers candidates **only** via the type index named by the query's `meta` discriminator (`vct_values` for SD-JWT-VC, `type_values` for W3C). There is no enumeration primitive, by design.

The consequence is deliberate and worth stating plainly: a query carrying no type discriminator contributes **no** candidates. It does not mean "everything". A holder MUST NOT blind-scan its wallet to answer a query, because a verifier that can phrase an unconstrained query can map a wallet it was never entitled to see.

## Answer, defer, or refuse

A matched query is gated by the holder's consent policy:

- A **pre-trusted** verifier is auto-consented; the holder answers immediately with [`present`](../../present/0.1/) on the same thread.
- Any other verifier is **deferred**: the query is persisted, the verifier is told `consentRequired`, and the decision moves out of band to [`pending/list`](../../pending/list/0.1/) → [`pending/approve`](../../pending/approve/0.1/) or [`pending/deny`](../../pending/deny/0.1/). If approved, the holder presents on the original thread.

`consentRequired` is `retryable: true` in the narrow sense that the exchange may still complete — but a verifier MUST NOT read it as an invitation to re-send. Re-querying does not advance a decision that is waiting on a human, and each attempt persists more state at the holder.

## Conformance

Producer: send a DCQL query with a type discriminator, a fresh `nonce`, and a non-empty `purpose` stating the actual reason. Expect three outcomes: a presentation, `consentRequired`, or `noMatch`.

Consumer: gather candidates only through the type index. Gate every match on consent policy. Never disclose which credentials are held in a refusal — `noMatch` and "held but not consented" MUST be indistinguishable to the verifier, or refusal itself becomes an enumeration primitive.

## Security & Privacy

`exposure.discloses` is `none` for the query itself — it asks, it does not tell.

The privacy properties that matter here are the holder's, and they are enforced by the two rules above: no enumeration, and refusals that do not leak. `sideEffects` is `mutating` because a deferred query persists; a consumer SHOULD bound how much deferral state one verifier can create.
