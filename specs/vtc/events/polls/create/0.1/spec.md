---
slug: vtc/events/polls/create
version: '0.1'
title: VTC Events — Polls — Create
summary: 'Someone who runs a calendar opens a scheduling poll: candidate times put to an audience, answered yes, if need be or no.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: calendar runner
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'What the caller may do or see depends on who asks; the request must be attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would repeat a change.'
sideEffects:
  level: mutating
  rationale: 'Stores the poll.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'A poll''s title, times and audience; the poll.'
retention:
  class: durable
  rationale: 'A poll is kept until closed, and its answers until the community''s retention removes them.'
errorCodes:
- code: vtc/events/polls/create:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007).'
  retryable: false
- code: vtc/events/polls/create:calendarArchived
  meaning: 'The calendar is archived.'
  retryable: false
- code: vtc/events/polls/create:slotsInvalid
  meaning: 'A candidate time is malformed, in an unknown zone, or named twice.'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"slotId": {"type": "string", "maxLength": 128}, "reason": {"type": "string", "maxLength": 500}}}
- code: vtc/events/polls/create:audienceInvalid
  meaning: 'The audience is malformed or names a group that does not exist.'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"reason": {"type": "string", "maxLength": 500}}}
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/polls/show
- vtc/events/polls/answer
- vtc/events/polls/close
---

## Abstract

The **VTC Events — Polls — Create** Trust Task opens a scheduling poll (VTI-EVT-270): candidate times put to an audience, each answered yes, if need be or no, closed later into an event at the time chosen. Whether answerers see who answered what is fixed at creation (VTI-EVT-271).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not run the calendar (VTI-EVT-013) with `permissionDenied`; answer `notFound` for a calendar the caller may not see.
2. Refuse an archived calendar with `calendarArchived`, candidate times that are malformed, in an unknown zone or named twice with `slotsInvalid`, and an audience as `vtc/events/create` refuses one with `audienceInvalid` (VTI-EVT-020).
3. Fix `voters` for the poll's life (VTI-EVT-271).
4. Audit the poll by reference (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Authority over the calendar** (VTI-EVT-013). Never from a portal-held key (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The calendar runner (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/polls/create/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "calendarId": "cal_HarbourRowingMain",
    "kind": "meeting",
    "title": "AGM 2026",
    "slots": [
      {
        "slotId": "sat-5",
        "when": {
          "start": "2026-12-05T10:00:00",
          "timeZone": "Europe/London",
          "duration": "PT3H"
        }
      },
      {
        "slotId": "sun-6",
        "when": {
          "start": "2026-12-06T14:00:00",
          "timeZone": "Europe/London",
          "duration": "PT3H"
        }
      }
    ],
    "voters": "hidden"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:mary#key-1",
    "created": "2026-12-05T10:40:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/polls/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/polls/create/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "poll": {
      "pollId": "pol_AGM2026TimesPoll01",
      "calendarId": "cal_HarbourRowingMain",
      "kind": "meeting",
      "title": "AGM 2026",
      "slots": [
        {
          "slotId": "sat-5",
          "when": {
            "start": "2026-12-05T10:00:00",
            "timeZone": "Europe/London",
            "duration": "PT3H"
          }
        },
        {
          "slotId": "sun-6",
          "when": {
            "start": "2026-12-06T14:00:00",
            "timeZone": "Europe/London",
            "duration": "PT3H"
          }
        }
      ],
      "audience": {
        "scope": "members"
      },
      "voters": "hidden",
      "state": "open",
      "createdAt": "2026-11-20T09:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

A calendar, a title, candidate times, an audience and the voters setting; the poll.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable** until closed; answers until the community's retention removes them.

### Consent/purpose

Finding a time that suits.
