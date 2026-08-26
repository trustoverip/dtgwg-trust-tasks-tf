---
slug: vtc/relationships/graph
version: "0.2"
title: VTC Relationships — Graph
summary: Return a Verifiable Trust Community's published relationship credentials as a node-and-edge graph of who has vouched for whom.
status: draft
targetFrameworkVersion: "0.2"
category: reputation
keywords:
  - vtc
  - relationships
  - graph
  - vrc
  - social
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only projection of published relationships. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Projects stored relationship credentials; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
---

## Abstract

The **VTC Relationships — Graph** Trust Task projects the community's live Verifiable Relationship Credentials into a graph: `nodes` are the DIDs involved, `edges` are the relationships, each directed from `issuerDid` to `subjectDid`.

### Changes from 0.1

`0.1` called each credential an edge. That made a DTG edge inexpressible.

Two directed halves between the same pair of identifiers are **one
relationship**, and a flat list of credentials leaves every consumer to
re-derive that: sort the DIDs, group by pair, and decide for itself what
"complete" means. Two implementations doing that independently will disagree
at the margins — which is the reasoning a schema exists to settle once.

`0.2` makes an edge a pair. `endpoints` carries the two DIDs, sorted so the
pair has one identity whichever half was published first; `halves` carries
every credential between them; and `complete` states whether both parties have
asserted. That last distinction is the one worth having in the model rather
than in each client: **an edge asserted by one party is a claim, and an edge
asserted by both is a relationship.**

`GraphHalf` also carries `personaDid` — the persona an issuer has asserted on
that half. It is the one place deliberate correlation becomes visible: two
pairwise halves carrying the same persona are the same party, said so by that
party. A consumer that cannot read it cannot honour a correlation its subject
chose to publish.

It is the whole-community view of what [`vtc/relationships/list`](../../list/0.1/) returns per relationship, shaped for rendering rather than pagination.

## Conformance

Producer: no payload members are required.

Consumer: verify the community-admin capability. Emit one node per distinct DID appearing in any live edge, and one edge per unrevoked relationship credential. Revoked relationships MUST NOT appear — the graph shows the trust that currently holds, not its history. A DID left with no live edges MUST NOT appear as an isolated node, since it is the edges that put it in the graph at all.

## Security & Privacy

The relationship graph is the community's social structure in one payload. Individually a relationship credential is unremarkable; assembled, the graph reveals clusters, brokers, and isolates — inferences no single credential supports. That aggregation is why the task is admin-gated even though every edge it draws comes from an already-published credential.

Excluding revoked edges is a privacy property as much as a correctness one: a withdrawn vouching should stop being visible, not persist as history in a view an operator reads routinely.
