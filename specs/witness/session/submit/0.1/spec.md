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
    identifierScope: pairwise
  - role: witness
    requirement: REQUIRED
    member: recipient
    identifierScope: public
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
  ingests: secret
  actsAsSubject: true
  rationale: "The request presents the holder's own credential material under the session challenge; the response returns credential material the holder retains. On the inbound side `vp` is typed only as an object — the presentation is opaque to this schema — so every claim the party's wallet chose to disclose arrives at the witness in full and in the clear, and the witness must read all of it in order to verify it. That is confidential material the witness holds on the producer's behalf, which is why `ingests` is `secret` rather than `personal`."
retention:
  class: durable
  rationale: The `#response` is the session's terminal success document and the outcome evidence a Verifiable Witness Credential's `taskContext` points at; a holder retains it, together with the `witness/session` document that opened the session, for the useful life of the VWC, because a verifier that lacks it cannot establish that the credential belongs to a session that actually completed. The durable obligation attaches to the response, not to the inbound `vp` — see Security & Privacy → Retention for why a witness that keeps presentations is holding credential material with no evidentiary role.
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
   [SPEC.md §4.9.1](/SPEC.md#491-naming-an-exchange-from-outside-the-framework) —
   **and** the VWC's `taskDigestMultibase` **MUST** be the *task digest* of
   that same document, computed per
   [SPEC.md §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names).
   The `id` locates the session document; the digest binds it. An `id` alone
   proves nothing about *which* document opened the session, because anyone
   may write a different one bearing the same `id`.

   `taskDigestMultibase` is a member of the credential, whose schema belongs
   to DTG Core Credentials; this specification states only the obligation
   that the value pair with the session document, and §4.9.3 states how the
   value is computed. Nothing about the session document's own wire shape
   changes.
2. A holder later presenting that VWC as proof the witnessing occurred
   **MUST** retain this `#response` **and the `witness/session` document that
   opened the session**, and ship both with the presentation — the digest
   check is not performable without the document it is taken over. A
   verifier pairing them checks: the session document's `id` equals the VWC's
   `taskContext` **and the VWC's `taskDigestMultibase` reproduces over that
   document** under §4.9.3, comparing decoded multihash bytes rather than
   encoded strings; the evidence's `threadId` equals the
   VWC's `taskContext`; the evidence's `type` is this specification's
   `#response`; the evidence's own REQUIRED proof verifies; the evidence's
   `issuer` is the witness that issued the VWC; and the presented credential's
   digest equals the evidence's `vwcDigestMultibase`.
3. A `trust-task-error` terminating this exchange is diagnostic for the
   parties; it is **not** verifier-facing outcome evidence, and no credential
   may cite an exchange that terminated in one as completed.

### What the digest check settles, and what carries the rest

*This subsection is non-normative.*

`witness/session` declares `proofRequirement.request: OPTIONAL`, so the session
document a holder ships may carry no `proof` of its own — and per
[§4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)
the task digest is taken over the document with any top-level `proof` removed,
so a signed and an unsigned copy of the same session document reproduce the
same value. Neither fact weakens the pairing, and it is worth being exact about
why.

The digest is not evidence the *holder* produces; it is a value the **witness
signed into the VWC**. A witness computes it over the session document it
actually received, and a counterfeit that borrowed the `id` cannot reproduce
that value without reproducing the content — which is the attack this rule
exists to stop. What the holder ships is only the input a verifier needs in
order to recompute; a substituted input fails, and a `proof`-stripped copy of
the genuine document is still the genuine document's content.

Attribution of the exchange therefore rests where this specification already
puts it: on the `#response`, whose `proof` is REQUIRED precisely because it is
retained and relied upon by parties who were never in the exchange, and whose
`issuer` a verifier checks against the VWC's issuer. The task digest answers
*which session document*; the response's proof answers *that this witness
conducted it*. A verifier needs both, and neither substitutes for the other.

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

1. Applies the [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) pipeline.
2. Verifies the presentation's binding to the session challenge **before** any other check; a wrong binding is `witness/session/submit:challengeMismatch`, distinct from `presentationInvalid`, because the operator responses differ — a replay versus a defect.
3. Verifies that the submitting party is the one that opened the session; a submission on a session opened by another party is `witness/session/submit:challengeMismatch`, since it can only have been made with a challenge that was not issued to it.
4. On success, **MUST** return the `#response` delivering the VWC and its `vwcDigestMultibase`, under the REQUIRED proof, with the `taskContext` rule of this document honoured.

## Authorization

*Declared under [SPEC.md §7.3](/SPEC.md#73-specification-requirements) item 15.*

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
[SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 10,
verifying it establishes *who submitted*, not that they may. The inner
presentation's holder binding is likewise evidence about the credential
material, not about entitlement to this session.

Whether to mint a Verifiable Witness Credential for a submission satisfying
both checks remains the **witness's own decision** under its policy. This
declaration describes the evidence the task assumes; it obliges no witness to
attest anything.

## Security & Privacy

### Data carried

The request has exactly one substantive member, and it is the widest one in the
family. `vp` is typed as a bare `object` — "opaque here", as the schema puts it —
so nothing in this specification narrows what a presentation may contain. Whatever
the party's wallet chose to disclose arrives at the witness whole, and the witness
must read all of it, because verifying the presentation's own proof, its holder
binding, and its binding to `{challenge, domain}` is the task. This is the point
at which a witness stops seeing identifiers and starts seeing claims.

It follows that data minimisation here is not a property of the document — there is
nothing in the payload to trim — but of the presentation the party constructs.
A producer **SHOULD** present the narrowest derivation its credential format
supports, and **SHOULD NOT** ship a full credential where a selective disclosure or
a derived proof would satisfy the witness's checks. A witness needs to establish
that the submitting party holds the relationship credential material the session
concerns; it does not need every attribute that credential happens to carry, and
this specification cannot tell the difference on the party's behalf.

The response carries `vwc` — a signed Verifiable Witness Credential, likewise
opaque to this schema — and `vwcDigestMultibase` over its RFC 8785
canonicalization. The credential is a statement *about the party*, and unlike the
presentation it is meant to travel: the party will show it to verifiers who were
never in the exchange.

### Correlation

This is the strongest correlation event in the witnessing family, and it is worth
being blunt about why. Up to this point the parties are protected by pairwise
relationship DIDs that join to nothing outside the exchange. The presentation
breaks that containment *at the witness*: a VP carries credential-level
identifiers — the issuer of each credential, a credential `id`, a subject
identifier, a revocation or status-list index — and the witness now holds all of
them alongside the pairwise DID from the session and the counterparty's pairwise
DID from the session's `parties` pair. A status-list index in particular is a
stable handle that other verifiers see too. The witness is therefore the one party
positioned to link a party's pairwise relationship identity to the credential
identity it presents everywhere else, and nothing in the protocol prevents it.

The Verifiable Witness Credential is a correlator in its own right, deliberately.
Its `taskContext` names this session's opening document and its
`taskDigestMultibase` binds to that document's content, so any verifier shown two
VWCs can tell whether they came from the same session — which is what makes the
credential meaningful and also what makes it linkable. A party presenting the same
VWC to several verifiers gives them a shared, exact join key.

The witness declares `identifierScope: public` and the participating party
`pairwise`. The witness's public scope is a genuine narrowing of its own privacy
and it buys a specific property: the verifier's pairing rule above requires
checking that the `#response`'s `issuer` is the same party as the VWC's issuer, and
that both parties' sessions ran with the same witness. A pairwise witness
identifier would make those the same string only by accident, and a verifier would
have no way to establish that one witness attested both halves of an exchange. The
participating party stays pairwise because its relationship DID is minted for this
exchange and nothing about the pairing rule needs it to be recognisable elsewhere.

### Retention

**The `#response` is durable and the holder owns that obligation.** Evidence
discovery and retrieval are out of scope here as in the depending credential
specification — evidence the holder does not ship is evidence that does not exist.
A holder retains this response *and* the `witness/session` document that opened the
session for the useful life of the VWC, because the digest check is not performable
without the document the digest was taken over.

**The wall between evidence and credential survives that retention.** The VWC
verifies as a credential on its own issuer proof, indefinitely. What *this
response* proves is that the session reached its terminal success state. Neither
substitutes for the other: a valid VWC with no paired evidence proves a witness once
signed something, not that this session completed.

**The inbound presentation is the one thing here that should not be kept.** Once
the witness has verified `vp` and minted the credential, the presentation has
discharged its function. It has no evidentiary role afterwards: the VWC is bound to
the *session document*, by `taskContext` and task digest, and to the *credential*,
by `vwcDigestMultibase` — never to the presentation. A witness that retains
presentations is therefore accumulating other parties' credential material that
proves nothing it could not prove without it, which is the worst possible ratio. A
witness **SHOULD** discard `vp` once the response is emitted, retaining only what
its own revocation decision needs — revocation of a VWC is the witness's own act
per this specification's `sideEffects` rationale, and that requires a record of
what it attested, not a copy of what it was shown.

### Consent/purpose

The party discloses its credential material for one purpose: to let a witness it
selected establish that the party submitting is the party that opened the session
and holds the relationship credential material the session concerns, so that the
witness can attest that participation. The challenge binding is the record that the
disclosure was made into *this* session and no other, which is why a presentation
bound elsewhere is `challengeMismatch` rather than an acceptable submission.

Being witnessed is not the same as being verified, and the limit follows from that.
Material presented so a witness can attest an exchange is not material presented so
the witness can evaluate the party's claims for its own purposes, enrich a profile,
or re-present them onward. The witness is an observer of an exchange, not a relying
party to the credentials it observes. This specification provides no member through
which a party can attach that limit to the presentation and no mechanism by which
it could detect a breach — so, as with the session it belongs to, the operative
control is which witness the parties chose and what retention policy that witness
publishes. Whether to mint a credential for a submission that passes both checks
remains the witness's own decision under its policy.
