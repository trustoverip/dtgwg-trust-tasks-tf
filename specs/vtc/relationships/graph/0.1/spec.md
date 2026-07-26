---
slug: vtc/relationships/graph
version: "0.1"
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
errorCodes:
  - code: vtc/relationships/graph:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Relationships — Graph** Trust Task projects the community's live Verifiable Relationship Credentials into a graph: `nodes` are the DIDs involved, `edges` are the relationships, each directed from `issuerDid` to `subjectDid`.

It is the whole-community view of what [`vtc/relationships/list`](../../list/0.1/) returns per relationship, shaped for rendering rather than pagination.

## Conformance

Producer: no payload members are required.

Consumer: verify the community-admin capability. Emit one node per distinct DID appearing in any live edge, and one edge per unrevoked relationship credential. Revoked relationships MUST NOT appear — the graph shows the trust that currently holds, not its history. A DID left with no live edges MUST NOT appear as an isolated node, since it is the edges that put it in the graph at all.

## Security & Privacy

The relationship graph is the community's social structure in one payload. Individually a relationship credential is unremarkable; assembled, the graph reveals clusters, brokers, and isolates — inferences no single credential supports. That aggregation is why the task is admin-gated even though every edge it draws comes from an already-published credential.

Excluding revoked edges is a privacy property as much as a correctness one: a withdrawn vouching should stop being visible, not persist as history in a view an operator reads routinely.
