---
slug: vtc/relationships/list
version: "0.3"
title: VTC Relationships — List
summary: List the Verifiable Relationship Credentials published about a community member.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
  - list
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
  rationale: Read-only listing of a member's relationships. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the relationship store; persists nothing."
subjectPath: /did
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The request carries `did` — the member whose relationships to read, an identifier the community already holds — plus paging controls. Nothing new about anyone travels inbound; the whole weight of this task is in the response, which returns the credential bodies themselves."
retention:
  class: transient
  rationale: The community reads its relationship store and persists nothing from the request. The credentials it returns are already durable, lodged by `vtc/relationships/publish`; this task neither extends nor bounds their life, and revocation rather than deletion is how a relationship is withdrawn.
errorCodes:
  - code: vtc/relationships/list:notFound
    meaning: No member with the supplied did exists.
    retryable: false
---

## Abstract

The **VTC Relationships — List** Trust Task returns the Verifiable Relationship Credentials recorded for a member `did` — each with its `id`, `issuerDid`/`subjectDid`, the `vrcJsonld` body, a `vrcDigestMultibase`, and `createdAt`. Paged by `cursor`/`limit`.

## Changes from 0.2

The response's **`nextCursor`** changes type from `["string", "null"]` to **`"string"`**. It is present when more items remain and **absent** on the last page; a `null` value now fails validation. A producer that detected the last page by `nextCursor === null` **MUST** detect it by absence instead.

This is the registry-wide pagination convention: a continuation token is an optional string whose absence marks the last page, so there is one spelling of "no more pages" instead of two.

The request's `cursor` and `limit` already followed the convention and are unchanged on the wire — same types and bounds; only their schema descriptions are reworded to the registry's paging text. `totalEstimate` is unchanged. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains published.

## Conformance

Producer: supply `did`; optionally `cursor`/`limit`.

Consumer: resolve the member (`notFound` if absent). Return the relationships where the member is issuer or subject, clamping `limit` to 1..=200 and setting `nextCursor` when more remain.

## Security & Privacy

### Data carried

The request names a member — `did` — plus `cursor` and `limit`. The response
returns, for each relationship, the `id`, the `issuerDid` and `subjectDid` that
name both ends of it, the full `vrcJsonld` credential body, a
`vrcDigestMultibase`, and `createdAt`.

`exposure.discloses` is `metadata`, and the entries are indeed descriptive rather
than secret — a published VRC was lodged in order to be read. But "metadata"
should not be read as "thin". `vrcJsonld` is the whole credential, including any
claims the issuing member chose to put in it beyond the bare fact of a
relationship, and this specification constrains none of them: what a VRC asserts
is decided at [`request`](../../request/0.2/spec.md) and
[`publish`](../../publish/0.2/spec.md), and arrives here verbatim.

The one minimisation lever is the consumer's. `limit` is clamped to 1..=200,
which is a ceiling rather than a recommendation, and a caller rendering a
relationship count or a set of edges needs `issuerDid`, `subjectDid` and
`createdAt` — not the credential bodies. Returning the full `vrcJsonld` on every
entry because the schema requires it is worth noticing as a cost, particularly on
a paged walk.

### Correlation

This task is a correlation instrument, and unusually the registry should say so
without apology: reading who has vouched for whom is what it is for.

The DIDs in `items[].issuerDid` and `items[].subjectDid` are not parties to this
exchange, so they carry no `identifierScope` declaration of their own — but they
are necessarily public-scoped, for the reason
[`request`](../../request/0.2/spec.md) sets out. A relationship credential means
something only where a third party recognises both named identifiers as the ones
it sees elsewhere; pairwise identifiers would leave a VRC asserting a
relationship between two parties nobody could resolve. The privacy cost of that
choice is realised *here*, in the task that reads the graph back.

Paging is the multiplier. A single member's entry list is a description of that
member's position in the community; a walk of `nextCursor` across many members
reconstructs the community's social graph, and `createdAt` timestamps every edge
so the graph can be replayed over time. `vrcDigestMultibase` is stable across
stores, so edges read here can be matched against the same credentials appearing
in another community's records without comparing bodies. None of this requires
any privilege beyond the read gate.

The administrator declares `identifierScope: pairwise`. The identity that a graph
read is attributed to belongs inside the community whose graph is being read;
keeping it community-scoped means an operator's reading pattern is not joinable
by identifier to their activity in another community. That is a narrow protection
given what the response contains, and it is the only one this task's parties
afford.

### Retention

The request is transient — a page is read, nothing is written. The credentials it
returns were made durable when they were lodged, and this task neither extends
nor bounds their life.

The durability worth understanding is the one a member cannot undo. A published
VRC is withdrawn by [`revoke`](../../revoke/0.1/spec.md), which supersedes it with
a new signed statement rather than erasing it, so a relationship that existed
generally remains visible as having existed. Consumers **SHOULD NOT** cache
responses to this task beyond the render they were fetched for: a stale cache of
relationship entries is a graph that keeps asserting edges after they were
revoked, which is the specific failure this family's revocation model is designed
to avoid.

### Consent/purpose

The purpose is community transparency — members and administrators can see the
attestations the community holds, which is what makes a relationship credential
worth having. The consent underpinning each entry was given twice and in the
right order: the issuing member agreed to assert the relationship at
[`request`](../../request/0.2/spec.md), and the holder agreed to make it
community-visible at [`publish`](../../publish/0.2/spec.md). That two-step is why
this read is legitimate at all, and why a VRC held privately never appears here.

What no member consented to is bulk extraction. Agreeing that one relationship be
visible to a community is not agreeing that the community's whole graph be walked,
exported, and analysed — and the difference between those two is `nextCursor`,
not any member of the payload. A consumer that enumerates the graph is doing
something the individual publications do not on their own authorise, and it will
look identical on the wire to a console rendering one member's profile.

Whether bulk reads warrant a narrower capability than the ordinary read gate,
and whether they should be audited, are consumer policy questions; per
[SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification
describes the exposure and does not require a gate on it.
