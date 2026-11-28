---
slug: vtc/events/checkin/challenge
version: '0.2'
title: VTC Events — Check-in — Challenge
summary: 'Obtain a fresh door challenge for an event: a short-lived random value the door shows, which a registrant signs over to check in.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: door staff or host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'A challenge admits whoever signs over it to the event; only the event''s door may ask for one.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Each answer is a new challenge the community will accept for a short time; a replayed request would widen what it accepts.'
sideEffects:
  level: mutating
  rationale: 'Records a new challenge the community accepts for a short time, then forgets.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'An event identifier in; a random challenge and its expiry out.'
retention:
  class: transient
  rationale: 'A challenge is kept only until it expires (VTI-EVT-161).'
errorCodes:
- code: vtc/events/checkin/challenge:notPublished
  meaning: 'The event is a draft; it has no door yet.'
  retryable: false
- code: vtc/events/checkin/challenge:eventCancelled
  meaning: 'The event was cancelled.'
  retryable: false
- code: vtc/events/checkin/challenge:eventEnded
  meaning: 'The event has ended.'
  retryable: false
- code: vtc/events/checkin/challenge:occurrenceRequired
  meaning: "The event is a series registered by occurrence, and the request names no `occurrence`."
  retryable: false
- code: vtc/events/checkin/challenge:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
- code: vtc/events/checkin/challenge:occurrenceCancelled
  meaning: "The occurrence named is cancelled (VTI-EVT-196)."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/checkin/record
- vtc/events/checkin/list
---

## Abstract

The **VTC Events — Check-in — Challenge** Trust Task gives an event's door a fresh **door challenge** (VTI-EVT-161): at least 128 random bits, bound to the event, accepted for a short time the answer states. The door shows it — typically as a QR code — and a registrant's wallet signs a [`vtc/events/checkin/record`](../../record/0.2/spec.md) request carrying it. A screenshot of an old challenge admits nobody.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer only the event's door staff (`vtc.events.checkin` qualified `event:<eventId>`) and its hosts (VTI-EVT-160); anyone else as for an event they may not see.
2. Refuse a draft with `notPublished`, a cancelled event with `eventCancelled`, an ended one with `eventEnded`.
3. Issue a new challenge on every request — at least 128 bits from a cryptographically secure source, bound to the event — and accept it until `expiresAt`, no longer than five minutes after issue; MAY accept several at once (VTI-EVT-161).

### Series and anonymity (0.2)

0.2 adds `occurrence`: a challenge is for one occurrence of a series (VTI-EVT-195).

## Authorization

**The event's door.** Door staff hold `vtc.events.checkin` qualified by the event, appointed by its hosts; its hosts hold `vtc.events.manage` for it. Never signed by a member portal's delegated key.

## Definitions

- `EventDoorChallengeValue`, `EventId`, `EventInstant` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/challenge/0.2#request",
  "issuer": "did:example:dan",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:dan#key-1",
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
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/challenge/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:dan",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
    "challenge": "kQ3nV8xR2bZ7tL4pW9mD6sYc",
    "expiresAt": "2026-11-30T18:27:00Z"
  },
  "threadId": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000001"
}
```

## Security & Privacy

### Data carried

An event identifier in; a random challenge and its expiry out. Nothing about a registrant.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the DID its members and guests already know it by. Members are named by their community-scoped `entryId`; guests and partners' members by the identifier they registered with, shown to hosts only (VTI-EVT-134, VTI-EVT-153).

### Retention

The challenge until it expires.

### Consent/purpose

Letting the door prove a registrant was there now, without a code anyone can photograph and reuse. **Events confer nothing**: the community never reads an event, a registration, a check-in, a ticket or an attendance in an authorization decision (VTI-EVT-002).
