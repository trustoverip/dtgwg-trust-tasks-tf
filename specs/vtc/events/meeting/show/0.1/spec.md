---
slug: vtc/events/meeting/show
version: '0.1'
title: VTC Events — Meeting — Show
summary: 'Read a meeting''s agenda, chair and secretary, quorum, motions and minutes.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: member, registrant or host
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
sideEffects:
  level: none
  rationale: 'Reads a meeting''s records; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'Discloses a meeting''s agenda, roles by entry, quorum counts, motions by election, and the state of its minutes.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes:
- code: vtc/events/meeting/show:notAMeeting
  meaning: 'The event is not of kind `meeting`; this task applies to meetings only (VTI-EVT-255).'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/meeting/agenda/update
- vtc/events/meeting/quorum/show
- vtc/events/show
---

## Abstract

The **VTC Events — Meeting — Show** Trust Task reads what a meeting holds beyond its event (VTI-EVT-255 – 266): its agenda, who chairs and who takes the minutes, where quorum stands, the motions it put to elections, and its minutes.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023), and refuse an event of any kind but `meeting` with `notAMeeting` (VTI-EVT-255).
2. Answer the agenda, the chair and secretary, the motions and the state of the minutes to anyone who may see the meeting; the quorum counts to its hosts and to registrants who are going (VTI-EVT-261).
3. Answer each motion's election by identifier only; its standing is the election's to answer (VTI-EVT-262).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Standing to see the meeting** (VTI-EVT-021). A portal holding a key the member authorized for `vtc/events` **MAY** sign it (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The member, registrant or host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/show/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/meeting/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "agenda": [
      {
        "itemId": "agi_Budget2027Item0001",
        "title": "2027 budget",
        "owner": "dir_KwameOsei000000000",
        "timeboxMinutes": 20
      }
    ],
    "chair": "dir_MaryOkafor00000000",
    "secretary": "dir_LenaBrandt00000000",
    "quorumRule": {
      "of": "role",
      "role": "member",
      "fractionPercent": 50
    },
    "quorum": {
      "present": 23,
      "eligible": 40,
      "required": 20,
      "met": true
    },
    "motions": []
  }
}
```

## Security & Privacy

### Data carried

A meeting; its agenda, roles, quorum, motions and minutes.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Following a meeting.
