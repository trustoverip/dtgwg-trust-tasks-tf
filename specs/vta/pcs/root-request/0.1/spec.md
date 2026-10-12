---
slug: vta/pcs/root-request
version: "0.1"
title: "VTA PCS — Root Request"
summary: "A vetter's client asks its VTA for the blinded request that enrols the vetter with a community for a vetting period; the VTA creates the vetter's PCS key on first use and keeps the blinding state."
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
    On first use for a context and community, creates the vetter engine and derives its PCS key.
    Every use records the blinding state of the request, which root-accept needs to unblind the
    community's answer.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA exercises the vetter's PCS key to build a blinded request in the vetter's name. Out:
    the request the community blind-signs — a commitment it cannot open — and the vetter's PCS
    identifier. The key and the blinding state stay in the VTA.
retention:
  class: durable
  rationale: >-
    The vetter engine and its key last for the vetter's whole life in the community; the
    blinding state lasts until the answer is accepted.
errorCodes:
  - code: vta/pcs/root-request:unsupportedSuite
    meaning: "The parameters name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/root-request:parametersInvalid
    meaning: "The community parameters cannot be decoded."
    retryable: false
  - code: vta/pcs/root-request:memberMismatch
    meaning: "A vetter engine exists for this context and community under another member DID."
    retryable: false
  - code: vta/pcs/root-request:labelNotLive
    meaning: "`label` is not among the parameters' `vetterLabels`."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-root
  - vta/pcs/root-accept
  - vta/pcs/vetter/show
---

## Abstract

The **VTA PCS — Root Request** Trust Task is the first half of a vetter's enrolment with a community that runs hidden vetting. The VTA builds the blinded request of [`vtc/vetting/vetters/pcs-root`](../../../../vtc/vetting/vetters/pcs-root/0.1/spec.md) for the named class label, from the vetter's PCS key, and keeps the blinding state. The client sends the request to the community, signed by the vetter's member DID, and hands the community's answer to [`vta/pcs/root-accept`](../../../../vta/pcs/root-accept/0.1/spec.md).

The first request for a context and community creates the vetter engine and its key, which then lasts for the vetter's whole life in that community ([conventions §3](../../../../vta/_shared/0.4/CONVENTIONS.md#3-one-vetter-key-per-community)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Send `request` as the payload of a `vtc/vetting/vetters/pcs-root/0.1` document signed by `memberDid`, and pass the community's response to `vta/pcs/root-accept`.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Create the vetter engine for (`contextId`, `community`) if none exists, deriving its key and recording `memberDid`; refuse with `vta/pcs/root-request:memberMismatch` if one exists under another member DID.
4. Refuse a `label` the parameters do not name as live with `vta/pcs/root-request:labelNotLive`.
5. Build the request with the engine's one key, keep its blinding state against `label` (replacing any earlier state for that label), and return the request — never the key or the blinding state.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVetter`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`** — the vetter engine.
- **`memberDid`** — the vetter's member DID in the community.
- **`parameters`** — the community's published parameters.
- **`label`** — the class label to enrol under, `vetter/<YYYY-MM>`.
- **`request`** (response) — the `vtc/vetting/vetters/pcs-root` payload to send.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000675031",
  "type": "https://trusttasks.org/spec/vta/pcs/root-request/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000675039",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "memberDid": "did:webvh:QmMember:vta.example.com:alice",
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
    "label": "vetter/2026-10"
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000675032",
  "type": "https://trusttasks.org/spec/vta/pcs/root-request/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000675039",
  "payload": {
    "request": {
      "label": "vetter/2026-10",
      "id": "z4FvpxtP31f2HAEo4V9ffW3VRH1yZJAXY",
      "request": {
        "encoding": "zdjd2scJfUZZT8yfFVfG4UK8RADYCWWni",
        "t0": "zpKvJZS5uaUfGsbMyNW1JVTu1PXZPynmc",
        "proof": "zUCxgm6Xt5yph6DZFZW57AHAA1PdCt1Ny"
      }
    }
  }
}
```

## Security & Privacy

### Data carried

The response carries the vetter's PCS identifier and a commitment the community cannot open. The identifier is recorded by the community once and required at every later label, so a member cannot enrol twice.

### Correlation

The identifier is stable for this vetter in this community — by design, as above — and is never used for another community.

### Retention

The VTA keeps the engine for as long as the vetter vets for the community, and the blinding state until root-accept consumes it.

### Consent/purpose

To obtain the class credential a vetter attests under. Nothing else. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
