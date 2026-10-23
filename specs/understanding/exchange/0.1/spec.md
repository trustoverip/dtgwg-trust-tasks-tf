---
slug: understanding/exchange
version: "0.1"
title: "Understanding — Exchange"
summary: "A joining party answers from the terms it was offered, demonstrating comprehension rather than merely acknowledging receipt; the host records whether the answers show it."
status: draft
targetFrameworkVersion: "0.6.0"
category: consent
parties:
  - role: joining-party
    requirement: REQUIRED
    member: issuer
  - role: host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The answers are the evidentiary record `understanding/seal` and any later relationship
    credential rely on: a joining party who could later deny giving a given answer, or a host
    who could alter the recorded outcome after the fact, defeats the reason the ceremony exists.
    Both directions carry a proof so neither side can repudiate its half.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    This document answers against the terms version named by a specific understanding/offer
    response. Replayed, it would bind an old set of answers to a terms version that may since
    have changed, with the acceptance window the only thing standing between a stale answer
    and one presented as current.
sideEffects:
  level: mutating
  rationale: >-
    The host records the joining party's answers and its own determination as the outcome a
    later understanding/seal cites. Recoverable only by conducting a new exchange; this task
    has no separate "retract" counterpart because an exchange, once recorded, is a fact about
    what happened, not a standing grant to revoke.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries the joining party's answers to the host's own comprehension questions
    about its terms — ordinarily facts about the terms, not about the joining party, though a
    host that writes questions eliciting personal information changes that; a host SHOULD write
    questions that test comprehension of the terms rather than solicit personal data. The
    response discloses only a pass/fail determination and an optional opaque score.
errorCodes:
  - code: understanding/exchange:offerNotFound
    meaning: The `offerId` does not name a live understanding/offer response this host issued.
    retryable: false
  - code: understanding/exchange:issuerMismatch
    meaning: >-
      The request's `issuer` is not the party the understanding/offer response named by `offerId`
      was addressed to — that response's `recipient`, the identifier the opening request was issued
      under. A `proposedIdentifier` carried by the opening request does not satisfy this rule.
    retryable: false
  - code: understanding/exchange:termsStale
    meaning: The terms version named by `offerId` is no longer current; the joining party must open a new understanding/offer.
    retryable: false
related:
  - understanding/offer
  - understanding/seal
---

## Abstract

The **Understanding — Exchange** Trust Task is where a joining party demonstrates comprehension
of the terms named in an `understanding/offer` response, rather than merely acknowledging that
it received them. The joining party answers a set of host-defined questions drawn from the
terms document; the host records both the answers and its own determination of whether they
show understanding. This has to be a Trust Task, and not a plain form submission, because the
resulting determination is what `understanding/seal` — and anything that later relies on the
seal, such as a relationship credential — is entitled to cite as evidence that understanding
was actually tested, not assumed.

This task tests comprehension, not agreement. A joining party can demonstrate that it
understood the terms and still decline to be bound by them; `understanding/seal` is the
separate, later step where agreement is recorded.

