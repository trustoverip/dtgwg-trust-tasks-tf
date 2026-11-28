---
slug: vtc/events/list
version: '0.4'
title: VTC Events — List
summary: 'List the events the caller may see in a time range, by calendar or kind, or only those they registered for, with the caller''s status on each.'
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
  rationale: 'Which events are listed, what of each, and the caller''s status on each depend on who asks; an unattributable caller gets nothing.'
sideEffects:
  level: none
  rationale: 'Reads events and the caller''s registrations; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'Discloses the events the caller may see, as the caller may see them, and the caller''s own status on each.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes:
- code: vtc/events/list:rangeInvalid
  meaning: '`to` is not after `from`, or the range is longer than 400 days.'
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
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/show
- vtc/events/mine/list
- vtc/events/calendars/list
---

## Abstract

The **VTC Events — List** Trust Task lists the events a caller may see, as [`vtc/events/show`](../../show/0.4/spec.md) would show each, with the caller's own registration status in `myStatus`. It is what a member portal's events page and calendar view read.

An event is in the range when its time overlaps `[from, to)`. Unlisted events are left out (VTI-EVT-022); drafts appear only to their hosts, and only when asked for; cancelled events only when asked for. With `mine`, only events the caller has a live registration for. A member's agent that keeps the member's calendar reads [`vtc/events/mine/list`](../../mine/list/0.3/spec.md) instead, which lists registrations rather than events.

## Status of this Document

Version 0.2 answers the Event and Registration of `_shared/0.2`, which carry invitation settings and partners' members (VTI Community Events chapter, VTI-EVT-100 – 135). Its Event and Registration are `vtc/events/_shared/0.2`'s.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`; answer an empty list while events are turned off (VTI-EVT-007).
2. Include only events the caller may see as vtc/events/show states it (VTI-EVT-020, VTI-EVT-021), leaving out unlisted events unless the caller hosts them or holds a registration for them (VTI-EVT-022).
3. Include drafts only with `includeDrafts`, and then only those the caller hosts or holds calendar authority over; include cancelled events only with `includeCancelled`, or where the caller holds a registration for them.
4. Where `from` or `to` is given, include an event only when `[start, start + duration)`, as instants in its time zone, overlaps `[from, to)`; refuse `to` not after `from`, or a range over 400 days, with `vtc/events/list:rangeInvalid`.
5. With `calendarId`, `kind` or `mine`, include only events in that calendar, of that kind, or with a registration of the caller's that is `going`, `maybe`, `pending` or `waitlisted`.
6. Show each event as vtc/events/show would (VTI-EVT-054, VTI-EVT-061), with `myStatus` the caller's registration status where they have one.
7. Order by start instant, earliest first, then by `eventId`; page with `cursor` and `limit`.

### Identity, guests, tickets and attendance (0.3)

The 0.3 Event adds `identity`, `guests`, `tickets`, `countsAttendance` and `attendanceBadge` (VTI-EVT-140, 150, 170, 180, 186). A conforming consumer **MUST** answer them as the event holds them; `identity.authorityValidUntil` only to hosts. A guest (VTI-EVT-151) is answered an event they may see — `invited` naming them, or `open` — with nothing a host alone sees.

### Series and anonymity (0.4)

0.4: a series is answered once per occurrence that overlaps `from`–`to`, each item carrying `occurrence` and that occurrence's `when` and status; without both bounds, a series is answered once, as the series (VTI-EVT-192).

## Authorization

**Standing in the community, bounded by each event's audience** (VTI-EVT-020). A member's read is an act of the member (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `Event`, `EventId`, `CalendarId`, `EventKind`, `RegistrationStatus`, `EventInstant` — see [`event.schema.json`](../../_shared/0.4/event.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Upcoming events in December

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000131",
  "type": "https://trusttasks.org/spec/vtc/events/list/0.4#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "from": "2026-11-30T00:00:00Z",
    "to": "2027-01-01T00:00:00Z",
    "limit": 50
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

### A backwards range

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000133",
  "type": "https://trusttasks.org/spec/vtc/events/list/0.4#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "from": "2027-01-01T00:00:00Z",
    "to": "2026-12-01T00:00:00Z"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/list/0.4#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Answered

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000132",
  "type": "https://trusttasks.org/spec/vtc/events/list/0.4#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000131",
  "payload": {
    "items": [
      {
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
        "myStatus": "going"
      }
    ]
  }
}
```

### Refused

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000134",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000133",
  "payload": {
    "code": "vtc/events/list:rangeInvalid",
    "message": "The end of the range is before its start.",
    "retryable": false,
    "details": {
      "field": "to",
      "reason": "not after from"
    }
  }
}
```

## Security & Privacy

### Data carried

Filters in; the events the caller may see and the caller's own status on each out. Never another registrant, never access details unless public.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID. The list leaves out events outside the caller's audience entirely, so it discloses no event the caller could not open.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Lets a member see what is on. **An event confers nothing** (VTI-EVT-002).
