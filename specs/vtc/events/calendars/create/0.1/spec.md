---
slug: vtc/events/calendars/create
version: '0.1'
title: VTC Events — Calendars — Create
summary: 'Create a calendar: a further community calendar, or a group''s own calendar that the group''s managers run.'
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
  rationale: 'Creates a calendar that holds events in the community''s name and, for a group calendar, hands the group''s managers authority over its events; must stay attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed create would make a second calendar.'
sideEffects:
  level: mutating
  rationale: 'Stores a calendar. Recoverable with vtc/events/calendars/update (`state: archived`).'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'The request carries a name, a description, a colour, a group identifier and defaults; the response is the calendar record.'
retention:
  class: durable
  rationale: 'A calendar persists until archived, and an archived calendar is kept with its events.'
errorCodes:
- code: vtc/events/calendars/create:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007). Nothing was stored.'
  retryable: false
- code: vtc/events/calendars/create:groupNotFound
  meaning: '`groupId` names no group the signer may see.'
  retryable: false
- code: vtc/events/calendars/create:groupHasCalendar
  meaning: 'The group already has an active calendar (VTI-EVT-012). Its identifier is in `details.calendarId`.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      calendarId:
        type: string
        maxLength: 128
- code: vtc/events/calendars/create:settingsInvalid
  meaning: 'A setting is malformed: a reminder that is not a duration before the start, or a colour that is not `#RRGGBB`.'
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
- vtc/events/calendars/update
- vtc/events/calendars/list
- vtc/events/create
---

## Abstract

The **VTC Events — Calendars — Create** Trust Task creates a calendar. Every event belongs to exactly one calendar (VTI-EVT-010). The community's own calendar always exists and is never created with this task (VTI-EVT-011). This task creates either a further calendar of the community's (a topic such as *Socials*), with no `groupId`, or a **group's calendar**, naming the group in `groupId`: the group's managers then run that calendar and the events in it (VTI-EVT-013). A group has at most one active calendar (VTI-EVT-012).

A calendar is the unit a member follows and the scope a host's authority comes from. Its `defaultRoster` and `defaultReminderMinutes` are what a new event in it starts with; they change nothing about events already in it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `vtc/events/calendars/create:eventsOff` while the community has events turned off (VTI-EVT-007).
2. Without `groupId`, accept only a signer holding `vtc.events.manage` unqualified. With `groupId`, accept a signer holding `vtc.events.manage` unqualified or qualified `group:<groupId>`, or `vtc.groups.manage` for that group (VTI-EVT-013); refuse anyone else with `permissionDenied`.
3. Answer `vtc/events/calendars/create:groupNotFound` for a `groupId` that names no group the signer may see, and never create a calendar for a group that does not exist.
4. Refuse a second active calendar for one group with `vtc/events/calendars/create:groupHasCalendar`, naming the existing one (VTI-EVT-012).
5. Assign a fresh `CalendarId`, set `state` to `active`, never set `community` (VTI-EVT-011), and return the calendar as stored.
6. Record the creation in its audit trail with the signer (VTI-EVT-091).

## Authorization

**Calendar authority** (VTI-EVT-013): `vtc.events.manage` unqualified for a calendar of the community's; for a group's calendar, `vtc.events.manage` unqualified or qualified `group:<groupId>`, or `vtc.groups.manage` for that group. The proof identifies the signer ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Never signed by a member portal's delegated key (VTI-EVT-095).

## Definitions

- `Calendar`, `CalendarId`, `RosterSetting`, `JSCalendarDuration`, `Color`, `EventTitle`, `EventDescription` — see [`event.schema.json`](../../../_shared/0.1/event.schema.json).
- `GroupId` — see the groups family's [`group.schema.json`](../../../../groups/_shared/0.1/group.schema.json).

## Request

The calendar administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Create the Electronics group's calendar

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000101",
  "type": "https://trusttasks.org/spec/vtc/events/calendars/create/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "name": "Electronics",
    "description": "Workshops and build nights run by the Electronics group.",
    "color": "#5B5BD6",
    "groupId": "grp_ElectronicsGrp00001",
    "defaultRoster": "hidden",
    "defaultReminderMinutes": [1440, 60]
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/calendars/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Created

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000102",
  "type": "https://trusttasks.org/spec/vtc/events/calendars/create/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000101",
  "payload": {
    "calendar": {
      "calendarId": "cal_ElectronicsCal00001",
      "name": "Electronics",
      "description": "Workshops and build nights run by the Electronics group.",
      "color": "#5B5BD6",
      "groupId": "grp_ElectronicsGrp00001",
      "defaultRoster": "hidden",
      "defaultReminderMinutes": [1440, 60],
      "state": "active",
      "createdAt": "2026-11-19T15:00:01Z"
    }
  }
}
```

### Refused: the group already has a calendar

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000103",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000101",
  "payload": {
    "code": "vtc/events/calendars/create:groupHasCalendar",
    "message": "The Electronics group already has an active calendar.",
    "retryable": false,
    "details": {
      "calendarId": "cal_ElectronicsCal00001"
    }
  }
}
```

## Security & Privacy

### Data carried

A name, description, colour, group identifier and defaults in; the calendar record out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID. A group calendar names its group by `groupId`, which the group's own records already show.

### Retention

The calendar persists until archived, with the audit record of its creation.

### Consent/purpose

Lets a community or a group keep its events together. **A calendar confers nothing** beyond the authority VTI-EVT-013 states over its own events (VTI-EVT-002).
