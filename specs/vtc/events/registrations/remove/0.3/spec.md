---
slug: vtc/events/registrations/remove
version: '0.3'
title: VTC Events — Registrations — Remove
summary: 'A host removes a registration from an event, with a note the registrant reads. A place it held passes to the waitlist, and the registrant cannot re-open it.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- registration
- moderation
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
  rationale: Removal takes a member off an event in the host's name; it must be attributable to the host after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed removal would act on a registration the host never meant.
sideEffects:
  level: mutating
  rationale: Sets the registration to removed and, where it held a place, moves the waitlist. The registrant cannot re-open it; nothing restores it but the member registering for another event.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: The request names a registration and a note; the response is that registration, which the host already reads.
retention:
  class: durable
  rationale: The removal stands on the registration, with the host's note.
errorCodes:
- code: vtc/events/registrations/remove:eventEnded
  meaning: The event has ended; its registrations are its record and are no longer changed. Nothing changed.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/registrations/list
- vtc/events/registrations/decide
- vtc/events/registrations/withdraw
---

## Abstract

The **VTC Events — Registrations — Remove** Trust Task is how a host takes a registration off an event: someone who should not be there, a registration made in error, a guest list cut down. It ends a registration in any open status — `going`, `maybe`, `pending` or `waitlisted` — and records it `removed`, with the host's note where they give one, which the registrant reads (VTI-EVT-073). A place it held passes to the waitlist (VTI-EVT-033). The registrant cannot re-open a removal ([`vtc/events/registrations/register`](../../register/0.3/spec.md) answers `hostDecided`).

A registrant ending their own registration uses [`vtc/events/registrations/withdraw`](../../withdraw/0.3/spec.md) instead, recorded as theirs.

## Status of this Document

Version 0.2 answers the Event and Registration of `_shared/0.2`, which carry invitation settings and partners' members (VTI Community Events chapter, VTI-EVT-100 – 135). Its Event and Registration are `vtc/events/_shared/0.2`'s.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for a registration that does not exist and one for an event the caller does not host, and while the community has events turned off (VTI-EVT-007).
2. Refuse a registration for an event that has ended with `eventEnded`: after the event, its registrations are its record.
3. Set a registration that is `going`, `maybe`, `pending` or `waitlisted` to `removed`, record `note` where given, and answer `changed` true. Answer a registration already `withdrawn`, `declined` or `removed` as it stands with `changed` false.
4. Where the registration was `going`, free its places and move the waitlist: the first `waitlisted` registration that fits becomes `going`, then the next, in order (VTI-EVT-033). Where it was `waitlisted`, remove it from the queue.
5. Remove a registration also when its registrant leaves the community (VTI-EVT-039), without a host's request, as if a host had removed it with no note.
6. Answer the registration as it now stands and an `EventRef` of the event; the registrant reads the same on [`vtc/events/show`](../../../show/0.4/spec.md) and [`vtc/events/mine/list`](../../../mine/list/0.3/spec.md).
7. Audit the removal by reference — event, registration, entry, host — never with the registrant's answers (VTI-EVT-091).

### Series and anonymity (0.3)

0.3 answers the 0.4 Registration, which may name an occurrence.

## Authorization

**Hosting the event:** `vtc.events.manage` covering the event — a host's resource grant qualified `event:<eventId>` (VTI-EVT-070), or the authority over the event's calendar (VTI-EVT-013) — read from the signer's access-control entry at execution time. A host removes any registration for the event they host, another host's included; removing a host from hosting is [`vtc/events/hosts/remove`](../../../hosts/remove/0.1/spec.md), a different act. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A removal is the host's own act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-EVT-095).

## Definitions

- `RegistrationId`, `Registration`, `EventRef` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Remove a registration made in error

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000461",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/remove/0.3#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-25T12:00:00Z",
  "payload": {
    "registrationId": "reg_Mp5tW2nQx8Rv4Ck9",
    "note": "This session is for the Electronics group; the open Build Night is on Thursday."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-25T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/registrations/remove/0.3#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Remove a registration made in error — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000462",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/remove/0.3#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-25T12:00:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000461",
  "payload": {
    "registration": {
      "registrationId": "reg_Mp5tW2nQx8Rv4Ck9",
      "eventId": "evt_IntroSoldering2611",
      "entryId": "dir_DevSharma00000000",
      "status": "removed",
      "guests": 0,
      "roster": "hidden",
      "eventSequence": 1,
      "note": "This session is for the Electronics group; the open Build Night is on Thursday.",
      "createdAt": "2026-11-24T20:11:03Z",
      "updatedAt": "2026-11-25T12:00:01Z"
    },
    "event": {
      "eventId": "evt_IntroSoldering2611",
      "title": "Intro to Soldering",
      "when": { "start": "2026-11-30T18:30:00", "timeZone": "Europe/London", "duration": "PT1H30M" },
      "status": "published",
      "sequence": 1
    },
    "changed": true
  }
}
```

## Security & Privacy

### Data carried

A registration and a note in; the registration and a summary of the event out. Who took a freed place is never in the answer.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Registrants are named by community-scoped `entryId`, never by DID.

### Retention

**Durable**: the removal and the note stand on the registration; answers are deleted on the event's retention schedule as any other (VTI-EVT-090).

### Consent/purpose

Letting a host keep an event to its purpose. A removal is recorded as the host's and shown to the registrant with the host's note, so it is never silent. **An event confers nothing** (VTI-EVT-002).
