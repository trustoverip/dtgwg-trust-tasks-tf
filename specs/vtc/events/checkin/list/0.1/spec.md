---
slug: vtc/events/checkin/list
version: '0.1'
title: VTC Events — Check-in — List
summary: 'List an event''s check-ins for its door and its hosts: who came, how and when, and how many.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: door staff or host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Who came to an event is shown only to its door and its hosts.'
sideEffects:
  level: none
  rationale: 'Reads check-ins; persists nothing.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'An event in; its check-ins, by entry or registrant identifier, out.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/checkin/record
- vtc/events/registrations/list
---

## Abstract

The **VTC Events — Check-in — List** Trust Task lists an event's check-ins for its door staff and hosts (VTI-EVT-160, 165): each with the registrant (by entry, or by identifier for a guest or a partner's member), how they checked in and when, and the number checked in. There is no list of who did not come.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer only the event's door staff and hosts; anyone else as for an event they may not see.
2. Answer every check-in, earliest first, paged by `cursor` and `limit` as `vtc/events/registrations/list` pages.
3. Never answer a registration without a check-in as a no-show (VTI-EVT-165).

## Authorization

**The event's door**: `vtc.events.checkin` at `event:<eventId>`, or its hosts. Never signed by a portal key.

## Definitions

- `EventCheckIn` — see [`event.schema.json`](../../../_shared/0.3/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000005",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/list/0.1#request",
  "issuer": "did:example:dan",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
    "limit": 50
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:dan#key-1",
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
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000006",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:dan",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
    "items": [
      {
        "registrationId": "reg_Hk3Wq9Ls2Dn6Hy4B",
        "entryId": "dir_Pr1yaNatarajan00",
        "registrantKind": "member",
        "method": "challenge",
        "checkedInAt": "2026-11-30T18:25:01Z",
        "counts": true
      }
    ],
    "checkedIn": 1
  },
  "threadId": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000005"
}
```

## Security & Privacy

### Data carried

An event in; its check-ins out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the DID its members and guests already know it by. Members are named by their community-scoped `entryId`; guests and partners' members by the identifier they registered with, shown to hosts only (VTI-EVT-134, VTI-EVT-153).

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Running the door, and letting hosts see who came. **Events confer nothing**: the community never reads an event, a registration, a check-in, a ticket or an attendance in an authorization decision (VTI-EVT-002).
