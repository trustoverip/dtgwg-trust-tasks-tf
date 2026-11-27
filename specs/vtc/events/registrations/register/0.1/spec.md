---
slug: vtc/events/registrations/register
version: '0.1'
title: VTC Events — Registrations — Register
summary: 'A member registers for an event, or changes their registration — going or maybe, guests, answers, the code of conduct acknowledged. The community decides whether that makes them going, pending or waitlisted, and its signed answer is the member''s receipt.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- registration
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
  rationale: A registration takes a place in the member's name and carries their answers; it must be attributable to the member after the transport has closed, and the community's signed answer is only a receipt if the request it answers is.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed registration would take back a place the member had since given up.
sideEffects:
  level: mutating
  rationale: Creates or changes the member's registration and may take or free a place, moving the waitlist. Recoverable with vtc/events/registrations/withdraw.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: The request may carry answers about the member as a person — dietary needs, access needs — which the community holds for the event's hosts alone and deletes on the event's retention schedule. The response is the member's own registration and a summary of the event.
retention:
  class: durable
  rationale: The registration stands until withdrawn, declined or removed, and its status is kept after the event; its answers are deleted a community-set period after the event ends (VTI-EVT-090).
errorCodes:
- code: vtc/events/registrations/register:notOpen
  meaning: The event takes no registrations — it is in mode `closed`, or not yet published. Nothing was stored.
  retryable: false
- code: vtc/events/registrations/register:eventCancelled
  meaning: The event is cancelled and takes no registrations. Nothing was stored.
  retryable: false
- code: vtc/events/registrations/register:windowClosed
  meaning: Outside the event's registration window — before `opensAt`, or after `closesAt` (by default the event's start). Withdrawal is still possible. Nothing was stored.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      opensAt:
        type: string
        format: date-time
      closesAt:
        type: string
        format: date-time
- code: vtc/events/registrations/register:full
  meaning: Going (with the guests asked for) would exceed the event's capacity and it keeps no waitlist. Nothing was stored; a registration held is unchanged.
  retryable: true
- code: vtc/events/registrations/register:tooManyGuests
  meaning: '`guests` is more than the event''s `plusOnes`.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - plusOnes
    properties:
      plusOnes:
        type: integer
        minimum: 0
- code: vtc/events/registrations/register:answersInvalid
  meaning: A required question is unanswered, an answer names a question the event does not ask, or an answer does not fit its question's kind or choices.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      questionId:
        type: string
        maxLength: 32
      reason:
        type: string
        maxLength: 500
- code: vtc/events/registrations/register:codeOfConductRequired
  meaning: The event names a code of conduct and `codeOfConductAcknowledged` is absent or is not its current `sha256`. Read the code at its `url` and acknowledge that one.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - url
    - sha256
    properties:
      url:
        type: string
        format: uri
        maxLength: 2048
      sha256:
        type: string
        pattern: ^[0-9a-f]{64}$
- code: vtc/events/registrations/register:maybeNotAccepted
  meaning: The event is in mode `approval`, where every registration waits for a host's decision; `maybe` is not offered. Register `going`.
  retryable: false
- code: vtc/events/registrations/register:hostDecided
  meaning: A host declined or removed the member's registration; the member cannot re-open it. The registration is unchanged.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - status
    properties:
      status:
        type: string
        enum:
        - declined
        - removed
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/registrations/withdraw
- vtc/events/registrations/decide
- vtc/events/show
- vtc/events/mine/list
---

## Abstract

The **VTC Events — Registrations — Register** Trust Task is how a member says they are coming to an event, or that they might. It creates the member's registration, or changes the one they hold — from maybe to going, to bring a guest, to change an answer — since a member has at most one registration per event (VTI-EVT-030).

The member says `going` or `maybe`. What that makes them is the community's to decide, by the event's settings (VTI-EVT-031 – 033): `going` at once when the event is `open` and has room; `waitlisted` when it is full and keeps a waitlist; `pending` when the event is `approval`, until a host decides with [`vtc/events/registrations/decide`](../../decide/0.1/spec.md). `maybe` holds no place.

The community's signed answer is the member's **receipt** (VTI-EVT-092): the registration as it now stands, with the roster setting it was made under, and the event as it stood — title, time, status, `sequence`. The member's agent keeps it. It is evidence of what the community said, and confers nothing (VTI-EVT-002).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decisions 16 and 21.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Answer `notFound` alike for an event that does not exist, a draft, and an event whose audience does not admit the caller, reading the audience at execution time and never from a notification the caller holds (VTI-EVT-020, VTI-EVT-021, VTI-EVT-023); and while the community has events turned off (VTI-EVT-007).
3. Refuse a cancelled event with `eventCancelled`, an event in mode `closed` with `notOpen` (VTI-EVT-031), and a request outside the registration window — before `opensAt`, or after `closesAt` or, without one, the event's start — with `windowClosed` (VTI-EVT-038).
4. Refuse a caller whose registration a host `declined` or `removed` with `hostDecided`; the registrant cannot re-open a host's decision. A `withdrawn` registration is the member's own and is re-opened by registering again.
5. Validate the request against the event before changing anything: `guests` at most `plusOnes` (`tooManyGuests`); every required question answered, no answer to a question the event does not ask, each answer of its question's kind and, for `choice`, one of its `choices` (`answersInvalid`, VTI-EVT-034); where the event names a code of conduct, `codeOfConductAcknowledged` equal to its current `sha256` (`codeOfConductRequired`, VTI-EVT-035). On a change of an existing registration, a held acknowledgement of an earlier `sha256` is kept and not asked again unless the member sends a new one.
6. Decide the status, counting `capacity` over registrations that are `going` and their guests (VTI-EVT-033):
   - In mode `open`, `going` is `going` where it fits, else `waitlisted` (last in the queue) where the event keeps a waitlist, else refused with `full` and nothing stored. `maybe` is `maybe` and takes no place.
   - In mode `approval`, a new registration is `pending` and takes no place until a host decides; `maybe` is refused with `maybeNotAccepted`. A registration already `going` stays `going`.
   - A registration already `going` that asks for more guests than now fit is refused with `full` and left as it was. One that goes to `maybe`, or asks for fewer guests, frees places, and the community **MUST** then move the waitlist: the first `waitlisted` registration that fits becomes `going`, then the next, in order (VTI-EVT-033).
   - A `waitlisted` registration asking again for `going` keeps its place in the queue.
