---
slug: vtc/events/publication/set
version: '0.1'
title: VTC Events — Publication — Set
summary: 'Someone who runs a calendar makes it, or one of its events, public: projected to the community''s website and its public calendar feed.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: calendar runner or host
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
  rationale: 'Sets whether a calendar or an event is public.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'A calendar or event and a flag.'
retention:
  class: durable
  rationale: 'The setting stands until changed.'
errorCodes:
- code: vtc/events/publication/set:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007).'
  retryable: false
- code: vtc/events/publication/set:targetInvalid
  meaning: 'Neither or both of `calendarId` and `eventId`.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/calendars/update
- vtc/website/deploy
---

## Abstract

The **VTC Events — Publication — Set** Trust Task makes a calendar, or one event, public (VTI-EVT-280 – 282): its listed, published events are projected to the community's website as schema.org `Event` data and to its public calendar feed, showing nothing a non-member could not be shown. The website and the feed are the community's own public pages, read without a Trust Task.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse neither or both of `calendarId` and `eventId` with `targetInvalid`.
2. For a calendar, require authority over it (VTI-EVT-013); for an event, hosting it (VTI-EVT-070); answer `notFound` for one the caller may not see.
3. Project only events that are public, published and `listed` (VTI-EVT-280), and only what VTI-EVT-281 permits: never access details unless their own `public` is set, never registrants, a count only where `showCount` is set.
4. Audit the change by reference (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Authority over the calendar, or hosting the event** (VTI-EVT-013, VTI-EVT-070). Never from a portal-held key (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The calendar runner or host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/publication/set/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "calendarId": "cal_RiversideOpenDays",
    "public": true
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/publication/set/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/publication/set/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "calendarId": "cal_RiversideOpenDays",
    "public": true
  }
}
```

## Security & Privacy

### Data carried

A calendar or event and a flag.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable**, until changed.

### Consent/purpose

Telling the world about public events, and nothing else.
