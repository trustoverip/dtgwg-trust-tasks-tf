---
slug: vtc/events/announcements/list
version: '0.1'
title: VTC Events — Announcements — List
summary: 'Read an event''s announcements: its hosts, and its registrants who are going, maybe, pending or waitlisted, newest first.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- announcement
parties:
- role: member or event host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Whether announcements are shown depends on who asks; an unattributable caller gets nothing.'
sideEffects:
  level: none
  rationale: 'Reads announcements; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'A host''s announcements to the event''s registrants.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes:
- code: vtc/events/announcements/list:notRegistered
  meaning: 'The caller neither hosts the event nor holds a registration that is going, maybe, pending or waitlisted (VTI-EVT-120).'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/announcements/send
- vtc/events/mine/list
---

## Abstract

The **VTC Events — Announcements — List** Trust Task answers an event's announcements, newest first (VTI-EVT-120), to its hosts and to its registrants who are going, maybe, pending or waitlisted. A member's agent reads them when its sync shows the event changed.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 22, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023).
2. Refuse a caller who neither hosts the event nor holds a live registration with `notRegistered`.
3. Answer the announcements newest first, with the posting host's entry.

## Authorization

**Standing in the community.** The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and the community reads their standing from its own records at execution time. A member's act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `EventId` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventAnnouncement` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The member or event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read the announcements

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000591",
  "type": "https://trusttasks.org/spec/vtc/events/announcements/list/0.1#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-20T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/announcements/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Read the announcements — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000592",
  "type": "https://trusttasks.org/spec/vtc/events/announcements/list/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000591",
  "payload": {
    "items": [
      {
        "announcementId": "ann_FirmwareLaptops00",
        "eventId": "evt_IntroSoldering2611",
        "text": "Bring a laptop if you have one — we will flash the badge's firmware.",
        "postedBy": "dir_PriyaNatarajan000",
        "createdAt": "2026-11-20T10:00:01Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

An event; its announcements.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Registrants reading what their hosts told them.
