---
slug: vtc/events/calendars/list
version: '0.2'
title: VTC Events — Calendars — List
summary: 'List the calendars the caller may see: the community''s own, its further calendars, and the calendars of groups the caller may see.'
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
  rationale: 'Which group calendars are shown depends on who asks; an unattributable caller gets nothing.'
sideEffects:
  level: none
  rationale: 'Reads calendar records; persists nothing. May create the community''s own calendar on first need (VTI-EVT-011), which is not an effect of the caller''s.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'Discloses calendars'' names, colours, defaults and their groups'' identifiers to callers who may see those groups.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/calendars/create
- vtc/events/list
---

## Abstract

The **VTC Events — Calendars — List** Trust Task lists the calendars a caller may see: the community's own calendar, which always exists (VTI-EVT-011); the community's further calendars; and the calendar of every group the caller may see. It is what a member portal shows as the filters beside its events, and what a member's agent reads to name and colour the events it syncs.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the VTI Community Events chapter (VTI-EVT-*). The design is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. Answer an empty list while the community has events turned off (VTI-EVT-007).
3. Always include the community's own calendar, creating it on first need with `community` true (VTI-EVT-011).
4. Include a group's calendar only where the group's own `visibility` admits the caller, so this task never discloses a group the groups family would not (VTI-EVT-012).
5. Leave archived calendars out unless `includeArchived` is true.
6. With `groupId`, answer only that group's calendar, or none.
7. Order the community's own calendar first, then by `name`.

### Self check-in and attendance (0.2)

The 0.2 Calendar adds `selfCheckIn` (`none`, `allowed`, `counting`; VTI-EVT-164, 181) and `countsAttendance`, the default for its events (VTI-EVT-180). A conforming consumer **MUST** store and answer them as given.

## Authorization

**Standing in the community**, bounded for each group calendar by the group's visibility. A member's read is an act of the member (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `Calendar`, `CalendarId` — see [`event.schema.json`](../../../_shared/0.3/event.schema.json).
- `GroupId` — see the groups family's [`group.schema.json`](../../../../groups/_shared/0.1/group.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### List calendars

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000108",
  "type": "https://trusttasks.org/spec/vtc/events/calendars/list/0.2#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {},
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/calendars/list/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Answered

```json
{
  "id": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000109",
  "type": "https://trusttasks.org/spec/vtc/events/calendars/list/0.2#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7e1f2c3a-4b5d-4e6f-8a9b-000000000108",
  "payload": {
    "items": [
      {
        "calendarId": "cal_RiversideCommunity1",
        "name": "Riverside Makers",
        "community": true,
        "state": "active",
        "createdAt": "2026-11-01T10:00:00Z"
      },
      {
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
    ]
  }
}
```

## Security & Privacy

### Data carried

Optional filters in; calendar records out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Members are named by their community-scoped `entryId`, never by DID. A group calendar is shown only to a caller the group's visibility admits, so the list reveals no group the caller could not otherwise see.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Lets a member see where the community's events are kept and pick what to follow.
