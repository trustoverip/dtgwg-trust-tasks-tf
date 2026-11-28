---
slug: vtc/booking/bookings/book
version: '0.1'
title: VTC Booking — Bookings — Book
summary: 'Book a free slot: the community creates an unlisted meeting event for the host and the booker and registers the booker on it. The reply is the receipt.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- booking
- calendar
parties:
- role: a booker in any tier
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Who may book, see or change a page is decided by the proven signer and the tier they prove; an unattributable request gets nothing (VTI-BKG-031).'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor; a replay would book, change or cancel again.'
sideEffects:
  level: mutating
  rationale: 'Changes booking pages, bookings or the events that are bookings, as the task states.'
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: 'Pages, slots and bookings as the caller may see them; answers only to the host, busy time to nobody (VTI-BKG-041).'
retention:
  class: durable
  rationale: 'Kept as the community keeps pages and events; answers deleted after the meeting (VTI-EVT-090).'
errorCodes:
- code: vtc/booking/bookings/book:slotTaken
  meaning: 'The slot was booked by someone else first. Nothing was stored; `details.nextSlots` offers the next free slots.'
  retryable: true
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"nextSlots": {"type": "array", "maxItems": 10, "items": {"type": "object", "additionalProperties": false, "properties": {"start": {"type": "string", "maxLength": 64}, "end": {"type": "string", "maxLength": 64}}}}}}
- code: vtc/booking/bookings/book:slotNotOffered
  meaning: 'The slot is not one the caller''s tier is offered now (VTI-BKG-051).'
  retryable: false
- code: vtc/booking/bookings/book:tierNotOffered
  meaning: 'The caller''s proven tier may not book this meeting type.'
  retryable: false
- code: vtc/booking/bookings/book:limitReached
  meaning: 'The caller has made the tier''s limit of bookings for the period, or the day''s cap is reached (VTI-BKG-021).'
  retryable: false
- code: vtc/booking/bookings/book:answersInvalid
  meaning: 'The answers do not fit the meeting type''s questions (VTI-BKG-011).'
  retryable: false
- code: vtc/booking/bookings/book:ownPage
  meaning: 'The caller hosts this page (VTI-BKG-053).'
  retryable: false
- code: vtc/booking/bookings/book:presentationInvalid
  meaning: 'The presentation does not verify, or names no partner the `known` tier admits.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/booking/slots/list
- vtc/booking/bookings/list
- vtc/events/show
---

## Abstract

The **VTC Booking — Bookings — Book** Trust Task: book a free slot: the community creates an unlisted meeting event for the host and the booker and registers the booker on it. The reply is the receipt. It implements VTI-BKG-031 – 032, 050 – 053 of the VTI Community Booking Pages chapter. A booking is an event (vtc/events/*), so everything the events family states of an event holds for one, except where this family says otherwise.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, decision 28), and the requirements in the VTI Community Booking Pages chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Place the caller in a tier as vtc/booking/pages/show does; refuse `ownPage` where the caller hosts the page; charge a guest's request to the bounds for senders without standing (VTI-BKG-032).
2. Check the slot is still offered to the caller's tier at the moment of booking, under one lock with every other booking of the host's pages; refuse a slot booked first by another with `slotTaken`, offering the next free slots (VTI-BKG-051). Hold no slot while a booker answers questions.
3. Create an event in the host's calendar: an unlisted `meeting`, roster `hidden`, audience the host and the booker only, the host as its host, the meeting type's place as its access details; register the booker `going`, or `pending` where the tier needs approval (VTI-BKG-050).
4. Answer the booking, the event reference and the registration: the booker's receipt (VTI-EVT-092).

## Authorization

**A booker in any tier**, proven by the document's signature ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A booker's tier is what they prove — membership, a verified presentation, a relationship the community's records hold — never what they assert (VTI-BKG-031). A host's acts are never signed by a member portal's delegated key on a booker's behalf.

## Definitions

- `BookingPage`, `BookingMeetingType`, `BookingTierRule`, `BookingAvailability`, `BookingSlot`, `BookingBusyInterval`, `Booking` — see [`booking.schema.json`](../../../_shared/0.1/booking.schema.json).
- `EventRef`, `Registration`, `RegistrationAnswers`, `MembershipPresentation` — see the events family's [`event.schema.json`](../../../../events/_shared/0.4/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5b2c7d1e-3f4a-4b8c-9d0e-000000000101",
  "type": "https://trusttasks.org/spec/vtc/booking/bookings/book/0.1#request",
  "issuer": "did:example:ben",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:05:00Z",
  "payload": {
    "pageId": "bkp_Mt7Qw2Lk9Rn4Bv1X",
    "meetingTypeId": "intro",
    "start": "2026-12-01T14:00:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ben#key-1",
    "created": "2026-11-28T10:05:00Z",
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
  "id": "urn:uuid:5b2c7d1e-3f4a-4b8c-9d0e-000000000102",
  "type": "https://trusttasks.org/spec/vtc/booking/bookings/book/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:ben",
  "issuedAt": "2026-11-28T10:05:01Z",
  "threadId": "urn:uuid:5b2c7d1e-3f4a-4b8c-9d0e-000000000101",
  "payload": {
    "booking": {
      "bookingId": "bkg_Hk3Wq9Ls2Dn6Hy4B",
      "pageId": "bkp_Mt7Qw2Lk9Rn4Bv1X",
      "meetingTypeId": "intro",
      "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
      "start": "2026-12-01T14:00:00Z",
      "end": "2026-12-01T14:30:00Z",
      "tier": "members",
      "status": "going",
      "bookerEntryId": "dir_BenOkafor0000000",
      "createdAt": "2026-11-28T10:05:00Z",
      "updatedAt": "2026-11-28T10:05:00Z"
    },
    "event": {
      "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
      "title": "Intro call",
      "when": {
        "start": "2026-12-01T14:00:00",
        "timeZone": "Europe/London",
        "duration": "PT30M"
      },
      "status": "published",
      "sequence": 0
    },
    "registration": {
      "registrationId": "reg_Hk3Wq9Ls2Dn6Hy4C",
      "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
      "entryId": "dir_BenOkafor0000000",
      "status": "going",
      "guests": 0,
      "roster": "hidden",
      "eventSequence": 0,
      "createdAt": "2026-11-28T10:05:00Z",
      "updatedAt": "2026-11-28T10:05:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

What the request and response schemas state, and nothing else: pages, meeting types, slots and bookings as the caller may see them. Busy time the host's agent supplies is never answered to anyone (VTI-BKG-041).

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, partners and guests already know it by. Members are named by their community-scoped `entryId`; a booker who is not a member by their own identifier, shown only to the host. A booking's event is unlisted with a hidden roster, so no third party learns who booked whom (VTI-BKG-050).

### Retention

Pages until deleted; bookings as events, their answers deleted after the meeting on the community's retention schedule (VTI-EVT-090); supplied busy time until replaced, withdrawn or the page is deleted (VTI-BKG-040).

### Consent/purpose

A member offering their time, and others booking it. A booking confers nothing (VTI-EVT-002); nobody learns why a slot is not free (VTI-BKG-041).
