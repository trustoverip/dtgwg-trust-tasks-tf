---
slug: vta/pcs/attest
version: "0.1"
title: "VTA PCS — Attest"
summary: "A vetter's client asks its VTA to attest, after the human check, that an applicant was vetted: the VTA spends a token and makes an attestation that names no vetter."
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
  level: mutating
  rationale: >-
    Spends one of the vetter's attestation tokens, which is not recoverable, and records the
    attestation in the engine's log so it can be refreshed after a rotation.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA exercises the vetter's PCS key and class credential to vouch, as an eligible vetter
    of the community, that the applicant was checked. Out: the attestation the applicant
    receives. In: the applicant's PCS identifier and what the vetter checked.
retention:
  class: durable
  rationale: >-
    The attestation is the vetter's word that a person was checked; the engine keeps a log of it
    so it can be refreshed.
errorCodes:
  - code: vta/pcs/attest:notFound
    meaning: "No vetter engine exists for this context and community."
    retryable: false
  - code: vta/pcs/attest:notEnrolled
    meaning: "The vetter holds no class credential under a label the parameters name as live."
    retryable: false
  - code: vta/pcs/attest:noToken
    meaning: "The vetter holds no free token under a live label."
    retryable: false
  - code: vta/pcs/attest:applicantInvalid
    meaning: "`applicantId` is not a valid PCS identifier."
    retryable: false
  - code: vta/pcs/attest:communityMismatch
    meaning: "`meta.community` is not the engine's community."
    retryable: false
  - code: vta/pcs/attest:unsupportedSuite
    meaning: "The parameters name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/attest:parametersInvalid
    meaning: "The community parameters cannot be decoded."
    retryable: false
related:
  - vetting/attestation
  - vta/pcs/verify-attestation
  - vta/pcs/tokens/request
  - vetting/request
---

## Abstract

The **VTA PCS — Attest** Trust Task is a vetter's statement, under hidden vetting, that it checked an applicant. After the human check, the client passes the applicant's PCS identifier and what the vetter checked; the VTA spends one of the vetter's attestation tokens, binds the token and the statement metadata into the attestation, and returns it. The client sends it to the applicant as a [`vetting/attestation`](../../../../vetting/attestation/0.1/spec.md).

The attestation carries a tag that is deterministic for this vetter and this applicant and unlinkable across applicants, and no identifier of the vetter. The family's shared rules are in [`CONVENTIONS.md`](../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Ask only after the vetting session's human check, for the identifier the applicant presented in its `vetting/request`.
2. Send the result as the payload of a `vetting/attestation/0.1` document to the applicant.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Attest under the newest class label the parameters name as live and the engine holds a credential for; refuse with `vta/pcs/attest:notEnrolled` if there is none, before reserving a token.
4. Spend one free token under a live label — `preferLabel` when it is live and a token is free under it — binding its label and serial into the metadata; refuse with `vta/pcs/attest:noToken` if none is free.
5. Record the attestation in the engine's log, and return it.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVetter`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

An attestation is the vetter's word that a person was checked, and it is relied on by a community that will never learn whose word it was. A VTA's own policy may place a further gate on this task before it runs; this specification states only that the act is consequential ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

## Definitions

- **`contextId`**, **`community`** — the vetter engine.
- **`parameters`** — the community's published parameters, as they stand now.
- **`applicantId`** — the applicant's PCS identifier.
- **`meta`** — what the vetter checked (`AttestMeta`).
- **`preferLabel`** — optionally, the token label to spend under.
- **`attestation`** (response) — the attestation to send the applicant.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000007822621",
  "type": "https://trusttasks.org/spec/vta/pcs/attest/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000007822629",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
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
    "applicantId": "zkChBGjt2GvefLiGF1uuJCsv4eFaSFRuF",
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
      "validUntil": "2027-01-10"
    }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000007822622",
  "type": "https://trusttasks.org/spec/vta/pcs/attest/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000007822629",
  "payload": {
    "attestation": {
      "suite": "ps-ddh-bls12381",
      "attestation": "zgFwGTaQuWjXvuAeQU8b8U51JGX4MhRrp",
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
  }
}
```

## Security & Privacy

### Data carried

The request carries the applicant's PCS identifier and what the vetter checked — method, claims, a liveness flag, the declared relationship, an identity commitment and a card digest, dates. The response carries the attestation, which names no vetter. Dates, never times: an exact time would let a community line an attestation up with a vetter's activity.

### Correlation

The VTA sees the vetter and the applicant's identifier together; that pairing is what hidden vetting keeps from the community, and the engine's audit must not record it in clear ([conventions §6](../../../../vta/_shared/0.4/CONVENTIONS.md#6-audit)). The VTA operator must not be the community's operator, or one colluding with it.

### Retention

The engine's log keeps each attestation's identifier and metadata, so a rotation can be refreshed without another session; it must be protected as the key is.

### Consent/purpose

To vouch, once, that this applicant was checked as `meta` states, for this community's admission. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
