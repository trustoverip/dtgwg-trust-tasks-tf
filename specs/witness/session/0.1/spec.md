---
slug: witness/session
version: "0.1"
title: Witness — Session
summary: Opens one participating party's witness session as its own exchange, nested in a relationship exchange via parentThreadId. The witness's response issues the session challenge; this document's id is the value that party's Verifiable Witness Credential later carries as taskContext.
status: draft
targetFrameworkVersion: "0.4"
category: credentials
keywords:
  - witness
  - session
  - vwc
  - relationship
  - challenge
authors:
  - Alberto L (https://github.com/albertoleon7794)
parties:
  - role: participating party
    requirement: REQUIRED
    member: issuer
  - role: witness
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: >-
    The request is consumed in-exchange by the witness under transport
    authentication (SPEC.md §4.7.1). The response is REQUIRED because it
    issues the session challenge — the value the party's presentation binds
    to — and because this session's documents are retained as the context of
    outcome evidence relied on by parties outside the exchange: a forged or
    unattributable challenge poisons everything built on it.
sideEffects:
  level: mutating
  rationale: "The witness opens session state it must hold until the session terminates or expires."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: witness/session:refused
    meaning: The witness declines to open a session — policy, capacity, or the named exchange is not one it will witness.
    retryable: false
related:
  - witness/session/submit
  - vrc/relationships/propose
---

## Abstract

**Witnessing**: a third party observes a relationship exchange and later
attests, in a Verifiable Witness Credential, that it did. This specification
opens one participating party's session with that witness. The witness's
`#response` issues the **session challenge** the party binds its presentation
to under [`witness/session/submit`](../../session/submit/0.1/spec.md).

## One session per participating party

A session is **bilateral**: one participating party, one witness. A witnessed
relationship exchange between two parties therefore runs **two sessions** with
the same witness — each party opens its own — and both name the same
`parties` pair, which is how the witness knows the two sessions observe one
exchange and how a verifier holding both can see that they do.

This follows from the framework's model rather than from preference. A *Trust
Task* is bilateral ([SPEC.md §2](../../../../SPEC.md#2-terminology)), and the
challenge travels in a `#response`, which reaches the party that sent the
request and nobody else. A single shared session would leave the second party
with no conforming way to obtain the challenge it is required to bind to;
inventing one would mean either a bespoke relay of the witness's own
`#response` through the relationship exchange, or a third party submitting on
a thread it never opened.

The two sessions are separate exchanges, so each carries its own challenge.
Nothing is lost by that: it is the **witness** that binds them, and its
attestation — not a shared nonce — is what says the two presentations belong
to one witnessed exchange. Distinct challenges also mean neither party's
presentation can be replayed into the other's session.

Where the pair of sessions needs to be named as a single flow rather than
inferred from the `parties` pair, that is what a *Trust Ceremony* is for
([§4.11](../../../../SPEC.md#411-the-ceremony-member)): the `ceremony` member
is carried on the document rather than in `payload`, and §4.11.1 forbids a
*Trust Task specification* from declaring anything about it — so this
specification says nothing, and composing these sessions into an enactment
requires no change here and no new version.

## The nesting, and what a `taskContext` names

A session is **its own exchange**, conducted inside a relationship exchange.
Two rules follow, and they are the reason this specification exists in this
exact shape:

1. **This document's `id` is the session's name.** Per
   [SPEC.md §4.9.1](../../../../SPEC.md#491-naming-an-exchange-from-outside-the-framework),
   a citation naming an exchange as evidence names the *innermost* exchange
   that attests the event, by the `id` of the document that initiated it. The
   witnessing is attested by *this* exchange — not by the surrounding
   relationship exchange, whose own responses say nothing about whether a
   witness observed anything. A Verifiable Witness Credential's `taskContext`
   therefore carries **this document's `id`**, never the outer exchange's
   thread. Since each party opens its own session, each party's VWC anchors to
   its own — which is correct, because each attests that party's participation.
2. **Every document of the session carries `parentThreadId`.** A *producer*
   **MUST** set `parentThreadId`
   ([§4.9.2](../../../../SPEC.md#492-the-parentthreadid-member)) to the
   containing relationship exchange's `threadId` on every document of this
   exchange, including responses and error responses. Per §4.9.2 the member
   is navigation only: a *consumer* **MUST NOT** reject a document solely for
   its absence.

## Conformance

A conforming **participating party** (`issuer`):

1. Emits a document whose `type` is `https://trusttasks.org/spec/witness/session/0.1`, addressed to the witness, with a fresh `id`, `threadId` equal to that `id`, and `parentThreadId` per the rule above.
2. Names in `parties` the relationship DIDs of the exchange to be witnessed — **both** of them, including its own, in either order.
3. Opens **its own** session. A party **MUST NOT** submit under a session opened by its counterparty, and **MUST NOT** treat a challenge relayed to it by any party other than the witness as this session's challenge.

A conforming **witness** (`recipient`):

1. Applies the [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) pipeline.
2. Verifies that the session's `issuer` is one of the DIDs named in `parties`; a session opened by a party that is not in the exchange it claims to witness is refused with `witness/session:refused`.
3. On accepting, returns the `#response` carrying a fresh, unpredictable `challenge` and its `domain`, under the REQUIRED proof.
4. On declining, returns a `trust-task-error` with `witness/session:refused`.

## Security & Privacy

**The challenge is the session's binding value.** Presentations under `submit`
are bound to `{challenge, domain}`. A challenge is single-use and
unpredictable; a witness **MUST NOT** reuse one across sessions, including
across the two sessions of a single witnessed exchange — a shared value would
let either party's presentation satisfy the other's session. *The pair may be
superseded by a canonical session transcript* — binding protocol and profile
versions, context, purpose, scope, session and epoch — as that work is
ratified; this specification deliberately isolates the binding material in one
place so that upgrade replaces a member rather than the exchange.

**A witness learns who is forming relationships.** Opening a session
discloses the parties' relationship DIDs to the witness by necessity. Those
are pairwise values, but the witness can correlate the sessions it serves;
parties choose witnesses with that in view, and a witness's own retention is
governed by the policies it publishes, not by this specification.
