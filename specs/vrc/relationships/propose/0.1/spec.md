---
slug: vrc/relationships/propose
version: "0.1"
title: VRC Relationships — Propose
summary: One person proposes a peer-to-peer relationship to another, naming the pairwise DID they will use for it; the counterparty accepts or declines, and says whether the exchange will be witnessed. Opens the relationship exchange every later document of the exchange threads to.
status: draft
targetFrameworkVersion: "0.4"
category: credentials
keywords:
  - vrc
  - relationship
  - peer-to-peer
  - handshake
authors:
  - Alberto L (https://github.com/albertoleon7794)
parties:
  - role: proposing party
    requirement: REQUIRED
    member: issuer
  - role: counterparty
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: OPTIONAL
  response: OPTIONAL
  rationale: >-
    Both documents are consumed inside the exchange by the connected peer and
    retained by nobody else; the transport's authcrypt supplies sender
    authentication per SPEC.md §4.7.1, and no third party ever evaluates
    these documents — the exchange's durable artifacts are the credentials
    delivered by vrc/relationships/issue, which carry their own proofs.
sideEffects:
  level: none
  rationale: "Nothing is persisted by proposing or accepting; state changes only when vrc/relationships/issue delivers credentials."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vrc/relationships/propose:declined
    meaning: The counterparty declines the relationship. The human-readable reason, if any, travels in the error payload's `message`.
    retryable: false
related:
  - vrc/relationships/issue
  - witness/session
---

## Abstract

Two people establish a **peer-to-peer relationship**: each will issue the other
a Verifiable Relationship Credential under pairwise relationship DIDs. This
specification opens that exchange — the proposing party names the relationship
DID it will use and whether it requests a witnessed exchange; the counterparty
accepts (naming its own relationship DID) or declines.

This is the peer-to-peer sibling of the community-mediated
[`vtc/relationships/request`](../../../../vtc/relationships/request/0.1/spec.md):
that specification obtains one credential from a community member; this
exchange is **mutual** (both parties issue, via
[`vrc/relationships/issue`](../../issue/0.1/spec.md)) and optionally
**witnessed** (via [`witness/session`](../../../../witness/session/0.1/spec.md)).

## The exchange this document opens

The `id` of a `propose` document names the relationship exchange: every later
document of the exchange — the acceptance, both `issue` legs — carries it as
`threadId` per [SPEC.md §4.9](../../../../../SPEC.md#49-the-threadid-member).
Where the exchange is witnessed, each party's witness session is a **separate,
nested exchange**: its documents carry their own `threadId` and name this
exchange via `parentThreadId`
([§4.9.2](../../../../../SPEC.md#492-the-parentthreadid-member)). No document
of *this* specification references a session — witnessing is additive, never a
precondition, and a party that never opens one still completes the exchange.

## Conformance

A conforming **proposing party** (`issuer`):

1. Emits a document whose `type` is `https://trusttasks.org/spec/vrc/relationships/propose/0.1`, addressed to the counterparty.
2. Sets `relationshipDid` to the pairwise DID it will use for this relationship — a DID minted for this relationship, not a long-term identifier.
3. **MAY** set `witnessed: true` to ask that the exchange be witnessed. Which witness, and on what terms, is settled under the `witness/*` specifications; this member settles only whether the parties are proceeding on that basis.

A conforming **counterparty** (`recipient`):

1. Applies the [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) pipeline.
2. On accepting, returns a `#response` with `accept: true` and its own `relationshipDid`.
3. Sets `witnessed: true` on that response where it agrees the exchange will be witnessed. An acceptance **silent** on `witnessed` accepts an **unwitnessed** exchange, whatever the proposal asked for; a counterparty unwilling to proceed unwitnessed **MUST** decline rather than accept, since this specification defines no renegotiation.
4. On declining, returns a `trust-task-error` with `vrc/relationships/propose:declined`. A decline is an error response, not a message type, on the same grounds `vtc/relationships/request` records.

## Security & Privacy

**Relationship DIDs are pairwise by design.** The values exchanged here are
minted for this relationship; carrying a long-term DID in `relationshipDid`
would make every later presentation of the resulting credentials linkable.
`exposure.discloses` is `metadata` because even the pairwise DIDs are
correlation handles between these two parties.

**Acceptance is not issuance.** An `accept: true` response commits nobody: no
credential exists until `vrc/relationships/issue` delivers one, and either
party may still walk away. Consumers **MUST NOT** treat an accepted proposal
as evidence a relationship was established — that is what the credentials,
and where applicable the witness sessions' evidence, are for.

**Witnessing is agreed, not assumed.** `witnessed` on the request is a request;
`witnessed` on the response is the answer. Treating the request as settling it
lets one party believe an exchange was observed when the other never opened a
session — and the resulting credentials carry no witness evidence to contradict
the belief. This is why silence resolves to *unwitnessed*: the failure mode of
under-claiming is a party who asks again, and of over-claiming is a relationship
recorded as witnessed that nobody witnessed.
