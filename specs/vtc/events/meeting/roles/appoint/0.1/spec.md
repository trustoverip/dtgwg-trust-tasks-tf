---
slug: vtc/events/meeting/roles/appoint
version: '0.1'
title: VTC Events — Meeting — Roles — Appoint
summary: 'A host appoints an active member the meeting''s chair or secretary.'
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
  rationale: 'Writes or removes a resource grant for the role on the member''s access-control entry.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'Names a member by entry; answers the meeting''s chair and secretary by entry.'
retention:
  class: durable
  rationale: 'An appointment stands until removed, the meeting is deleted or its holder leaves the community.'
errorCodes:
- code: vtc/events/meeting/roles/appoint:notAMeeting
  meaning: 'The event is not of kind `meeting`; this task applies to meetings only (VTI-EVT-255).'
  retryable: false
- code: vtc/events/meeting/roles/appoint:notEligible
  meaning: 'The entry is not an active member of the community.'
  retryable: false
- code: vtc/events/meeting/roles/appoint:roleHeld
  meaning: 'Someone already holds the role; remove them first.'
  retryable: false
- code: vtc/events/meeting/roles/appoint:eventCancelled
  meaning: 'The meeting is cancelled.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/meeting/show
- vtc/events/hosts/appoint
---

## Abstract

The **VTC Events — Meeting — Roles — Appoint** Trust Task appoints a meeting's chair or secretary (VTI-EVT-257). The chair runs the meeting — its agenda, motions and the approval of its minutes; the secretary records the minutes and the action items. Each is held by appointment, as hosts are.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023), and refuse an event of any kind but `meeting` with `notAMeeting` (VTI-EVT-255).
2. Refuse a caller who does not host the meeting with `permissionDenied`, except that the holder of a role may end their own appointment (VTI-EVT-257).
3. Refuse an entry that is not an active member with `notEligible`, a role someone holds with `roleHeld`, and a cancelled meeting with `eventCancelled`.
4. Record the appointment as hosts' are recorded — a grant on the holder's access-control entry, reviewed when its granter departs — never as a field of the meeting (VTI-EVT-257).
5. Audit the change by reference (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Authority: hosting the meeting** (VTI-EVT-070); the holder of a role may end their own. Never from a portal-held key (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/roles/appoint/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "role": "chair",
    "entryId": "dir_MaryOkafor00000000"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/meeting/roles/appoint/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/roles/appoint/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "chair": "dir_MaryOkafor00000000"
  }
}
```

## Security & Privacy

### Data carried

A meeting, a role and a member; the meeting's role holders.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable**, as hosts' grants are.

### Consent/purpose

Running a meeting.
