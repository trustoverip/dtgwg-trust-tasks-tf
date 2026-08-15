---
slug: witness/session/submit
version: "0.1"
title: Witness — Session Submit
summary: A participating party submits its presentation, bound to its session challenge; the witness's mandatory response delivers the Verifiable Witness Credential and its digest — the outcome evidence a VWC presentation must ship, and the terminal document a taskContext points at.
status: draft
targetFrameworkVersion: "0.4"
category: credentials
keywords:
  - witness
  - session
  - vwc
  - presentation
  - outcome-evidence
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
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    On the request, because the submitting party acts as the subject of its
    own presentation — the exposure class this registry floors at REQUIRED —
    and the inner presentation's holder binding does not attribute the outer
    document on a relayed path. On the response, because this is the terminal
    document a Verifiable Witness Credential's taskContext points at: it is
    retained by the holder and evaluated by verifiers who were never party to
    the exchange, which is SPEC.md §4.7.1's retained-and-relied-upon
    condition in its purest form. An unproofed response here would make the
    VWC's completion evidence unattributable — the exact defect the
    qualifying profile exists to exclude.
sideEffects:
  level: mutating
  rationale: "Successful execution mints a Verifiable Witness Credential. Not compensatable by this exchange; revocation is the witness's own act."
exposure:
  discloses: secret
  actsAsSubject: true
  rationale: "The request presents the holder's own credential material under the session challenge; the response returns credential material the holder retains."
errorCodes:
  - code: witness/session/submit:challengeMismatch
    meaning: The presentation is not bound to this session's challenge and domain — a replay from another session, or a stale binding.
    retryable: false
  - code: witness/session/submit:presentationInvalid
    meaning: The presentation failed verification — its own proof, its holder binding, or its party bindings against the session.
    retryable: false
related:
  - witness/session
  - vrc/relationships/issue
---

## Abstract

Within an open witness session
([`witness/session`](../../0.1/spec.md)), the participating party that opened
it submits its **presentation**, bound to that session's `{challenge,
domain}`. The witness verifies it and — in the **mandatory** `#response` —
delivers the **Verifiable Witness Credential** attesting the witnessed
exchange.

## This response is the outcome evidence

The `#response` of this specification is the session's **terminal success
document**, and it is what a `taskContext`-bearing credential's evidence
obligation refers to:

1. The delivered VWC's `taskContext` **MUST** equal the `id` of the
   `witness/session` document that opened **this** session — the innermost
   exchange that attests the witnessing, per
   [SPEC.md §4.9.1](../../../../../SPEC.md#491-naming-an-exchange-from-outside-the-framework).
2. A holder later presenting that VWC as proof the witnessing occurred
   **MUST** retain this `#response` and ship it with the presentation. A
   verifier pairing the two checks: the evidence's `threadId` equals the
   VWC's `taskContext`; the evidence's `type` is this specification's
   `#response`; the evidence's own REQUIRED proof verifies; the evidence's
   `issuer` is the witness that issued the VWC; and the presented credential's
   digest equals the evidence's `vwcDigestMultibase`.
3. A `trust-task-error` terminating this exchange is diagnostic for the
   parties; it is **not** verifier-facing outcome evidence, and no credential
   may cite an exchange that terminated in one as completed.

Documents of this exchange carry `parentThreadId` per the rule
[`witness/session`](../../0.1/spec.md) states; it applies here unchanged.

A witnessed relationship exchange produces **two** of these responses — one
per participating party, on that party's own session. Each attests its own
party's participation and nothing about the other's. A verifier asking whether
*the exchange* was witnessed needs the evidence belonging to the party whose
claim it is evaluating; it **MUST NOT** infer one party's participation from
the other's evidence.

### Why the response carries the credential

Recorded because the alternative is reasonable and was weighed. `submit#response`
could carry a **reference and digest** instead of the credential, leaving
delivery to a separate credential-exchange flow — smaller retained evidence,
and a clean split between "credential delivery" and "completion evidence".

It carries the credential because the response is **mandatory** and a
second flow would not be: a witness that has verified the presentation has
everything it needs to issue, and requiring a further exchange adds a protocol
dependency and a failure mode — a session that succeeded but whose credential
never arrived — for a saving that matters only at scale this specification does
not yet have. The digest is carried alongside regardless, so the binding a
reference-and-digest design would give is available here too, and a later
version can drop `vwc` to a reference without changing how a verifier pairs
the two.

## Conformance

A conforming **participating party** (`issuer`):

1. Emits a document whose `type` is `https://trusttasks.org/spec/witness/session/submit/0.1`, on the session's thread (`threadId` = the `witness/session` document's `id`), and **only** on a session it opened itself.
2. Carries in `vp` a presentation of its own relationship credential material, bound to the session's `challenge` and `domain`, under the REQUIRED envelope proof.

A conforming **witness** (`recipient`):

1. Applies the [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) pipeline.
2. Verifies the presentation's binding to the session challenge **before** any other check; a wrong binding is `witness/session/submit:challengeMismatch`, distinct from `presentationInvalid`, because the operator responses differ — a replay versus a defect.
3. Verifies that the submitting party is the one that opened the session; a submission on a session opened by another party is `witness/session/submit:challengeMismatch`, since it can only have been made with a challenge that was not issued to it.
4. On success, **MUST** return the `#response` delivering the VWC and its `vwcDigestMultibase`, under the REQUIRED proof, with the `taskContext` rule of this document honoured.

## Authorization

*Declared under [SPEC.md §7.3](../../../../../SPEC.md#73-specification-requirements) item 15.*

Two pieces of evidence, both verified by the witness under Conformance above:
the submitting party is **the party that opened this session** (item 3), and
the presentation is **bound to the challenge issued into that session**
(item 2).

The challenge is the operative evidence. It is single-use and unpredictable,
and it was issued to exactly one party, in one `#response`, on a session that
party opened; a submission that cannot produce it is not entitled to be
witnessed under that session. This is why a submission on a session opened by
another party is reported as `challengeMismatch` rather than
`presentationInvalid` — the presentation may be perfectly well-formed, and the
defect is one of entitlement, not of construction.

The envelope `proof` is REQUIRED here and it is **not** the authorization: it
attributes the outer document to its sender so that a presentation cannot be
relayed anonymously on a party's behalf. Per
[SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) item 10,
verifying it establishes *who submitted*, not that they may. The inner
presentation's holder binding is likewise evidence about the credential
material, not about entitlement to this session.

Whether to mint a Verifiable Witness Credential for a submission satisfying
both checks remains the **witness's own decision** under its policy. This
declaration describes the evidence the task assumes; it obliges no witness to
attest anything.

## Security & Privacy

**The wall between evidence and credential.** The VWC verifies as a
credential on its own issuer proof, indefinitely. What *this response*
proves is that the session reached its terminal success state. Neither
substitutes for the other: a valid VWC with no paired evidence proves a
witness once signed something, not that this session completed.

**Retention is the holder's obligation.** Evidence discovery and retrieval
are out of scope here as in the depending credential specification —
evidence the holder does not ship is evidence that does not exist. Holders
retain this response for the useful life of the VWC.

**The presentation discloses the holder's credential material to the
witness.** That is the point of witnessing, and the reason `exposure`
declares `discloses: secret` and `actsAsSubject: true`. Witness selection
and witness retention policy carry the privacy weight here.
