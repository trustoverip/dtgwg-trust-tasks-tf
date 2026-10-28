---
slug: understanding/seal
version: "0.1"
title: "Understanding — Seal"
summary: "The two parties to a passed understanding/exchange record a bilateral agreement, each independently signing the same terms, through a handshake modelled on IEEE 7012-2025 that does not claim to conform to it."
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
    This is the family's evidentiary center: a relationship credential or VAC formed afterward
    may cite it as the record that both parties agreed. Verifying a proof establishes only that
    each party's own half is attributable and unaltered — not that either party was entitled
    to seal, which is the subject of Authorization below — but without it neither half is
    worth citing at all.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The agreement carries its own `effectiveFrom`, but the documents that form it are placed
    in time independently so a replayed proposal cannot be re-accepted after the referenced
    exchange, or the terms it was grounded in, have moved on.
sideEffects:
  level: mutating
  rationale: >-
    Records a bilateral agreement a relationship credential, a VAC, or other durable artifact
    may later cite as its evidentiary basis. This task has no revoke counterpart: an agreement is
    amended or terminated by a separate, later, equally bilateral act, never unilaterally
    withdrawn by re-emitting this task.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a reference to the agreed terms (roster items and pointers to their
    legal and machine-readable registers) and a reference to the exchange that grounds it —
    metadata about the engagement, not secret material. Neither party's underlying identity
    documentation is disclosed by this task.
