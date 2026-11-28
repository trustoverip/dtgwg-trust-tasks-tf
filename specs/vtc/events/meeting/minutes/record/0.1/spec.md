---
slug: vtc/events/meeting/minutes/record
version: '0.1'
title: VTC Events — Meeting — Minutes — Record
summary: 'The secretary or the chair records a meeting''s minutes: a record in its room, or the text where it has none.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: secretary or chair
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
  rationale: 'Stores the minutes as a draft, replacing a draft held.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'A room record''s identifier or the minutes'' text; the minutes.'
retention:
  class: durable
  rationale: 'Minutes are kept with the meeting.'
errorCodes:
- code: vtc/events/meeting/minutes/record:notAMeeting
  meaning: 'The event is not of kind `meeting`; this task applies to meetings only (VTI-EVT-255).'
  retryable: false
- code: vtc/events/meeting/minutes/record:minutesApproved
  meaning: 'The minutes are approved and fixed; a correction is a later record (VTI-EVT-265).'
  retryable: false
- code: vtc/events/meeting/minutes/record:minutesInvalid
  meaning: 'Neither or both of `roomRecordId` and `text`, or `roomRecordId` on a meeting with no room.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/meeting/minutes/approve
---

## Abstract

The **VTC Events — Meeting — Minutes — Record** Trust Task records a meeting's minutes as a draft (VTI-EVT-265): a record the secretary wrote in the meeting's room, named here, or — for a meeting with no room — the text itself.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023), and refuse an event of any kind but `meeting` with `notAMeeting` (VTI-EVT-255).
2. Refuse a caller who is neither the meeting's secretary nor its chair with `permissionDenied` (VTI-EVT-265).
3. Refuse approved minutes with `minutesApproved`, and neither or both of the two members — or a room record on a meeting with no room — with `minutesInvalid`.
4. Keep the minutes in the meeting's room where it has one (VTI-EVT-265), naming the record; the community holds no room key and does not write the record itself.
5. Audit the change by reference, never the text (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Authority: the meeting's secretary or chair** (VTI-EVT-265). Never from a portal-held key (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The secretary or chair (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/minutes/record/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "roomRecordId": "rec_MinutesAGM2026"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/meeting/minutes/record/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/minutes/record/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "minutes": {
      "state": "draft",
      "roomRecordId": "rec_MinutesAGM2026",
      "recordedAt": "2026-12-05T13:10:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

A meeting and its minutes or where they are.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable**, with the meeting.

### Consent/purpose

Keeping a record of what a meeting decided.
