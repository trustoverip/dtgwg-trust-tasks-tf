---
slug: vtc/events/access/show
version: '0.1'
title: VTC Events — Access — Show
summary: 'Read where an event happens and how to join it — its places, its online links, its notes — answered to a registrant who is going and to the event''s hosts, from the event''s revealAt if it has one.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- location
- access
parties:
- role: registrant or host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The answer is released only to a registrant who is going or a host; an unattributable caller is neither.
sideEffects:
  level: none
  rationale: Reads the event's access details; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses an address, a door code or a join link and passcode that the event withholds from its wider audience — not secret in the framework's sense, but released to the caller alone.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/events/access/show:notGoing
  meaning: The caller may see the event but holds no registration in `going` for it, and is not a host. Register, or wait for a host's decision, and ask again.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      status:
        type: string
        enum:
        - none
        - going
        - maybe
        - pending
        - waitlisted
        - withdrawn
        - declined
        - removed
        description: The caller's registration status, or `none` without one.
- code: vtc/events/access/show:notYet
  meaning: The caller is going, but the hosts release the access details only from `revealAt`. Ask again then.
  retryable: true
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - revealAt
    properties:
      revealAt:
        type: string
        format: date-time
- code: vtc/events/access/show:eventCancelled
  meaning: The event is cancelled; its access details are no longer released to registrants. Hosts still read them.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/show
- vtc/events/registrations/register
- vtc/events/update
---

## Abstract

The **VTC Events — Access — Show** Trust Task reads where an event happens and how to join it: its places (name, address, directions, a door code), its online links (a video call and its passcode) and its notes. An event's access details are **released, not published** (VTI-EVT-060, VTI-EVT-061): they are never in [`vtc/events/list`](../../../list/0.1/spec.md), never in [`vtc/events/show`](../../../show/0.1/spec.md) unless the hosts made them public, never in a notification, a calendar feed or a public page. The audience sees the event's `placeSummary` — `Central London`, `Online` — and a registrant who is going reads the rest here, from the moment the hosts chose.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 11.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Answer `notFound` alike for an event that does not exist and one the caller may not see — a draft to anyone but its hosts, or an event whose audience does not admit the caller (VTI-EVT-020, VTI-EVT-023) — and while the community has events turned off (VTI-EVT-007).
3. Answer the event's hosts whatever the event's status and `revealAt`.
4. Otherwise answer only a caller whose registration for the event is `going` (VTI-EVT-060); refuse any other caller who may see the event with `notGoing`, naming their status. Guests the registrant brings are covered by the registrant's answer; they hold no registration of their own.
5. Refuse a going registrant before the event's `revealAt` with `notYet`, naming it; and refuse one for a cancelled event with `eventCancelled`.
6. Answer the access details as they stand and the event's current `sequence`, so a reader holding an older copy knows it changed (VTI-EVT-063). Leave `public` and `revealAt` in the answer: they are the hosts' settings, not secrets.
7. Answer an event whose access details are `public` to every caller who may see it, as [`vtc/events/show`](../../../show/0.1/spec.md) already does.

## Authorization

**A going registration for the event, or hosting it.** The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); the community reads the caller's registration, or their `vtc.events.manage` grant for the event (VTI-EVT-070), at execution time. A registration confers nothing beyond this read (VTI-EVT-002).

Reading the access details is an act of the member (VTI-EVT-095): a member portal that holds a key the member authorized to sign for them for `vtc/events` (`auth/oob` `signingScope`) **MAY** sign it.

## Definitions

- `EventId`, `AccessDetails`, `Place`, `VirtualPlace` — see [`event.schema.json`](../../../_shared/0.1/event.schema.json).

## Request

The registrant or host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Where is it?

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000411",
  "type": "https://trusttasks.org/spec/vtc/events/access/show/0.1#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-30T16:30:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-30T16:30:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/access/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Where is it? — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000412",
  "type": "https://trusttasks.org/spec/vtc/events/access/show/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-30T16:30:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000411",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "access": {
      "places": [
        {
          "name": "The Makerspace",
          "address": "Unit 4, Mill Lane, London",
          "description": "Door code 4719. Step-free entrance on the left."
        }
      ],
      "revealAt": "2026-11-30T16:30:00Z"
    },
    "sequence": 2
  }
}
```

### Refused: too early

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000413",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-29T10:00:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000411",
  "payload": {
    "code": "vtc/events/access/show:notYet",
    "message": "The hosts release where this happens two hours before it starts.",
    "retryable": true,
    "details": { "revealAt": "2026-11-30T16:30:00Z" }
  }
}
```

## Security & Privacy

### Data carried

An event identifier in; the event's access details and its `sequence` out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. The request names one event and is made by a member the community already knows is registered for it, so it adds nothing to what the community holds. A member's agent **SHOULD** read the access details close to the event rather than for every event as soon as it is going, which keeps the read meaningful and its timing unremarkable.

### Retention

Nothing beyond the audit record of the request. The member's agent keeps what it reads; the community does not track who read the access details.

### Consent/purpose

Getting to an event one is going to. The access details are withheld from the wider audience because an address and a door code in the hands of everyone who may see a listing is how a members-only event stops being one. A host who wants them shown to the whole audience sets `public`.
