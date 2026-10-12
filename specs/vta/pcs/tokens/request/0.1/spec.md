---
slug: vta/pcs/tokens/request
version: "0.1"
title: "VTA PCS — Tokens — Request"
summary: "A vetter's client asks its VTA for one draw of the attestation-token drip: blinded serials bound to the vetter's member DID, the label and the tick."
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
    Records the draw as in flight, with the serials it committed to, so the community's answer
    can be taken into the wallet.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA builds commitments to serials only it knows, with opening proofs bound to the
    vetter's member DID. Out: the draw to send the community. The serials stay in the VTA.
retention:
  class: exchange
  rationale: >-
    The draw is kept until tokens/accept or tokens/abandon settles it.
errorCodes:
  - code: vta/pcs/tokens/request:notFound
    meaning: "No vetter engine exists for this context and community."
    retryable: false
  - code: vta/pcs/tokens/request:labelNotLive
    meaning: "`label` is not among the parameters' `tokenLabels`."
    retryable: false
  - code: vta/pcs/tokens/request:countExceedsDrip
    meaning: "`count` is more than the parameters' `dripPerTick`."
    retryable: false
  - code: vta/pcs/tokens/request:alreadyInFlight
    meaning: "A draw for this label and tick is already in flight."
    retryable: false
  - code: vta/pcs/tokens/request:unsupportedSuite
    meaning: "The parameters name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/tokens/request:parametersInvalid
    meaning: "The community parameters cannot be decoded."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-tokens
  - vta/pcs/tokens/accept
  - vta/pcs/tokens/abandon
  - vta/pcs/vetter/show
---

## Abstract

The **VTA PCS — Tokens — Request** Trust Task builds one draw of a vetter's attestation-token drip: the [`vtc/vetting/vetters/pcs-tokens`](../../../../../vtc/vetting/vetters/pcs-tokens/0.1/spec.md) request for a label and tick, with blinded serials bound to the vetter's member DID. The client sends it to the community, signed by that DID, and settles it with [`vta/pcs/tokens/accept`](../../../../../vta/pcs/tokens/accept/0.1/spec.md) or [`vta/pcs/tokens/abandon`](../../../../../vta/pcs/tokens/abandon/0.1/spec.md).

The drip is unconditional by design: a vetter draws on its schedule whether or not it has vetted anyone, because a draw that tracked activity would report activity. A client **SHOULD** draw every tick. The family's shared rules are in [`CONVENTIONS.md`](../../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Send `request` as the payload of a `vtc/vetting/vetters/pcs-tokens/0.1` document signed by the engine's member DID.
2. Settle the draw: pass the community's response to `vta/pcs/tokens/accept`, or, on a refusal (`tickNotYet`, `alreadyServed`), call `vta/pcs/tokens/abandon`.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse a `label` the parameters do not name as live, a `count` over `dripPerTick`, and a draw already in flight for the same label and tick.
4. Bind each opening proof to the engine's recorded member DID, the label and the tick, record the draw as in flight, and return it — never the serials.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVetter`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`** — the vetter engine.
- **`parameters`** — the community's published parameters.
- **`label`**, **`tick`** — the draw.
- **`count`** — tokens asked for.
- **`request`** (response) — the `vtc/vetting/vetters/pcs-tokens` payload to send.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000002672331",
  "type": "https://trusttasks.org/spec/vta/pcs/tokens/request/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000002672339",
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
    "label": "token/2026-10",
    "tick": 4,
    "count": 3
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000002672332",
  "type": "https://trusttasks.org/spec/vta/pcs/tokens/request/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000002672339",
  "payload": {
    "request": {
      "label": "token/2026-10",
      "tick": 4,
      "requests": [
        {
          "commitment": "ztPZLiNivk8UXd37mcK2545avGVPPvW9U",
          "openingProof": "zSUDeNeEvYst3bf7CMW61Fu17ZpPFajWo"
        },
        {
          "commitment": "znqmFBWoGfJ8nDFS9W1u67SaksfujbpCa",
          "openingProof": "ztbTa5j82aA3QXQ3VddhG6RPr8gax8EgK"
        },
        {
          "commitment": "zG8wG6rJQGrK1xFaYQ9E7kPCVrQgzz979",
          "openingProof": "zDiKeAK3zJYyDM7GQhtT6WDc5eUDH1fx7"
        }
      ]
    }
  }
}
```

## Security & Privacy

### Data carried

Commitments and opening proofs out; the serials they commit to stay in the VTA, and are what make a token spendable.

### Correlation

The draw is bound to the vetter's member DID, by design — the community caps each vetter's rate by it. Its regularity is what keeps the draw from correlating with vetting activity.

### Retention

The draw in flight is kept until settled.

### Consent/purpose

To hold attestation tokens, which cap how many attestations a vetter can make. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
