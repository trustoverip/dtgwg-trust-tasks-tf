---
slug: vtc/events/create
version: '0.1'
title: VTC Events — Create
summary: 'Create an event or meeting as a draft in a calendar: its time, place, audience, registration settings and roster setting. The creator is its first host.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: calendar administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Creates an event the community will show to its members in its own name, and makes the signer its host; must stay attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed create would make a second event.'
sideEffects:
  level: mutating
  rationale: 'Stores a draft event and appoints its creator as its first host. A draft is seen by its hosts only and is removed with vtc/events/delete.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'The request carries the event''s description, time, place, audience and settings; the response is the event record as its host sees it.'
retention:
  class: durable
  rationale: 'A draft persists until published or deleted; see vtc/events/delete.'
errorCodes:
- code: vtc/events/create:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007). Nothing was stored.'
  retryable: false
- code: vtc/events/create:calendarArchived
  meaning: 'The calendar is archived and takes no new events (VTI-EVT-014).'
  retryable: false
- code: vtc/events/create:timeZoneUnknown
  meaning: '`when.timeZone` is not a zone the community''s time zone database knows (VTI-EVT-005).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      timeZone:
        type: string
        maxLength: 64
- code: vtc/events/create:whenInvalid
  meaning: '`when` is inconsistent: an all-day event (`showWithoutTime`) whose `start` is not `T00:00:00` or whose `duration` is not whole days, or a zero duration (VTI-EVT-005).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      field:
        type: string
        maxLength: 128
      reason:
        type: string
        maxLength: 500
- code: vtc/events/create:audienceInvalid
  meaning: '`audience` is inconsistent or names something that does not exist: `groups` without scope `groups` or the reverse, a group the signer may not see, a role the community does not define, or a rule that is malformed or reads a count fact (VTI-EVT-020).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      field:
        type: string
        maxLength: 128
      reason:
        type: string
        maxLength: 500
- code: vtc/events/create:registrationInvalid
  meaning: '`registration` is inconsistent: a duplicate `questionId`, `choices` on a question that is not `choice` or missing on one that is, `closesAt` not after `opensAt`, or `waitlist` without `capacity`.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      field:
        type: string
        maxLength: 128
      reason:
        type: string
        maxLength: 500
- code: vtc/events/create:policyRefused
  meaning: 'The community''s `events` policy refused the event (VTI-EVT-080). Its reason is in `details.reason`. Nothing was stored.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      reason:
        type: string
        maxLength: 500
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/update
- vtc/events/publish
- vtc/events/delete
- vtc/events/show
- vtc/events/calendars/create
---

## Abstract

The **VTC Events — Create** Trust Task creates an event, or a meeting (`kind: meeting`, VTI-EVT-003), as a **draft** in a calendar (VTI-EVT-004, VTI-EVT-010). A draft is seen only by its hosts and takes no registrations; [`vtc/events/publish`](../../publish/0.1/spec.md) shows it to its audience. The signer becomes its first host (VTI-EVT-070).

An event's time is a local `start` in an IANA `timeZone` with a `duration`, as JSCalendar states it (VTI-EVT-005). Its `audience` says who may see and register for it (VTI-EVT-020). Its `access` — where it happens and how to join — is released only to registrants who are going unless marked `public` (VTI-EVT-060, VTI-EVT-061); `placeSummary` is the coarse hint everyone in the audience sees (VTI-EVT-062). Its `roster` setting may change while it is a draft and is fixed once it is published (VTI-EVT-051).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `vtc/events/create:eventsOff` while the community has events turned off (VTI-EVT-007).
2. Use the community's own calendar when `calendarId` is absent (VTI-EVT-011); answer `notFound` for a calendar the signer may not see and `vtc/events/create:calendarArchived` for an archived one (VTI-EVT-014).
3. Accept only a signer with calendar authority over the calendar (VTI-EVT-013); refuse anyone else with `permissionDenied`.
4. Refuse an unknown `when.timeZone` with `vtc/events/create:timeZoneUnknown` and an inconsistent `when` with `vtc/events/create:whenInvalid` (VTI-EVT-005).
5. Refuse an inconsistent `audience` with `vtc/events/create:audienceInvalid` (VTI-EVT-020). Where `audience` is absent, use scope `groups` with the calendar's group for a group calendar, and scope `members` otherwise.
6. Refuse inconsistent `registration` settings with `vtc/events/create:registrationInvalid`. Where `registration` is absent, use mode `open` with no capacity.
7. Default `roster` to the calendar's `defaultRoster`, else `hidden`, and `suggestedReminderMinutes` to the calendar's `defaultReminderMinutes` (VTI-EVT-050).
8. Evaluate the community's `events` policy over the request; refuse with `vtc/events/create:policyRefused` where it refuses, and never let it widen what the request asked for (VTI-EVT-080).
9. Assign a fresh `EventId`, store the event with `status` `draft`, `sequence` 0 and the signer's entry as its only host — appointed as hosts are (VTI-EVT-070) — and return it as its host sees it.
10. Record the creation in its audit trail with the signer (VTI-EVT-091).

## Authorization

**Calendar authority** over the event's calendar (VTI-EVT-013). The proof identifies the signer ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Never signed by a member portal's delegated key (VTI-EVT-095).

## Definitions

- `Event`, `EventId`, `CalendarId`, `EventKind`, `When`, `AccessDetails`, `PlaceSummary`, `Audience`, `RegistrationSettings`, `RosterSetting`, `Accessibility`, `Duration` — see [`event.schema.json`](../../_shared/0.1/event.schema.json).

## Request

The calendar administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Create a workshop in the Electronics calendar

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000110",
  "type": "https://trusttasks.org/spec/vtc/events/create/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "calendarId": "cal_ElectronicsCal00001",
    "kind": "event",
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
    "accessibility": {
      "stepFree": true,
      "quietSpace": true
    }
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Created

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000111",
  "type": "https://trusttasks.org/spec/vtc/events/create/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000110",
  "payload": {
    "event": {
      "eventId": "evt_S0lderIntroNov2026",
      "calendarId": "cal_ElectronicsCal00001",
      "kind": "event",
      "status": "draft",
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
        "exact": 0
      },
      "full": false,
      "sequence": 0,
      "createdAt": "2026-11-20T09:00:01Z",
      "updatedAt": "2026-11-20T09:00:01Z"
    }
  }
}
```

### Refused: an unknown time zone

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000112",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000110",
  "payload": {
    "code": "vtc/events/create:timeZoneUnknown",
    "message": "Europe/Riverside is not a time zone this community knows.",
    "retryable": false,
    "details": {
      "timeZone": "Europe/Riverside"
    }
  }
}
```

## Security & Privacy

### Data carried

The event's description, time, place, audience and settings in; the draft out, to its host.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID. The access details stay with the community: a draft is seen by its hosts only, and once published they are answered only as VTI-EVT-060 allows.

### Retention

The draft persists until published or deleted; the audit record of its creation names the signer.

### Consent/purpose

Lets a host prepare an event before anyone sees it. **An event confers nothing** (VTI-EVT-002): no event, registration or attendance is read by an authorization decision.
