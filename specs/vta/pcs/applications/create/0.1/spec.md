---
slug: vta/pcs/applications/create
version: "0.1"
title: "VTA PCS — Applications — Create"
summary: "An applicant's client asks its VTA to start a hidden-vetting application: the VTA derives a fresh PCS key for it, keeps the key, and returns the PCS identifier vetters attest to."
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
    Creates an applicant engine and derives a fresh PCS secret key for it in the named context.
    The key is kept until vta/pcs/applications/delete removes the application.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context, a community DID, the DID the applicant joins as, and the community's public
    parameters. Out: the application's handle and its PCS identifier — a public value the
    applicant hands to every vetter it sits with. The key the identifier is made from is never
    disclosed.
retention:
  class: durable
  rationale: >-
    The application, and the key behind its identifier, are what every later attestation and the
    final proof are bound to. They are worth keeping until the application ends.
errorCodes:
  - code: vta/pcs/applications/create:unsupportedSuite
    meaning: "The parameters name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/applications/create:parametersInvalid
    meaning: "The community parameters cannot be decoded."
    retryable: false
related:
  - vta/pcs/applications/show
  - vta/pcs/applications/delete
  - vta/pcs/verify-attestation
  - vta/pcs/prove
  - vetting/request
  - vtc/vetting/hidden/publish
---

## Abstract

The **VTA PCS — Applications — Create** Trust Task starts an applicant engine. The VTA derives a PCS secret key for this application alone, from the named context's derivation base, and returns the application's handle and its **PCS identifier** — the public value a vetter attests to. The key stays in the VTA. Every later step of the application — taking in an attestation, proving to the community — runs against it through [`vta/pcs/verify-attestation`](../../../../../vta/pcs/verify-attestation/0.1/spec.md) and [`vta/pcs/prove`](../../../../../vta/pcs/prove/0.1/spec.md).

This is what lets a client with no PCS implementation apply to a community that runs hidden vetting: it never has to make, hold or use the key. The family's shared rules are in [`CONVENTIONS.md`](../../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Copy `parameters` verbatim from the criterion's `vetting.ext` in the community's join manifest.
2. Hand the returned `id` to each vetter it asks to vet it, in its `vetting/request`'s `ext` under the community's hidden-vetting namespace, together with `suite`.
3. Keep `applicationId`: it is the only handle to the application, and the engine does not enumerate applications.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Derive a PCS secret key for this application from `contextId`'s derivation base, at a position no other application has used ([conventions §2–3](../../../../../vta/_shared/0.4/CONVENTIONS.md#2-custody)), and create the applicant engine bound to `joinDid` and `community`.
4. Return the application's summary, and **never** the key.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsApplicant`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`** — the context the application belongs to. Its key is derived from this context's base, and the caller must have standing in it.
- **`community`** — the community's DID.
- **`joinDid`** — the DID the applicant will join as. The proof binds the PCS identifier to it.
- **`parameters`** — the community's published parameters (`CommunityParameters`).
- **`application`** (response) — the new application (`ApplicationSummary`): its `applicationId`, its PCS `id`, and an empty list of attestations.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000008789991",
  "type": "https://trusttasks.org/spec/vta/pcs/applications/create/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000008789999",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "joinDid": "did:key:z6MkpJoinPersonaExampleKey22222222222222222222",
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
    }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000008789992",
  "type": "https://trusttasks.org/spec/vta/pcs/applications/create/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000008789999",
  "payload": {
    "application": {
      "applicationId": "app-7f3c2a",
      "contextId": "personal/communities",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "joinDid": "did:key:z6MkpJoinPersonaExampleKey22222222222222222222",
      "suite": "ps-ddh-bls12381",
      "id": "zkChBGjt2GvefLiGF1uuJCsv4eFaSFRuF",
      "attestations": [],
      "createdAt": "2026-10-12T09:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a context identifier, a community DID, the applicant's join DID and the community's public parameters. The response carries the application's handle and its PCS identifier. Neither carries a key; the identifier is a public value by design — the applicant gives it to each vetter it sits with.

### Correlation

The PCS identifier is fresh for every application, so two applications by one person share nothing on the wire. Within one application it is stable: every vetter who sees it, and the community at submission, can join on it, which is what binding attestations to one applicant requires. The join DID is visible to the VTA and, at submission, to the community.

### Retention

The VTA keeps the application and its key until it is deleted. It should be deleted once the application is decided: a key kept after that serves nothing, and an identifier the VTA can still prove for is one more thing that could be misused.

### Consent/purpose

The key and identifier exist to collect attestations for one application to one community and to prove them to that community. They are not an identity for anything else, and the identifier is not a stable handle on the applicant. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
