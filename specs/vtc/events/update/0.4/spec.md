---
slug: vtc/events/update
version: '0.4'
title: VTC Events — Update
summary: 'Change an event: its time, place, access details, audience, registration settings or presentation. The roster setting changes only while it is a draft.'
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
  rationale: 'Changes what the community tells its members about an event, possibly its time or place; must stay attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window.'
sideEffects:
  level: mutating
  rationale: 'Changes the event record and, for a published event, may increment its `sequence`. Registrations are not changed.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'The request carries the members that change; the response is the event record as its host sees it.'
retention:
  class: durable
  rationale: 'The event persists; the audit record names what changed.'
errorCodes:
- code: vtc/events/update:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007). Nothing was changed.'
  retryable: false
- code: vtc/events/update:eventCancelled
  meaning: 'The event is cancelled and changes no more.'
  retryable: false
- code: vtc/events/update:rosterLocked
  meaning: 'The event is published, so its roster setting is fixed (VTI-EVT-051). Nothing was changed. To change it, cancel the event and create a new one.'
  retryable: false
- code: vtc/events/update:timeZoneUnknown
  meaning: '`when.timeZone` is not a zone the community''s time zone database knows (VTI-EVT-005).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      timeZone:
        type: string
        maxLength: 64
- code: vtc/events/update:whenInvalid
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
- code: vtc/events/update:audienceInvalid
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
- code: vtc/events/update:registrationInvalid
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
- code: vtc/events/update:policyRefused
  meaning: 'The community''s `events` policy refused the event as changed (VTI-EVT-080). Its reason is in `details.reason`. Nothing was changed.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      reason:
        type: string
        maxLength: 500
- code: vtc/events/update:partnerInvalid
  meaning: 'An audience partner is neither a member of the community nor a party it holds a relationship in force with (VTI-EVT-130). Nothing was stored.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      partner:
        type: string
        maxLength: 2048
- code: vtc/events/update:badgeNotFound
  meaning: '`attendanceBadge` names no active community badge (VTI-EVT-186).'
  retryable: false
- code: vtc/events/update:recurrenceInvalid
  meaning: "`recurrence` falls outside VTI-EVT-190's subset."
  retryable: false
- code: vtc/events/update:recurrenceLocked
  meaning: "The event is published: its rule changes only by vtc/events/series/split, and single occurrences by vtc/events/occurrences/update."
  retryable: false
- code: vtc/events/update:registrationScopeLocked
  meaning: "The event is published, and `registrationScope` would change (VTI-EVT-194)."
  retryable: false
- code: vtc/events/update:identifiedLocked
  meaning: "The event is published, and `identified` would change (VTI-EVT-231)."
  retryable: false
- code: vtc/events/update:anonymityUnavailable
  meaning: "`roster` is set to `anonymous`, and the community does not run its predicate credential system for its members."
  retryable: false
- code: vtc/events/update:audienceNotProvable
  meaning: "The event is or becomes `anonymous`, and the audience or the registration settings cannot be proven anonymously (VTI-EVT-211, 216)."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/create
- vtc/events/publish
- vtc/events/cancel
- vtc/events/show
---

## Abstract

The **VTC Events — Update** Trust Task changes an event. Members present replace what is stored; `description`, `placeSummary`, `access` and `accessibility` may be `null` to clear them. An event's kind and calendar never change (VTI-EVT-003).

While the event is a draft anything else may change. Once it is published, its **roster setting is fixed**: a request that changes `roster` is refused, not parked (VTI-EVT-051). A change of `when` or `access` to a published event increments its `sequence`, the number a registrant's agent compares to notice that something it was told has changed (VTI-EVT-006, VTI-EVT-063).

Registrations are never changed by an update. A narrower audience applies to registrations made afterwards; a smaller `capacity` moves nobody off the event, it only stops new registrations becoming going until there is room (VTI-EVT-021, VTI-EVT-033).

## Status of this Document

