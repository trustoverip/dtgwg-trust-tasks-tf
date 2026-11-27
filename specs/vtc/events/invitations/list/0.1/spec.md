---
slug: vtc/events/invitations/list
version: '0.1'
title: VTC Events — Invitations — List
summary: 'List invitations: a host lists an event''s invitations and their answers; a member lists their own invitations across the community''s events.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- invitation
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
  rationale: 'Which invitations are shown depends on who asks; an unattributable caller gets nothing.'
sideEffects:
  level: none
  rationale: 'Reads invitation records; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'Invitations by reference: entries, who invited, the answer.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/invitations/send
- vtc/events/invitations/respond
---

## Abstract

The **VTC Events — Invitations — List** Trust Task reads invitations (VTI-EVT-106). Given an `eventId` by one of its hosts, it answers every invitation to the event — who was invited, by whom, and each answer — so that under `expect` the hosts see who has not answered. Without an `eventId`, or from anyone else, it answers the caller's own invitations, each with a summary of its event.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. With `eventId`, from a host of the event: answer its invitations, `revoked` ones included, oldest first, each with `inviterEntryId` where a host or member invited.
2. With `eventId`, from anyone else: answer only the caller's own invitation to it, if any; answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023).
3. Without `eventId`: answer the caller's own invitations across the community's events, newest first, never `revoked` ones, each with `event` as `EventRef`, and never `inviterEntryId` (VTI-EVT-106).
4. Filter by `answers` where given.
5. Never answer one member's invitation to another.

## Authorization

**Standing in the community.** The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and the community reads their standing from its own records at execution time. A member's act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it. A host's reading of an event's invitations is a host's read and **MUST NOT** be accepted from the portal's key.

## Definitions

- `EventId` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventInvitation` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventInvitationAnswer` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventRef` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The member or event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A host lists who has answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000511",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/list/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "answers": [
      "none"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/invitations/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A host lists who has answered — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000512",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/list/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000511",
  "payload": {
    "items": [
      {
        "invitation": {
          "invitationId": "inv_W5k2N8xRb3Qz7TpL",
          "eventId": "evt_IntroSoldering2611",
          "entryId": "dir_AdaLovelace0000000",
          "invitedBy": "host",
          "inviterEntryId": "dir_PriyaNatarajan000",
          "replyExpected": false,
          "answer": "none",
          "createdAt": "2026-11-20T10:00:01Z"
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Invitations by reference and, on a member's own listing, a summary of each event.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

A host following up an event they expect replies to; a member seeing what they have been invited to.
