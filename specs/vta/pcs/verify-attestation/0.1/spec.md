---
slug: vta/pcs/verify-attestation
version: "0.1"
title: "VTA PCS — Verify Attestation"
summary: "A client hands its VTA an attestation a vetter sent; the VTA checks it and its token against the community's parameters and keeps it for the application's proof."
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
    Adds the attestation to the application, or replaces an earlier attestation from the same
    vetter.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: an application handle, the community's parameters, and an attestation — which names no
    vetter. Out: the application's summary and the metadata of the attestation taken.
retention:
  class: durable
  rationale: >-
    The attestation is what the application's proof is built from; it is kept until the
    application is deleted.
errorCodes:
  - code: vta/pcs/verify-attestation:notFound
    meaning: "No application has this handle in a context the caller may act in."
    retryable: false
  - code: vta/pcs/verify-attestation:unsupportedSuite
    meaning: "The parameters or the attestation name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/verify-attestation:parametersInvalid
    meaning: "The community parameters cannot be decoded."
    retryable: false
  - code: vta/pcs/verify-attestation:attestationInvalid
    meaning: "The attestation, or its token, does not verify under the parameters; or it is for another community, or for another identifier."
    retryable: false
related:
  - vetting/attestation
  - vta/pcs/attest
  - vta/pcs/prove
  - vta/pcs/applications/show
---

## Abstract

The **VTA PCS — Verify Attestation** Trust Task takes an attestation a vetter sent the applicant — the payload of a [`vetting/attestation`](../../../../vetting/attestation/0.1/spec.md) — into the application it was made for. The VTA checks it on arrival: that it verifies under the community's parameters for this application's identifier, and that its token verifies under the community's token key. An attestation that fails is refused while the vetter is still there to ask, rather than discovered at submission. The family's shared rules are in [`CONVENTIONS.md`](../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the `payload` of the vetter's `vetting/attestation` document as `attestation`, without its `ext`, after verifying that document's envelope proof attributes it to the vetter the applicant sat with.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Verify the attestation against the application's identifier under `parameters`, and its token under the published token key; refuse with `vta/pcs/verify-attestation:attestationInvalid` if either fails, or if `attestation.meta.community` is not the application's community.
4. Keep the attestation with the application. An attestation from a vetter the application already holds one from (the same tag) replaces it rather than being kept beside it: two from one vetter count once at the community, and a replacement is how a vetter refreshes after a rotation.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsApplicant`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`applicationId`** — the application the attestation was made for.
- **`parameters`** — the community's published parameters.
- **`attestation`** — the attestation (`Attestation`).
- **`application`** (response) — the application after the attestation was taken.
- **`accepted`** (response) — the metadata of the attestation taken.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000008672251",
  "type": "https://trusttasks.org/spec/vta/pcs/verify-attestation/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000008672259",
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

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000008672252",
  "type": "https://trusttasks.org/spec/vta/pcs/verify-attestation/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000008672259",
  "payload": {
    "application": {
      "applicationId": "app-7f3c2a",
      "contextId": "personal/communities",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "joinDid": "did:key:z6MkpJoinPersonaExampleKey22222222222222222222",
      "suite": "ps-ddh-bls12381",
      "id": "zkChBGjt2GvefLiGF1uuJCsv4eFaSFRuF",
      "attestations": [
        {
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
        }
      ],
      "createdAt": "2026-10-12T09:00:00Z"
    },
    "accepted": {
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
    }
  }
}
```

## Security & Privacy

### Data carried

The attestation carries a tag, a shown class credential, a proof, the statement metadata and the token spent — and no vetter identifier. The VTA learns the tag, which is deterministic for this vetter and this applicant and unlinkable across applicants.

### Correlation

The VTA holding several attestations for one application can tell they are from distinct vetters, which it must in order to prove. It cannot tell who they are from the attestations alone. The client, which carried each from a vetter it sat with, can; the VTA's audit must not join the two ([conventions §6](../../../../vta/_shared/0.4/CONVENTIONS.md#6-audit)).

### Retention

Kept with the application until it is deleted.

### Consent/purpose

Collected only to prove this application to this community. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
