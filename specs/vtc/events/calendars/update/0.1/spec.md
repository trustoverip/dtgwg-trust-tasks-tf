---
slug: vtc/events/calendars/update
version: '0.1'
title: VTC Events — Calendars — Update
summary: 'Change a calendar''s name, description, colour or defaults, or archive it so it takes no new events.'
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
  rationale: 'Changes or archives a calendar the community answers for; must stay attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window.'
sideEffects:
  level: mutating
  rationale: 'Changes the calendar record. Archiving is undone by setting `state` back to `active`.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'The request carries the fields that change; the response is the calendar record.'
retention:
  class: durable
  rationale: 'The calendar persists; the audit record names what changed.'
errorCodes:
- code: vtc/events/calendars/update:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007). Nothing was changed.'
  retryable: false
- code: vtc/events/calendars/update:communityCalendar
  meaning: 'The community''s own calendar cannot be archived (VTI-EVT-011).'
  retryable: false
- code: vtc/events/calendars/update:groupHasCalendar
  meaning: 'The calendar is a group''s and the group already has another active calendar, so this one cannot be made active again (VTI-EVT-012). The other is named in `details.calendarId`.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      calendarId:
        type: string
        maxLength: 128
- code: vtc/events/calendars/update:settingsInvalid
  meaning: 'A setting is malformed.'
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
- vtc/events/calendars/create
- vtc/events/calendars/list
---

## Abstract

The **VTC Events — Calendars — Update** Trust Task changes a calendar. Members present replace what is stored; `description` and `color` may be `null` to clear them. Setting `state` to `archived` stops the calendar taking new events and keeps every event already in it (VTI-EVT-014); setting it back to `active` reopens it. A calendar's group and whether it is the community's own never change. Its defaults apply to events created afterwards, never to events already in it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `vtc/events/calendars/update:eventsOff` while the community has events turned off (VTI-EVT-007).
2. Accept only a signer with calendar authority over the calendar as vtc/events/calendars/create states it (VTI-EVT-013); answer `notFound` for a calendar the signer may not see, and `permissionDenied` for one they may see but not run.
3. Refuse to archive the community's own calendar with `vtc/events/calendars/update:communityCalendar` (VTI-EVT-011).
4. Refuse to make a group's calendar active while the group has another active calendar, with `vtc/events/calendars/update:groupHasCalendar` (VTI-EVT-012).
5. Replace each member present, clear `description` or `color` given as `null`, change no event already in the calendar, and return the calendar as stored.
6. Record the change in its audit trail with the signer and the members changed (VTI-EVT-091).

## Authorization

**Calendar authority** over the calendar named (VTI-EVT-013), as for [`vtc/events/calendars/create`](../../create/0.1/spec.md). Never signed by a member portal's delegated key (VTI-EVT-095).

## Definitions

- `Calendar`, `CalendarId`, `RosterSetting`, `Duration`, `Color`, `Title`, `Description` — see [`event.schema.json`](../../../_shared/0.1/event.schema.json).

## Request

The calendar administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Archive a calendar

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000104",
  "type": "https://trusttasks.org/spec/vtc/events/calendars/update/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "calendarId": "cal_ElectronicsCal00001",
    "state": "archived"
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

### Archive the community's calendar

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000106",
  "type": "https://trusttasks.org/spec/vtc/events/calendars/update/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "calendarId": "cal_RiversideCommunity1",
    "state": "archived"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/calendars/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Archived

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000105",
  "type": "https://trusttasks.org/spec/vtc/events/calendars/update/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000104",
  "payload": {
    "calendar": {
      "calendarId": "cal_ElectronicsCal00001",
      "name": "Electronics",
      "description": "Workshops and build nights run by the Electronics group.",
      "color": "#5B5BD6",
      "groupId": "grp_ElectronicsGrp00001",
      "defaultRoster": "hidden",
      "defaultReminderMinutes": [1440, 60],
      "state": "archived",
      "createdAt": "2026-11-19T15:00:01Z"
    }
  }
}
```

### Refused

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000107",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000106",
  "payload": {
    "code": "vtc/events/calendars/update:communityCalendar",
    "message": "The community's own calendar cannot be archived.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A calendar identifier and the members that change in; the calendar record out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

The calendar persists; the audit record names what changed and who changed it.

### Consent/purpose

Keeps a calendar's presentation and defaults current. **A calendar confers nothing** beyond VTI-EVT-013 (VTI-EVT-002).