Version 0.2 adds the invitation mode, members' invitations and partner communities in the audience (VTI Community Events chapter, VTI-EVT-100 – 135). Its Event and Registration are `vtc/events/_shared/0.2`'s.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `vtc/events/update:eventsOff` while the community has events turned off (VTI-EVT-007).
2. Accept only a host of the event or a signer with calendar authority over its calendar (VTI-EVT-013, VTI-EVT-070); answer `notFound` for an event the signer may not see and `permissionDenied` for one they may see but not host.
3. Refuse any change to a cancelled event with `vtc/events/update:eventCancelled`.
4. Refuse a request carrying a `roster` different from the stored one once the event is published, with `vtc/events/update:rosterLocked`, changing nothing (VTI-EVT-051). A `roster` equal to the stored one is not a change.
5. Validate `when`, `audience` and `registration` as vtc/events/create does, answering this task's codes, and evaluate the `events` policy over the event as changed (VTI-EVT-080).
6. Replace each member present and clear each member given as `null`; leave every other member as stored.
7. For a published event, increment `sequence` by one when `when` or `access` changes (VTI-EVT-006, VTI-EVT-063), and set `updatedAt`.
8. Change no registration: never remove, move or re-decide a registration because the audience, capacity or mode changed (VTI-EVT-021). Where a larger capacity frees places, move waitlisted registrations to going as VTI-EVT-033 states.
9. Record the change in its audit trail with the signer, the members changed and `reason` (VTI-EVT-091).
10. Refuse an audience naming a partner that is neither a member of the community nor a party it holds a relationship in force with, with `partnerInvalid` (VTI-EVT-130).
11. Once the event is published with `invite` `notify` or `expect` — or is changed to one of them while published — invite its audience as VTI-EVT-101 requires; send each named partner its `event` notification (VTI-EVT-131). Changing `invite` to `none` withdraws nothing (VTI-EVT-108).

### Identity, guests, tickets and attendance (0.3)

The 0.3 Event adds `identity`, `guests`, `tickets`, `countsAttendance` and `attendanceBadge` (VTI-EVT-140, 150, 170, 180, 186). A conforming consumer **MUST** answer them as the event holds them; `identity.authorityValidUntil` only to hosts. A guest (VTI-EVT-151) is answered an event they may see — `invited` naming them, or `open` — with nothing a host alone sees.

An event's identity is fixed at creation (VTI-EVT-140): `update` has no `identity` member.

### Series and anonymity (0.4)

0.4 adds `recurrence`, `registrationScope` and `identified`. A conforming consumer **MUST** let them change only while the event is a draft (`recurrenceLocked`, `registrationScopeLocked`, `identifiedLocked`), check a rule against VTI-EVT-190, and refuse a change that leaves an `anonymous` event with an audience or settings that cannot be proven anonymously (VTI-EVT-211, 216).

## Authorization

**A host of the event**, or calendar authority over its calendar (VTI-EVT-013, VTI-EVT-070). Never signed by a member portal's delegated key (VTI-EVT-095).

## Definitions

- `Event`, `EventId`, `When`, `AccessDetails`, `PlaceSummary`, `EventAudience`, `RegistrationSettings`, `RosterSetting`, `Accessibility`, `JSCalendarDuration` — see [`event.schema.json`](../../_shared/0.4/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Move the workshop half an hour later

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000113",
  "type": "https://trusttasks.org/spec/vtc/events/update/0.4#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "eventId": "evt_S0lderIntroNov2026",
    "when": {
      "start": "2026-11-30T19:00:00",
      "timeZone": "Europe/London",
      "duration": "PT1H30M"
    },
    "reason": "The room is booked until 19:00."
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

### Make the roster visible after publishing

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000115",
  "type": "https://trusttasks.org/spec/vtc/events/update/0.4#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "eventId": "evt_S0lderIntroNov2026",
    "roster": "visible"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/update/0.4#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Updated: the sequence moves on

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000114",
  "type": "https://trusttasks.org/spec/vtc/events/update/0.4#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000113",
  "payload": {
    "event": {
      "eventId": "evt_S0lderIntroNov2026",
      "calendarId": "cal_ElectronicsCal00001",
      "kind": "event",
      "status": "published",
      "title": "Intro to Soldering",
      "description": "Learn to solder through-hole parts and build a blinking LED badge to take home. Suitable for complete beginners.",
      "when": {
        "start": "2026-11-30T19:00:00",
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
      "sequence": 1,
      "createdAt": "2026-11-20T09:00:01Z",
      "updatedAt": "2026-11-24T08:00:01Z",
      "publishedAt": "2026-11-20T09:30:01Z"
    }
  }
}
```

### Refused: the roster setting is fixed

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000116",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000115",
  "payload": {
    "code": "vtc/events/update:rosterLocked",
    "message": "This event is published, so its roster setting cannot change. Cancel it and create a new event to change it.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

An event identifier and the members that change in; the event as its host sees it out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

The event persists; the audit record names what changed, who changed it and why.

### Consent/purpose

Keeps an event accurate. The roster setting is fixed at publication because registrants chose to register under it: a member who registered for an event with a hidden roster must never find themselves on a visible list (VTI-EVT-051, VTI-EVT-052).
