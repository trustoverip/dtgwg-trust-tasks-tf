---
slug: vta/pcs/prove
version: "0.1"
title: "VTA PCS — Prove"
summary: "A client asks its VTA for the zero-knowledge proof a hidden-vetting join submission carries, bound to the community's single-use challenge."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - hidden-vetting
  - vetting
  - zero-knowledge
  - vta
parties:
  - role: Client driving the PCS engine
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: PCS engine (the VTA)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request makes the VTA act with, or change, a PCS key its principal's standing in the
    community rests on. Where the transport does not already authenticate the caller, the proof
    is the only thing between that key and anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor: a duplicate
    can only be absorbed inside a bounded window.
sideEffects:
  level: none
  rationale: >-
    The engine reads the application and builds a proof. Nothing it holds changes; the challenge
    the proof is bound to is spent at the community, not here.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA exercises the application's PCS key to prove, on the applicant's behalf, that
    distinct eligible vetters attested for it. Out: the proof, with the metadata and tokens of
    the attestations it carries — what the community counts. In: the community's challenge and
    requirements digest.
retention:
  class: transient
  rationale: >-
    The proof is bound to a single-use challenge and is useless after the submission it was
    built for.
errorCodes:
  - code: vta/pcs/prove:notFound
    meaning: "No application has this handle in a context the caller may act in."
    retryable: false
  - code: vta/pcs/prove:unsupportedSuite
    meaning: "The parameters name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/prove:parametersInvalid
    meaning: "The community parameters cannot be decoded."
    retryable: false
  - code: vta/pcs/prove:noUsableAttestation
    meaning: "No attestation the application holds is usable under the parameters as they stand: each was made under a class label or token label that is no longer live, or none is held."
    retryable: false
related:
  - vta/pcs/applications/create
  - vta/pcs/verify-attestation
  - vtc/vetting/pcs-challenge
---

## Abstract

The **VTA PCS — Prove** Trust Task builds the proof an applicant submits to a community that runs hidden vetting: that distinct eligible vetters of the community attested for it, without naming them. The VTA builds it from the application's key and the attestations it holds, bound to the community's single-use challenge ([`vtc/vetting/pcs-challenge`](../../../../vtc/vetting/pcs-challenge/0.1/spec.md)), to the requirements digest the applicant is applying under, and to the DID it joins as. The client places the result in its join submission's `extensions` under `hiddenVetting`. The family's shared rules are in [`CONVENTIONS.md`](../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Obtain `challenge` from the community through `vtc/vetting/pcs-challenge` immediately before asking: it is single-use and expires.
2. Submit the returned `hiddenVetting` unchanged, as the `hiddenVetting` member of the join submission's `extensions`.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Leave out of the proof every held attestation the parameters make unusable — one under a class label not in `vetterLabels`, or spending a token under a label not in `tokenLabels` — because one stale attestation fails the whole proof at the community. Refuse with `vta/pcs/prove:noUsableAttestation` when none is left.
4. Bind the proof to `challenge`, to `requirementsDigest`, to the community and to the application's join DID, and return it; never the key.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsApplicant`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`applicationId`** — the application to prove for.
- **`parameters`** — the community's published parameters, as they stand now.
- **`requirementsDigest`** — the digest of the criterion applied under.
- **`challenge`** — the challenge the community issued for this submission.
- **`hiddenVetting`** (response) — the proof and the statements it carries (`HiddenVettingSubmission`), ready for the submission's `extensions`.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000007870271",
  "type": "https://trusttasks.org/spec/vta/pcs/prove/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000007870279",
  "payload": {
    "applicationId": "app-7f3c2a",
    "parameters": {
      "suite": "ps-ddh-bls12381",
      "helperKey": "z6MkhVerificationKey1abcXYZ",
      "tokenKey": "z6MktVerificationKey1defXYZ",
      "vetterLabels": [
        "vetter/2026-10",
        "vetter/2026-09"
      ],
      "tokenLabels": [
        "token/2026-10"
      ],
      "dripPerTick": 3,
      "tickLength": "P3D",
      "events": []
    },
    "requirementsDigest": "zbJGeTtipUG586UbekG3W4rqiWuqs7DPL",
    "challenge": "zxkTnTcyyDriAinL5aWFMKsYgcGf4qwCp"
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000007870272",
  "type": "https://trusttasks.org/spec/vta/pcs/prove/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000007870279",
  "payload": {
    "hiddenVetting": {
      "suite": "ps-ddh-bls12381",
      "id": "zkChBGjt2GvefLiGF1uuJCsv4eFaSFRuF",
      "joinDid": "did:key:z6MkpJoinPersonaExampleKey22222222222222222222",
      "idBinding": "4f9b2c7e1a0d3e8f6b5c4a39281706f5e4d3c2b1a09f8e7d6c5b4a3928170615",
      "challenge": "zxkTnTcyyDriAinL5aWFMKsYgcGf4qwCp",
      "statements": [
        {
          "meta": {
            "community": "did:webvh:QmCommunity:vtc.example.org",
            "requirementsDigest": "zbJGeTtipUG586UbekG3W4rqiWuqs7DPL",
            "method": "video",
            "claimsVerified": [
              "name.legal"
            ],
            "livenessConfirmed": true,
            "declaredRelationship": "none",
            "identityCommitment": "zGM7YYdXbJfRjqVT55f65JrrEvwRGccND",
            "cardDigestMultibase": "zQFrySRbScLd9ggxzQ31yHzA3gRqJThA5",
            "validFrom": "2026-10-12",
            "validUntil": "2027-01-10",
            "tokenLabel": "token/2026-10",
            "tokenSerial": "zHfL5AtsgzkyoYFY7sjQwK8Fn7ZkHWjjw"
          },
          "token": {
            "label": "token/2026-10",
            "serial": "zHfL5AtsgzkyoYFY7sjQwK8Fn7ZkHWjjw",
            "shown": "zRoYgpRvsnJhvPtZ3ZioCyLuJ2ae4Ug5z"
          }
        }
      ],
      "proof": "z9SBEJP7Eb4degZkKxwoGjnKPVaL3zVr9"
    }
  }
}
```

## Security & Privacy

### Data carried

The proof carries the application's PCS identifier, the join DID, the challenge, and for each attestation its metadata and spent token. It carries no vetter identifier and no key. The community counts the statements and records the token serials.

### Correlation

Everything in the proof is something the community needs and will see. Within the proof, the statements are unlinkable to the vetters who made them; across applications, the identifier changes.

### Retention

Nothing is retained by the VTA beyond its audit record of the operation.

### Consent/purpose

To admit this applicant to this community, once. The proof is bound to a single challenge and cannot be replayed to another submission. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
