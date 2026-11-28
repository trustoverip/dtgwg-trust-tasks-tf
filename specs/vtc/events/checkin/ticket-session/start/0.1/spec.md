---
slug: vtc/events/checkin/ticket-session/start
version: '0.1'
title: VTC Events — Check-in — Ticket Session — Start
summary: 'Open an OpenID4VP presentation session at an event''s door asking for that event''s ticket, so a registrant''s wallet can present it and be checked in.'
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
  rationale: 'A door session admits whoever presents a valid ticket to it; only the event''s door may open one.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Each answer is a new session with a fresh nonce; a replayed request would open sessions nobody asked for.'
sideEffects:
  level: mutating
  rationale: 'Records a new presentation session, with its nonce, that the community accepts an answer to for a few minutes.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'An event identifier in; a session identifier, the wallet invocation URI, the community''s verifier client identifier and an expiry out.'
retention:
  class: transient
  rationale: 'A session is kept until it is answered or expires, and its outcome only as long as the door reads it; the check-in it records follows the event''s retention (VTI-EVT-090).'
errorCodes:
- code: vtc/events/checkin/ticket-session/start:notPublished
  meaning: 'The event is a draft; it has no door yet.'
  retryable: false
- code: vtc/events/checkin/ticket-session/start:eventCancelled
  meaning: 'The event was cancelled.'
  retryable: false
- code: vtc/events/checkin/ticket-session/start:eventEnded
  meaning: 'The event has ended.'
  retryable: false
- code: vtc/events/checkin/ticket-session/start:ticketsNotIssued
  meaning: 'The event issues no tickets, so there is nothing to present at its door; use a door challenge (vtc/events/checkin/challenge).'
  retryable: false
- code: vtc/events/checkin/ticket-session/start:occurrenceRequired
  meaning: "The event is a series registered by occurrence, and the request names no `occurrence`."
  retryable: false
- code: vtc/events/checkin/ticket-session/start:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
- code: vtc/events/checkin/ticket-session/start:occurrenceCancelled
  meaning: "The occurrence named is cancelled (VTI-EVT-196)."
  retryable: false
- code: vtc/events/checkin/ticket-session/start:verifierNotConfigured
  meaning: 'The community holds no OpenID4VP verifier identity of the requested kind (or none at all). Its operator configures one.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/checkin/ticket-session/status
- vtc/events/checkin/challenge
- vtc/events/checkin/record
- vtc/events/checkin/list
- vtc/events/tickets/show
---

## Abstract

The **VTC Events — Check-in — Ticket Session — Start** Trust Task opens a **door session** for an event that issues tickets (VTI-EVT-170 – 174): an OpenID4VP 1.0 presentation request asking for that event's ticket, with a fresh nonce, answered by a registrant's wallet rather than by a signed Trust Task. The door shows `walletUri` as a QR code; the registrant's wallet presents the ticket to the community's verifier; the door reads the outcome with [`vtc/events/checkin/ticket-session/status`](../../status/0.1/spec.md). An admitted ticket records a check-in with method `ticket`, exactly as a check-in over a door challenge records one with method `challenge` ([`vtc/events/checkin/record`](../../../record/0.2/spec.md)).

