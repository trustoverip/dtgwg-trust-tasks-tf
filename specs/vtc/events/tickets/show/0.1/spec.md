---
slug: vtc/events/tickets/show
version: '0.1'
title: VTC Events — Tickets — Show
summary: 'Read your own ticket for an event that issues tickets: a credential you present at a door the community does not run.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
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
  rationale: 'A ticket is shown only to its subject.'
sideEffects:
  level: none
  rationale: 'Reads the registrant''s ticket; the ticket was issued when the registration became going.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'An event in; the registrant''s own ticket credential out.'
retention:
  class: transient
  rationale: 'A read; the ticket itself lives until the event ends.'
errorCodes:
- code: vtc/events/tickets/show:noTickets
  meaning: 'The event issues no tickets (VTI-EVT-170).'
  retryable: false
- code: vtc/events/tickets/show:notGoing
  meaning: 'The caller holds no registration that is `going`, so holds no ticket (VTI-EVT-170).'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      status:
        type: string
        maxLength: 32
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/checkin/record
- vtc/events/registrations/register
---

## Abstract

The **VTC Events — Tickets — Show** Trust Task gives a registrant their own ticket for an event that issues tickets (VTI-EVT-170 – 174): a credential naming them and the event, valid until it ends, revocable through the community's shared status set. They present it, with a fresh nonce per presentation, at a door that checks it through the community's OpenID4VP verifier (VTI-EVT-173) — a door run by another party, or offline.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse an event that issues no tickets with `noTickets`.
2. Answer only the ticket's subject: a member, a partner's member with a `presentation` verified first (VTI-EVT-132), or a guest (VTI-EVT-151); refuse a caller with no registration `going` with `notGoing`.
3. Answer the credential exactly as issued, never re-serialised.
4. Never answer a ticket to anyone but its subject (VTI-EVT-174).

## Authorization

**The ticket's subject.** A member's read is their act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` MAY sign it.

## Definitions

- `EventId`, `EventInstant`, `MembershipPresentation` — see [`event.schema.json`](../../../_shared/0.3/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000007",
  "type": "https://trusttasks.org/spec/vtc/events/tickets/show/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#key-1",
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
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000008",
  "type": "https://trusttasks.org/spec/vtc/events/tickets/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
    "credential": {
      "@context": [
        "https://www.w3.org/ns/credentials/v2"
      ],
      "type": [
        "VerifiableCredential",
        "EventTicket"
      ],
      "issuer": "did:example:event",
      "validFrom": "2026-11-28T10:00:00Z",
      "validUntil": "2026-11-30T20:00:00Z",
      "credentialSubject": {
        "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "event": "evt_Sd8Kq2Lm4Rt8Vx1Z"
      }
    },
    "validUntil": "2026-11-30T20:00:00Z"
  },
  "threadId": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000007"
}
```

## Security & Privacy

### Data carried

An event in; the caller's own ticket out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the DID its members and guests already know it by. Members are named by their community-scoped `entryId`; guests and partners' members by the identifier they registered with, shown to hosts only (VTI-EVT-134, VTI-EVT-153).

### Retention

Nothing beyond the audit record of the request; the ticket expires with the event.

### Consent/purpose

Admission at a door the community does not run. **Events confer nothing**: the community never reads an event, a registration, a check-in, a ticket or an attendance in an authorization decision (VTI-EVT-002).
