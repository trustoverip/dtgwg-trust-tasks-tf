---
slug: vtc/events/occurrences/update
version: '0.1'
title: "VTC Events — Occurrences — Update"
summary: "Change or cancel one occurrence of a series, kept against its original start; the other occurrences are unchanged."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- recurrence
parties:
- role: "host"
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: "community maintainer"
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "What the task does depends on who asks, and must be attributable after the transport has closed."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor: a replay could only be absorbed inside a bounded window."
sideEffects:
  level: mutating
  rationale: "Stores a change to one occurrence of a series, raising the series' sequence where it changes the occurrence's time, access details or status."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries one occurrence's changed time, words, place hint or access details; the response is the series and the occurrence."
retention:
  class: durable
  rationale: "The change is part of the event until the event's records are deleted."
errorCodes:
- code: vtc/events/occurrences/update:eventsOff
  meaning: "The community has events turned off. Nothing was stored."
  retryable: false
- code: vtc/events/occurrences/update:notSeries
  meaning: "The event is not a series."
  retryable: false
- code: vtc/events/occurrences/update:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
- code: vtc/events/occurrences/update:eventCancelled
  meaning: "The series is cancelled."
  retryable: false
- code: vtc/events/occurrences/update:occurrenceEnded
  meaning: "The occurrence has ended; an ended occurrence does not change."
  retryable: false
- code: vtc/events/occurrences/update:whenInvalid
  meaning: "`start` with `duration` gives no valid time in the series' zone."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/update
- vtc/events/series/split
- vtc/events/show
---
## Abstract

The **VTC Events — Occurrences — Update** Trust Task changes one occurrence of a series — its time, words, place hint or access details — or cancels it, kept against the occurrence's original start (VTI-EVT-191, 193, 196). The rest of the series is unchanged. A change from one occurrence onward is [`vtc/events/series/split`](../../../series/split/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Refuse a caller who is not one of the event's hosts (VTI-EVT-070) with `permissionDenied`, and answer `notFound` for an event the caller may not see (VTI-EVT-023).
2. Refuse `notSeries` for an event that is not a series, `occurrenceUnknown` for an original start the rule does not give, `eventCancelled` for a cancelled series and `occurrenceEnded` for an occurrence that has ended.
3. Keep the change against the occurrence's original start (VTI-EVT-191, 193); members absent leave the series' own in force; `cancelled` false removes a cancellation.
4. Raise the series' `sequence` where the change moves the occurrence, changes its access details or its status (VTI-EVT-006, 193).
5. On cancellation keep every registration for the occurrence and show it cancelled, release no access details for it and count no attendance for it (VTI-EVT-196).
6. Record the change in its audit trail by reference (VTI-EVT-091).

## Authorization

**The event's hosts** (VTI-EVT-070): the calendar's authority or an appointed host. Never a member portal's delegated key.

## Definitions

- `Event`, `EventOccurrence`, `LocalDateTime` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).
- **Original start** — the local date-time the rule gives an occurrence, before any change (VTI-EVT-191).

## Request

The host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Cancel the Christmas Eve build night

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/occurrences/update/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
    "occurrence": "2026-12-24T18:00:00",
    "cancelled": true,
    "reason": "The makerspace is closed for the holiday."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-28T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/occurrences/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Cancelled

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/occurrences/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "event": {
      "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
      "calendarId": "cal_W5k2N8xRb3Qz7TpLAAAA",
      "kind": "event",
      "status": "published",
      "title": "Thursday Build Night",
      "when": {
        "start": "2026-12-03T18:00:00",
        "timeZone": "Europe/London",
        "duration": "PT3H"
      },
      "audience": {
        "scope": "members"
      },
      "registration": {
        "mode": "open"
      },
      "roster": "visible",
      "listed": true,
      "sequence": 3,
      "recurrence": {
        "recurrenceRules": [
          {
            "frequency": "weekly",
            "byDay": [
              {
                "day": "th"
              }
            ],
            "count": 20
          }
        ],
        "recurrenceOverrides": {
          "2026-12-24T18:00:00": {
            "cancelled": true
          }
        }
      },
      "registrationScope": "series",
      "createdAt": "2026-11-20T09:00:00Z",
      "updatedAt": "2026-11-28T10:00:01Z"
    },
    "occurrence": {
      "occurrence": "2026-12-24T18:00:00",
      "when": {
        "start": "2026-12-24T18:00:00",
        "timeZone": "Europe/London",
        "duration": "PT3H"
      },
      "status": "cancelled"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:community#key-1",
    "created": "2026-11-28T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

One occurrence's changed time, words, place hint or access details in; the series and the occurrence out, as its host sees them.

### Correlation

The community maintainer is addressed by its public DID; hosts by their own.

### Retention

The change is kept as part of the event; the audit record names the host and the occurrence, never the reason's text beyond the record.

### Consent/purpose

Hosts keep a series accurate occurrence by occurrence. A change confers nothing (VTI-EVT-002).
