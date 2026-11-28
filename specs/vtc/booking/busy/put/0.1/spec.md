---
slug: vtc/booking/busy/put
version: '0.1'
title: VTC Booking — Busy — Put
summary: 'The host''s agent supplies the host''s busy time to their booking page: merged, quarter-hour-widened intervals at most ninety days ahead, replacing the last supply.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- booking
- calendar
parties:
- role: the host's agent, as the host
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
  ingests: metadata
  actsAsSubject: false
  rationale: 'Pages, slots and bookings as the caller may see them; answers only to the host, busy time to nobody (VTI-BKG-041).'
retention:
  class: durable
  rationale: 'Kept as the community keeps pages and events; answers deleted after the meeting (VTI-EVT-090).'
errorCodes:
- code: vtc/booking/busy/put:intervalsInvalid
  meaning: 'An interval does not start and end on a quarter hour, ends before it starts, overlaps or touches another, or ends more than ninety days ahead (VTI-CAL-061, VTI-BKG-040). Nothing was stored.'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"field": {"type": "string", "maxLength": 128}, "reason": {"type": "string", "maxLength": 500}}}
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vta/calendar/busy/share
- vtc/booking/slots/list
---

## Abstract

The **VTC Booking — Busy — Put** Trust Task: the host's agent supplies the host's busy time to their booking page: merged, quarter-hour-widened intervals at most ninety days ahead, replacing the last supply. It implements VTI-BKG-040 – 041, VTI-CAL-060 – 061 of the VTI Community Booking Pages chapter. A booking is an event (vtc/events/*), so everything the events family states of an event holds for one, except where this family says otherwise.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, decision 28), and the requirements in the VTI Community Booking Pages chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Accept a supply only from the page's host, signed as the host's membership identifier; answer `notFound` for any other signer (VTI-BKG-040).
2. Refuse with `intervalsInvalid` intervals that are not merged, quarter-hour aligned and within ninety days; store nothing.
3. Replace the stored busy time with the supply; an empty supply withdraws it.
4. Use the busy time only as an absence of slots (VTI-BKG-022), and disclose it to no one (VTI-BKG-041).

## Authorization

**The host's agent, as the host**, proven by the document's signature ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A booker's tier is what they prove — membership, a verified presentation, a relationship the community's records hold — never what they assert (VTI-BKG-031). A host's acts are never signed by a member portal's delegated key on a booker's behalf.

## Definitions

- `BookingPage`, `BookingMeetingType`, `BookingTierRule`, `BookingAvailability`, `BookingSlot`, `BookingBusyInterval`, `Booking` — see [`booking.schema.json`](../../../_shared/0.1/booking.schema.json).
- `EventRef`, `Registration`, `RegistrationAnswers`, `MembershipPresentation` — see the events family's [`event.schema.json`](../../../../events/_shared/0.4/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5b2c7d1e-3f4a-4b8c-9d0e-000000000101",
  "type": "https://trusttasks.org/spec/vtc/booking/busy/put/0.1#request",
  "issuer": "did:example:ben",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:05:00Z",
  "payload": {
    "pageId": "bkp_Mt7Qw2Lk9Rn4Bv1X",
    "intervals": [
      {
        "start": "2026-12-01T15:00:00Z",
        "end": "2026-12-01T16:30:00Z"
      },
      {
        "start": "2026-12-03T14:00:00Z",
        "end": "2026-12-03T14:45:00Z"
      }
    ]
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
  "type": "https://trusttasks.org/spec/vtc/booking/busy/put/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:ben",
  "issuedAt": "2026-11-28T10:05:01Z",
  "threadId": "urn:uuid:5b2c7d1e-3f4a-4b8c-9d0e-000000000101",
  "payload": {
    "pageId": "bkp_Mt7Qw2Lk9Rn4Bv1X",
    "intervals": 2,
    "suppliedAt": "2026-11-28T10:00:00Z"
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
