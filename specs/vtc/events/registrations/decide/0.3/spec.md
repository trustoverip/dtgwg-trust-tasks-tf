---
slug: vtc/events/registrations/decide
version: '0.3'
title: VTC Events — Registrations — Decide
summary: 'A host decides a pending registration for an event in approval mode: approve it — going, or waitlisted where the event is full — or decline it, with a note the registrant reads.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- registration
- approval
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
  rationale: A decision admits a member to an event, or turns them away, in the host's name; it must be attributable to the host after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed decision of a registration since withdrawn and re-made would decide a request the host never saw.
sideEffects:
  level: mutating
  rationale: Moves a pending registration to going, waitlisted or declined, taking a place where it approves. A decline is not recoverable by the registrant; a host may remove an approved registration with vtc/events/registrations/remove.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: The request names a registration and a decision; the response is that registration, answers included, which the host already reads.
retention:
  class: durable
  rationale: The decision stands on the registration, with the host's note.
errorCodes:
- code: vtc/events/registrations/decide:notPending
  meaning: The registration is not pending — already decided, withdrawn or removed. Nothing changed.
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
        - going
        - maybe
        - pending
        - waitlisted
        - withdrawn
        - declined
        - removed
- code: vtc/events/registrations/decide:full
  meaning: Approving would exceed the event's capacity and it keeps no waitlist. The registration stays pending; free a place, raise the capacity, or decline it.
  retryable: true
- code: vtc/events/registrations/decide:eventCancelled
  meaning: The event is cancelled; its pending registrations are not decided. Nothing changed.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/registrations/list
- vtc/events/registrations/register
- vtc/events/registrations/remove
---

## Abstract

The **VTC Events — Registrations — Decide** Trust Task is how a host answers a registration for an event in mode `approval` (VTI-EVT-031). Every registration for such an event waits, `pending`, until a host decides it (VTI-EVT-073):

- **Approve** makes the registrant `going`, or `waitlisted` where the event is full and keeps a waitlist, last in the queue (VTI-EVT-033).
- **Decline** turns the registration away. The registrant cannot re-open it ([`vtc/events/registrations/register`](../../register/0.3/spec.md) answers `hostDecided`).

A decline is a **decision**, answered in the response like an approval — never an error. The answer is signed, so it is evidence the host and the registrant can both keep: the registrant reads it, with the host's note, on their registration.

## Status of this Document

Version 0.2 answers the Event and Registration of `_shared/0.2`, which carry invitation settings and partners' members (VTI Community Events chapter, VTI-EVT-100 – 135). Its Event and Registration are `vtc/events/_shared/0.2`'s.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for a registration that does not exist and one for an event the caller does not host, and while the community has events turned off (VTI-EVT-007).
2. Refuse a cancelled event with `eventCancelled`, and a registration that is not `pending` with `notPending`, naming its status. A decision is made once: deciding again is refused, not repeated.
3. On `approve`, make the registration `going` where the registrant and their guests fit the event's capacity (VTI-EVT-033); otherwise `waitlisted`, last in the queue, where the event keeps a waitlist; otherwise refuse with `full` and leave it `pending`.
4. On `decline`, make the registration `declined`. Free nothing: a pending registration held no place.
5. Record `note`, where given, on the registration, where the registrant reads it, and the event's `sequence` as `eventSequence`.
6. Answer the decision, the registration as it now stands and an `EventRef` of the event. The answer is a receipt (VTI-EVT-092): the registrant reads the same registration on [`vtc/events/show`](../../../show/0.4/spec.md) and [`vtc/events/mine/list`](../../../mine/list/0.3/spec.md).
7. Audit the decision by reference — event, registration, entry, decision, host — never with the registrant's answers (VTI-EVT-091).

### Series and anonymity (0.3)

0.3 answers the 0.4 Registration, which may name an occurrence.

## Authorization

**Hosting the event:** `vtc.events.manage` covering the event — a host's resource grant qualified `event:<eventId>` (VTI-EVT-070), or the authority over the event's calendar (VTI-EVT-013) — read from the signer's access-control entry at execution time. A decision admits a member to an event; it confers nothing else (VTI-EVT-002). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A decision is the host's own act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-EVT-095).

## Definitions

- `RegistrationId`, `Registration`, `EventRef` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Approve

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000451",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/decide/0.3#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-21T08:05:00Z",
  "payload": {
    "registrationId": "reg_Kf3nP8qLs2Vx7Bd1",
    "decision": "approve"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-21T08:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/registrations/decide/0.3#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document; a decline is not a failure.

### Approve — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000452",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/decide/0.3#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-21T08:05:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000451",
  "payload": {
    "decision": "approve",
    "registration": {
      "registrationId": "reg_Kf3nP8qLs2Vx7Bd1",
      "eventId": "evt_IntroSoldering2611",
      "entryId": "dir_AdaLovelace000000",
      "status": "going",
      "guests": 1,
      "answers": { "dietary": "Vegetarian", "step-free": true },
      "codeOfConductAcknowledged": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
      "roster": "hidden",
      "eventSequence": 1,
      "createdAt": "2026-11-20T19:02:11Z",
      "updatedAt": "2026-11-21T08:05:01Z"
    },
    "event": {
      "eventId": "evt_IntroSoldering2611",
      "title": "Intro to Soldering",
      "when": { "start": "2026-11-30T18:30:00", "timeZone": "Europe/London", "duration": "PT1H30M" },
      "status": "published",
      "sequence": 1
    }
  }
}
```

## Security & Privacy

### Data carried

A registration, a decision and a note in; the registration, the decision and a summary of the event out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Registrants are named by community-scoped `entryId`, never by DID.

### Retention

**Durable**: the decision and the note stand on the registration.

### Consent/purpose

Letting a host decide who comes to an event that needs deciding — a workshop with limited equipment, a meeting for a particular purpose. The registrant consented by registering; the decision is the host's, recorded as theirs. **An event confers nothing** (VTI-EVT-002).
