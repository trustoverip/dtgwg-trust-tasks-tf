---
slug: vtc/events/invitations/send
version: '0.1'
title: VTC Events — Invitations — Send
summary: 'Invite members to an event by name: a host invites any active member, and the invitation admits them to the audience; where the event allows it, a member of the audience invites another member of it.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- invitation
parties:
- role: event host or member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'An invitation is sent in the inviter''s name and, from a host, admits the invitee to the event''s audience; it must be attributable to the inviter after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed invitation would invite again someone a host has since revoked.'
sideEffects:
  level: mutating
  rationale: 'Records one invitation per invitee and has the community send each an `event` notification. Recoverable: a host revokes an invitation; an invitee declines it.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'The request names entries; the response is the invitations, by reference.'
retention:
  class: durable
  rationale: 'An invitation stands until it is revoked or the event ends (VTI-EVT-102).'
errorCodes:
- code: vtc/events/invitations/send:notPublished
  meaning: 'The event is a draft; invitations go out once it is published. Nothing was recorded.'
  retryable: false
- code: vtc/events/invitations/send:eventCancelled
  meaning: 'The event is cancelled. Nothing was recorded.'
  retryable: false
- code: vtc/events/invitations/send:notEligible
  meaning: 'An entry is not an active member of the community. Nothing was recorded.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      entryId:
        type: string
        maxLength: 128
- code: vtc/events/invitations/send:notInAudience
  meaning: 'A member''s invitation names someone the event''s audience does not admit; a member''s invitation never widens it (VTI-EVT-105). Nothing was recorded.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      entryId:
        type: string
        maxLength: 128
- code: vtc/events/invitations/send:membersMayNotInvite
  meaning: 'The signer is not a host and the event does not let members invite (VTI-EVT-105).'
  retryable: false
- code: vtc/events/invitations/send:limitReached
  meaning: 'The signer has made as many invitations today as the community allows a member (VTI-EVT-105). Nothing was recorded.'
  retryable: true
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/invitations/list
- vtc/events/invitations/respond
- vtc/events/invitations/links/create
---

## Abstract

The **VTC Events — Invitations — Send** Trust Task invites members to an event by name. A host may invite any active member, and the invitation admits that member to the event's audience while it stands, so an event can be seen only by those its hosts invite (VTI-EVT-103). Where the event sets `membersMayInvite`, a member of its audience may invite other members of it, within the community's daily limit, never widening the audience (VTI-EVT-105). Each invitee is sent one `event` notification, carrying the event's identifier and period and nothing else (VTI-NTF-064). An invitation registers nobody (VTI-EVT-100).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the signer may not see (VTI-EVT-023), and while events are turned off (VTI-EVT-007).
2. Refuse a draft with `notPublished` and a cancelled event with `eventCancelled`.
3. From a signer who does not host the event: refuse with `membersMayNotInvite` unless the event sets `membersMayInvite`; refuse an entry the audience does not admit with `notInAudience`; and count the signer's invitations against the community's daily limit, refusing with `limitReached` (VTI-EVT-105).
4. Refuse an entry that is not an active member with `notEligible`. Refuse the whole request on any refusal: record nothing.
5. Record one invitation per invitee, recording it before anything is sent, and answer an existing one — not a second — for a member already invited (VTI-EVT-102). Never invite a member who holds a registration.
6. Treat a member a host invited as in the event's audience while the invitation stands (VTI-EVT-103).
7. Send each invitee an `event` notification from the one place that retries it (VTI-EVT-102, VTI-NTF-064), never from the task.
8. Audit each invitation by reference: the event, the invitee's entry and who invited (VTI-EVT-091).

## Authorization

**A host, or a member of the audience where the event allows it.** A host: `vtc.events.manage` covering the event (VTI-EVT-070). A member: an active member the event's audience admits, the event setting `membersMayInvite`. The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A member's invitation is the member's act and **MAY** be signed by the portal's key for the member (VTI-EVT-095); a host's is not.

## Definitions

- `EventId` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventInvitation` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The event host or member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Invite two members

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000501",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/send/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "entryIds": [
      "dir_AdaLovelace0000000"
    ]
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/invitations/send/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Invite two members — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000502",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/send/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000501",
  "payload": {
    "invitations": [
      {
        "invitationId": "inv_W5k2N8xRb3Qz7TpL",
        "eventId": "evt_IntroSoldering2611",
        "entryId": "dir_AdaLovelace0000000",
        "invitedBy": "host",
        "inviterEntryId": "dir_PriyaNatarajan000",
        "replyExpected": false,
        "answer": "none",
        "createdAt": "2026-11-20T10:00:01Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

An event and the entries invited; the invitations, by reference. No title, place or roster.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

**Durable** until the invitation is revoked, or the event ends; the record of who was invited by whom is audited.

### Consent/purpose

Telling members an event is for them. An invitation is a pointer: the event says what it is to the invitee when they open it, and nothing about the event travels in the notification (VTI-NTF-064).