This comprehension check is this framework's own addition, not a requirement of IEEE
7012-2025 — see [`understanding/seal`](../../seal/0.1/spec.md#relationship-to-ieee-7012-2025)
for the family's full account of what comes from that standard and what does not. 7012's own
approach to making a choice safe to make without testing it is to keep the roster of possible
agreements small, not to examine whether a given party understood the one it picked; this task
gates entry to `understanding/seal`'s 7012-style handshake with an explicit check instead.

This specification declares no `outcomeEvidence` ([SPEC §7.3 item 20](/SPEC.md#73-specification-requirements)):
the only party that currently cites this exchange's outcome is `understanding/seal`, by a bare
`exchangeId`, and the party checking it is the host — the same party that issued the response
being cited. That is a first-party lookup against the host's own record, not the third-party
citation item 20's digest binding exists to protect: a stranger who was not this exchange's host
has no way to verify an `exchangeId` independently, and this version of the specification does
not provide one. A future need for a third party to cite this exchange's outcome directly would
require adding that declaration; it is a known gap, not an oversight.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets
framework version 0.6.0 and may change without a version bump while it remains a draft
([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described
in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all
capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements)
in addition to the requirements stated here.

## Authorization

The entitlement to open an exchange is being the joining party that a live
`understanding/offer` response this host issued was addressed to. Two checks establish it, and a
recipient **MUST** make both before evaluating the answers:

1. `offerId` **MUST** resolve to a live `understanding/offer` response this host issued, per
   `understanding/exchange:offerNotFound`.
2. The request's `issuer` **MUST** be the party that response was addressed to — its
   `recipient`, which is the `issuer` the opening `understanding/offer` request was issued under.
   A recipient **MUST** refuse any other issuer with `understanding/exchange:issuerMismatch`.
   A `proposedIdentifier` the opening request carried does **not** satisfy this rule: control of
   that identifier was never proven in the offer step — the offer's proof was made under
   `issuer`, not under it — so letting it sign the exchange would let an unproven key take over
   the thread. `proposedIdentifier` governs only the identity recorded at `understanding/seal`,
   not who may continue the engagement.

The second check is what makes the first worth anything. `threadId` and a document's `id` are
correlation handles, not secrets: the offer response's `id` passes through whatever transport
carried it, and any party that learns it can cite it as `offerId`. Without the issuer binding, a
third party could answer against someone else's offer, and the host's record would then name that
party as the one that demonstrated understanding of the terms — a record `understanding/seal`
inherits through `exchangeId`. With it, the entitlement is complete: this task is open to the
joining party the host already answered with an `understanding/offer` response, and to no one
else, with no further precondition of membership, payment, or prior relationship.

Verifying the request's proof establishes that the named joining party gave these answers and
that they are unaltered; it does not establish that the answers are correct. Correctness is
the host's own determination, stated in the response's `outcome`, and this specification does
not constrain how a host reaches it.

## Definitions

* **Joining party** (`joining-party`). The party answering, continuing the thread it opened with
  `understanding/offer`; identified by `issuer`, which is the party the referenced offer response
  was addressed to — the same identifier it opened the offer under, never a `proposedIdentifier`
  (Authorization, above).
* **Host** (`host`). The party that named the terms in its `understanding/offer` response and now
  evaluates the answers; identified by `recipient`.
* **Offer reference.** `offerId` — the `id` of the `understanding/offer` response this exchange
  answers against, naming which terms version the answers are about.
* **Answer.** One `{ questionId, answer }` pair. `questionId` and the universe of valid answers
  are defined by the host's terms document, out of band; both are opaque strings to this
  specification.
* **Outcome.** The host's determination of whether the answers demonstrate understanding of
  the named terms: `understood` or `insufficient`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/understanding/exchange/0.1`
with a payload that validates against the top-level schema in
[`payload.schema.json`](payload.schema.json). Issued by the joining party, addressed to the
host, with `parentThreadId` set to the `understanding/offer` request's `threadId` per
[SPEC §4.9.2](/SPEC.md#492-the-parentthreadid-member).

### The joining party answers

```json
{
  "id": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/understanding/exchange/0.1",
  "issuer": "did:example:alice",
  "recipient": "did:example:host",
  "issuedAt": "2026-09-24T10:05:00Z",
  "threadId": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000001",
  "parentThreadId": "urn:uuid:1a2b3c4d-0001-4000-8000-000000000001",
  "payload": {
    "offerId": "urn:uuid:1a2b3c4d-0001-4000-8000-000000000002",
    "answers": [
      { "questionId": "retention-period", "answer": "12-months" },
      { "questionId": "third-party-sharing", "answer": "none" }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:alice#key-1",
    "created": "2026-09-24T10:05:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type:
https://trusttasks.org/spec/understanding/exchange/0.1#response`, with a payload that validates
against the `$anchor: "response"` sub-schema in `payload.schema.json`. Issued by the host.

The response payload is `{ outcome, score? }`. A joining party proceeding to
`understanding/seal` **MUST** carry this document's `id` forward as `exchangeId`.

Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response`
variant. `outcome: insufficient` is not a failure — it is the host's normal, evidentiary
determination that the answers did not demonstrate understanding, and it is carried in a
`#response` document like any other outcome.

### The host confirms understanding

```json
{
  "id": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/understanding/exchange/0.1#response",
  "threadId": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000001",
  "parentThreadId": "urn:uuid:1a2b3c4d-0001-4000-8000-000000000001",
  "issuer": "did:example:host",
  "recipient": "did:example:alice",
  "issuedAt": "2026-09-24T10:05:01Z",
  "payload": {
    "outcome": "understood"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:host#key-1",
    "created": "2026-09-24T10:05:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6ab..."
  }
}
```

## Security & Privacy

### Data carried

The request carries `offerId` and the joining party's `answers`. A host **MUST NOT** write
comprehension questions that solicit personal data the terms do not themselves require — the
task's payload is intended to carry facts about the terms, echoed back, not facts about the
joining party. The response carries only `outcome` and an optional opaque `score`; it never
carries the answers back or the host's grading rationale.

### Correlation

`parentThreadId` links this exchange to the `understanding/offer` it answers, by design — the
same correlation the family depends on throughout. `answers` **MAY** be correlatable across
exchanges with the same host if `questionId` values are reused verbatim across joining
parties, but the answers themselves are about the terms, not about the joining party, so this
does not by itself identify anyone.

### Retention

A host retains the request and response for at least as long as it may need to answer a
dispute over whether an `understanding/seal` was properly grounded in a passed exchange —
this is the record `understanding/seal:exchangeNotFound` and similar checks are answered
against. It has no obligation to retain the record indefinitely once the engagement it
supported has concluded or lapsed.

### Consent/purpose

The joining party's answers are collected to test comprehension of the named terms and for no
other purpose. This specification does not require the host to obtain consent, approval, or a
step-up before evaluating them — it only requires the response to say what it determined.
