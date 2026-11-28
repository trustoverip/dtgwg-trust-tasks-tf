---
slug: vtc/events/meeting/actions/list
version: '0.1'
title: VTC Events — Meeting — Actions — List
summary: 'List action items: a meeting''s, or the caller''s own across every meeting.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: member, assignee or host
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
  rationale: 'Reads action items; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'Discloses action items the caller may see, with assignees by entry.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes:
- code: vtc/events/meeting/actions/list:filterInvalid
  meaning: 'Neither or both of `eventId` and `assignedToMe`.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/meeting/actions/add
- vtc/events/meeting/actions/update
---

## Abstract

The **VTC Events — Meeting — Actions — List** Trust Task lists action items (VTI-EVT-266): one meeting's, as the caller may see them, or the caller's own across every meeting — which their agent reads to show them in their calendar (VTI-CAL-024).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse neither or both of `eventId` and `assignedToMe: true` with `filterInvalid`.
2. For a meeting: answer `notFound` alike for one that does not exist and one the caller may not see (VTI-EVT-023); answer its action items to its hosts, chair and secretary, and to registrants who are going on a `visible` roster; to anyone else only the items assigned to them (VTI-EVT-266).
3. For `assignedToMe`: answer the caller's own action items only.

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Standing to see the meeting, or the caller's own items** (VTI-EVT-266). A member's read: a portal holding a key the member authorized for `vtc/events` **MAY** sign it (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The member, assignee or host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/actions/list/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "assignedToMe": true,
    "statuses": [
      "open"
    ]
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/meeting/actions/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/actions/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "items": [
      {
        "actionId": "act_ShedQuotes0000001",
        "eventId": "evt_AGM2026HarbourRowing",
        "title": "Get three quotes for the boat shed",
        "assignee": "dir_KwameOsei000000000",
        "dueAt": "2027-01-15T17:00:00Z",
        "status": "open",
        "createdAt": "2026-12-05T12:30:00Z",
        "updatedAt": "2026-12-05T12:30:00Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

A filter; action items.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Seeing what one has to do.
