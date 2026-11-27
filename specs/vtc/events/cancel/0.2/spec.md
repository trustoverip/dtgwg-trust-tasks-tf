---
slug: vtc/events/cancel
version: '0.2'
title: VTC Events — Cancel
summary: 'Cancel a published event: it will not happen, takes no more registrations, and is kept so its registrants and their agents see that it was cancelled.'
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
  rationale: 'Tells every registrant, in the community''s name, that an event will not happen; must stay attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window.'
sideEffects:
  level: destructive
  rationale: 'The event will not happen and cannot be published again; a host who wants it back creates a new event. Registrations are kept with their status, as the record of who had registered.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'The request names an event and a reason; the response is the event record as its host sees it.'
retention:
  class: durable
  rationale: 'A cancelled event and its registrations are kept until the community''s retention removes them (VTI-EVT-090).'
errorCodes:
- code: vtc/events/cancel:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007). Nothing was changed.'
  retryable: false
- code: vtc/events/cancel:notPublished
  meaning: 'The event is a draft (delete it with vtc/events/delete) or is already cancelled.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      status:
        type: string
        maxLength: 16
- code: vtc/events/cancel:eventEnded
  meaning: 'The event has already ended.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/publish
- vtc/events/delete
- vtc/events/show
---

## Abstract

The **VTC Events — Cancel** Trust Task cancels a published event (VTI-EVT-004). The event stays, with `status` `cancelled`, the host's `cancelReason` and an incremented `sequence` (VTI-EVT-006), so a registrant's agent that syncs it sees the cancellation rather than an event that has vanished. It takes no more registrations; a registrant may still withdraw, which changes nothing but their own record (VTI-EVT-037). Registrations keep their status as the record of who had registered.

A cancelled event cannot be published again. To change an event's roster setting, a host cancels it and creates a new one (VTI-EVT-051).

## Status of this Document

Version 0.2 answers the event as `vtc/events/_shared/0.2` states it — with its invitation mode, whether members may invite, and partner communities in its audience (VTI-EVT-100 – 135) — as `create/0.2` and `update/0.2` do. Its request is 0.1's.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `vtc/events/cancel:eventsOff` while the community has events turned off (VTI-EVT-007).
2. Accept only a host of the event or a signer with calendar authority over its calendar (VTI-EVT-013, VTI-EVT-070).
3. Refuse an event that is not published with `vtc/events/cancel:notPublished`, naming its status, and one that has ended with `vtc/events/cancel:eventEnded`.
4. Set `status` to `cancelled`, `cancelledAt` and, where given, `cancelReason`; increment `sequence` by one (VTI-EVT-006).
5. Change no registration's status, and from then on refuse every registration and change except withdrawal (VTI-EVT-037).
6. Record the cancellation in its audit trail with the signer and the reason (VTI-EVT-091).

## Authorization

**A host of the event**, or calendar authority over its calendar (VTI-EVT-013, VTI-EVT-070). Never signed by a member portal's delegated key (VTI-EVT-095).

## Definitions

- `Event`, `EventId` — see [`event.schema.json`](../../_shared/0.2/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Cancel the workshop

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000120",
  "type": "https://trusttasks.org/spec/vtc/events/cancel/0.2#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "eventId": "evt_S0lderIntroNov2026",
    "reason": "The workshop leader is unwell. We will run it again in January."
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

### Cancel a draft

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000122",
  "type": "https://trusttasks.org/spec/vtc/events/cancel/0.2#request",
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

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/cancel/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Cancelled

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000121",
  "type": "https://trusttasks.org/spec/vtc/events/cancel/0.2#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000120",
  "payload": {
    "event": {
      "eventId": "evt_S0lderIntroNov2026",
      "calendarId": "cal_ElectronicsCal00001",
      "kind": "event",
      "status": "cancelled",
      "title": "Intro to Soldering",
      "description": "Learn to solder through-hole parts and build a blinking LED badge to take home. Suitable for complete beginners.",
      "when": {
        "start": "2026-11-30T18:30:00",
        "timeZone": "Europe/London",
        "duration": "PT1H30M"
      },
      "placeSummary": "Riverside, near the station",
      "access": {
        "places": [
          {
            "name": "The Makerspace",
            "address": "Unit 4, Mill Lane, Riverside",
            "description": "Ring the bell; the door code is in your confirmation."
          }
        ],
        "notes": "Wear closed shoes. Irons, solder and kits are provided."
      },
      "audience": {
        "scope": "groups",
        "groups": [
          "grp_ElectronicsGrp00001"
        ]
      },
      "registration": {
        "mode": "approval",
        "capacity": 12,
        "waitlist": true,
        "plusOnes": 1,
        "questions": [
          {
            "questionId": "dietary",
            "label": "Any dietary needs?",
            "kind": "text"
          },
          {
            "questionId": "experience",
            "label": "Have you soldered before?",
            "kind": "choice",
            "choices": [
              "Never",
              "A little",
              "Often"
            ],
            "required": true
          }
        ]
      },
      "roster": "hidden",
      "showCount": true,
      "countThreshold": 5,
      "listed": true,
      "accessibility": {
        "stepFree": true,
        "quietSpace": true
      },
      "suggestedReminderMinutes": [1440, 60],
      "hosts": [
        "dir_Pr1yaNatarajan00"
      ],
      "goingCount": {
        "exact": 9
      },
      "full": false,
      "sequence": 1,
      "createdAt": "2026-11-20T09:00:01Z",
      "updatedAt": "2026-11-28T12:00:01Z",
      "publishedAt": "2026-11-20T09:30:01Z",
      "cancelReason": "The workshop leader is unwell. We will run it again in January.",
      "cancelledAt": "2026-11-28T12:00:01Z"
    }
  }
}
```

### Refused: a draft is deleted, not cancelled

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000123",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000122",
  "payload": {
    "code": "vtc/events/cancel:notPublished",
    "message": "This event is a draft. Delete it instead.",
    "retryable": false,
    "details": {
      "status": "draft"
    }
  }
}
```

## Security & Privacy

### Data carried

An event identifier and a reason in; the cancelled event out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

The cancelled event and its registrations are kept until the community's retention removes them (VTI-EVT-090), so registrants' agents see the cancellation.

### Consent/purpose

Tells the people who registered that the event will not happen. A cancellation is kept rather than deleted because a registrant's record of an event must not silently disappear.
