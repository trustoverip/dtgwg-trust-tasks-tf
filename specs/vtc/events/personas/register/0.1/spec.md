---
slug: vtc/events/personas/register
version: '0.1'
title: "VTC Events — Personas — Register"
summary: "A member registers under another of their identifiers, proving membership by an attestation whose tag keeps them to one place."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- pcs
- privacy
- persona
parties:
- role: "persona"
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor: a replay could only be absorbed inside a bounded window."
sideEffects:
  level: mutating
  rationale: "Creates or changes a registration named by the persona identifier, and records the attestation's tag so the member cannot register twice."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request may carry answers to the event's questions; the response is the registration, named by the persona identifier."
retention:
  class: durable
  rationale: "The registration is kept until the event's records are deleted; answers until the event's retention (VTI-EVT-090)."
errorCodes:
- code: vtc/events/personas/register:identifiedRequired
  meaning: "The event requires identified registration (VTI-EVT-231)."
  retryable: false
- code: vtc/events/personas/register:audienceNotProvable
  meaning: "The event's audience cannot be proven anonymously (VTI-EVT-211, 232)."
  retryable: false
- code: vtc/events/personas/register:attestationInvalid
  meaning: "The attestation does not verify under the community's key, to this event's identity, bound to this document (VTI-EVT-213, 214)."
  retryable: false
- code: vtc/events/personas/register:classNotInAudience
  meaning: "The attestation's class is not a live class of the event's audience in the current period (VTI-EVT-213)."
  retryable: false
- code: vtc/events/personas/register:attestationReplayed
  meaning: "This attestation was carried before (VTI-EVT-214)."
  retryable: false
- code: vtc/events/personas/register:alreadyRegistered
  meaning: "This member already holds a registration for the event under another identifier: the tag is taken."
  retryable: false
- code: vtc/events/personas/register:eventCancelled
  meaning: "The event is cancelled."
  retryable: false
- code: vtc/events/personas/register:notOpen
  meaning: "The event takes no registrations."
  retryable: false
- code: vtc/events/personas/register:windowClosed
  meaning: "Registrations are not open now."
  retryable: false
- code: vtc/events/personas/register:full
  meaning: "The event is full and keeps no waitlist."
  retryable: true
- code: vtc/events/personas/register:tooManyGuests
  meaning: "More guests than the event allows."
  retryable: false
- code: vtc/events/personas/register:answersInvalid
  meaning: "An answer does not fit its question, or a required one is missing."
  retryable: false
- code: vtc/events/personas/register:codeOfConductRequired
  meaning: "The event requires the code of conduct to be acknowledged."
  retryable: false
- code: vtc/events/personas/register:maybeNotAccepted
  meaning: "The event takes registrations by approval: `maybe` is not accepted."
  retryable: false
- code: vtc/events/personas/register:hostDecided
  meaning: "A host declined or removed this registration."
  retryable: false
- code: vtc/events/personas/register:occurrenceRequired
  meaning: "The event is a series registered by occurrence, and the request names no `occurrence`."
  retryable: false
- code: vtc/events/personas/register:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
- code: vtc/events/personas/register:occurrenceCancelled
  meaning: "The occurrence named is cancelled."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/personas/withdraw
- vtc/events/registrations/register
- vta/pcs/member/attest
---
## Abstract

The **VTC Events — Personas — Register** Trust Task lets a member register under another of their identifiers (VTI-EVT-230): the persona signs, and an attestation proves it belongs to a member of the event's audience without saying whose. Its tag keeps the member to one place at the event.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Accept the document from a signer that is not a member — the persona — and charge it to the operation budget for senders without standing.
2. Refuse `identifiedRequired` on an event requiring identified registration and `audienceNotProvable` where its audience cannot be proven anonymously (VTI-EVT-231, 232).
3. Verify the attestation as vtc/events/anonymous/register does, bound to this document and its signer (VTI-EVT-213, 214); refuse `alreadyRegistered` where the tag already holds a registration under another identifier (VTI-EVT-230).
4. Apply the event's registration settings as vtc/events/registrations/register does, with its refusals under this task's codes.
5. Name the registration by the persona identifier (`registrantDid`, kind `persona`), and never show the member to the roster or the hosts (VTI-EVT-230); count no attendance and offer no attendance badge for it (VTI-EVT-232).

## Authorization

**A persona of a member of the event's audience**, proven by the attestation; the signer is the persona identifier, which carries no standing of its own.

## Definitions

- `Registration`, `EventPcsAttestation`, `EventRef` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).
- **Persona** — an identifier of a member other than the one they are a member by (VTI-EVT-230).

## Request

The persona (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Register as a persona

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/personas/register/0.1#request",
  "issuer": "did:example:carol-persona",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
    "status": "going",
    "attestation": {
      "suite": "pcs-bls12-381-v1",
      "attestation": "z3u2en7t5LR2WJQ2YcVPYa1XxYz8XoJ7v1q5pFh3pE6Kp6uJmKq9ZcN2",
      "tag": "zQ3shVc2UkAfJCdc1TR8E66J85h48P19mxcSvquTzPSnSqgxS",
      "classLabel": "member/2026-12"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol-persona#key-1",
    "created": "2026-11-28T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/personas/register/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Registered

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/personas/register/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol-persona",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "registration": {
      "registrationId": "reg_W5k2N8xRb3Qz7TpLAAAA",
      "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
      "registrantDid": "did:example:carol-persona",
      "status": "going",
      "guests": 0,
      "roster": "visible",
      "eventSequence": 2,
      "createdAt": "2026-11-28T10:00:01Z",
      "updatedAt": "2026-11-28T10:00:01Z"
    },
    "event": {
      "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
      "title": "Thursday Build Night",
      "when": {
        "start": "2026-12-03T18:00:00",
        "timeZone": "Europe/London",
        "duration": "PT3H"
      },
      "status": "published",
      "sequence": 2
    },
    "wasCreated": true
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

The persona identifier, an attestation and possibly answers in; the registration out.

### Correlation

The attestation names no member; the roster and the hosts see the persona identifier only (VTI-EVT-230). The tag stops one member taking two places without linking the persona to them.

### Retention

As any registration (VTI-EVT-090).

### Consent/purpose

A member who does not want to be recognised at an event registers as another of their identities (VTI-EVT-055, 230).
