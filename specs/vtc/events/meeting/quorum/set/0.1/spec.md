---
slug: vtc/events/meeting/quorum/set
version: '0.1'
title: VTC Events — Meeting — Quorum — Set
summary: 'A host sets or clears a meeting''s quorum rule, before the meeting starts.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: event host
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
  rationale: 'Stores or clears the meeting''s quorum rule.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'A quorum rule; the rule and where quorum stands.'
retention:
  class: durable
  rationale: 'The rule is kept with the meeting.'
errorCodes:
- code: vtc/events/meeting/quorum/set:notAMeeting
  meaning: 'The event is not of kind `meeting`; this task applies to meetings only (VTI-EVT-255).'
  retryable: false
- code: vtc/events/meeting/quorum/set:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007).'
  retryable: false
- code: vtc/events/meeting/quorum/set:meetingStarted
  meaning: 'The meeting has started; its quorum rule is fixed (VTI-EVT-260).'
  retryable: false
- code: vtc/events/meeting/quorum/set:quorumInvalid
  meaning: 'Not exactly one of `count` and `fractionPercent`, or `role` present without `of: role` or absent with it.'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"reason": {"type": "string", "maxLength": 500}}}
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/meeting/quorum/show
- vtc/events/meeting/motions/open
---

## Abstract

The **VTC Events — Meeting — Quorum — Set** Trust Task sets how many must be present for a meeting to decide (VTI-EVT-260): a count or a percentage, of the meeting's roster or of the community's members holding a role. It is fixed once the meeting starts.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023), and refuse an event of any kind but `meeting` with `notAMeeting` (VTI-EVT-255).
2. Refuse a caller who does not host the meeting with `permissionDenied`, and a meeting that has started with `meetingStarted` (VTI-EVT-260).
3. Refuse a rule that is not exactly one of a count and a percentage, or whose `role` does not match `of`, with `quorumInvalid`.
4. Audit the change by reference (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Authority: hosting the meeting** (VTI-EVT-070). Never from a portal-held key (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/quorum/set/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "quorumRule": {
      "of": "role",
      "role": "member",
      "fractionPercent": 50
    }
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/meeting/quorum/set/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/quorum/set/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "quorumRule": {
      "of": "role",
      "role": "member",
      "fractionPercent": 50
    },
    "quorum": {
      "present": 0,
      "eligible": 40,
      "required": 20,
      "met": false
    }
  }
}
```

## Security & Privacy

### Data carried

A meeting and a quorum rule; the rule and the counts.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable**, with the meeting.

### Consent/purpose

Deciding when a meeting may decide.
