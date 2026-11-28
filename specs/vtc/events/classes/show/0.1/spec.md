---
slug: vtc/events/classes/show
version: '0.1'
title: "VTC Events — Classes — Show"
summary: "Read what a member's agent needs to take part in anonymous and persona registration: the community's verification key, the current period and the live classes."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- pcs
- zero-knowledge
parties:
- role: "member"
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: "community maintainer"
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "What the task does depends on who asks, and must be attributable after the transport has closed."
sideEffects:
  level: none
  rationale: "Reads the community's published predicate-credential-system parameters."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Discloses the community's public verification key, the current period and the class labels the caller may hold."
retention:
  class: transient
  rationale: "A read."
errorCodes:
- code: vtc/events/classes/show:anonymityUnavailable
  meaning: "The community does not run its predicate credential system for its members (VTI-EVT-210)."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/classes/enrol
- vtc/events/anonymous/register
- vta/pcs/member/enrol-request
---
## Abstract

The **VTC Events — Classes — Show** Trust Task answers what a member's agent needs to take part in anonymous and persona registration (VTI-EVT-210 – 218, 230): the community's verification key, the deployment, the current period and when it ends, the class labels the caller may hold, and the floor an anonymous event's audience must reach.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Refuse `anonymityUnavailable` where it runs no member classes.
3. Answer only the classes the caller may hold this period (VTI-EVT-212): the membership's and those of the groups the caller belongs to.

## Authorization

**An active member**, signing as themselves. A member's act: a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `EventClassLabel`, `EventMultibase` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read the classes

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/classes/show/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-28T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/classes/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Answered

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/classes/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "suite": "pcs-bls12-381-v1",
    "verificationKey": "zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme",
    "deployment": "did:example:community#events-pcs",
    "period": "2026-12",
    "periodEndsAt": "2027-01-01T00:00:00Z",
    "classes": [
      "member/2026-12",
      "group/grp_W5k2N8xRb3Qz7TpL/2026-12"
    ],
    "floor": 10
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:community#key-1",
    "created": "2026-11-28T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

Nothing in; public parameters and the caller's own classes out.

### Correlation

The class labels are shared by every holder (VTI-EVT-212): naming the ones a caller may hold says only which groups they are in, which the caller already knows.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

What a member's agent needs before it enrols (VTI-EVT-212).
