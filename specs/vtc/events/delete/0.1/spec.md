---
slug: vtc/events/delete
version: '0.1'
title: VTC Events — Delete
summary: 'Delete a draft event. A published event is cancelled, never deleted.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: event host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Removes an event the community holds; must stay attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window.'
sideEffects:
  level: destructive
  rationale: 'Removes the draft and its host appointments. A draft has no registrations, so nobody else is affected.'
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: 'The request names an event; the response repeats its identifier.'
retention:
  class: transient
  rationale: 'The draft is gone; the audit record of the deletion remains.'
errorCodes:
- code: vtc/events/delete:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007). Nothing was deleted.'
  retryable: false
- code: vtc/events/delete:notDraft
  meaning: 'The event is published or cancelled; cancel a published event with vtc/events/cancel (VTI-EVT-093).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      status:
        type: string
        maxLength: 16
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/create
- vtc/events/cancel
---

## Abstract

The **VTC Events — Delete** Trust Task removes a **draft** and its host appointments. Only a draft is ever deleted (VTI-EVT-093): it has been seen by its hosts alone and has no registrations. A published event is cancelled instead ([`vtc/events/cancel`](../../cancel/0.2/spec.md)), so that nobody who registered for it finds it gone without a word.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `vtc/events/delete:eventsOff` while the community has events turned off (VTI-EVT-007).
2. Accept only a host of the event or a signer with calendar authority over its calendar (VTI-EVT-013, VTI-EVT-070).
3. Refuse an event that is not a draft with `vtc/events/delete:notDraft`, naming its status (VTI-EVT-093).
4. Remove the draft and withdraw its host appointments, and answer its identifier.
5. Record the deletion in its audit trail with the signer (VTI-EVT-091).

## Authorization

**A host of the event**, or calendar authority over its calendar (VTI-EVT-013, VTI-EVT-070). Never signed by a member portal's delegated key (VTI-EVT-095).

## Definitions

- `EventId` — see [`event.schema.json`](../../_shared/0.1/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Delete a draft

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000124",
  "type": "https://trusttasks.org/spec/vtc/events/delete/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "eventId": "evt_BuildNightDraft0001"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-20T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### Delete a published event

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000126",
  "type": "https://trusttasks.org/spec/vtc/events/delete/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "eventId": "evt_S0lderIntroNov2026"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-20T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/delete/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Deleted

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000125",
  "type": "https://trusttasks.org/spec/vtc/events/delete/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000124",
  "payload": {
    "eventId": "evt_BuildNightDraft0001"
  }
}
```

### Refused: cancel it instead

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000127",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000126",
  "payload": {
    "code": "vtc/events/delete:notDraft",
    "message": "This event is published. Cancel it instead.",
    "retryable": false,
    "details": {
      "status": "published"
    }
  }
}
```

## Security & Privacy

### Data carried

An event identifier in and out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

Nothing of the draft remains but the audit record of its deletion.

### Consent/purpose

Lets a host discard an event nobody else has seen.
