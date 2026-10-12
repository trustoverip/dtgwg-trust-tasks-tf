---
slug: vta/pcs/tokens/accept
version: "0.1"
title: "VTA PCS — Tokens — Accept"
summary: "A vetter's client hands its VTA the community's answer to a token draw; the VTA unblinds the tokens into the vetter's wallet."
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
    Adds the tokens to the vetter's wallet, settles the draw, and drops tokens whose label the
    parameters no longer name as live.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA unblinds the community's signatures over serials only it knows. Out: how many tokens
    were received, and the engine's summary.
retention:
  class: durable
  rationale: >-
    The tokens are kept until spent or until their label closes.
errorCodes:
  - code: vta/pcs/tokens/accept:notFound
    meaning: "No vetter engine exists for this context and community."
    retryable: false
  - code: vta/pcs/tokens/accept:noDrawInFlight
    meaning: "No draw is in flight for the answer's label and tick."
    retryable: false
  - code: vta/pcs/tokens/accept:tokensInvalid
    meaning: "A token does not verify under the parameters' token key, or the answer's length does not match the draw."
    retryable: false
  - code: vta/pcs/tokens/accept:unsupportedSuite
    meaning: "The parameters name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/tokens/accept:parametersInvalid
    meaning: "The community parameters cannot be decoded."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-tokens
  - vta/pcs/tokens/request
  - vta/pcs/tokens/abandon
---

## Abstract

The **VTA PCS — Tokens — Accept** Trust Task settles a draw the community answered: the client passes the [`vtc/vetting/vetters/pcs-tokens`](../../../../../vtc/vetting/vetters/pcs-tokens/0.1/spec.md) response, and the VTA unblinds each signature into a token it can later spend in [`vta/pcs/attest`](../../../../../vta/pcs/attest/0.1/spec.md). The family's shared rules are in [`CONVENTIONS.md`](../../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the community's response payload as `response`, without its `ext`.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse with `vta/pcs/tokens/accept:tokensInvalid`, and take none, if any token fails to verify; keep the draw in flight.
4. Otherwise take every token, settle the draw, and drop tokens under labels no longer in `tokenLabels`.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVetter`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`** — the vetter engine.
- **`parameters`** — the community's published parameters.
- **`response`** — the community's answer.
- **`received`** (response) — tokens taken.
- **`vetter`** (response) — the engine after the draw was settled.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000009291151",
  "type": "https://trusttasks.org/spec/vta/pcs/tokens/accept/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000009291159",
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
    "response": {
      "label": "token/2026-10",
      "tick": 4,
      "preCredentials": [
        "zSn8ZVBX1gMqPjEWrfFr6aqGtHjePA7i1",
        "zrd4TnUUQg26cCuPXNtsTwjZzvWGdNJ1x",
        "zXKsKcpDtzM1Yesx7C43jNKGjWL4hUReR"
      ]
    }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000009291152",
  "type": "https://trusttasks.org/spec/vta/pcs/tokens/accept/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000009291159",
  "payload": {
    "received": 3,
    "vetter": {
      "contextId": "personal/communities",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "memberDid": "did:webvh:QmMember:vta.example.com:alice",
      "suite": "ps-ddh-bls12381",
      "id": "z4FvpxtP31f2HAEo4V9ffW3VRH1yZJAXY",
      "enrolledLabels": [
        "vetter/2026-10"
      ],
      "pendingLabels": [],
      "tokensFree": 3,
      "tokensHeld": 3,
      "drawsInFlight": [],
      "attested": 0
    }
  }
}
```

## Security & Privacy

### Data carried

Blind signatures in; counts out. Tokens never leave the VTA unspent.

### Correlation

None beyond the draw.

### Retention

Tokens are kept until spent or until their label closes.

### Consent/purpose

To hold attestation tokens. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