outcomeEvidence:
  response: https://trusttasks.org/spec/understanding/seal/0.1#response
  binding:
    id: /payload/taskContext
    taskDigest: /payload/taskDigestMultibase
  rationale: >-
    A relationship credential or VAC formed after a seal typically cites it from the host's
    side, and the host is this exchange's *responder*, not the accepted request's producer — so
    the freshness-challenge binding form (SPEC §7.3 item 20.5's second form) is unavailable
    here, since it requires the initiating document's producer to also be the citing artifact's
    issuer. This declaration therefore uses the first form: the id-plus-task-digest binding of
    SPEC §4.9.3, computed over the accepted request in full. That also closes a gap the family's
    earlier `agreementDigestMultibase` construction left open by scoping the digest to the
    `agreement` member alone: the full-document task digest additionally binds `exchangeId` and,
    where present, `respondsTo`. The response reports every decision — accepted, countered or
    refused (SPEC §8, "A decision is not a failure") — so a signed refusal is citable evidence
    too; a party citing it as the record that an agreement was reached checks `/payload/decision`
    is `accepted`.
retention:
  class: durable
  rationale: >-
    A sealed agreement is exactly the kind of exchange SPEC §7.3 item 20 exists for: cited later
    by a relationship credential, a VAC, or a dispute, none of which this task's own documents
    retain a copy for on the citing party's behalf.
errorCodes:
  - code: understanding/seal:exchangeNotFound
    meaning: The `exchangeId` does not name a live understanding/exchange response between these two parties.
    retryable: false
  - code: understanding/seal:issuerMismatch
    meaning: >-
      The request's `issuer` is not the party the understanding/exchange response named by
      `exchangeId` was addressed to — that response's `recipient`.
    retryable: false
  - code: understanding/seal:understandingInsufficient
    meaning: The exchange named by `exchangeId` recorded outcome "insufficient". A seal cannot be grounded in a failed exchange.
    retryable: false
  - code: understanding/seal:negotiationInProgress
    meaning: >-
      This `exchangeId` already grounds a prior understanding/seal request from this pair, and
      the new request is not the single permitted follow-up to a counter. One exchangeId grounds
      at most one negotiation — see Authorization, "One negotiation per exchange".
    retryable: false
  - code: understanding/seal:negotiationExhausted
    meaning: >-
      This request's `respondsTo` does not name a request, from this same first party on this
      same `exchangeId`, that this recipient countered and has not already answered a follow-up
      to. IEEE 7012-2025 §5.2.2 permits no more than one countered round; a further round MUST
      be refused rather than answered.
    retryable: false
  - code: understanding/seal:agreementMismatch
    meaning: >-
      A follow-up's `agreement` does not match, under JSON Canonicalization, the
      `counterAgreement` of the countered response it answers. Re-proposing different terms is
      negotiation beyond the single choice IEEE 7012-2025 §5.2.2 forbids, not an acceptance of
      the counter.
    retryable: false
related:
  - understanding/offer
  - understanding/exchange
---

## Abstract

The **Understanding — Seal** Trust Task records a bilateral, MyTerms-style (IEEE 7012-2025)
agreement between the two parties to a passed `understanding/exchange`. The *first party*
proposes an agreement grounded in that exchange; the *second party* independently signs
acceptance of the identical content, bound by digest. Neither party's acceptance is inferred
from the other's proposal — each half is its own proof-bearing document, following the same
pattern this ecosystem already uses for bidirectional relationship credentials, where a
relationship is only as real as the two independently signed halves that attest it.

This task carries the agreement's *reference* — which terms were agreed, and pointers to their
canonical registers — not a durable graph artifact in its own right. A relationship credential
or other durable record formed afterward cites this exchange as its evidentiary basis (see
`outcomeEvidence`, above) rather than re-embedding the agreement; the agreement and the
credential are deliberately kept as separate layers, so that revising how relationships are
represented on a trust graph never requires renegotiating what was actually agreed.

Graduated access following a seal — for example, a scoped permission to act — is out of scope
for this task. It is expected to be conferred by a Verifiable Authority Credential (a VAC,
defined in the sibling DTG Core Credentials specification), issued by the host and citing this
exchange as outcome evidence, rather than by a vault-mediated session grant: exercising an
already-held credential and settling whether an agreement was reached at all are different
questions, and this task answers only the second.

Outside attestation — a witness able to confirm the seal occurred, without becoming the
authority for what it means — is not composed into this task. The registry's existing
witnessing mechanism (a Verifiable Witness Credential) attests to a *credential's* claims; a
seal is a Trust Task document, not a credential, so witnessing one is not yet expressible with
that mechanism as it stands. Extending witnessing to Trust Task documents generally, rather than
only to credentials, is left as work outside this specification's scope.

## Relationship to IEEE 7012-2025

This task's proposal/response/counter shape follows the handshake IEEE Std 7012-2025 clause 5
("Action and Interaction") specifies, adapted rather than reproduced verbatim — the differences
matter and are stated below rather than left implicit.

**This is not a carried protocol.** This specification does not declare `carries`, and is not a
*protocol family* for IEEE 7012 in the sense of [Carried Protocols](/SPEC.md#94-carried-protocols):
it departs from 7012 in the ways stated below, and SPEC §9.4 rule 6 forbids a specification that
carries a protocol from departing from it. An implementation that needs to *be* a MyTerms agent —
an individual proposing from a neutral roster to an entity, under 7012's own rules — needs a
separate family that carries 7012 faithfully. This family borrows 7012's handshake for a
host-governed engagement, and makes no claim of 7012 conformance.

**What follows the standard directly.** §5.3.1 gives the responding party exactly three moves:
accept the proposed terms (a), respond with a single alternative from the same structured and
bounded list (b), or reject (c). §5.2.2 adds the limit: "There shall be no negotiation beyond the
single choice by either party." This specification implements that three-way choice as the
response's `decision` — `accepted`, `countered` or `refused` — (the `respondsTo` follow-up and
its refusal codes are this specification's own mechanism, not something 7012 itself specifies:
the mechanics of enforcement are this specification's design, the *shape* of the choice is
7012's) and enforces the one-round limit in Authorization, below, as a Trust Task rule rather
than leaving each deployment to reimplement §5.2.2 for itself. §5.2.3 requires
the completed handshake to carry a unique, pseudonymous-or-explicit contract identifier and to
notify both parties: the accepted request's own `id`, carried forward as `taskContext` in the
response (above), serves as that identifier, and the response's delivery to the first party over
the same channel that carried the request is the completion notice — no separate field is added
for either.

**What this specification extends beyond 7012's own scope.** 7012 scopes this handshake to an
individual, who "shall always be the first party," and an organizational entity, who "shall
always be the second party" (§5.4.3) — it does not contemplate two individuals, two
organizations, or an agent acting for either, negotiating as peers. This specification
deliberately extends the same handshake to any two parties this framework's Trust Tasks connect
— not a claim that doing so is required, or even addressed, by 7012's own text, but a natural
reading of its underlying intent: 7012 exists to let a party proffer standard, machine-readable
terms and have another party bind to them on equal footing, the way Creative Commons lets an
individual license creative work without hiring a lawyer. That idea does not depend on one side
being a natural person and the other an organization; it holds equally between two peers of any
kind. The mapping here is: the party that opened `understanding/offer` (identified as the
*joining party* there) takes 7012's first-party/proposer role in this task, and the party that
named terms in response (the *host*) takes its second-party/responder role, regardless of what
kind of entity each actually is.

**A second departure this specification does not resolve, and states plainly instead.** 7012
also assumes the roster a first party chooses from is held by "a neutral nonprofit entity or
neutral party that is not economically dependent on terms use" (§4.2), and its own Purpose
(§1.2) is built around an individual proffering *its own* terms to an organization — not the
reverse. This specification does not require a neutral roster: `termsRef` in
`understanding/offer` may point at a roster the host itself authored, and a host is free to
publish only the terms it is willing to accept — which §5.3.2 itself requires an entity to display
publicly, but as the terms it would accept *from* a proposing individual, not as the roster that
individual chooses from. Where a host does so, the result is structurally
closer to the adhesion pattern 7012 exists to counter than to the individual-empowerment pattern
it is modeled on — the party with more leverage is the one proffering the terms the other side
chooses from, not the reverse. This specification does not resolve that tension, and does not
attempt to: mandating a neutral registrar for every peer engagement this framework might connect
is not generally enforceable, and would foreclose the peer-to-peer extension above entirely.
Where an independent party — a governance framework, a registrar neither engagement party
controls — publishes a roster instead, a host's `termsRef` **SHOULD** point there rather than at
a self-authored one; this specification does not require it, because it cannot always be
satisfied, but a deployment whose threat model includes an imbalance of leverage between the
parties should not treat this as optional.

**Requirements this specification does not reach.** §5.4.4 also calls for location to be
recorded alongside date and time — a provision suited to a standard whose interactions may run
through physically sited agents. This specification does not carry a location member: a peer
engagement between two DIDs has no location this framework's identity model represents, and a
field invented only to satisfy the clause would carry no verifiable meaning. An implementation
for which location genuinely matters **MAY** carry it in `ext` (SPEC §4.5.1). §5.4.5's identifier
floor — "at a minimum, all interactions in digital form include IP addresses" — is already
exceeded here: this framework's DID-based identity requirements are a stricter superset of that
floor, not a gap against it. §5.3.1(c) requires that a refusal not bar an individual from free or
public services; this task confers no access of its own (see Abstract, above), so the
constraint binds whatever access a host separately provides, not this task.

**Recording, restated at this registry's strength.** 7012 places the duty to record on both
sides, with "shall": the individual's recorder (§5.2.4) records the agreement and handshake terms,
date, time stamp and identifiers, and records rejected agreements with the date, time and refusing
entity; the entity (§5.3.1) signs, records and deposits a reached contract with both parties'
agents, and generates a record documenting a rejection; and §5.4.4 requires identical, immutable
copies in each party's data store. This specification keeps that symmetry — it is 7012's, not an
extension — but states the duty as a **SHOULD** (Retention, below) rather than repeat 7012's
"shall", matching this registry's general retention convention (SPEC §7.3 item 18) rather than
7012's own strength here. That is a deliberate weakening and is recorded as one.

**The comprehension gate is this specification's own, not 7012's, and a counter can move past
what it tested.** The comprehension check in
[`understanding/exchange`](../../exchange/0.1/spec.md) that this task requires as a precondition
(`exchangeId`, below) is this framework's own addition — 7012's own approach to making a choice
safe to make without testing it is to keep the roster of possible agreements small (§4.1), not
to examine whether a given party understood the one it picked. Because a counter may name
different roster items than the ones an `understanding/exchange` tested, a first party accepting
a counter is agreeing to terms it may not have been tested on; this specification does not
require a fresh exchange before a follow-up, and a deployment whose threat model requires
understanding of the *exact* terms accepted, not merely the terms originally offered, should
account for this gap rather than assume `understanding/exchange` closes it for every possible
counter.

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

The entitlement to propose a seal is having been the joining party in the `understanding/offer`
that opened this engagement, with a passed `understanding/exchange` — recorded `outcome`
`understood` — grounding the proposal. `exchangeId` **MUST** name that exchange, and the
request's `issuer` **MUST** be the party that exchange's response was addressed to — its
`recipient`. A recipient **MUST** verify both before evaluating the proposal: that the exchange
resolved with that outcome, per `understanding/seal:understandingInsufficient`, and that the
issuer matches, refusing any other issuer with `understanding/seal:issuerMismatch`. The binding
is not optional: `threadId`, `parentThreadId` and a document's `id` are correlation handles, not
secrets, so without it a third party that learned an `exchangeId` could propose against someone
else's passed exchange, and the host's acceptance — the record this family exists to produce —
would then name that party as the one that agreed. The joining party is always the first party and
proposer; the host is always the second party and responder — per Relationship to IEEE
7012-2025, above, this specification does not let the roles float between documents of the same
engagement, and every conforming request under this specification is therefore issued by the
joining party. There is no case in which the host issues a request under this specification, so
no refusal code is defined for one.

Roster items in `agreement`, and in any counter's `counterAgreement`, **MUST** be drawn from the
roster the grounding `understanding/offer` response's `termsRef` names — a party proposing or
countering with items from an unrelated vocabulary is starting a different negotiation, not
staying within the single bounded list IEEE 7012-2025 §5.3.1(b) requires. This specification
does not itself validate roster membership, since resolving and interpreting `termsRef` is out
of its scope; a conforming first party and host are each responsible for staying within it.

### One negotiation per exchange

`exchangeId` grounds **at most one** seal negotiation. A recipient **MUST** refuse, with
`understanding/seal:negotiationInProgress`, any request naming an `exchangeId` for which it has
already received a request under this specification — whether that prior request was accepted,
refused, countered, or is still awaiting a decision — **unless** the new request is the single
permitted follow-up described below. Without this rule, a per-`respondsTo` check alone leaves a
gap open: a first party could open a second, unchained request on the same `exchangeId` after
receiving a counter, and negotiate without limit by simply not naming what it is answering.

The host **MUST** decide each request one of exactly three ways, each reported as a `#response`
document per Response, below — a decision is the task's outcome, not a failure
([SPEC §8](/SPEC.md#8-error-responses), "A decision is not a failure"):

- **Accept** — `decision: "accepted"`. Available for any request, whether or not it carries
  `respondsTo`.
- **Counter** — `decision: "countered"`, carrying the alternative in `counterAgreement`.
  **MUST NOT** be used to answer a request that itself carries `respondsTo`: a countered request
  has already had its one round, and the host's only choices at that point are to accept or
  refuse.
- **Refuse** — `decision: "refused"`. Available for any request. No reason is given, and none is
  owed.

The error codes this specification declares report only that a request was not decided: it named
no live passed exchange, came from the wrong party, or broke the one-round rule below.

### Answering a counter

A first party answering a counter **MAY** issue exactly one further request, naming the
countered request in `respondsTo`. A recipient **MUST** refuse this follow-up, with
`understanding/seal:negotiationExhausted`, unless `respondsTo` names a request that satisfies
all four:

1. it grounds this same `exchangeId`;
2. it was issued by this same first party;
3. this recipient answered it with `decision: "countered"`; and
4. it has not already received a conforming follow-up.

A follow-up satisfying all four **MUST** carry an `agreement` identical, under JSON
Canonicalization ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)), to the `counterAgreement`
the recipient offered; a recipient **MUST** refuse a mismatched follow-up with
`understanding/seal:agreementMismatch` rather than treat it as a new proposal. Re-proposing
different terms is negotiation beyond the single choice §5.2.2 forbids, whichever document
carries it; a first party that wants different terms opens a fresh engagement under a new
`understanding/offer` instead of continuing this one.

Verifying either document's proof establishes who proposed, accepted, countered, or refused, and
that neither half was altered after signing; it does not establish that either party was who
they claimed for purposes beyond this exchange, which is a matter for whatever identity
assurance the parties separately agreed to rely on.

## Definitions

* **Joining party** (`joining-party`), **host** (`host`). The two parties to the agreement, as
  `understanding/offer` names them: the joining party that opened the offer, identified by
  `issuer`, proposes the agreement; the host that named terms in response, identified by
  `recipient`, accepts, refuses, or counters once.
* **First party, second party.** The same two parties in IEEE 7012-2025's own terminology, which
  this document uses when it speaks of the handshake. Fixed, not floating: the joining party is
  always the first party and proposer; the host is always the second party and responder. See
  Relationship to IEEE 7012-2025, above.
* **Exchange reference.** `exchangeId` — the `id` of the `understanding/exchange` response this
  seal is grounded in, verified by the host against its own record of that exchange (Authorization,
  above). This is a first-party check, not the external-citation binding SPEC §7.3 item 20
  exists for: the host verifying `exchangeId` is the same party that issued the cited exchange's
  response in the first place, so it already holds the record a stranger would need a digest to
  trust. A party that is *not* the host and needs to verify an `understanding/exchange` outcome
  independently has no declared `outcomeEvidence` to rely on for it — that is a known limit of
  this family, not addressed by this version.
* **Decision.** The response's `decision`: `accepted`, `countered` or `refused`. See
  Authorization, above.
* **Counter.** A response with `decision: "countered"`, carrying `counterAgreement`, the second
  party's single alternative agreement. A later request's `respondsTo` names the countered
  request, continuing the same engagement for exactly one further round under the conditions in
  Authorization above.
* **Agreement.** The terms being sealed: a name, the roster items agreed, and resolvable,
  hash-pinned pointers to the agreement's legal register and its machine-readable register,
  following the three-register pattern (plain language, legal, machine-readable) MyTerms
  negotiations use. Carried by reference; this task does not carry the plain-language text
  itself.
* **Outcome evidence binding.** `taskContext` and `taskDigestMultibase` in the response — the
  decided request's `id` and its task digest per SPEC §4.9.3, computed over the full request
  document rather than only its `agreement` member. Together they let a party outside this
  exchange — a relationship credential, a VAC — cite this response as evidence of the decision,
  per SPEC §7.3 item 20 and this specification's `outcomeEvidence` declaration, above; a party
  citing it as evidence that the agreement was reached also checks `decision` is `accepted`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/understanding/seal/0.1` with a
payload that validates against the top-level schema in
[`payload.schema.json`](payload.schema.json). Issued by the first party, addressed to the
second party, with `parentThreadId` set to the grounding `understanding/exchange` request's
`threadId`.

A request naming `respondsTo` is the single permitted follow-up to a counter — see Authorization,
above, for the conditions a recipient checks before honoring it rather than refusing with
`understanding/seal:negotiationExhausted` or `understanding/seal:agreementMismatch`. Every other
request on a given `exchangeId` after the first is refused with
`understanding/seal:negotiationInProgress`.

### The first party proposes the agreement

```json
{
  "id": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/understanding/seal/0.1",
  "issuer": "did:example:alice",
  "recipient": "did:example:host",
  "issuedAt": "2026-09-24T10:10:00Z",
  "threadId": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000001",
  "parentThreadId": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000001",
  "payload": {
    "exchangeId": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000002",
    "agreement": {
      "name": "Data retention (12 months)",
      "rosterItems": ["data-retention-12mo", "no-third-party-sharing"],
      "legalRef": {
        "uri": "https://mytermsregistry.example/terms/2026-3.2/legal",
        "version": "2026-3.2",
        "digestMultibase": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbd"
      },
      "machineReadableRef": {
        "uri": "https://mytermsregistry.example/terms/2026-3.2/machine.jsonld",
        "version": "2026-3.2",
        "digestMultibase": "zQmb9x2u1Nf1Xg2C8mZeK7v3s6D4tL5aP9qR1sT2uV3wXy4",
        "mediaType": "application/ld+json"
      },
      "effectiveFrom": "2026-09-24T10:10:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:alice#key-1",
    "created": "2026-09-24T10:10:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type:
https://trusttasks.org/spec/understanding/seal/0.1#response`, with a payload that validates
against the `$anchor: "response"` sub-schema in `payload.schema.json`. Issued by the second
party, and reports its decision — accepted, countered or refused; see Authorization, above.

The response payload is `{ decision, taskContext, taskDigestMultibase, counterAgreement? }`,
binding the decision to the exact request decided per SPEC §4.9.3. A conforming second party
**MUST** compute `taskDigestMultibase` over the full decided request, exactly as received, with
its top-level `proof` removed; a recipient of the response **MUST** recompute the digest over the
request it holds and reject the pair if they disagree. A response with `decision: "countered"`
**MUST** carry `counterAgreement`, and the schema refuses `counterAgreement` beside any other
decision.

A request that is not decided at all — no live passed exchange, the wrong issuer, a broken
one-round rule — is answered with `trust-task-error` and one of the codes in the front matter
([SPEC.md §8](/SPEC.md#8-error-responses)), never with this response.

### The second party accepts

```json
{
  "id": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/understanding/seal/0.1#response",
  "threadId": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000001",
  "parentThreadId": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000001",
  "issuer": "did:example:host",
  "recipient": "did:example:alice",
  "issuedAt": "2026-09-24T10:10:01Z",
  "payload": {
    "decision": "accepted",
    "taskContext": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000001",
    "taskDigestMultibase": "zQmT6nR3sVwXeJhK4d8f2gY1pQ9mN7bC5aE3wD6rL8sU2v"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:host#key-1",
    "created": "2026-09-24T10:10:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6ab..."
  }
}
```

### The second party counters instead

A counter is a `#response` like an acceptance, with the alternative in `counterAgreement`:

```json
{
  "id": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/understanding/seal/0.1#response",
  "threadId": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000001",
  "parentThreadId": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000001",
  "issuer": "did:example:host",
  "recipient": "did:example:alice",
  "issuedAt": "2026-09-24T10:10:01Z",
  "payload": {
    "decision": "countered",
    "taskContext": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000001",
    "taskDigestMultibase": "zQmT6nR3sVwXeJhK4d8f2gY1pQ9mN7bC5aE3wD6rL8sU2v",
    "counterAgreement": {
      "name": "Data retention (6 months)",
      "rosterItems": ["data-retention-6mo", "no-third-party-sharing"],
      "effectiveFrom": "2026-09-24T10:10:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:host#key-1",
    "created": "2026-09-24T10:10:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6cd..."
  }
}
```

A first party accepting this counter re-proposes the identical countered terms as a new request
naming the countered request in `respondsTo`:

```json
{
  "id": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/understanding/seal/0.1",
  "issuer": "did:example:alice",
  "recipient": "did:example:host",
  "issuedAt": "2026-09-24T10:10:05Z",
  "threadId": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000004",
  "parentThreadId": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000001",
  "payload": {
    "exchangeId": "urn:uuid:1a2b3c4d-0003-4000-8000-000000000002",
    "respondsTo": "urn:uuid:1a2b3c4d-0004-4000-8000-000000000001",
    "agreement": {
      "name": "Data retention (6 months)",
      "rosterItems": ["data-retention-6mo", "no-third-party-sharing"],
      "effectiveFrom": "2026-09-24T10:10:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:alice#key-1",
    "created": "2026-09-24T10:10:05Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3ef..."
  }
}
```

The host's response to this second request **MUST** be `decision: "accepted"` (`taskContext`
naming this request's `id`) or `decision: "refused"` — a conforming host does not counter a
request that carries `respondsTo`, and a first party **MUST** treat such a counter as a protocol
violation rather than as an offer it may answer.

## Security & Privacy

### Data carried

Both document kinds — the request and the response, whatever its decision — carry only
references: roster item identifiers, pointers to the agreement's legal and machine-readable
registers, and the digest binding a decision to a specific request. Neither the request nor the
response carries the plain-language text of the agreement, personal data about either party
beyond their own signing identifiers, or anything from the `understanding/exchange` this seal is
grounded in beyond its `id`.

### Correlation

A negotiation's documents are chained, not merely siblings: a follow-up names its countered
request by `respondsTo`, which is what establishes their order, not the `parentThreadId` they
also happen to share. That shared `parentThreadId` — carried by every document of the
engagement, together with the `understanding/offer` and `understanding/exchange` documents that
preceded it — is what makes the whole engagement a fully correlatable record, by design: that is
what makes a seal citable as evidence. A party that wants a *separate* agreement uncorrelated
with a prior engagement opens a fresh `understanding/offer` under a fresh `threadId`, rather than
sealing repeatedly under one.

### Retention

Both parties **SHOULD** retain this exchange for as long as either might rely on it — as the
evidentiary basis for a relationship credential or VAC, in a dispute over what was agreed, or in
an audit — since neither `agreement`'s referenced registers nor this exchange's documents are
retained anywhere else on either party's behalf by this specification. This duty is symmetric
and applies whatever the engagement's final state: a first party **SHOULD** retain a refused
decision, and a countered-but-unaccepted proposal, for the same reasons and the same duration it
would retain a sealed one. IEEE 7012-2025 states this duty on both sides as a "shall" (§5.2.4,
§5.3.1, §5.4.4); see Relationship to IEEE 7012-2025, above, for why this specification carries it
at a SHOULD rather than repeat that strength verbatim.

### Consent/purpose

The agreement reference is exchanged to record that both parties independently agreed to the
named terms, and for no other purpose. This specification does not require additional consent,
approval, or a step-up beyond the second party's own decision to accept, counter, or refuse —
that decision is the act this task exists to record.
