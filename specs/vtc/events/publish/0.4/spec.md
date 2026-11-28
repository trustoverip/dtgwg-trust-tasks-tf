---
slug: vtc/events/publish
version: '0.4'
title: VTC Events — Publish
summary: 'Publish a draft event: it is shown to its audience and takes registrations, and its roster setting is fixed from now on.'
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
  rationale: 'Shows an event to the community''s members in its name and fixes the roster setting registrants rely on; must stay attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window.'
sideEffects:
  level: mutating
  rationale: 'Moves the event from draft to published. Undone only by cancelling it (vtc/events/cancel); a published event is never deleted (VTI-EVT-093).'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'The request names an event; the response is the event record as its host sees it.'
retention:
  class: durable
  rationale: 'A published event persists; it is cancelled, never deleted.'
errorCodes:
- code: vtc/events/publish:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007). Nothing was changed.'
  retryable: false
- code: vtc/events/publish:notDraft
  meaning: 'The event is already published or is cancelled.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      status:
        type: string
        maxLength: 16
- code: vtc/events/publish:calendarArchived
  meaning: 'The event''s calendar is archived (VTI-EVT-014).'
  retryable: false
- code: vtc/events/publish:eventEnded
  meaning: 'The event''s time has already passed.'
  retryable: false
- code: vtc/events/publish:policyRefused
  meaning: 'The community''s `events` policy refused publication (VTI-EVT-080). Its reason is in `details.reason`. The event stays a draft.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      reason:
        type: string
        maxLength: 500
- code: vtc/events/publish:anonymityUnavailable
  meaning: "The event is `anonymous`, and the community does not run its predicate credential system for its members now."
  retryable: false
- code: vtc/events/publish:anonymitySetTooSmall
  meaning: "The event is `anonymous`, and fewer members hold a credential for a live class of its audience than the community's floor (VTI-EVT-217). Details name both numbers."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/create
- vtc/events/update
- vtc/events/cancel
---

## Abstract

The **VTC Events — Publish** Trust Task moves a draft to **published** (VTI-EVT-004). From then on the event is shown to its audience — listed or, when `listed` is false, answered to a member who names it (VTI-EVT-022) — and takes registrations as its settings say. Its **roster setting is fixed** at this moment (VTI-EVT-051): every registration made afterwards records it (VTI-EVT-052), and a member portal shows it before anyone registers.

A published event is never deleted: it is cancelled ([`vtc/events/cancel`](../../cancel/0.4/spec.md)), so registrants and their agents see that it was (VTI-EVT-093).

## Status of this Document

Version 0.2 answers the event as `vtc/events/_shared/0.2` states it — with its invitation mode, whether members may invite, and partner communities in its audience (VTI-EVT-100 – 135) — as `create/0.2` and `update/0.2` do. Its request is 0.1's.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `vtc/events/publish:eventsOff` while the community has events turned off (VTI-EVT-007).
2. Accept only a host of the event or a signer with calendar authority over its calendar (VTI-EVT-013, VTI-EVT-070).
3. Refuse an event that is not a draft with `vtc/events/publish:notDraft`, naming its status; refuse an event in an archived calendar with `vtc/events/publish:calendarArchived` (VTI-EVT-014) and one whose time has passed with `vtc/events/publish:eventEnded`.
4. Evaluate the community's `events` policy over the event; where it refuses, answer `vtc/events/publish:policyRefused` and leave the event a draft (VTI-EVT-080).
5. Set `status` to `published` and `publishedAt`, leave `sequence` as it is, and from then on refuse any change of `roster` (VTI-EVT-051).
6. Record the publication in its audit trail with the signer (VTI-EVT-091).

### Identity, guests, tickets and attendance (0.3)

The 0.3 Event adds `identity`, `guests`, `tickets`, `countsAttendance` and `attendanceBadge` (VTI-EVT-140, 150, 170, 180, 186). A conforming consumer **MUST** answer them as the event holds them; `identity.authorityValidUntil` only to hosts. A guest (VTI-EVT-151) is answered an event they may see — `invited` naming them, or `open` — with nothing a host alone sees.

### Series and anonymity (0.4)

0.4: publishing an `anonymous` event **MUST** make it a predicate-credential-system identity used for it alone and fill `anonymity` (VTI-EVT-213), and **MUST** be refused `anonymitySetTooSmall` below the community's floor, which is never below 10 (VTI-EVT-217). Publishing fixes `registrationScope` and `identified` (VTI-EVT-194, 231).

## Authorization

**A host of the event**, or calendar authority over its calendar (VTI-EVT-013, VTI-EVT-070). Never signed by a member portal's delegated key (VTI-EVT-095).

## Definitions

- `Event`, `EventId`, `RosterSetting` — see [`event.schema.json`](../../_shared/0.4/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Publish the workshop

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000117",
  "type": "https://trusttasks.org/spec/vtc/events/publish/0.4#request",
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/publish/0.4#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Published

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000118",
  "type": "https://trusttasks.org/spec/vtc/events/publish/0.4#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000117",
  "payload": {
    "event": {
      "eventId": "evt_S0lderIntroNov2026",
      "calendarId": "cal_ElectronicsCal00001",
      "kind": "event",
      "status": "published",
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
      "updatedAt": "2026-11-20T09:30:01Z",
      "publishedAt": "2026-11-20T09:30:01Z"
    }
  }
}
```

### Refused by the community's events policy

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000119",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000117",
  "payload": {
    "code": "vtc/events/publish:policyRefused",
    "message": "The events policy refused publication.",
    "retryable": false,
    "details": {
      "reason": "Events with a capacity above 50 need an administrator to publish them."
    }
  }
}
```

## Security & Privacy

### Data carried

An event identifier in; the event as its host sees it out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

The published event persists until it ends and the community's retention applies; it is never deleted.

### Consent/purpose

Shows an event to the people it is for. Fixing the roster setting here is what lets a member decide whether to register knowing who will see that they did (VTI-EVT-051).
