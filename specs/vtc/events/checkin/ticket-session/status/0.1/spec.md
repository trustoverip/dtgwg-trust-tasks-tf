---
slug: vtc/events/checkin/ticket-session/status
version: '0.1'
title: VTC Events — Check-in — Ticket Session — Status
summary: 'Read where a door session stands: waiting, the ticket admitted with the check-in it recorded, refused with a reason, or expired.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- openid4vp
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
  rationale: 'An admitted session names the registrant checked in; only the event''s door reads it.'
sideEffects:
  level: none
  rationale: 'Reads a session''s state; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'A session identifier in; its state, and for an admitted ticket the check-in recorded (the registrant by entry or identifier), out.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes:
- code: vtc/events/checkin/ticket-session/status:sessionNotFound
  meaning: 'No door session the caller may read has this identifier — never opened, opened for an event the caller is not door staff or a host of, or no longer kept.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/checkin/ticket-session/start
- vtc/events/checkin/list
---

## Abstract

The **VTC Events — Check-in — Ticket Session — Status** Trust Task lets an event's door read the outcome of a door session it opened with [`vtc/events/checkin/ticket-session/start`](../../start/0.1/spec.md) (VTI-EVT-173): `pending` while no wallet has answered; `admitted` with the check-in the ticket recorded (method `ticket`); `refused` with one reason from a closed set — `ticketInvalid`, `wrongEvent`, `ticketRevoked`, `notGoing`, `ticketUsed`; or `expired`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer only the event's door staff (`vtc.events.checkin` qualified `event:<eventId>`) and its hosts; any other caller, and any session it no longer keeps or never opened, with `sessionNotFound` alike.
2. Answer `occurrence` where the session is for an occurrence of a series (VTI-EVT-195), and `state` as [`ticket-session/start`](../../start/0.1/spec.md) item 7 sets it; `checkIn` exactly when `admitted` (the check-in recorded, method `ticket`); `refusal` exactly when `refused`, as start item 4 chose it.
3. Answer the same outcome on every read once the session is answered: a door that reads twice is told the same thing, and reading never admits.
4. Name the registrant in `checkIn` as [`vtc/events/checkin/list`](../../../list/0.2/spec.md) names them — a member by `entryId`, a partner's member or a guest by `registrantDid` — and never answer anything of the presentation beyond that.

## Authorization

**The event's door**, as for [`ticket-session/start`](../../start/0.1/spec.md). Never a portal's delegated key. The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `EventId`, `EventCheckIn` — see [`event.schema.json`](../../../../_shared/0.4/event.schema.json).
- `TicketSessionState`, `TicketRefusal` — this task's own `$defs`.

## Request

The door staff member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read the door session

```json
{
  "id": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000511",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/ticket-session/status/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-30T18:21:00Z",
  "payload": {
    "sessionId": "Qm7Lk2Tn9Bv4Xs1Hr8Wp3Zd6"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-30T18:21:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/checkin/ticket-session/status/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Admitted

```json
{
  "id": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000512",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/ticket-session/status/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-30T18:21:01Z",
  "threadId": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000511",
  "payload": {
    "sessionId": "Qm7Lk2Tn9Bv4Xs1Hr8Wp3Zd6",
    "eventId": "evt_Rk4mT9xQ2nWb7LpZ",
    "state": "admitted",
    "checkIn": {
      "registrationId": "reg_Hs7Qm2Lk9Tn4Bv1X",
      "registrantKind": "guest",
      "registrantDid": "did:example:rosa",
      "method": "ticket",
      "checkedInAt": "2026-11-30T18:20:47Z",
      "counts": false
    }
  }
}
```

### Refused: the ticket already admitted its holder

```json
{
  "id": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000513",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/ticket-session/status/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-30T18:23:01Z",
  "threadId": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000511",
  "payload": {
    "sessionId": "Qm7Lk2Tn9Bv4Xs1Hr8Wp3Zd6",
    "eventId": "evt_Rk4mT9xQ2nWb7LpZ",
    "state": "refused",
    "refusal": "ticketUsed"
  }
}
```

## Security & Privacy

### Data carried

A session identifier in; the session's state and, once admitted, the check-in it recorded. Never a claim value or the presented credential.

### Correlation

The community maintainer's identifier scope is public. The registrant appears only to the door that opened the session, exactly as the check-in list shows them to it.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Telling the door whether to let someone in. A refusal names a reason from a closed set so the door can say what to do next, and nothing about the ticket's holder.
