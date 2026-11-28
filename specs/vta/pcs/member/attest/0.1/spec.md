---
slug: vta/pcs/member/attest
version: '0.1'
title: "VTA PCS — Member — Attest"
summary: "A member's client asks its VTA for an anonymous or persona events document: the VTA makes the attestation to the event's identity and signs the document with a key made for it alone, or as the persona."
status: draft
targetFrameworkVersion: 0.6.0
category: identity
keywords:
- pcs
- events
- zero-knowledge
- privacy
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
  rationale: "Exercises a class credential to make an attestation, and signs a document with a key made for it alone (discarded) or as a persona the VTA holds."
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The client's intended operation in; the signed document carrying the attestation out. The member's key and credentials stay in the VTA."
retention:
  class: transient
  rationale: "The one-time key is discarded once the document is signed."
errorCodes:
- code: vta/pcs/member/attest:unsupportedSuite
  meaning: "The community's parameters name a suite the engine does not implement."
  retryable: false
- code: vta/pcs/member/attest:parametersInvalid
  meaning: "The community's parameters cannot be decoded."
  retryable: false
- code: vta/pcs/member/attest:noCredential
  meaning: "The VTA holds no credential for a live class of the event's audience this period: enrol first."
  retryable: false
- code: vta/pcs/member/attest:personaUnknown
  meaning: "`personaDid` is not an identifier the VTA holds keys of in this context."
  retryable: false
- code: vta/pcs/member/attest:operationInvalid
  meaning: "`payload` does not fit the operation named."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/anonymous/register
- vtc/events/anonymous/withdraw
- vtc/events/anonymous/checkin
- vtc/events/personas/register
- vta/pcs/member/enrol-accept
---
## Abstract

The **VTA PCS — Member — Attest** Trust Task has the member's VTA build an anonymous or persona events document: it makes the attestation to the event's identity with a class credential it holds (VTI-EVT-213), binds it to the document (VTI-EVT-214), and signs the document with a key made for it alone — or as the persona (VTI-EVT-230). The client sends it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Refuse a caller without authority in `contextId` with `permissionDenied`.
2. Choose a credential the VTA holds for a live class of the event's audience (`noCredential` where none), and make the attestation to the event's identity, bound to the event, the operation's task name, the document's `id` and its signer (VTI-EVT-213, 214).
3. For an anonymous operation, sign the document with a key made for it alone and discard the key; for `personaRegister`, sign as `personaDid` (`personaUnknown` where the VTA holds no keys of it).
4. Never return the member's key, a credential or the one-time key, and never send the document itself: the client sends it, as a ballot is cast (VTI-EVT-214).

## Authorization

**A party with authority in the context.**

## Definitions

- `Event`, `EventMultibase` — see the events family's [`event.schema.json`](../../../../../vtc/events/_shared/0.4/event.schema.json).

## Request

The client driving the PCS engine (`issuer`) sends the PCS engine (the VTA) (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Build an anonymous registration

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vta/pcs/member/attest/0.1#request",
  "issuer": "did:example:carol-client",
  "recipient": "did:example:vta",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "contextId": "ctx-riverside",
    "community": "did:example:community",
    "operation": "anonymousRegister",
    "event": {
      "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
      "calendarId": "cal_W5k2N8xRb3Qz7TpLAAAA",
      "kind": "event",
      "status": "published",
      "title": "Thursday Build Night",
      "when": {
        "start": "2026-12-03T18:00:00",
        "timeZone": "Europe/London",
        "duration": "PT3H"
      },
      "audience": {
        "scope": "members"
      },
      "registration": {
        "mode": "open"
      },
      "roster": "anonymous",
      "listed": true,
      "sequence": 2,
      "anonymity": {
        "suite": "pcs-bls12-381-v1",
        "pcsIdentity": "zQ3shtxV1FrJfhqE1dvxYRcCknWNjHc3c5X1y4ZPYuVuXjZRn",
        "classes": [
          "member/2026-12"
        ],
        "period": "2026-12",
        "periodEndsAt": "2027-01-01T00:00:00Z",
        "holderCount": 42
      },
      "createdAt": "2026-11-20T09:00:00Z",
      "updatedAt": "2026-11-25T09:00:00Z"
    },
    "payload": {
      "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
      "status": "going"
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

The PCS engine (the VTA) answers with a document of type `https://trusttasks.org/spec/vta/pcs/member/attest/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The signed document

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vta/pcs/member/attest/0.1#response",
  "issuer": "did:example:vta",
  "recipient": "did:example:carol-client",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "document": {
      "type": "https://trusttasks.org/spec/vtc/events/anonymous/register/0.1#request"
    },
    "tag": "zQ3shVc2UkAfJCdc1TR8E66J85h48P19mxcSvquTzPSnSqgxS"
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

The operation in; the signed document and its tag out.

### Correlation

The one-time key is made for this document and discarded, so nothing ties two anonymous documents to each other or to the member; a persona document is signed as the persona.

### Retention

Nothing kept beyond the audit record of the request.

### Consent/purpose

The member's agent does the zero-knowledge work so the client never holds the key (VTI-EVT-212 – 215, 230).
