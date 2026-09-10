---
slug: process-attestation
version: "0.1"
title: "Process Attestation"
summary: "Evidence that an artifact was produced through a captured process, attested by the party that captured it and bound to a Verifier-supplied challenge."
status: draft
targetFrameworkVersion: "0.5.0"
category: framework
keywords: [process, attestation, provenance, authorship, evidence]
authors:
  - David L. Condrey (https://github.com/dcondrey)
knownImplementations:
  - https://github.com/LF-Decentralized-Trust-labs/proof-of-effort
  - https://writersproof.com
parties:
  - role: Verifier
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Holder
    requirement: REQUIRED
    member: recipient
    identifierScope: any
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: >-
    A response is retained by the Verifier as evidence and may be relied upon
    after the transport that carried it has closed, which is the condition under
    which SPEC §4.7.1 makes a proof mandatory. The request carries no authority
    and is protected against replay by the challenge it supplies, so requiring a
    proof on it would make the task unreachable on bindings whose integrity is
    out-of-band without buying a property the challenge does not already give.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The response is evidentiary and its anti-replay property depends on placing
    it in a bounded window alongside the challenge. A response that cannot be
    placed in time leaves a Verifier no window in which duplicate-execution
    protection is implementable.
sideEffects:
  level: none
  rationale: >-
    A read of evidence about an event that has already happened; it persists no
    state at the recipient.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    Returns derived aggregates only: a determination and its confidence, the
    artifact binding, session timeline bounds, integrity state, and any external
    timestamp anchors. Raw behavioral sample data and behavioral fingerprints are
    out of scope for this version; a profile that discloses them is disclosing
    `secret` material and is a different classification. The request carries an
    artifact identifier, which may itself reveal that a Verifier is interested in
    a specific document.
retention:
  class: durable
  rationale: >-
    A Verifier retains the response as its record of the determination it relied
    on, and the value of that record is precisely that it outlives the exchange.
errorCodes: []
related:
  - trust-ceremony-receipt
  - trust-task-error
---

## Abstract

This specification defines a Trust Task for verifying that an artifact was produced
through a captured process, with cryptographic evidence bound to the act of
production rather than to a downstream claim about the producer.

The Trust Task standardizes the request and response interaction and the Holder's
cryptographic commitment to the response. It does **not** define the receipt format,
the capture primitive, or the cryptography the receipt uses. Those remain the
concern of the underlying primitive and are verified independently under that
primitive's own specification.

## Status of this Document

Draft. Targets framework 0.5.0.

## What the Holder does and does not attest

A conforming response attests that the Holder possesses a process-attestation
receipt that the Holder associates with the identified artifact, and that the
response was produced by the Holder in reply to the supplied challenge.

It does **not** attest that:

- the receipt is cryptographically valid. A Verifier MUST obtain and verify the
  receipt under its own specification;
- the determination the receipt carries is correct;
- the artifact is unmodified since production, beyond whatever binding the
  `artifact` member's digest establishes;
- any particular person operated the process.

A valid envelope `proof` establishes that the response came from the claimed
Holder. It establishes nothing about the receipt. These are separate layers and a
Verifier MUST evaluate both before treating a determination as attested.

## Conformance

A conforming Holder:

1. MUST reply to a `#request` document with either a `#response` document or a
   `trust-task-error` document.
2. MUST echo the request's `challenge` verbatim in the response payload.
3. MUST set `result` to `unavailable` where it holds no receipt for the identified
   artifact, and MUST NOT distinguish that case from a refusal to disclose at the
   envelope level.
4. MUST NOT return a disclosure category the request did not ask for.
5. MUST NOT include `assessment`, `receiptReference` or `disclosed` when `result`
   is `unavailable`.
6. MUST attach a `proof` to the response.

A conforming Verifier:

1. MUST NOT treat a response as authoritative unless `proof.verificationMethod`
   resolves to verification material controlled by the claimed `issuer` under a
   resolution mechanism the Verifier's trust framework accepts.
2. MUST treat a response whose `challenge` does not match the one it supplied as
   unverifiable, regardless of proof validity.
3. MUST verify the referenced receipt under the receipt's own specification before
   relying on `assessment`.
4. MUST NOT infer from `unavailable` that no receipt exists.

## Authorization

This task is not *consequential* under [SPEC §2](/SPEC.md#2-terminology), so
[SPEC §7.3 item 15](/SPEC.md#73-specification-requirements) does not bind it. This
section is carried anyway because the layering below is easy to get wrong, and its
presence is not a conformance claim.

The Holder's authority to respond is possession of the receipt. There is no
registry, governing body, or third-party intermediary in the interaction, and no
authority statement is consulted. A Holder MAY delegate response handling to a
service it controls, provided the binding between the response `proof` and the
Holder's identifier is preserved.

The envelope `proof` is not the authorization. It establishes that the response is
attributable and unaltered, which is what makes the later comparison against the
receipt possible; it does not entitle the Holder to any outcome, and it says nothing
about whether the receipt is valid. Where a Verifier applies policy, accepting some
Holders and not others or requiring a particular capture primitive, that policy is
the Verifier's and is out of scope here.

## Definitions

**Artifact.** The object whose production is in question, identified by a digest.

**Receipt.** Evidence produced by a capture primitive at the time of production,
independently specified and independently verifiable.

**Capture primitive.** The system that produced the receipt. Named in the response
so a Verifier knows which specification to verify the receipt under.

**Disclosure category.** A named group of derived fields a Verifier may request.

## Request

Produced by the Verifier and sent to the Holder. Described by the top-level schema
in [`payload.schema.json`](payload.schema.json).

The Verifier identifies an artifact by digest, supplies a fresh `challenge`, and
names the disclosure categories it wants. A Holder MAY return fewer categories than
requested; it MUST NOT return more.

### Requesting a determination with timeline detail

```json
{
  "id": "9f1c3a72-5e4b-4c8a-9d21-7b6e0f3a1c58",
  "type": "https://trusttasks.org/spec/process-attestation/0.1#request",
  "issuer": "did:web:verifier.example",
  "recipient": "did:web:author.example",
  "issuedAt": "2026-09-09T14:02:11Z",
  "threadId": "1c9d4e60-8a2f-4b13-bd77-2e5a6c0918af",
  "payload": {
    "artifact": {
      "digest": "hcHFbGVGLXVQqZ8mRk2vN4pT7sYwD1jB6xK0aC3eI9M",
      "algorithm": "sha-256"
    },
    "challenge": "9tKpX2mQzR7vLbN4hJ8cWfA6dY0sE5uT3gO1iZ",
    "disclosure": ["document", "timeline"]
  }
}
```

## Response

Produced by the Holder, which is the recipient of the request. Described by the
sub-schema reachable via `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json).

The payload members:

- **`result`** is `attested` or `unavailable`. Both use the same response type, so
  the envelope carries no observable distinction between them.
- **`challenge`** is the request's challenge echoed verbatim. It is what binds this
  response to this Verifier's question.
- **`artifact`** repeats the requested digest, so the response stands alone as a
  record without the request beside it.
- **`assessment`** is present only when `result` is `attested`. It is opaque at the
  task layer: `primitive` names the specification a Verifier must verify the receipt
  under, and `determination` belongs to that primitive's vocabulary. This
  specification deliberately does not normalize determinations across primitives,
  because a determination means only what the primitive that produced it defines it
  to mean.
- **`receiptReference`** is present only when `result` is `attested`. It is a tagged
  union of `uri` and `inline`; `inline` exists for Verifiers that cannot dereference
  a URI at verification time and is bounded accordingly.
- **`disclosed`** carries the requested categories, and only those.

`unavailable` is a task-level **result**, not the [SPEC §8.3](/SPEC.md#83-standard-error-codes)
error code that shares its spelling. A result travels in a `#response` document and
means the Holder answered; the error code travels in a `trust-task-error` document
and means it did not. Do not map one onto the other.

Malformed requests, unsupported versions, and a missing required proof are answered
with `trust-task-error` carrying the standard §8.3 codes (`malformedRequest`,
`unsupportedVersion`, `proofRequired`), not with a `#response` document. This
specification defines no extended error codes of its own.

### An attested response

```json
{
  "id": "4b8e1f05-6c3d-42a9-8e71-0d5f2a7c91b3",
  "type": "https://trusttasks.org/spec/process-attestation/0.1#response",
  "issuer": "did:web:author.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-09-09T14:02:14Z",
  "threadId": "1c9d4e60-8a2f-4b13-bd77-2e5a6c0918af",
  "payload": {
    "result": "attested",
    "challenge": "9tKpX2mQzR7vLbN4hJ8cWfA6dY0sE5uT3gO1iZ",
    "artifact": {
      "digest": "hcHFbGVGLXVQqZ8mRk2vN4pT7sYwD1jB6xK0aC3eI9M",
      "algorithm": "sha-256"
    },
    "assessment": {
      "primitive": "https://lf-decentralized-trust-labs.github.io/proof-of-effort/",
      "determination": "V1_VerifiedHuman",
      "confidence": 94
    },
    "receiptReference": {
      "type": "uri",
      "uri": "https://evidence.author.example/r/4b8e1f05.cpoe",
      "mediaType": "application/cpoe+cose"
    },
    "disclosed": {
      "document": {
        "byteLength": 18432,
        "mediaType": "text/markdown"
      },
      "timeline": {
        "startedAt": "2026-09-08T09:14:02Z",
        "endedAt": "2026-09-08T11:47:35Z",
        "checkpointCount": 41
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-09T14:02:14Z",
    "verificationMethod": "did:web:author.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4oey6cnBVUCyx9YKvQZ8mR2vN4pT7sYwD1jB6xK0aC3eI9Mh"
  }
}
```

### An unavailable response

Returned where the Holder has no receipt for the artifact **and** where it declines
to say. A Verifier cannot distinguish the two, by design.

```json
{
  "id": "7d2a9c41-3f8b-4e05-a6d9-1b4c8e07f52a",
  "type": "https://trusttasks.org/spec/process-attestation/0.1#response",
  "issuer": "did:web:author.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-09-09T14:02:14Z",
  "threadId": "1c9d4e60-8a2f-4b13-bd77-2e5a6c0918af",
  "payload": {
    "result": "unavailable",
    "challenge": "9tKpX2mQzR7vLbN4hJ8cWfA6dY0sE5uT3gO1iZ",
    "artifact": {
      "digest": "hcHFbGVGLXVQqZ8mRk2vN4pT7sYwD1jB6xK0aC3eI9M",
      "algorithm": "sha-256"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-09T14:02:14Z",
    "verificationMethod": "did:web:author.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3xKp7mQzR2vLbN9hJ4cWfA6dY0sE5uT8gO1iZBVUCyx9"
  }
}
```

### The failure path

A request the Holder cannot parse is answered with `trust-task-error`, paired to the
request by `threadId`. Here the `challenge` fell below the 16-character floor, so the
request never reached task semantics and no `#response` document is produced.

```json
{
  "id": "c05f9a3b-71d4-4e28-9a6f-3e8b0d52147c",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:author.example",
  "recipient": "did:web:verifier.example",
  "issuedAt": "2026-09-09T14:02:12Z",
  "threadId": "1c9d4e60-8a2f-4b13-bd77-2e5a6c0918af",
  "payload": {
    "code": "malformedRequest",
    "message": "challenge is shorter than the 16-character minimum",
    "retryable": true
  }
}
```

`retryable` is `true` because the Verifier can reissue with a conforming challenge.
A Verifier MUST NOT read this as `unavailable`: an error means the Holder did not
answer the question, not that it has no receipt.

## Security & Privacy

### Data carried

The request carries an artifact digest, a challenge, and a list of category names.
The response carries a determination, a receipt reference, and the derived
aggregates for the requested categories.

Disclosure categories in this version return derived values only: `document` (the
artifact binding), `timeline` (session bounds and checkpoint count), `integrity`
(chain and signature state), and `anchors` (external timestamp anchors). The
determination is not a category, because `assessment` already carries it on every
attested response; making it one would put the same value in two places and invite
them to disagree.

Raw behavioral sample data, behavioral fingerprints, and per-region edit topology
are **out of scope for this version**. A capture primitive that holds such data MUST
NOT return it under these categories. Disclosing it is disclosing `secret` material
under [SPEC §7.3 item 14](/SPEC.md#73-specification-requirements), which makes the
task consequential and changes what a consumer is obliged to do; a profile that
needs it is a different specification with a different classification, not a
category added to this one.

### Correlation

A stable Holder signing key correlates every response that key signs. Where a Holder
responds to many Verifiers about many artifacts, that is a linkage surface across
all of them. This version does not define an unlinkability mechanism; a profile that
needs one defines the key strategy and verification semantics without altering the
task semantics here.

This specification is **not** `bearer`. `trust-ceremony-receipt` is, because a
ceremony receipt is a public record of a completed flow that any party verifying the
proof may legitimately rely on. A process-attestation response is scoped to the
challenge one Verifier supplied, and accepting it audience-free would expose it to
cross-recipient replay under [SPEC §10.1](/SPEC.md#101-replay): a response obtained
by one Verifier could be presented to another as though freshly obtained.

The artifact digest in a request may itself be sensitive. Where the existence of a
receipt for a given artifact is confidential, a Holder SHOULD require an
authenticated request as local policy, and the `unavailable` conflation above is
what keeps refusal indistinguishable from absence.

### Retention

A Verifier retains the response as its record of the determination. It is `durable`
by nature; a Verifier that does not need a durable record SHOULD discard it once its
decision is made, because a retained response is a retained correlation.

### Consent/purpose

This specification states what authority the task assumes and does not oblige any
consumer to authorize anything. Whether a Holder responds at all, and to whom, is
Holder policy under its own governance framework.
