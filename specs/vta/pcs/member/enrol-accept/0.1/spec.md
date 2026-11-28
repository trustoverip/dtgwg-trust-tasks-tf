---
slug: vta/pcs/member/enrol-accept
version: '0.1'
title: "VTA PCS — Member — Enrol Accept"
summary: "A member's client hands its VTA the community's answer to an enrolment; the VTA unblinds and keeps the class credentials."
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
  rationale: "Unblinds each pre-credential and keeps the credential, wrapped, for the period."
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The community's blind pre-credentials in; the classes now held out. The credentials stay in the VTA."
retention:
  class: durable
  rationale: "The credentials are kept for their period."
errorCodes:
- code: vta/pcs/member/enrol-accept:unsupportedSuite
  meaning: "The community's parameters name a suite the engine does not implement."
  retryable: false
- code: vta/pcs/member/enrol-accept:parametersInvalid
  meaning: "The community's parameters cannot be decoded."
  retryable: false
- code: vta/pcs/member/enrol-accept:noEnrolmentPending
  meaning: "No enrolment is pending for this context, community and period."
  retryable: false
- code: vta/pcs/member/enrol-accept:preCredentialInvalid
  meaning: "A pre-credential does not unblind into a credential that verifies."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vta/pcs/member/enrol-request
- vta/pcs/member/attest
- vtc/events/classes/enrol
---
## Abstract

The **VTA PCS — Member — Enrol Accept** Trust Task hands the VTA the community's blind pre-credentials; the VTA unblinds them into class credentials and keeps them for the period (VTI-EVT-212).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Refuse a caller without authority in `contextId` with `permissionDenied`.
2. Unblind each pre-credential with the kept blinding state, verify it under the community's key, keep it wrapped for its period, and discard the blinding state.
3. Refuse `noEnrolmentPending` and `preCredentialInvalid` as their meanings state.

## Authorization

**A party with authority in the context.**

## Definitions

- `EventClassLabel` — see the events family's [`event.schema.json`](../../../../../vtc/events/_shared/0.4/event.schema.json).

## Request

The client driving the PCS engine (`issuer`) sends the PCS engine (the VTA) (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Accept the enrolment

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vta/pcs/member/enrol-accept/0.1#request",
  "issuer": "did:example:carol-client",
  "recipient": "did:example:vta",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "contextId": "ctx-riverside",
    "community": "did:example:community",
    "response": {
      "period": "2026-12"
    }
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

The PCS engine (the VTA) answers with a document of type `https://trusttasks.org/spec/vta/pcs/member/enrol-accept/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Held

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vta/pcs/member/enrol-accept/0.1#response",
  "issuer": "did:example:vta",
  "recipient": "did:example:carol-client",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "period": "2026-12",
    "classes": [
      "member/2026-12"
    ]
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

Blind pre-credentials in; class labels out.

### Correlation

As enrol-request.

### Retention

The credentials for their period.

### Consent/purpose

What anonymous and persona registration use (VTI-EVT-212).
