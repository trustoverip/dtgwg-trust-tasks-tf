---
slug: vtc/events/registrations/withdraw
version: '0.2'
title: VTC Events — Registrations — Withdraw
summary: 'A member withdraws their registration for an event: always allowed, never held, at any time. A place it held passes to the waitlist, and the community''s signed answer is the member''s receipt.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- registration
- withdraw
parties:
- role: registrant
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Withdrawal gives up a place in the registrant's name and may hand it to another member; it must be attributable to the registrant after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed withdrawal would give up a place the member had since taken again.
sideEffects:
  level: mutating
  rationale: Sets the registration to withdrawn and, where it held a place, moves the waitlist. Recoverable by registering again with vtc/events/registrations/register, subject to the event's capacity at that time.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: The request names an event; the response is the member's own registration and a summary of the event.
retention:
  class: durable
  rationale: The withdrawn status is kept with the registration, so the member's agent and the hosts see it ended.
errorCodes:
- code: vtc/events/registrations/withdraw:notRegistered
  meaning: The caller holds no registration for this event. Nothing changed.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/registrations/register
- vtc/events/mine/list
- vtc/events/show
---

## Abstract

The **VTC Events — Registrations — Withdraw** Trust Task is how a member takes themselves off an event. It is **always allowed** (VTI-EVT-037): before or after the registration window closes, whatever the event's mode, from any status the member holds — going, maybe, pending or waitlisted — and it is **never held** for anyone's decision. A place the registration held passes to the first registration on the waitlist that fits (VTI-EVT-033).

The community's signed answer is the member's **receipt** (VTI-EVT-092), as [`vtc/events/registrations/register`](../../register/0.2/spec.md)'s is.

## Status of this Document

Version 0.2 answers the Event and Registration of `_shared/0.2`, which carry invitation settings and partners' members (VTI Community Events chapter, VTI-EVT-100 – 135). Its Event and Registration are `vtc/events/_shared/0.2`'s.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Answer `notFound` for an event that does not exist, and while the community has events turned off (VTI-EVT-007). An event the caller holds a registration for is one they may name here even where its audience no longer admits them: withdrawing is never refused for losing the audience.
3. Refuse a caller who holds no registration for the event with `notRegistered`.
4. Set a registration that is `going`, `maybe`, `pending` or `waitlisted` to `withdrawn`, whatever the registration window, the event's mode or its time (VTI-EVT-037), and **MUST NOT** hold the request for an approval, a consent decision or a step-up. Answer `changed` true.
5. Where the registration was `going`, free its places — the registrant and their guests — and move the waitlist: the first `waitlisted` registration that fits becomes `going`, then the next, in order (VTI-EVT-033). Where it was `waitlisted`, remove it from the queue.
6. Answer a registration that is already `withdrawn`, `declined` or `removed`, or one for a cancelled event, as it stands with `changed` false. Withdrawing from a cancelled event changes nothing: the event will not happen and no place is held.
7. Keep the registration's `registrationId`, its roster setting and its history; delete its answers on the event's retention schedule as any other (VTI-EVT-090).
8. Answer the registration and an `EventRef` of the event. The answer is the member's receipt (VTI-EVT-092). Audit the withdrawal by reference (VTI-EVT-091).

## Authorization

**Being the registrant.** A registration is withdrawn by the member who holds it and nobody else; no capability is involved. Hosts end a registration with [`vtc/events/registrations/remove`](../../remove/0.2/spec.md) instead, which is recorded as theirs. The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and the registration withdrawn is that member's.

Withdrawing is an act of the member (VTI-EVT-095): a member portal that holds a key the member authorized to sign for them for `vtc/events` (`auth/oob` `signingScope`) **MAY** sign it.

## Definitions

- `EventId`, `Registration`, `EventRef` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The registrant (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Can't make it

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000431",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/withdraw/0.2#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-29T21:40:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-29T21:40:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/registrations/withdraw/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Can't make it — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000432",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/withdraw/0.2#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-29T21:40:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000431",
  "payload": {
    "registration": {
      "registrationId": "reg_Kf3nP8qLs2Vx7Bd1",
      "eventId": "evt_IntroSoldering2611",
      "entryId": "dir_AdaLovelace000000",
      "status": "withdrawn",
      "guests": 1,
      "roster": "hidden",
      "eventSequence": 2,
      "createdAt": "2026-11-20T19:02:11Z",
      "updatedAt": "2026-11-29T21:40:01Z"
    },
    "event": {
      "eventId": "evt_IntroSoldering2611",
      "title": "Intro to Soldering",
      "when": { "start": "2026-11-30T18:30:00", "timeZone": "Europe/London", "duration": "PT1H30M" },
      "status": "published",
      "sequence": 2
    },
    "changed": true
  }
}
```

## Security & Privacy

### Data carried

An event identifier in; the member's registration and a summary of the event out. Who took the freed place is never in the answer.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. The member withdraws as their identity at this community.

### Retention

**Durable** for the withdrawn status, kept with the registration so the member's agent and the hosts see it ended. Answers are deleted on the event's retention schedule (VTI-EVT-090).

### Consent/purpose

Changing one's mind. A member who said they would come is never held to it: withdrawal is the member's own act, refused only where there is nothing to withdraw. **An event confers nothing** (VTI-EVT-002).
