---
slug: vta/pcs/applications/delete
version: "0.1"
title: "VTA PCS — Applications — Delete"
summary: "A client tells its VTA to forget a hidden-vetting application: its PCS key and every attestation it holds are destroyed."
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
  level: destructive
  rationale: >-
    Destroys the application's PCS key and the attestations it holds. The attestations cannot be
    recovered: a vetter would have to sit with the applicant again, and the new attestation
    would be for a new identifier.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: an application handle. Out: confirmation of deletion. Nothing of the application is
    disclosed.
retention:
  class: durable
  rationale: >-
    The deletion is recorded in the VTA's audit trail; the application itself is gone.
errorCodes:
  - code: vta/pcs/applications/delete:notFound
    meaning: "No application has this handle in a context the caller may act in."
    retryable: false
related:
  - vta/pcs/applications/create
  - vta/pcs/applications/show
---

## Abstract

The **VTA PCS — Applications — Delete** Trust Task ends an applicant engine: the VTA destroys the application's PCS key and every attestation it holds. A client sends it once an application is decided, or abandoned. The family's shared rules are in [`CONVENTIONS.md`](../../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Send it once the application is decided or abandoned. An identifier the VTA can still prove for, after its purpose has ended, is a liability and not a convenience.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Destroy the application's key and its attestations, so that no later task can prove for its identifier.
4. Answer `vta/pcs/applications/delete:notFound` both for a handle that names nothing and for one in a context the caller has no standing in.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsApplicant`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`applicationId`** — the application to delete.
- **`deleted`** (response) — always `true`.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000005729901",
  "type": "https://trusttasks.org/spec/vta/pcs/applications/delete/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000005729909",
  "payload": {
    "applicationId": "app-7f3c2a"
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000005729902",
  "type": "https://trusttasks.org/spec/vta/pcs/applications/delete/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000005729909",
  "payload": {
    "applicationId": "app-7f3c2a",
    "deleted": true
  }
}
```

## Security & Privacy

### Data carried

An application handle in, the same handle out.

### Correlation

None beyond the handle, which is local to the VTA.

### Retention

The VTA records that the application was deleted, by whom and when, and nothing of its content.

### Consent/purpose

Ends the application's life at the VTA. A deletion cannot be reversed, and a client should confirm it with the applicant first. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
