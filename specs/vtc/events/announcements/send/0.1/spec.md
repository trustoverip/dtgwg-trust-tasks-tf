---
slug: vtc/events/announcements/send
version: '0.1'
title: VTC Events — Announcements — Send
summary: 'A host posts an announcement to an event''s registrants. It never moves the event, and its text is never in a notification.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- announcement
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
  rationale: 'An announcement speaks for the event''s hosts to its registrants; it must be attributable to the host who posted it.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed announcement would post twice.'
sideEffects:
  level: mutating
  rationale: 'Stores the announcement and marks the event changed for registrants'' agents. Not recoverable: a host posts a correction.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'The announcement''s text, which a host writes.'
retention:
  class: durable
  rationale: 'Kept with the event''s records.'
errorCodes:
- code: vtc/events/announcements/send:notPublished
  meaning: 'The event is a draft; announcements are for registrants, and a draft has none.'
  retryable: false
- code: vtc/events/announcements/send:limitReached
  meaning: 'The event has had as many announcements today as the community allows. Nothing was posted.'
  retryable: true
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/announcements/list
- vtc/events/update
---

## Abstract

The **VTC Events — Announcements — Send** Trust Task posts a host's announcement to an event — "bring a laptop", "we start fifteen minutes late" (VTI-EVT-120). It never changes the event's time, place or status, which a host changes with [`vtc/events/update`](../../../update/0.3/spec.md) so members' calendars stay right (VTI-EVT-121). Registrants read it with [`vtc/events/announcements/list`](../../list/0.1/spec.md); the community marks the event changed for their agents and puts the text in no notification.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 22, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller does not host and may not see (VTI-EVT-023).
2. Refuse a draft with `notPublished`; accept a cancelled event, so its hosts can say why.
3. Refuse beyond the community's daily limit per event with `limitReached`.
4. Store the announcement with the posting host's entry, and mark the event changed so its registrants' agents read it at their next sync (VTI-EVT-121).
5. Never put an announcement's text in a notification, and never change the event's `sequence` for one.
6. Audit by reference, never the text.

## Authorization

**Authority: `vtc.events.manage` covering the event** — a host's grant qualified `event:<eventId>`, or the authority over the event's calendar (VTI-EVT-013, VTI-EVT-070) — read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A host's act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-EVT-095).

## Definitions

- `EventId` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventAnnouncement` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Post an announcement

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000581",
  "type": "https://trusttasks.org/spec/vtc/events/announcements/send/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "text": "Bring a laptop if you have one — we will flash the badge's firmware."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-20T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/announcements/send/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Post an announcement — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000582",
  "type": "https://trusttasks.org/spec/vtc/events/announcements/send/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000581",
  "payload": {
    "announcement": {
      "announcementId": "ann_FirmwareLaptops00",
      "eventId": "evt_IntroSoldering2611",
      "text": "Bring a laptop if you have one — we will flash the badge's firmware.",
      "postedBy": "dir_PriyaNatarajan000",
      "createdAt": "2026-11-20T10:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

An event and a host's text.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

**Durable** with the event's records.

### Consent/purpose

Telling the people coming something they need, without a channel of its own.
