---
slug: vetting/attestation
version: "0.1"
title: "Vetting — Attestation"
summary: A vetter gives an applicant an attestation that carries the facts of a vetting session and names nobody, for the applicant to prove over when it applies to the community.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - vetting
  - hidden-vetting
  - attestation
  - zero-knowledge
  - unlinkability
parties:
  - role: vetter
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: applicant
    requirement: REQUIRED
    member: recipient
    identifierScope: any
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The applicant must know the attestation came from the vetter it sat with — it verifies it on
    arrival rather than discovering at submit that it holds something unusable. The envelope proof
    is what attributes the delivery; the attestation inside it deliberately attributes nothing, and
    is the only thing that reaches the community.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The applicant checks the attestation against the session it just held. A document that cannot be placed in time cannot be told from one replayed from an earlier session with the same vetter.
sideEffects:
  level: mutating
  rationale: The applicant records the attestation against its application, and the vetter has spent a token producing it.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The bound facts say what the vetter checked about the applicant — method, claim *types*,
    liveness, the card's commitment and digest — and never a claim value: the values were shown in
    the session and stayed there. It is an assertion about the applicant, delivered to the
    applicant. Nothing in it is personal data about the vetter, which is the property the whole
    exchange exists to preserve.
retention:
  class: durable
  rationale: The applicant holds the attestation until it submits, and afterwards for as long as it may need to supplement. It is evidence, and it is the applicant's own.
errorCodes:
  - code: vetting/attestation:doesNotVerify
    meaning: "The attestation does not verify under the community's published parameters, or is not bound to the card the applicant showed."
    retryable: false
  - code: vetting/attestation:unknownSuite
    meaning: "`suite` is not one this applicant implements. It cannot be mixed with attestations under another suite."
    retryable: false
  - code: vetting/attestation:notForThisApplication
    meaning: "`meta.community` or `meta.requirementsDigest` is not the criterion this applicant is gathering for."
    retryable: false
related:
  - vetting/session
  - vetting/request
  - vetting/decline
  - vtc/join-requests/submit
---

## Abstract

On the named vetting path a vetter signs a statement, and its signature is its name. This task is
the same moment on the hidden path: the same facts, established the same way in the same session,
delivered without an issuer.

What makes it trustworthy is not a signature on the attestation — there is none — but what the
applicant can later prove over it: that a pairwise-distinct holder of a live class credential in
this community produced it, and spent a token to do so. The community counts that proof with the
rule it already uses for named statements.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

Any party may send an applicant an attestation; nothing is authorized by this document arriving.
The applicant's entitlement to *count* it comes from the attestation itself verifying under the
community's published parameters, and the community's from the proof the applicant later submits.

