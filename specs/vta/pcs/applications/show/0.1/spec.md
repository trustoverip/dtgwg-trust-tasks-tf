---
slug: vta/pcs/applications/show
version: "0.1"
title: "VTA PCS — Applications — Show"
summary: "A client reads one of its hidden-vetting applications back from its VTA: its PCS identifier and the metadata of the attestations it holds, never its key."
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
    Reads the applicant engine. Nothing changes.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: an application handle. Out: the application's summary — its PCS identifier, its join DID
    and the metadata of each attestation held. Statement metadata names no vetter.
retention:
  class: transient
  rationale: >-
    A read; the response is a view of state the VTA already keeps.
errorCodes:
  - code: vta/pcs/applications/show:notFound
    meaning: "No application has this handle in a context the caller may act in."
    retryable: false
related:
  - vta/pcs/applications/create
  - vta/pcs/applications/delete
  - vta/pcs/verify-attestation
---

## Abstract

The **VTA PCS — Applications — Show** Trust Task reads an applicant engine back: the application's PCS identifier, the DID it joins as, and the metadata of every attestation it holds. A client uses it to show the applicant how far its application has come, and to recover the identifier after a restart.

The family deliberately has no `list`. A client created every application it drives and holds its handle; an engine that enumerated applications would let any caller with standing in the context discover which communities the principal is applying to. A read by handle discloses nothing to a caller that did not already hold the handle. The family's shared rules are in [`CONVENTIONS.md`](../../../../../vta/_shared/0.4/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Return the application's summary, and **never** its key.
4. Answer `vta/pcs/applications/show:notFound` both for a handle that names nothing and for one in a context the caller has no standing in, so a refusal does not confirm that an application exists.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../../vta/_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsApplicant`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`applicationId`** — the handle [`vta/pcs/applications/create`](../../../../../vta/pcs/applications/create/0.1/spec.md) returned.
- **`application`** (response) — the application (`ApplicationSummary`).

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000008835181",
  "type": "https://trusttasks.org/spec/vta/pcs/applications/show/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-12T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000008835189",
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
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000008835182",
  "type": "https://trusttasks.org/spec/vta/pcs/applications/show/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-12T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000008835189",
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
    }
  }
}
```

## Security & Privacy

### Data carried

An application handle in; the application's summary out. Statement metadata describes how a vetting was done — method, claims verified, dates, the token label — and carries no vetter identifier.

### Correlation

The response repeats what the client already knows of its own application. The handle is local to the VTA and never sent to a community.

### Retention

Nothing is retained beyond the engine state the read reports.

### Consent/purpose

Lets the client present the application's progress to the applicant and recover its identifier. The metadata is not to be used to infer which vetter made which attestation. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
