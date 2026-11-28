---
slug: vtc/events/guests/register
version: '0.1'
title: VTC Events — Guests — Register
summary: 'A guest — a party who is not a member — registers for an event that admits guests, or changes their registration. The reply is their receipt.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: guest
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'A guest is known only by the identifier that signs; the registration is theirs and the receipt must say so.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor; a replayed registration would take a place.'
sideEffects:
  level: mutating
  rationale: 'Creates or changes the guest''s registration, which counts against capacity; withdrawn with vtc/events/guests/withdraw.'
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: 'The guest''s answers may carry dietary and access needs, read only by hosts; the response is the guest''s own registration and a reference to the event.'
retention:
  class: durable
  rationale: 'Kept as any registration; answers deleted after the event (VTI-EVT-090).'
errorCodes:
- code: vtc/events/guests/register:guestsNotAdmitted
  meaning: 'The event admits no guests, or not this one: `none`, or `invited` without an invitation for guests that this guest redeemed (VTI-EVT-150, 151).'
  retryable: false
- code: vtc/events/guests/register:alreadyMember
  meaning: 'The signer is a member of the community; members register with vtc/events/registrations/register.'
  retryable: false
- code: vtc/events/guests/register:eventCancelled
  meaning: 'The event was cancelled.'
  retryable: false
- code: vtc/events/guests/register:notOpen
  meaning: 'The event takes no registrations (`closed`).'
  retryable: false
- code: vtc/events/guests/register:windowClosed
  meaning: 'Registrations are not open now (VTI-EVT-038).'
  retryable: false
- code: vtc/events/guests/register:full
  meaning: 'The event is full and keeps no waitlist (VTI-EVT-033).'
  retryable: false
- code: vtc/events/guests/register:tooManyGuests
  meaning: 'More plus-ones than the event admits.'
  retryable: false
- code: vtc/events/guests/register:answersInvalid
  meaning: 'The answers do not fit the event''s questions (VTI-EVT-034).'
  retryable: false
- code: vtc/events/guests/register:codeOfConductRequired
  meaning: 'The code of conduct was not acknowledged (VTI-EVT-035).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      sha256:
        type: string
        maxLength: 64
- code: vtc/events/guests/register:maybeNotAccepted
  meaning: 'The event decides each registration; say going.'
  retryable: false
- code: vtc/events/guests/register:hostDecided
  meaning: 'A host declined or removed this registration; it is not re-opened.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/guests/withdraw
- vtc/events/invitations/links/create
- vtc/events/show
- vtc/events/access/show
---

## Abstract

The **VTC Events — Guests — Register** Trust Task lets a party who is not a member of the community register for an event that admits guests (VTI-EVT-150 – 154): any party under `open`, or under `invited` one who holds an invitation link made for guests. It is apart from [`vtc/events/registrations/register`](../../../registrations/register/0.2/spec.md) because a guest is admitted by the event's guest setting, never by its audience, and holds no directory entry: everything else about the registration — capacity, waitlist, questions, code of conduct, the receipt — is as that task states.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a signer who is an active member with `alreadyMember`.
2. Refuse with `guestsNotAdmitted` an event whose `guests` is `none`, or `invited` without a `linkCode` naming a live link made for guests (VTI-EVT-151); never admit a guest by the audience.
3. Refuse a cancelled event with `eventCancelled`, and then apply `registrations/register/0.2`'s rules — `notOpen`, `windowClosed`, `full`, `tooManyGuests`, `answersInvalid`, `codeOfConductRequired`, `maybeNotAccepted`, `hostDecided` — under this task's codes.
4. Name the registration by `registrantDid`, the signer's identifier; count it against capacity; show it to hosts and never by identifier to other registrants (VTI-EVT-153).
5. Charge the request to the bounds for senders without standing (VTI-TRN-029), and refuse `open` where the events policy does (VTI-EVT-154).
6. Answer the receipt (VTI-EVT-092): the registration and the event reference.

## Authorization

**The guest**, by their own identifier. A guest has no standing in the community beyond this event (VTI-EVT-152).

## Definitions

- `Registration`, `EventRef`, `RegistrationAnswers` — see [`event.schema.json`](../../../_shared/0.3/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000011",
  "type": "https://trusttasks.org/spec/vtc/events/guests/register/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
    "status": "going",
    "guests": 0
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#key-1",
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
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000012",
  "type": "https://trusttasks.org/spec/vtc/events/guests/register/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "registration": {
      "registrationId": "reg_Hk3Wq9Ls2Dn6Hy4B",
      "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
      "registrantDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "status": "going",
      "guests": 0,
      "roster": "hidden",
      "eventSequence": 1,
      "createdAt": "2026-11-28T10:00:00Z",
      "updatedAt": "2026-11-28T10:00:00Z"
    },
    "event": {
      "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
      "title": "Intro to Soldering",
      "when": {
        "start": "2026-11-30T18:30:00",
        "timeZone": "Europe/London",
        "duration": "PT1H30M"
      },
      "status": "published",
      "sequence": 1
    },
    "wasCreated": true
  },
  "threadId": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000011"
}
```

## Security & Privacy

### Data carried

An event, a status, plus-ones, answers and an optional link code in; the guest's registration and a reference to the event out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the DID its members and guests already know it by. Members are named by their community-scoped `entryId`; guests and partners' members by the identifier they registered with, shown to hosts only (VTI-EVT-134, VTI-EVT-153).

### Retention

The registration as any; answers deleted after the event.

### Consent/purpose

Letting people outside the community come to an event that wants them, without joining it. **Events confer nothing**: the community never reads an event, a registration, a check-in, a ticket or an attendance in an authorization decision (VTI-EVT-002).
