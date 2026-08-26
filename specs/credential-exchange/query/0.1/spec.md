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
    identifierScope: public
  - role: Holder
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: OPTIONAL
  rationale: An authcrypted transport proves which DID is asking, and the verifier's identity is the input the holder's consent decision turns on. A document proof is permitted where the envelope's authentication does not survive relaying.
sideEffects:
  level: mutating
  rationale: A query the holder does not auto-consent to is PERSISTED as a deferral awaiting an out-of-band decision. Asking therefore leaves durable state at the holder even when nothing is presented — which is also why an unbounded query rate is a storage concern, not merely a nuisance.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
retention:
  class: exchange
  rationale: A query the holder does not auto-consent to is persisted as a deferral and lives exactly as long as the decision it is waiting on — until approval, denial, or `expiresAt`, after which the verifier's nonce is stale and the record cannot be acted on at all. Nothing about the query is kept past the decision it was stored to enable.
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

### Data carried

The request is three members, and all three are the verifier's own words. `dcql_query`
names credential types — through the `meta` discriminator, `vct_values` for SD-JWT-VC
or `type_values` for W3C — and the claim paths the verifier wants. That is descriptive
data about *credentials sought*, not attributes of the person holding them: the query
asks, it does not tell, and there is no response payload carrying claims. Disclosure is
[`present`](../../present/0.1/)'s job, and every rule here exists to keep the two
separate.

The one path by which a verifier can push personal data *into* a holder is DCQL's
claim-value filtering. A query may constrain the values it will accept, and "the
credential whose `family_name` is Schmidt" asserts an attribute rather than requesting
one. A producer **SHOULD NOT** use value constraints to state what it already believes
about the holder: a deferred query is persisted and put in front of a human, so the
assertion outlives the request and is read as though the holder's own agent had
recorded it.

`purpose` is REQUIRED, non-empty and free text, which makes it the member most likely
to carry more than the task needs. It is shown to whoever decides and bound into the
consent record, so it is the verifier's most durable statement in the whole exchange.
A producer **SHOULD** write the reason for this request and not the case file behind
it — a purpose string is read by a person and retained as evidence, and neither of
those is improved by additional detail about why the verifier is interested.

`purpose` is the payload's one free-text member and is bounded at 500
characters — the consent-surface figure, because that is exactly what it is: a
sentence a verifier writes and a holder reads while deciding. It is **REQUIRED**,
which departs from the SHOULD of
[SPEC.md §7.3](/SPEC.md#73-specification-requirements) item 19 and is the whole
point of purpose binding — a verifier that could omit it could ask without
saying why. It is authored by the *verifier*, who is not the party the holder
trusts, so it is **explicitly untrusted**: a consent surface MUST attribute it to
the requesting verifier, MUST NOT let it displace the credential types actually
being asked for, and MUST NOT treat it as a statement of what the verifier will
do with what it receives. Its only reader is the holder, or the approver acting
for them; where the query is deferred it is retained for the life of that
deferral, per *Retention* below, and not beyond it.

### Correlation

The verifier's identifier is the pivot this task turns on. The holder's consent policy
is a decision about *which verifier is asking* — a pre-trusted one is auto-consented,
anyone else is deferred — and a verifier the holder cannot recognise from one exchange
to the next cannot be pre-trusted, cannot be audited across disclosures, and cannot be
matched against an ecosystem trust list. That is why the Verifier party declares
`identifierScope: public`: a pairwise verifier identifier would make every query look
like a first contact and would collapse the consent decision into a coin toss. The
Holder faces no such constraint and declares `identifierScope: pairwise` — nothing in
this task requires the holder to be recognisable to anyone but the verifier in front of
it, and a holder using a distinct identifier per verifier keeps its queries from being
joined across them.

Refusal is the subtler channel. `noMatch` and "held, but not consented" **MUST** be
indistinguishable to the verifier, because a refusal that discriminates between them is
an enumeration primitive wearing an error code: a verifier that can tell the two apart
maps a wallet one query at a time without ever receiving a presentation. The
no-enumeration rule closes the direct route — candidates are gathered only through the
type index named by `meta`, and a query carrying no discriminator contributes **no**
candidates rather than all of them — but the rule is worthless if the failure path
leaks what the success path withheld.

Timing is left. A deferred query tells the verifier that the holder's consent policy
did not pre-trust it, and the interval before a presentation arrives is a measure of
how long a human took. A consumer that wants to narrow that channel varies its
`consentRequired` response time rather than answering as fast as the deferral is
written.

### Retention

Exchange-scoped, and bounded by a value the holder does not control. A deferred query
is persisted with the verifier's `nonce` intact so that an approval can present against
the original request byte-faithfully; once `expiresAt` passes, the verifier has stopped
accepting that nonce and the record is unactionable — [`pending/list`](../../pending/list/0.1/)
**MUST** omit it and [`pending/approve`](../../pending/approve/0.1/) **MUST** refuse it.
Expiry is therefore the deletion trigger as well as the freshness rule, and a consumer
has no reason to keep the record past it.

This is also why `sideEffects` is `mutating` rather than `none`: asking leaves durable
state at the holder even when nothing is presented. A verifier that re-sends on
`consentRequired` does not advance a decision waiting on a human — it accumulates
deferrals. A consumer **SHOULD** bound how much deferral state one verifier can create,
which is a storage limit and a privacy limit at once, since the backlog is a record of
that verifier's interest in this holder.

### Consent/purpose

`purpose` is what makes the rest of the family a consent flow rather than an access
protocol. A verifier cannot ask without saying why, and a consumer **MUST** surface the
stated purpose to whoever decides and bind it into any consent record it keeps, so that
a disclosure can later be audited against the reason given for it. Making the member
optional would render the well-behaved verifier's query indistinguishable from the one
that declines to explain itself, precisely where the difference matters.

What this specification does **not** do is require a human. The holder's policy decides:
a pre-trusted verifier is answered immediately, anyone else is deferred to
[`pending/approve`](../../pending/approve/0.1/) or [`pending/deny`](../../pending/deny/0.1/).
Which verifiers earn pre-trust, and whether a person is asked at all, are consumer
policy questions this task takes no position on — it supplies the stated purpose the
decision is made against, and leaves the gate to the deployment.