7. Record on the registration the event's roster setting at the time (VTI-EVT-052) and its `sequence` (`eventSequence`), and on a change keep the `registrationId`.
8. Answer the registration and an `EventRef` of the event, with `wasCreated` true where this request created the registration. The answer is the member's receipt (VTI-EVT-092) and **MUST** be signed by the community as any Trust Task response is.
9. Hold `answers` for the registrant and the event's hosts alone (VTI-EVT-034), delete them on the event's retention schedule (VTI-EVT-090), and audit the registration by reference — event, entry, status — never with its answers (VTI-EVT-091).
10. Send no reminder of its own (VTI-EVT-040). The event's `suggestedReminderMinutes` are for the member's agent.

A repeated request with the same content changes nothing and answers the registration as it stands, with `wasCreated` false.

## Authorization

**Membership of the event's audience.** Any active member the event's audience admits may register, read at execution time from the audience's scope and rule (VTI-EVT-020); a notification the member holds is never evidence that they may (VTI-EVT-021). The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and the registration is that member's alone. A registration confers nothing (VTI-EVT-002).

Registering is an act of the member (VTI-EVT-095): a member portal that holds a key the member authorized to sign for them for `vtc/events` (`auth/oob` `signingScope`) **MAY** sign it.

## Definitions

- `EventId`, `Registration`, `RegistrationStatus`, `EventRef`, `Answers`, `CodeOfConduct` — see [`event.schema.json`](../../../_shared/0.1/event.schema.json).
- **Receipt** — the community's signed answer to a registration, withdrawal or decision, naming the registration and the event as they stood (VTI-EVT-092).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Going, with a guest

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000421",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/register/0.1#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T19:02:10Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "status": "going",
    "guests": 1,
    "answers": { "dietary": "Vegetarian", "step-free": true },
    "codeOfConductAcknowledged": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-20T19:02:10Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/registrations/register/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Going, with a guest — the event is in approval mode, so pending

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000422",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/register/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T19:02:11Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000421",
  "payload": {
    "registration": {
      "registrationId": "reg_Kf3nP8qLs2Vx7Bd1",
      "eventId": "evt_IntroSoldering2611",
      "entryId": "dir_AdaLovelace000000",
      "status": "pending",
      "guests": 1,
      "answers": { "dietary": "Vegetarian", "step-free": true },
      "codeOfConductAcknowledged": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
      "roster": "hidden",
      "eventSequence": 1,
      "createdAt": "2026-11-20T19:02:11Z",
      "updatedAt": "2026-11-20T19:02:11Z"
    },
    "event": {
      "eventId": "evt_IntroSoldering2611",
      "title": "Intro to Soldering",
      "when": { "start": "2026-11-30T18:30:00", "timeZone": "Europe/London", "duration": "PT1H30M" },
      "status": "published",
      "sequence": 1
    },
    "wasCreated": true
  }
}
```

### Refused: full, no waitlist

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000423",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T19:02:11Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000421",
  "payload": {
    "code": "vtc/events/registrations/register:full",
    "message": "Intro to Soldering is full and keeps no waitlist.",
    "retryable": true
  }
}
```

## Security & Privacy

### Data carried

An event, a status, a guest count, answers and a code-of-conduct digest in; the member's registration and a summary of the event out. Answers may be about the member as a person — dietary and access needs — and are the one personal data this family carries in.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. The member registers as their identity at this community; nothing in the request names any other community or any other identity of theirs. Who else is registered is never in the answer: other registrants are read, where the roster setting allows, with [`vtc/events/registrations/list`](../../list/0.1/spec.md) (VTI-EVT-053).

### Retention

**Durable** for the registration and its status. Answers are deleted a community-set period after the event ends, 30 days unless the community sets otherwise (VTI-EVT-090); the audit records the act by reference, never the answers (VTI-EVT-091).

### Consent/purpose

Taking part in an event, and telling its hosts what they need to know to host it. The roster setting the member registered under is recorded on the registration and cannot change after the event is published (VTI-EVT-051, VTI-EVT-052), so a member who registered for a `hidden` event is never on a visible list. **An event confers nothing**: the community **MUST NOT** read a registration in any authorization decision of its own (VTI-EVT-002).
