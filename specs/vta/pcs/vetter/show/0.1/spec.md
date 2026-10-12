---
slug: vta/pcs/vetter/show
version: "0.1"
title: "VTA PCS — Vetter — Show"
summary: "A vetter's client reads back its VTA's vetter engine for a community: identifier, enrolled labels, token counts and draws in flight — never its key, credentials or tokens."
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
  requirement: RECOMMENDED
  rationale: >-
    A read of state the caller already has standing over. The transport's own authentication
    suffices; a proof adds attribution in the audit trail.
sideEffects:
  level: none
  rationale: >-
    Reads the vetter engine. Nothing changes.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context and a community. Out: the engine's summary.
retention:
  class: transient
  rationale: >-
    A read.
errorCodes:
  - code: vta/pcs/vetter/show:notFound
    meaning: "No vetter engine exists for this context and community."
    retryable: false
related:
  - vta/pcs/root-request
  - vta/pcs/tokens/request
  - vta/pcs/attest
---

## Abstract

The **VTA PCS — Vetter — Show** Trust Task reads a vetter engine back, so a client can decide what to do next: enrol under a new label, draw the next tick, or tell the vetter it has no token to attest with. The family's shared rules are in [`CONVENTIONS.md`](../../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Return the engine's summary, and never its key, credentials, tokens or blinding state.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVetter`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`** — the vetter engine.
- **`vetter`** (response) — its summary (`VetterSummary`).

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000002239531",
  "type": "https://trusttasks.org/spec/vta/pcs/vetter/show/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000002239539",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org"
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000002239532",
  "type": "https://trusttasks.org/spec/vta/pcs/vetter/show/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000002239539",
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

Labels and counts. Nothing that would let the client act as the vetter.

### Correlation

None.

### Retention

None.

### Consent/purpose

To drive the vetter's schedule and show the vetter its standing. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
