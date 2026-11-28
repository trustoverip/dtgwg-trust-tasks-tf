---
slug: vta/pcs/member/enrol-request
version: '0.1'
title: "VTA PCS — Member — Enrol Request"
summary: "A member's client asks its VTA for the blinded requests that enrol the member's events key for the current period's classes; the VTA keeps the blinding state."
status: draft
targetFrameworkVersion: 0.6.0
category: identity
keywords:
- pcs
- events
- zero-knowledge
- vta
parties:
- role: "client driving the PCS engine"
  requirement: REQUIRED
  member: issuer
  identifierScope: pairwise
- role: "PCS engine (the VTA)"
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "What the task does depends on who asks, and must be attributable after the transport has closed."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor: a replay could only be absorbed inside a bounded window."
sideEffects:
  level: mutating
  rationale: "On first use derives the member's events key for the context and community; every use records the blinding state enrol-accept needs."
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The VTA exercises the member's events key to build blinded requests; out goes the vtc/events/classes/enrol request, signed as the member, which the community cannot open."
retention:
  class: durable
  rationale: "The key is derived, never stored; the blinding state lasts until the answer is accepted."
errorCodes:
- code: vta/pcs/member/enrol-request:unsupportedSuite
  meaning: "The community's parameters name a suite the engine does not implement."
  retryable: false
- code: vta/pcs/member/enrol-request:parametersInvalid
  meaning: "The community's parameters cannot be decoded."
  retryable: false
- code: vta/pcs/member/enrol-request:memberUnknown
  meaning: "`memberDid` is not an identifier the VTA holds keys of in this context."
  retryable: false
- code: vta/pcs/member/enrol-request:classNotOffered
  meaning: "A class asked for is not among those vtc/events/classes/show answered. Details name it."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/classes/show
- vtc/events/classes/enrol
- vta/pcs/member/enrol-accept
- vta/pcs/member/attest
---
## Abstract

The **VTA PCS — Member — Enrol Request** Trust Task has the member's VTA build the blinded requests that enrol its events key for the current period's classes, and sign the community's `vtc/events/classes/enrol` request as the member (VTI-EVT-212). The client never holds the key.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Refuse a caller without authority in `contextId` with `permissionDenied`.
2. Derive the member's events key for this context and community for this purpose alone, never the hidden-vetting key (VTI-EVT-212), and never return it.
3. Build one blinded request per class, keep the blinding state, and answer the vtc/events/classes/enrol request signed as `memberDid`.

## Authorization

**A party with authority in the context.** The engine acts as the member.

## Definitions

- `EventClassLabel` — see the events family's [`event.schema.json`](../../../../../vtc/events/_shared/0.4/event.schema.json).

## Request

The client driving the PCS engine (`issuer`) sends the PCS engine (the VTA) (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Enrol for December

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vta/pcs/member/enrol-request/0.1#request",
  "issuer": "did:example:carol-client",
  "recipient": "did:example:vta",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "contextId": "ctx-riverside",
    "community": "did:example:community",
    "memberDid": "did:example:carol",
    "parameters": {
      "suite": "pcs-bls12-381-v1"
    },
    "classes": [
      "member/2026-12"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol-client#key-1",
    "created": "2026-11-28T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The PCS engine (the VTA) answers with a document of type `https://trusttasks.org/spec/vta/pcs/member/enrol-request/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The signed enrolment

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vta/pcs/member/enrol-request/0.1#response",
  "issuer": "did:example:vta",
  "recipient": "did:example:carol-client",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "document": {
      "type": "https://trusttasks.org/spec/vtc/events/classes/enrol/0.1#request"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:vta#key-1",
    "created": "2026-11-28T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

Parameters in; a signed, blinded enrolment out. The key and the blinding state stay in the VTA.

### Correlation

The client is the principal's own; the VTA acts only within the context.

### Retention

The blinding state until the answer is accepted.

### Consent/purpose

Enrolling is what lets the member take part anonymously (VTI-EVT-212).
