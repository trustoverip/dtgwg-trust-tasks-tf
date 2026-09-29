---
slug: vtc/relationships/revoke
version: "0.2"
title: VTC Relationships — Revoke
summary: The issuer revokes a Verifiable Relationship Credential they previously published — including one issued under a pairwise relationship DID, by proving control of it — or an administrator revokes it for moderation.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
  - revoke
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Only the credential's issuer, or an administrator, may revoke it; the proof signer is who the community checks control against, and the revocation must be attributable on every transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Revoking a relationship, replayed after it was re-established, severs the new one. Both parties act on the published state, so the severance is immediately load-bearing and not merely recorded.
sideEffects:
  level: mutating
  rationale: "Revokes a relationship credential; reversible by publishing a fresh one."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/relationships/revoke:notFound
    meaning: No relationship with the supplied id exists, or the proof signer established neither the issuer capacity (directly, or via pop) nor an administrator's.
    retryable: false
related:
  - vtc/relationships/publish
  - vtc/relationships/list
---

## Abstract

The **VTC Relationships — Revoke** Trust Task revokes a relationship credential `id` previously published via [`vtc/relationships/publish`](../../publish/0.2/spec.md). `0.1` let only the credential's own issuer revoke it, proven the same way publication was: the document's proof signer equal to that issuer DID. That check is false by construction for an edge published under a **pairwise relationship DID** — the party who published it never signs a document as that DID — so an edge published that way could be revoked over a signed document by nobody but an administrator. `0.2` closes that gap the way [`vtc/relationships/publish/0.2`](../../publish/0.2/spec.md) already closed it on the way in: an optional `pop`, a proof of possession of the relationship DID's key, bound to this document and to the edge being revoked.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

`0.1`'s payload carried `{ id }` alone. Revoking an edge published under a relationship DID had no route through this task at all: the community's bearer REST route accepted a `VrcRevokeAuthorization` proving control of that DID, but a signed document sent over TSP, DIDComm or HTTPS `/trust-tasks` had nothing equivalent to send, so an edge in that form could only be retracted by an administrator, never by the member who issued it.

`0.2` adds an optional `pop`: a proof of possession by the relationship's `issuerDid`, of the same shape [`vtc/relationships/publish/0.2`](../../publish/0.2/spec.md)'s `pop` already established for publication — `documentId` binding it to this document, a `proof` a verification method the `issuerDid` controls can be checked against — with one addition, `relationship`, naming which edge it authorizes (publication only ever authorizes the one VRC in the same document, so `pop` there needed no separate binding; revocation names its edge by `id` alone, so this document's `pop` binds to it explicitly) and one substitution, `type: "VrcRevokeAuthorization"` in place of `VrcPublishAuthorization`, so a signature made to authorize a *publish* cannot be replayed to authorize a *revoke*.

This is purely additive: every `0.1` document (`{ id }`, no `pop`) is a conforming `0.2` document, and a member who is the document's own proof signer — the ordinary, attributed case — still needs no `pop`. `0.1` is not retired.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

Producer: supply `id`. Carry a proof. Supply `pop` when the document's proof signer is neither the relationship's `issuerDid` nor an administrator — i.e. whenever the edge was published under a pairwise relationship DID and this revocation is not moderation. Sign `pop` with a verification method the relationship's `issuerDid` controls; set `documentId` to this document's own `id` and `relationship` to the edge's `id` (the same value as the top-level `id`).

Consumer:

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline.
2. Resolves the relationship named by `id`. If none exists, refuses with `notFound`.
3. **MUST** authorize by one of three routes, checked in this order:
   - **issuer** — the document's proof signer equals the relationship's `issuerDid`. No `pop` needed: the document's own proof already establishes control.
   - **administrator** — the proof signer holds the community-administrator capability. Moderation, keyed on the row `id` and not on issuer identity; no `pop` needed.
   - **pairwise** — `pop` is present, its `type` is `VrcRevokeAuthorization`, its `documentId` matches this document's `id`, its `relationship` matches the top-level `id`, and its `proof` verifies against a verification method the relationship's `issuerDid` controls.
4. **MUST** refuse with `notFound` — the same code as an unknown `id` — when none of the three routes holds, or a supplied `pop` fails any check.
5. Otherwise **MUST** revoke the relationship and return `{ id }`.
6. **MUST** audit the revocation, recording capacity `"issuer"` for the issuer and pairwise routes and `"admin"` for the administrator route: proving control of the issuing key *is* being the issuer, and recording it as an administrative action would misattribute a member's own decision in the one trail an operator uses to answer who did what.
7. **MUST NOT** retain `pop` after verifying it. It exists to answer one question at one moment, and storing it accumulates a durable link between the revoking member and a relationship DID that names nobody — which is the correlation publishing under a relationship DID exists to avoid.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this task presupposes is one of two: being the relationship's own issuer — directly, where the document's proof signer is that DID, or by proving control of it through `pop` — or holding the community-administrator capability. Neither substitutes for the other: an administrator's standing needs no `pop`, and a valid `pop` needs no administrator standing.

Note the deliberate choice in how a failure is reported, carried over from `0.1`: a caller who is neither route receives `notFound`, the same code as for a relationship that does not exist, and the same code a `pop` that fails to verify receives. That is an anti-probing measure, not an oversight. Distinguishing "wrong id" from "not your relationship" from "your `pop` didn't verify" would let a caller enumerate relationships it did not issue, or learn whether a guessed pairwise DID was the right one.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying `proof` (or `pop`) establishes who asked, never that they may.

## Security & Privacy

### Data carried

The request names one relationship `id`, and, on the pairwise route, a `pop` proving control of that edge's `issuerDid` — nothing beyond what `0.1` already carried, plus the proof itself. The response echoes `id`. Nothing about the edge's content, or its subject, is disclosed by this task.

### Correlation

**Issuer-gated, three ways.** Unknown id, not-your-relationship, and a `pop` that fails to verify collapse to one `notFound`, so revoke is not an oracle over others' relationships, nor over which pairwise DIDs a member controls. Only the issuer — proven directly or through `pop` — or an administrator can revoke an edge.

The relationship's issuer party keeps whatever `identifierScope` it published under; revocation asserts nothing new about it, whichever of the three routes authorized it. An administrator moderating an edge is a fact about this community's own records, not a new claim about the edge's identifiers.

**`pop` is verified and discarded.** It carries no `sessionId` or comparable transport binding — a signed document has no session to bind to — so its anti-replay properties are `documentId` (every Trust Task document is unique, [SPEC §4.3](/SPEC.md#43-the-id-member)) and `relationship` (this authorization is for one edge). A consumer that retained it would create a durable, queryable link between the revoking member's proof-signing identity and a relationship DID that otherwise names nobody — precisely the correlation a pairwise identifier exists to avoid.

### Retention

The community keeps only that the edge no longer exists — the same as `0.1`. Nothing about `pop`, or which of the three routes authorized a given revocation, is retained beyond the audit trail's `"issuer"` / `"admin"` capacity marker.

### Consent/purpose

The purpose is to let an edge's own issuer retract it, or an administrator moderate one, on the same terms `0.1` already stated; `0.2` only widens *who can prove they are the issuer*, not who may revoke. A consumer **MUST NOT** read a `pop`-authorized revocation as evidence of anything about the pairwise DID beyond that its controller asked for this one edge to end.
