---
slug: vtc/events/attachments/list
version: '0.1'
title: VTC Events — Attachments — List
summary: 'Read the attachments of an event the caller may see: materials only to registrants who are going and to hosts.'
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
  rationale: 'Reads the event''s attachments; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'Discloses the attachments the caller may see, a material''s address only to a registrant who is going.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/attachments/add
- vtc/events/access/show
---

## Abstract

The **VTC Events — Attachments — List** Trust Task reads an event's attachments. A material is released to a registrant who is going, as access details are (VTI-EVT-243); the rest are shown to anyone who may see the event.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023).
2. Answer a `material` only to a registrant who is going and to hosts (VTI-EVT-243); to anyone else leave it out, as access details are left out (VTI-EVT-060).
3. Answer every other attachment to anyone who may see the event.

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Standing to see the event** (VTI-EVT-021); a material only to a registrant who is going or a host. A member's read: a portal holding a key the member authorized for `vtc/events` **MAY** sign it (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The member, registrant or host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/attachments/list/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/attachments/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/attachments/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "items": [
      {
        "attachmentId": "att_KitListSoldering01",
        "eventId": "evt_IntroSoldering2611",
        "kind": "material",
        "material": {
          "name": "Kit list",
          "url": "https://riverside-makers.example/files/kit-list.pdf"
        },
        "attachedAt": "2026-11-18T10:00:01Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

An event; its attachments as the caller may see them.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Finding an event's room, board, election and materials.
