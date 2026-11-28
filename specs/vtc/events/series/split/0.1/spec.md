---
slug: vtc/events/series/split
version: '0.1'
title: "VTC Events — Series — Split"
summary: "Change a series from one occurrence onward: the series ends before it and a linked continuation starts with it, carrying its registrations."
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
  rationale: "Ends a series before an occurrence, creates its continuation and moves the registrations that concern the continuation's occurrences into it."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries the changes to apply from one occurrence onward; the response is both series."
retention:
  class: durable
  rationale: "Both series are kept until their records are deleted."
errorCodes:
- code: vtc/events/series/split:eventsOff
  meaning: "The community has events turned off. Nothing was stored."
  retryable: false
- code: vtc/events/series/split:notSeries
  meaning: "The event is not a series."
  retryable: false
- code: vtc/events/series/split:occurrenceUnknown
  meaning: "`from` is not an original start the series' rule gives."
  retryable: false
- code: vtc/events/series/split:firstOccurrence
  meaning: "`from` is the series' first occurrence: change the series itself with vtc/events/update."
  retryable: false
- code: vtc/events/series/split:occurrenceEnded
  meaning: "The occurrence named has ended."
  retryable: false
- code: vtc/events/series/split:eventCancelled
  meaning: "The series is cancelled."
  retryable: false
- code: vtc/events/series/split:recurrenceInvalid
  meaning: "`recurrence` falls outside VTI-EVT-190's subset."
  retryable: false
- code: vtc/events/series/split:whenInvalid
  meaning: "`when` is not a valid time."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/occurrences/update
- vtc/events/update
- vtc/events/show
---
## Abstract

The **VTC Events — Series — Split** Trust Task changes a series from one occurrence onward ("this and following"): the series ends before that occurrence and a continuation starting with it takes the change, linked to it and carrying the registrations that concern its occurrences (VTI-EVT-197).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Refuse a caller who is not one of the event's hosts with `permissionDenied`; answer `notFound` for an event the caller may not see.
2. Refuse `notSeries`, `occurrenceUnknown`, `firstOccurrence`, `occurrenceEnded` and `eventCancelled` as their meanings state, and check a new rule against VTI-EVT-190.
3. End the series before `from` and create the continuation from `from`, published as the series is, linked to it by `relatedTo` (`continues` on the continuation, `continuedBy` on the series) (VTI-EVT-197).
4. Carry into the continuation every registration that concerns its occurrences — a whole-series registration as a whole-series registration, an occurrence registration with its occurrence — keeping its state, guests and answers (VTI-EVT-197).
5. Revise every notification of the series for the change (VTI-EVT-198), and record the split in its audit trail (VTI-EVT-091).

## Authorization

**The event's hosts** (VTI-EVT-070). Never a member portal's delegated key.

## Definitions

- `Event`, `EventRecurrence`, `When` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Start an hour later from January

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/series/split/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
    "from": "2027-01-07T18:00:00",
    "when": {
      "start": "2027-01-07T19:00:00",
      "timeZone": "Europe/London",
      "duration": "PT3H"
    },
    "reason": "From January we start an hour later."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/series/split/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Split

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/series/split/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "ended": {
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
            "until": "2027-01-06T23:59:59"
          }
        ]
      },
      "relatedTo": [
        {
          "eventId": "evt_Hs7Qm2Lk9Tn4Bv1XAAAA",
          "relation": "continuedBy"
        }
      ],
      "createdAt": "2026-11-20T09:00:00Z",
      "updatedAt": "2026-11-28T10:00:01Z"
    },
    "continuation": {
      "eventId": "evt_Hs7Qm2Lk9Tn4Bv1XAAAA",
      "calendarId": "cal_W5k2N8xRb3Qz7TpLAAAA",
      "kind": "event",
      "status": "published",
      "title": "Thursday Build Night",
      "when": {
        "start": "2027-01-07T19:00:00",
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
      "sequence": 0,
      "recurrence": {
        "recurrenceRules": [
          {
            "frequency": "weekly",
            "byDay": [
              {
                "day": "th"
              }
            ],
            "count": 15
          }
        ]
      },
      "relatedTo": [
        {
          "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
          "relation": "continues"
        }
      ],
      "createdAt": "2026-11-28T10:00:01Z",
      "updatedAt": "2026-11-28T10:00:01Z"
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

Changes in; both series out, as a host sees them.

### Correlation

The community maintainer is addressed by its public DID.

### Retention

Both series are kept; the audit record names the host and both series.

### Consent/purpose

Hosts change a series from one point onward without rewriting its past (VTI-EVT-197). A change confers nothing (VTI-EVT-002).
