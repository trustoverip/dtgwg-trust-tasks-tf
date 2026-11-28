---
slug: vtc/events/classes/enrol
version: '0.1'
title: "VTC Events — Classes — Enrol"
summary: "A member's agent obtains, by blind issuance, its class credentials for the current period: the membership's and those of its groups."
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
  identifierScope: pairwise
- role: "community maintainer"
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
  rationale: "Blind-signs one pre-credential per class asked for and records that the member holds one for the period, so it is issued at most once per class and period."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries blinded requests the community cannot open and the member's predicate-credential-system identity; the response carries blind pre-credentials only the member's agent can unblind."
retention:
  class: durable
  rationale: "The record that a credential was issued for a class and period, kept until the period ends."
errorCodes:
- code: vtc/events/classes/enrol:anonymityUnavailable
  meaning: "The community does not run its predicate credential system for its members."
  retryable: false
- code: vtc/events/classes/enrol:periodNotCurrent
  meaning: "`period` is not the community's current period."
  retryable: false
- code: vtc/events/classes/enrol:classNotHeld
  meaning: "A class asked for is not one the caller holds this period: not a member of the group, or the label is not live. Details name it."
  retryable: false
- code: vtc/events/classes/enrol:alreadyIssued
  meaning: "A credential for this class and period was already issued to the caller. Details name the class."
  retryable: false
- code: vtc/events/classes/enrol:requestInvalid
  meaning: "A blinded request does not decode or its proof does not verify."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/classes/show
- vta/pcs/member/enrol-request
- vta/pcs/member/enrol-accept
---
## Abstract

The **VTC Events — Classes — Enrol** Trust Task is how a member's agent obtains its class credentials for the current period, by blind issuance (VTI-EVT-212): the membership's and those of the groups the member belongs to. The community records that it issued each one, at most once per class and period, and never sees the credentials themselves.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`, and refuse a document signed by a member portal's delegated key: enrolment is the member's agent's act.
2. Refuse `anonymityUnavailable`, `periodNotCurrent`, `classNotHeld`, `alreadyIssued` and `requestInvalid` as their meanings state, issuing nothing on any refusal.
3. Issue blind (VTI-EVT-212): never learn the credential it issues; one pre-credential per class asked for, under the class's shared label.
4. Record that the caller holds a credential for each class and period before answering, so a second request is `alreadyIssued`; count the holders per class and period for VTI-EVT-217.

## Authorization

**An active member's own agent**, signing as the member. Never a member portal's delegated key.

## Definitions

- `EventClassLabel`, `EventMultibase` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).
- **Blind issuance** — the community signs a commitment it cannot open; only the member's agent can unblind the credential (VTI-EVT-212).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Enrol for December

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/classes/enrol/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "period": "2026-12",
    "pcsIdentity": "zQ3shtxV1FrJfhqE1dvxYRcCknWNjHc3c5X1y4ZPYuVuXjZRn",
    "requests": [
      {
        "classLabel": "member/2026-12",
        "request": "zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme"
      }
    ]
  },
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/classes/enrol/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Issued

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/classes/enrol/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "period": "2026-12",
    "preCredentials": [
      {
        "classLabel": "member/2026-12",
        "preCredential": "zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme"
      }
    ]
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

The member's identity and blinded requests in; blind pre-credentials out.

### Correlation

The community learns that this member holds a credential for each class this period, as it learns their membership anyway; it cannot recognise the credential when it is shown, because the issuance is blind and every showing is re-randomised.

### Retention

The record of issue is kept until the period ends.

### Consent/purpose

Enrolment is what lets a member take part anonymously later (VTI-EVT-212).
