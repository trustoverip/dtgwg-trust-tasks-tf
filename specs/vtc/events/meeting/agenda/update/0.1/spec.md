---
slug: vtc/events/meeting/agenda/update
version: '0.1'
title: VTC Events — Meeting — Agenda — Update
summary: 'The chair, the secretary or a host replaces a meeting''s agenda: its items in order, each with an owner, a time box and links.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: chair, secretary or host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'What the caller may do or see depends on who asks; the request must be attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would repeat a change.'
sideEffects:
  level: mutating
  rationale: 'Replaces the meeting''s agenda.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'Agenda items with entries and links; the agenda.'
retention:
  class: durable
  rationale: 'The agenda is kept with the meeting.'
errorCodes:
- code: vtc/events/meeting/agenda/update:notAMeeting
  meaning: 'The event is not of kind `meeting`; this task applies to meetings only (VTI-EVT-255).'
  retryable: false
- code: vtc/events/meeting/agenda/update:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007).'
  retryable: false
- code: vtc/events/meeting/agenda/update:eventCancelled
  meaning: 'The meeting is cancelled; its agenda does not change.'
  retryable: false
- code: vtc/events/meeting/agenda/update:agendaInvalid
  meaning: 'An item names an `itemId` the agenda does not hold, names one twice, or names an owner who is not an active member.'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"itemId": {"type": "string", "maxLength": 128}, "reason": {"type": "string", "maxLength": 500}}}
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/meeting/show
---

## Abstract

The **VTC Events — Meeting — Agenda — Update** Trust Task replaces a meeting's agenda (VTI-EVT-256): the items in order, each with an optional owner, time box and links to a board idea, an election, a record in the meeting's room or a web page.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023), and refuse an event of any kind but `meeting` with `notAMeeting` (VTI-EVT-255).
2. Refuse a caller who is neither the meeting's chair, its secretary nor one of its hosts with `permissionDenied` (VTI-EVT-256).
3. Refuse a cancelled meeting with `eventCancelled`; an item naming an `itemId` the agenda does not hold, one named twice, or an owner who is not an active member with `agendaInvalid`.
4. Replace the agenda with the items as sent, in order, keeping the identity of each named item and giving each other item a new identifier.
5. Audit the change by reference (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Authority: the meeting's chair or secretary by appointment, or a host** (VTI-EVT-256, VTI-EVT-257). Never from a portal-held key (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The chair, secretary or host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/agenda/update/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "agenda": [
      {
        "title": "Apologies and roll call",
        "timeboxMinutes": 5
      },
      {
        "title": "2027 budget",
        "owner": "dir_KwameOsei000000000",
        "timeboxMinutes": 20
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:mary#key-1",
    "created": "2026-12-05T10:40:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/meeting/agenda/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/agenda/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "agenda": [
      {
        "itemId": "agi_Apologies000000001",
        "title": "Apologies and roll call",
        "timeboxMinutes": 5
      },
      {
        "itemId": "agi_Budget2027Item0001",
        "title": "2027 budget",
        "owner": "dir_KwameOsei000000000",
        "timeboxMinutes": 20
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

A meeting and its agenda.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable**, with the meeting.

### Consent/purpose

Planning a meeting.
