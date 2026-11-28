---
slug: vtc/events/show
version: '0.4'
title: VTC Events — Show
summary: 'Read one event the caller may see, with the caller''s own registration for it.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: member or administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Whether the event is shown at all, what of it is shown, and the caller''s own registration depend on who asks; an unattributable caller gets nothing.'
sideEffects:
  level: none
  rationale: 'Reads an event and the caller''s registration for it; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'Discloses one event as the caller may see it, and the caller''s own registration.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes:
- code: vtc/events/show:rangeInvalid
  meaning: "`to` is not after `from`, is more than 400 days after it, or only one of them is given."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/list
- vtc/events/access/show
- vtc/events/registrations/register
---

## Abstract

The **VTC Events — Show** Trust Task reads one event as the caller may see it, with the caller's own registration for it in `myRegistration`. It answers a member of the event's audience for a published or cancelled event, whether or not it is listed (VTI-EVT-022), and a host for a draft too. The audience is checked now, never taken from a notification the caller holds (VTI-EVT-021).

What the event shows depends on the caller: a host sees all of it; anyone else sees `access` only where it is public (VTI-EVT-061) and `goingCount` only where `showCount` is true, and then as a threshold allows (VTI-EVT-054). Where the event happens, for a registrant who is going, is [`vtc/events/access/show`](../../access/show/0.2/spec.md).

## Status of this Document

Version 0.2 lets a partner community's member read and register by presenting their partner membership credential (VTI Community Events chapter, VTI-EVT-100 – 135). Its Event and Registration are `vtc/events/_shared/0.2`'s.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023): a draft is seen by its hosts and by calendar authority over its calendar; a published or cancelled event by them and by members its audience admits now (VTI-EVT-020, VTI-EVT-021); a caller who holds a registration for it always sees it. Answer `notFound` while events are turned off (VTI-EVT-007).
3. Leave `access` out unless the caller is a host or the access details are `public` (VTI-EVT-061).
4. For a caller who is not a host, leave `goingCount` out unless `showCount` is true, and then give it as `exact` at or above `countThreshold` (default 5) and as `fewerThan` the threshold below it (VTI-EVT-054); always give `full`.
5. Return the caller's own registration in `myRegistration` where there is one, with its `answers`, and nobody else's.
6. With `presentation`, from a party who is not a member: verify it before, and without reference to, the event the request names, as VTI-VOT-016 verifies a presentation for an election; use the outcome only for an event whose audience names the credential's issuer as a partner; discard it once decided; and, without one that passes, answer as for an event the caller may not see (VTI-EVT-132, VTI-EVT-133).
7. Name a partner member's registration by `registrantDid`, never an `entryId` (VTI-EVT-134).

### Identity, guests, tickets and attendance (0.3)

The 0.3 Event adds `identity`, `guests`, `tickets`, `countsAttendance` and `attendanceBadge` (VTI-EVT-140, 150, 170, 180, 186). A conforming consumer **MUST** answer them as the event holds them; `identity.authorityValidUntil` only to hosts. A guest (VTI-EVT-151) is answered an event they may see — `invited` naming them, or `open` — with nothing a host alone sees.

### Series and anonymity (0.4)

0.4: for a series, a conforming consumer **MUST** answer `occurrences` for the window `from`–`to` (VTI-EVT-192), each with its original start, its `when` after any change and its status (VTI-EVT-191, 196), and refuse a window over 400 days with `rangeInvalid`. Without a window it answers the series alone.

## Authorization

**Standing in the community, bounded by the event's audience** (VTI-EVT-020). A member's read is an act of the member (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `Event`, `EventId`, `Registration`, `Count` — see [`event.schema.json`](../../_shared/0.4/event.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read the workshop

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000128",
  "type": "https://trusttasks.org/spec/vtc/events/show/0.4#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "eventId": "evt_S0lderIntroNov2026"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-20T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/show/0.4#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Answered to a registrant

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000129",
  "type": "https://trusttasks.org/spec/vtc/events/show/0.4#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000128",
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
      "sequence": 0,
      "createdAt": "2026-11-20T09:00:01Z",
      "updatedAt": "2026-11-20T09:30:01Z",
      "publishedAt": "2026-11-20T09:30:01Z"
    },
    "myRegistration": {
      "registrationId": "reg_Ad4L0vel4ceNov2026",
      "eventId": "evt_S0lderIntroNov2026",
      "entryId": "dir_AdaLovelace000001",
      "status": "going",
      "guests": 1,
      "answers": {
        "dietary": "Vegetarian",
        "experience": "Never"
      },
      "roster": "hidden",
      "eventSequence": 0,
      "createdAt": "2026-11-21T08:12:00Z",
      "updatedAt": "2026-11-22T10:00:00Z"
    }
  }
}
```

### Refused: not in the audience

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000130",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000128",
  "payload": {
    "code": "notFound",
    "message": "No event you may see has this identifier.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

An event identifier in; the event as the caller may see it and their own registration out. Never another registrant's registration, never the access details unless public.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID. Answering `notFound` alike for an event that does not exist and one outside the caller's audience keeps the task from disclosing which events exist (VTI-EVT-023).

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Lets a member decide whether to come. **An event confers nothing** (VTI-EVT-002).
