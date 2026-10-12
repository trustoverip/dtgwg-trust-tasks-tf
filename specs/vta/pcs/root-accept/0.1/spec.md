---
slug: vta/pcs/root-accept
version: "0.1"
title: "VTA PCS — Root Accept"
summary: "A vetter's client hands its VTA the community's answer to a root request; the VTA unblinds it into the vetter's class credential for the period and keeps it."
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
    Stores the vetter's class credential for the label, and discards the blinding state it was
    unblinded with.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA exercises the vetter's PCS key to unblind the community's answer into a credential
    the community has never seen. Out: the vetter engine's summary — labels and counts, never
    the credential.
retention:
  class: durable
  rationale: >-
    The class credential is what the vetter attests under for its period.
errorCodes:
  - code: vta/pcs/root-accept:notFound
    meaning: "No vetter engine exists for this context and community."
    retryable: false
  - code: vta/pcs/root-accept:noRequestInFlight
    meaning: "The engine holds no blinding state for the answer's label."
    retryable: false
  - code: vta/pcs/root-accept:credentialInvalid
    meaning: "The answer does not unblind into a credential that verifies under the parameters' helper key."
    retryable: false
  - code: vta/pcs/root-accept:unsupportedSuite
    meaning: "The parameters name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/root-accept:parametersInvalid
    meaning: "The community parameters cannot be decoded."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-root
  - vta/pcs/root-request
  - vta/pcs/vetter/show
---

## Abstract

The **VTA PCS — Root Accept** Trust Task is the second half of a vetter's enrolment. The client passes the community's [`vtc/vetting/vetters/pcs-root`](../../../../vtc/vetting/vetters/pcs-root/0.1/spec.md) response; the VTA unblinds it with the state [`vta/pcs/root-request`](../../../../vta/pcs/root-request/0.1/spec.md) kept, checks the credential against the community's helper key, and keeps it. The family's shared rules are in [`CONVENTIONS.md`](../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the community's response payload as `response`, without its `ext`.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Unblind `response.preCredential` with the blinding state kept for `response.label`; refuse with `vta/pcs/root-accept:credentialInvalid` if the result does not verify under the parameters' helper key, and keep the blinding state so a correct answer can still be accepted.
4. On success keep the credential under its period and discard the blinding state.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVetter`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`** — the vetter engine.
- **`parameters`** — the community's published parameters.
- **`response`** — the community's answer.
- **`vetter`** (response) — the engine after the credential was taken (`VetterSummary`).

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000003007111",
  "type": "https://trusttasks.org/spec/vta/pcs/root-accept/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000003007119",
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
      "label": "vetter/2026-10",
      "preCredential": "zv4FAsGW8WPSZcPpmLuwdzejotKtvy9ep"
    }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000003007112",
  "type": "https://trusttasks.org/spec/vta/pcs/root-accept/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000003007119",
  "payload": {
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
      "tokensFree": 0,
      "tokensHeld": 0,
      "drawsInFlight": [],
      "attested": 0
    }
  }
}
```

## Security & Privacy

### Data carried

A pre-credential in, useless to anyone without the blinding state. The engine's labels and counts out.

### Correlation

None beyond what root-request already established.

### Retention

The credential is kept for its period; credentials for periods the community no longer names as live may be discarded.

### Consent/purpose

To hold the vetter's class credential. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