This is worth stating plainly because the usual reading is inverted here: per
[SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the envelope's `issuer`
establishes who delivered the document — and on this path that is deliberately **not** what makes
the attestation count. A consumer MUST NOT treat the delivering `issuer` as the attesting party,
and MUST NOT record it against the attestation: doing so recreates, in the applicant's own store,
exactly the link the exchange removes.

## Definitions

**`suite`** — the hidden-vetting suite the attestation was made under, exactly as the community's
criterion publishes it. An applicant MUST refuse a suite it does not implement rather than hold an
attestation it cannot prove over, and MUST NOT mix suites in one proof.

**`attestation`** — the attestation itself: a tag, a shown class credential and a proof. The tag is
deterministic for this (vetter, applicant) pair — which is what makes two attestations from the
same vetter count once — and unlinkable across applicants.

**`meta`** — the facts the community will count, bound into the attestation: the community and
requirements digest it was made for, the method, the claim types checked, liveness, the declared
relationship, the applicant's identity commitment and the digest of the card it was shown on, and
the validity window. `validFrom` and `validUntil` are **dates, never timestamps**: an exact time
would let a community line an attestation up with a vetter's activity, which undoes the proof
without touching it.

**`token`** — the attestation token spent. Its serial is revealed to the community at submit and
recorded there; a second appearance of the same serial is a double spend.

## Request

The vetter issues the request to the applicant, on the thread of the session they just held; the
payload is the top-level object in [`payload.schema.json`](payload.schema.json).

### An attestation after an in-person session

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vetting/attestation/0.1#request",
  "issuer": "did:example:vetter",
  "recipient": "did:example:applicant",
  "issuedAt": "2026-09-20T14:31:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "suite": "ps-ddh-bls12381",
    "attestation": "zBbsywyzdGiuJ3aCc147SQheEgDJFJHkgNKctAcZPqHkJqPcYTwKcWQV6wYqPnB1e9N",
    "meta": {
      "community": "did:example:community",
      "requirementsDigest": "zQmdmF2WvyXF9dgZeb2W2VxxwYeSzqN3fBykoLTkru5QUYE",
      "method": "inPerson",
      "claimsVerified": ["name.legal"],
      "livenessConfirmed": true,
      "declaredRelationship": "none",
      "identityCommitment": "zQmPRccGTPKwUUB8nUdSGuK7oN8Ht7c6iEDsLqdK7LNsxKz",
      "cardDigestMultibase": "zQmRnZq46vDQnAzSuEnWz3zvfQoigbe23KtSp6S1Vm1UXuL",
      "validFrom": "2026-09-20",
      "validUntil": "2027-01-18",
      "tokenLabel": "token/2026-09",
      "tokenSerial": "zBKHjA2kQWBpQPgwiHBG1DrzCykjsvdMfnKBYrhiM4STW"
    },
    "token": {
      "label": "token/2026-09",
      "serial": "zBKHjA2kQWBpQPgwiHBG1DrzCykjsvdMfnKBYrhiM4STW",
      "shown": "z26q5oFrp6i2aTKLp6Y6jsLESJMoNfZQwk8VKXc37c24bJPj5fwBoUEsqv51ADbnUNM"
    }
  }
}
```

## Response

The applicant answers with a receipt, in the sub-schema reachable via `$anchor: "response"`. It
verifies the attestation **on arrival** — against the community's published parameters and the card
it showed in the session — so a vetter learns at once that what it sent is usable, rather than the
applicant discovering at submit that it is not. A refusal is a `trust-task-error` with one of the
codes above.

### The applicant confirms it verified

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vetting/attestation/0.1#response",
  "issuer": "did:example:applicant",
  "recipient": "did:example:vetter",
  "issuedAt": "2026-09-20T14:31:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "accepted": true,
    "heldCount": 2
  }
}
```

## Security & Privacy

### Data carried

The facts of one vetting session about one applicant, delivered to that applicant. A producer MUST
NOT put its own identifier, a display name, a session transcript or anything else identifying the
vetter in `ext` — not because the applicant does not already know who vetted them (it does), but
because whatever travels here travels on toward the community in the applicant's store.

### Correlation

The tag inside the attestation is deterministic for this (vetter, applicant) pair: the same vetter
attesting the same applicant twice is counted once, and the same vetter attesting somebody else
produces a value that cannot be linked to this one. The applicant, and only the applicant, knows
both ends. An applicant SHOULD store the attestation without the delivering `issuer` alongside it
once it has verified it, so that a copy of its own store is not the link the exchange removed.

Neither party is required to use a reusable identifier, and this task declares
`identifierScope: any` to say so. All the applicant needs is that the delivery arrives under the
same identifier the session was held under, whatever its scope; nothing here has to be recognised
by a third party, and the community — the one party that would make a reusable identifier
load-bearing — never sees this document at all. A pair that runs the whole vetting exchange under
pairwise identifiers loses nothing.

### Retention

The applicant keeps it until it has submitted and any supplement window has passed. The vetter has
no reason to keep a copy: it spent the token, and a second copy of the attestation is a second
chance for somebody else to read it.

### Consent/purpose

The facts are collected for one community's admission decision, named in `meta.community` and
pinned by `meta.requirementsDigest`. An attestation is not portable to another community, and a
consumer MUST refuse one whose `meta` names a criterion it is not applying for.
