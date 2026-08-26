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
    identifierScope: pairwise
  - role: Holder agent
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: The response enumerates which credentials the wallet holds and what a verifier asked of them. That is the wallet's contents by another name, so the caller must be authenticated and attributable — transport authentication alone is not enough for a surface whose output is an inventory.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Listing pending requests is performed under the subject's own authority, so a captured list request keeps enumerating the subject's exchange queue for as long as it remains acceptable. The window is what ends that.
sideEffects:
  level: none
  rationale: Reads the deferral backlog; decides nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: true
  rationale: "The caller is the holder asking about its own wallet, so nothing leaves the holder's control. What is returned is nonetheless sensitive — which credentials are held and who has asked for them — which is why the caller is authenticated rather than this being an open read. Nothing travels the other way: the request payload has no members at all, since the caller's authenticated identity is the entire scope."
retention:
  class: transient
  rationale: The request carries nothing and the response is a projection of state the agent already holds on the caller's behalf; neither party has anything new to keep once the read completes. The deferrals themselves are retained by `credential-exchange/query`, on that task's terms, and an audit line recording that a read happened is the consumer's own log rather than a copy of this document.
errorCodes: []
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

### Data carried

The request carries nothing at all — the payload has no members beyond `ext`, because
the caller's authenticated identity is the entire scope. That is the minimum a task can
ask for, and it is why `exposure.ingests` is `none`.

The response is the opposite, and is the most revealing document in the family that never
crosses a trust boundary. Each entry names who asked (`verifierDid`), why (`purpose`,
carried through verbatim from the query), when it was recorded and when it goes stale,
and then `requested` — for every held credential the query would touch, the
`credentialId` and the individual `claims` that answering would release. Taken together
that is an inventory of what the wallet holds *and* a log of who has been interested in
it, which is a sharper picture of the holder than any single credential in the wallet.
`exposure.discloses` is `metadata` because the caller is the holder reading its own
state, not because the content is mild.

What is deliberately absent is the original DCQL query. A consumer retains it so that an
approval can re-present byte-faithfully against the verifier's nonce, but it is machinery
rather than a decision input, and surfacing it would invite an operator to read the
query instead of the resolved disclosure — the same substitution of the abstract for the
concrete that `requested` exists to prevent.

`purpose` on each pending entry is free text, bounded at 500 characters — the
consent-surface figure, carried through verbatim from the query that raised the
deferral. It is the *verifier's* prose, not the agent's, so it is **untrusted**
wherever it is rendered: the surface listing pending decisions MUST attribute it
to the verifier that wrote it and MUST NOT let it stand in for the credential
types being requested. Its reader is the holder working through their pending
list. This task retains nothing — it is a projection — so the value lives
exactly as long as the deferral it belongs to.

### Correlation

The `requested` array is a join the holder performs on itself: it resolves the verifier's
query against the wallet as it actually stands, so each entry links a verifier to
specific held credentials. That is exactly the linkage an approver needs and exactly the
linkage a wallet should never perform for anybody else. A consumer **MUST** scope the
result to the calling agent's own deferrals; a listing that spanned wallets, or that
answered for a principal other than the caller, would hand out the correlation this task
computes as a service.

Both parties declare `identifierScope: pairwise`. Operator and agent are two components
of one principal's wallet, and nothing in this task asks either to be recognisable
outside that relationship — a public identifier on this leg would let an observer tie
wallet-administration traffic to the same party's presentations elsewhere, for no gain.

Read timing leaks a little on its own: a burst of `pending/list` calls says an operator
is working through a backlog, and the backlog's size is a function of how many verifiers
have been asking. This is a low-rate channel and it is internal, but a consumer that
audits reads — as it should — creates a durable record of the holder's attention, which
is worth holding to the same standard as the rest of the log.

### Retention

Nothing here is retained, because nothing here is new. The request is empty and the
response is a projection of state the agent already holds: the deferrals were persisted
by [`query`](../../../query/0.1/) and live on that task's terms, expiring with the
verifier's nonce. Rendering them into a list creates no second copy anyone is expected
to keep.

The obligation this task does carry is negative. A consumer **MUST** omit deferrals that
are terminal or past `expiresAt`, and **MUST** resolve `requested` against the wallet as
it stands now rather than as it stood when the query arrived — a credential deleted in
the meantime cannot be disclosed and must not be listed as though it could. Both rules
keep the listing from being a shadow copy of state that has already moved on.

### Consent/purpose

This task exists to make consent possible rather than to obtain it. Its purpose is to
put in front of the deciding party the three things a disclosure decision needs — who is
asking, the reason they gave, and precisely which claims of which credentials answering
would release — so that the decision at
[`pending/approve`](../../approve/0.1/) or [`pending/deny`](../../deny/0.1/) is made
against its inputs rather than against a verifier's name and a purpose string.

The listing itself is not a decision and confers nothing: reading a deferral neither
approves nor extends it. Because the response is an inventory, a consumer **SHOULD**
audit reads of it and **MUST NOT** treat the authorization that permits listing as
sufficient for approving — the two are different acts with different consequences, and
only one of them discloses anything. Whether a human is required for either is a
consumer policy question this specification does not answer.
