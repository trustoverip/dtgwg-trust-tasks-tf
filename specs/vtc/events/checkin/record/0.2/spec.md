---
slug: vtc/events/checkin/record
version: '0.2'
title: VTC Events — Check-in — Record
summary: 'Check in to an event: a registrant signing over the door challenge, or self check-in where the calendar allows it, or door staff checking a registrant in by name.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: registrant or door staff
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'A check-in records that this registrant came; it must be attributable to the registrant or to the door staff who recorded it.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor; a replayed check-in at another event must not count.'
sideEffects:
  level: mutating
  rationale: 'Records the check-in against the registration, counts it toward attendance where the event counts, and may offer the event''s attendance badge.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'An event, and a challenge or a registration identifier, in; the check-in and a reference to the event out.'
retention:
  class: durable
  rationale: 'Kept until the community''s retention period after the event ends (VTI-EVT-166); a monthly tally survives it (VTI-EVT-185).'
errorCodes:
- code: vtc/events/checkin/record:challengeInvalid
  meaning: 'The challenge is not one the community issued for this event, or it is no longer accepted. One code for all three (VTI-EVT-162).'
  retryable: false
- code: vtc/events/checkin/record:notGoing
  meaning: 'The registration is not `going`; only a registrant going checks in (VTI-EVT-162).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      status:
        type: string
        maxLength: 32
- code: vtc/events/checkin/record:alreadyCheckedIn
  meaning: 'The registration already has a check-in (VTI-EVT-162).'
  retryable: false
- code: vtc/events/checkin/record:selfCheckInNotAllowed
  meaning: 'No challenge was given and the event''s calendar does not take self check-in (VTI-EVT-164).'
  retryable: false
- code: vtc/events/checkin/record:notDuringEvent
  meaning: 'A self check-in outside the event''s time (VTI-EVT-164).'
  retryable: false
- code: vtc/events/checkin/record:eventCancelled
  meaning: 'The event was cancelled.'
  retryable: false
- code: vtc/events/checkin/record:occurrenceRequired
  meaning: "The event is a series registered by occurrence, and the request names no `occurrence`."
  retryable: false
- code: vtc/events/checkin/record:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
- code: vtc/events/checkin/record:occurrenceCancelled
  meaning: "The occurrence named is cancelled (VTI-EVT-196)."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/checkin/challenge
- vtc/events/checkin/list
- vtc/events/attendance/show
- vtc/badges/mine/accept
---

## Abstract

The **VTC Events — Check-in — Record** Trust Task records that a registrant came (VTI-EVT-162 – 165). A registrant signs it with the door's current challenge (`method: challenge`), or without one during the event where the calendar takes self check-in (`self`); door staff sign it naming a `registrationId` (`staff`). The answer is the check-in, a reference to the event, and — where the event names an attendance badge — the offer of it, which the member accepts or declines with [`vtc/badges/mine/accept`](../../../../badges/mine/accept/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. With `registrationId`: answer only the event's door staff and hosts, and record the check-in for that registration with `method: staff` and `checkedInBy` (VTI-EVT-163).
2. Otherwise: the signer is the registrant — a member, a partner's member with a `presentation` verified first (VTI-EVT-132, 133), or a guest (VTI-EVT-151).
3. With `challenge`: refuse with `challengeInvalid` a challenge it did not issue, one for another event, or one past `expiresAt` — the same code and message for all three (VTI-EVT-162).
4. Without `challenge` (and without `registrationId`): refuse with `selfCheckInNotAllowed` unless the event's calendar has `selfCheckIn` other than `none`, and with `notDuringEvent` outside the event's time (VTI-EVT-164).
5. Refuse a registration that is not `going` with `notGoing`, naming its status; one already checked in with `alreadyCheckedIn`; a cancelled event with `eventCancelled`.
6. Set `counts` as VTI-EVT-180 – 183 decide: false where the event or its calendar does not count, for `self` unless the calendar's `selfCheckIn` is `counting`, and for a guest's or a partner member's registration.
7. Where the event names an `attendanceBadge` and the registrant is a member, offer it (VTI-BDG-020) and answer its `badgeOffered`; never award it without their acceptance (VTI-EVT-186).
8. Record nothing about registrants who do not check in (VTI-EVT-165).

### Series and anonymity (0.2)

0.2 adds `occurrence`: a check-in is to one occurrence, and each occurrence is one event attended (VTI-EVT-195, 182).

## Authorization

**The registrant themselves**, or the event's **door staff** (`vtc.events.checkin` at `event:<eventId>`) and hosts for a check-in by name. A member's own check-in is their act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` MAY sign it. A check-in by name is never signed by a portal key.

## Definitions

- `EventCheckIn`, `EventCheckInMethod`, `EventDoorChallengeValue`, `EventRef`, `MembershipPresentation` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000003",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/record/0.2#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
    "challenge": "kQ3nV8xR2bZ7tL4pW9mD6sYc"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
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
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000004",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/record/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "checkIn": {
      "registrationId": "reg_Hk3Wq9Ls2Dn6Hy4B",
      "entryId": "dir_Pr1yaNatarajan00",
      "registrantKind": "member",
      "method": "challenge",
      "checkedInAt": "2026-11-30T18:25:01Z",
      "counts": true
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
    "badgeOffered": "awd_Q7m2K9xVb4Tz8LpR"
  },
  "threadId": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000003"
}
```

## Security & Privacy

### Data carried

An event and a challenge or registration in; the check-in, the event reference and a badge offer out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the DID its members and guests already know it by. Members are named by their community-scoped `entryId`; guests and partners' members by the identifier they registered with, shown to hosts only (VTI-EVT-134, VTI-EVT-153).

### Retention

The check-in until the retention period after the event; the monthly tally after that.

### Consent/purpose

Recording who came, for the door, the hosts, the registrant and attendance badges. **Events confer nothing**: the community never reads an event, a registration, a check-in, a ticket or an attendance in an authorization decision (VTI-EVT-002).