This is the door for a wallet that holds the ticket but cannot sign a Trust Task, a door run by another party, and a guest whose ticket is how an outside door knows them (VTI-EVT-153, 172). On a series, a session is for one occurrence (VTI-EVT-195): the door names it as [`vtc/events/checkin/challenge/0.2`](../../../challenge/0.2/spec.md) does.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer only the event's door staff (`vtc.events.checkin` qualified `event:<eventId>`) and its hosts (VTI-EVT-160); anyone else as for an event they may not see.
2. Refuse a draft with `notPublished`, a cancelled event with `eventCancelled`, an ended one with `eventEnded`, and an event that issues no tickets with `ticketsNotIssued`. On a series, the session is for one occurrence (VTI-EVT-195): refuse a request naming none where the series is registered by occurrence with `occurrenceRequired`, one naming an original start the rule does not give (or naming one on an event that is not a series) with `occurrenceUnknown`, and a cancelled occurrence with `occurrenceCancelled`; on a series registered as a whole, a request naming no occurrence is for the occurrence running now or next, and the answer names it.
3. Open a session whose OpenID4VP request asks for that event's ticket credential with key binding to its holder, carries a **fresh nonce generated for this session alone**, and names the community's verifier client identifier (`clientIdPrefix`, or the community's default; `verifierNotConfigured` where it holds none). Answer an unguessable `sessionId` of at least 128 bits, the invocation URI, the client identifier and an expiry measured in minutes.
4. When a wallet answers, verify the presentation against the session's nonce and client identifier and admit the ticket only if every one of these holds; otherwise record the session `refused` with the first reason that applies, in this order:
   1. the ticket was issued by the community or by the event's own identity (VTI-EVT-140 – 145), its proof verifies, the wallet's key binding verifies, and it has not expired — else `ticketInvalid`;
   2. it is a ticket for this event — else `wrongEvent`;
   3. its status in the **community's shared status set** is not revoked (VTI-CRD-014, VTI-EVT-171) — else `ticketRevoked`;
   4. the registration it names is `going` — else `notGoing`;
   5. it has not admitted anyone before — else `ticketUsed`.
5. Keep a **used-ticket record**, written under the same lock that admits, so that each ticket admits once (VTI-EVT-173) — once per occurrence for a ticket to a series registered as a whole, which is a registration for every occurrence; a second presentation of an admitted ticket for the same occurrence is `ticketUsed`, whatever session it arrives in.
6. On admitting, record a check-in for the ticket's registration — for the session's occurrence, on a series (VTI-EVT-195) — with method `ticket`, as [`vtc/events/checkin/record`](../../../record/0.2/spec.md) records one (VTI-EVT-165): it counts toward attendance as a check-in over a door challenge does (VTI-EVT-180 – 183), and offers the event's attendance badge where it names one (VTI-EVT-186). A registrant already checked in is admitted without a second check-in, and the ticket is still marked used.
7. Accept one answer per session: a session is answered once, then reads `admitted` or `refused` until the door stops reading it; one never answered reads `expired` after `expiresAt`.
8. Keep nothing of the presentation beyond the outcome and the used-ticket record: no claim value, no copy of the credential.

## Authorization

**The event's door.** Holders of `vtc.events.checkin` qualified `event:<eventId>`, and the event's hosts (VTI-EVT-160). Never a portal's delegated key: opening a door is not a member's own act. The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `EventId`, `EventInstant`, `LocalDateTime`, `EventCheckIn`, `EventCheckInMethod` — see [`event.schema.json`](../../../../_shared/0.4/event.schema.json).
- **Door session** — an OpenID4VP presentation session the community opens for one event's door, answered by a wallet presenting that event's ticket.
- **Used-ticket record** — the community's record of every ticket that has admitted its holder, by ticket identifier, so each admits once.

## Request

The door staff member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Open the door for a ticket

```json
{
  "id": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000501",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/ticket-session/start/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-30T18:20:00Z",
  "payload": {
    "eventId": "evt_Rk4mT9xQ2nWb7LpZ"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-30T18:20:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/checkin/ticket-session/start/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Open the door for a ticket — answered

```json
{
  "id": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000502",
  "type": "https://trusttasks.org/spec/vtc/events/checkin/ticket-session/start/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-30T18:20:01Z",
  "threadId": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000501",
  "payload": {
    "sessionId": "Qm7Lk2Tn9Bv4Xs1Hr8Wp3Zd6",
    "eventId": "evt_Rk4mT9xQ2nWb7LpZ",
    "walletUri": "openid4vp://?client_id=decentralized_identifier%3Adid%3Aexample%3Acommunity&request_uri=https%3A%2F%2Fcommunity.example%2Foid4vp%2Frequest%2FQm7Lk2Tn9Bv4Xs1Hr8Wp3Zd6",
    "clientId": "decentralized_identifier:did:example:community",
    "expiresAt": "2026-11-30T18:25:01Z"
  }
}
```

### Refused: the event issues no tickets

```json
{
  "id": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000503",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-30T18:20:01Z",
  "threadId": "urn:uuid:7c1e3b9d-4a2f-4c8b-9e0d-000000000501",
  "payload": {
    "code": "vtc/events/checkin/ticket-session/start:ticketsNotIssued",
    "message": "This event issues no tickets; use a door challenge.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

An event identifier in; a session identifier, an invocation URI, the community's verifier client identifier and an expiry out. The presentation itself goes from the wallet to the community's verifier, never through the door's device.

### Correlation

The community maintainer's identifier scope is public, as for every community task. A fresh nonce per session binds each presentation to one door at one moment, so a captured presentation admits nobody elsewhere or later; the used-ticket record makes a ticket admit once however many sessions it reaches.

### Retention

The session until it is answered or expires, then its outcome while the door reads it; the used-ticket record and the check-in follow the event's retention (VTI-EVT-090). Never a claim value or a copy of the credential.

### Consent/purpose

Checking a registrant in at the door. A ticket confers nothing beyond admission to its event (VTI-EVT-002).
