---
slug: vtc/events/attendance/show
version: '0.1'
title: VTC Events — Attendance — Show
summary: 'Read your own attendance: the counting events you checked in to, per calendar per month — what badges read.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'A member''s attendance is shown only to them.'
sideEffects:
  level: none
  rationale: 'Reads the caller''s tallies; persists nothing.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'Optional filters in; the caller''s own monthly tallies out.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes:
- code: vtc/events/attendance/show:rangeInvalid
  meaning: '`untilMonth` is before `sinceMonth`.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/checkin/record
- vtc/groups/create
---

## Abstract

The **VTC Events — Attendance — Show** Trust Task gives a member their own attendance (VTI-EVT-185): the number of counting events they checked in to, per calendar per month — the record a dynamic group's `eventsAttended` rule reads (VTI-EVT-184). Guests, partners' members and personas have none (VTI-EVT-183).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer only the caller's own tallies, as an active member.
2. Count only check-ins that count (VTI-EVT-180 – 183), each event once (VTI-EVT-182).
3. Refuse `untilMonth` before `sinceMonth` with `rangeInvalid`.

## Authorization

**The member themselves.** Their act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` MAY sign it.

## Definitions

- `EventAttendanceTally`, `CalendarId` — see [`event.schema.json`](../../../_shared/0.3/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000009",
  "type": "https://trusttasks.org/spec/vtc/events/attendance/show/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "calendarId": "cal_Rm4Kd9Vt2Nx6Pq8L",
    "sinceMonth": "2026-01"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-30T18:25:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000010",
  "type": "https://trusttasks.org/spec/vtc/events/attendance/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "tallies": [
      {
        "calendarId": "cal_Rm4Kd9Vt2Nx6Pq8L",
        "month": "2026-10",
        "count": 3
      },
      {
        "calendarId": "cal_Rm4Kd9Vt2Nx6Pq8L",
        "month": "2026-11",
        "count": 4
      }
    ],
    "total": 7
  },
  "threadId": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000009"
}
```

## Security & Privacy

### Data carried

Filters in; the caller's own monthly counts out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the DID its members and guests already know it by. Members are named by their community-scoped `entryId`; guests and partners' members by the identifier they registered with, shown to hosts only (VTI-EVT-134, VTI-EVT-153).

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Letting a member see what their attendance badges count. **Events confer nothing**: the community never reads an event, a registration, a check-in, a ticket or an attendance in an authorization decision (VTI-EVT-002).
