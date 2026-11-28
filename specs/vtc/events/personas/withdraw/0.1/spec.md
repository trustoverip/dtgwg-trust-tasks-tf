---
slug: vtc/events/personas/withdraw
version: '0.1'
title: "VTC Events — Personas — Withdraw"
summary: "A persona withdraws its registration for an event."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
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
  rationale: "Sets the persona's registration to withdrawn, freeing its places for the waitlist."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names the event; the response is the registration."
retention:
  class: durable
  rationale: "The registration's state is kept until the event's records are deleted."
errorCodes:
- code: vtc/events/personas/withdraw:notRegistered
  meaning: "The signer holds no registration for the event."
  retryable: false
- code: vtc/events/personas/withdraw:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/personas/register
---
## Abstract

The **VTC Events — Personas — Withdraw** Trust Task withdraws a persona's registration (VTI-EVT-037, 230). The persona signs, as it did to register.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Accept the document from the persona that registered, and charge it to the operation budget for senders without standing.
2. Refuse `notRegistered` where the signer holds no registration; otherwise set it to `withdrawn` (VTI-EVT-037), freeing its places, answering `changed` false where it was not in play.

## Authorization

**The persona that registered**, signing as itself.

## Definitions

- `Registration`, `EventRef` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The persona (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdraw a persona's registration

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/personas/withdraw/0.1#request",
  "issuer": "did:example:carol-persona",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/personas/withdraw/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Withdrawn

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/personas/withdraw/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol-persona",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "registration": {
      "registrationId": "reg_W5k2N8xRb3Qz7TpLAAAA",
      "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
      "registrantDid": "did:example:carol-persona",
      "status": "withdrawn",
      "guests": 0,
      "roster": "visible",
      "eventSequence": 2,
      "createdAt": "2026-11-28T10:00:01Z",
      "updatedAt": "2026-11-29T10:00:01Z"
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
    "changed": true
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

The event in; the registration out.

### Correlation

The persona identifier only.

### Retention

As any registration.

### Consent/purpose

Withdrawal is always allowed (VTI-EVT-037).
