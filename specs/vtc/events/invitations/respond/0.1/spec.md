---
slug: vtc/events/invitations/respond
version: '0.1'
title: VTC Events — Invitations — Respond
summary: 'An invited member answers: going or maybe, which registers them as vtc/events/registrations/register would, or decline, which records that they will not come and registers nothing.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- invitation
- rsvp
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'An answer registers the member or records their refusal in their name; it must be attributable to them after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed answer would re-register a member who had since withdrawn.'
sideEffects:
  level: mutating
  rationale: 'Records the answer on the invitation and, for going or maybe, creates or changes the member''s registration. Recoverable with vtc/events/registrations/withdraw, or by answering again.'
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: 'For going or maybe, the request may carry answers about the member as a person — dietary and access needs — held for the event''s hosts alone (VTI-EVT-034).'
retention:
  class: durable
  rationale: 'The answer stands with the invitation; a registration stands as vtc/events/registrations/register''s does.'
errorCodes:
- code: vtc/events/invitations/respond:notInvited
  meaning: 'The caller holds no invitation to this event that stands (none, or revoked).'
  retryable: false
- code: vtc/events/invitations/respond:eventCancelled
  meaning: 'The event is cancelled; nothing was recorded.'
  retryable: false
- code: vtc/events/invitations/respond:notOpen
  meaning: 'Going or maybe on an event that takes no registrations (`closed`, VTI-EVT-031).'
  retryable: false
- code: vtc/events/invitations/respond:windowClosed
  meaning: 'Going or maybe outside the event''s registration window (VTI-EVT-038).'
  retryable: false
- code: vtc/events/invitations/respond:full
  meaning: 'Going on an event that is full and keeps no waitlist (VTI-EVT-033).'
  retryable: true
- code: vtc/events/invitations/respond:tooManyGuests
  meaning: 'More guests than the event allows.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      plusOnes:
        type: integer
        minimum: 0
- code: vtc/events/invitations/respond:answersInvalid
  meaning: 'The answers do not fit the event''s questions (VTI-EVT-034).'
  retryable: false
- code: vtc/events/invitations/respond:codeOfConductRequired
  meaning: 'The event''s code of conduct was not acknowledged by its digest (VTI-EVT-035).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      sha256:
        type: string
        pattern: '^[0-9a-f]{64}$'
- code: vtc/events/invitations/respond:maybeNotAccepted
  meaning: 'Maybe on an event whose hosts decide each registration.'
  retryable: false
- code: vtc/events/invitations/respond:hostDecided
  meaning: 'A host declined or removed the caller''s registration; it is not re-opened by the registrant.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/invitations/list
- vtc/events/registrations/register
- vtc/events/registrations/withdraw
---

## Abstract

The **VTC Events — Invitations — Respond** Trust Task answers an invitation (VTI-EVT-104). `going` and `maybe` register the member exactly as [`vtc/events/registrations/register`](../../../registrations/register/0.3/spec.md) does — the same checks, the same receipt — and record the answer on the invitation. `decline` records on the invitation that the member will not come, and registers nothing: it is not a host's decline of a registration, and is never shown as one. A member who declined may answer again, or register, later.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023); refuse a caller with no invitation that stands with `notInvited`, and a cancelled event with `eventCancelled`.
2. For `going` or `maybe`: register the caller exactly as `vtc/events/registrations/register` item by item, refusing with this task's codes of the same names, and answer the registration as the receipt (VTI-EVT-092).
3. For `decline`: record the answer and `answeredAt` on the invitation; change no registration; never record `declined` as a registration status (VTI-EVT-104).
4. Answer the invitation as it then stands, and the event as `EventRef`.
5. Audit the answer by reference (VTI-EVT-091), never the registrant's answers to the event's questions.

## Authorization

**Standing in the community.** The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and the community reads their standing from its own records at execution time. A member's act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `EventId` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventInvitation` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `Registration` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `RegistrationAnswers` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventRef` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Accept an invitation

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000521",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/respond/0.1#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "answer": "going"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/invitations/respond/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Accept an invitation — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000522",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/respond/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000521",
  "payload": {
    "invitation": {
      "invitationId": "inv_W5k2N8xRb3Qz7TpL",
      "eventId": "evt_IntroSoldering2611",
      "entryId": "dir_AdaLovelace0000000",
      "invitedBy": "host",
      "inviterEntryId": "dir_PriyaNatarajan000",
      "replyExpected": false,
      "answer": "going",
      "createdAt": "2026-11-20T10:00:01Z",
      "answeredAt": "2026-11-20T10:00:01Z"
    },
    "registration": {
      "registrationId": "reg_Q7m2K9xVb4Tz8LpR",
      "eventId": "evt_IntroSoldering2611",
      "entryId": "dir_AdaLovelace0000000",
      "status": "going",
      "guests": 0,
      "roster": "hidden",
      "eventSequence": 0,
      "createdAt": "2026-11-20T10:00:01Z",
      "updatedAt": "2026-11-20T10:00:01Z"
    },
    "event": {
      "eventId": "evt_IntroSoldering2611",
      "title": "Intro to Soldering",
      "when": {
        "start": "2026-11-30T18:30:00",
        "timeZone": "Europe/London",
        "duration": "PT1H30M"
      },
      "status": "published",
      "sequence": 0
    }
  }
}
```

## Security & Privacy

### Data carried

An event and an answer; for going or maybe, what vtc/events/registrations/register carries.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

**Durable**: the answer stands with the invitation, and a registration as registrations/register's does; answers to the event's questions are deleted on the event's retention schedule (VTI-EVT-090).

### Consent/purpose

Answering an invitation. Saying no is recorded so hosts who expect replies stop waiting, and never reads as a host turning the member away.
