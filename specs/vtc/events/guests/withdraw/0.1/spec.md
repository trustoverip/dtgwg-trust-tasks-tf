---
slug: vtc/events/guests/withdraw
version: '0.1'
title: VTC Events — Guests — Withdraw
summary: 'A guest withdraws their registration for an event. Always allowed; the reply is their receipt.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: guest
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Withdrawing is the guest''s own act and the receipt must say so.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor.'
sideEffects:
  level: mutating
  rationale: 'Marks the guest''s registration withdrawn and frees its places for the waitlist.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'An event in; the guest''s registration and a reference to the event out.'
retention:
  class: durable
  rationale: 'The registration is kept withdrawn, as any.'
errorCodes:
- code: vtc/events/guests/withdraw:notRegistered
  meaning: 'The signer holds no registration for this event.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/guests/register
---

## Abstract

The **VTC Events — Guests — Withdraw** Trust Task withdraws a guest's registration, as [`vtc/events/registrations/withdraw`](../../../registrations/withdraw/0.2/spec.md) does a member's (VTI-EVT-037): always allowed, never parked, a repeat changing nothing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), and the requirements in the VTI Community Events chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Find the registration naming the signer by `registrantDid`; refuse with `notRegistered` if there is none.
2. Withdraw it if it is going, maybe, pending or waitlisted, freeing its places (VTI-EVT-033); otherwise change nothing and answer `changed: false`.
3. Answer the receipt (VTI-EVT-092).

## Authorization

**The guest**, by the identifier their registration names.

## Definitions

- `Registration`, `EventRef` — see [`event.schema.json`](../../../_shared/0.3/event.schema.json).

## Request

The issuer sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000013",
  "type": "https://trusttasks.org/spec/vtc/events/guests/withdraw/0.1#request",
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
  "id": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000014",
  "type": "https://trusttasks.org/spec/vtc/events/guests/withdraw/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-30T18:25:00Z",
  "payload": {
    "registration": {
      "registrationId": "reg_Hk3Wq9Ls2Dn6Hy4B",
      "eventId": "evt_Sd8Kq2Lm4Rt8Vx1Z",
      "registrantDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "status": "withdrawn",
      "guests": 0,
      "roster": "hidden",
      "eventSequence": 1,
      "createdAt": "2026-11-28T10:00:00Z",
      "updatedAt": "2026-11-28T10:00:00Z"
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
    "changed": true
  },
  "threadId": "urn:uuid:7e1c4a2b-5d3f-4a8e-9b6c-000000000013"
}
```

## Security & Privacy

### Data carried

An event in; the guest's registration and the event reference out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the DID its members and guests already know it by. Members are named by their community-scoped `entryId`; guests and partners' members by the identifier they registered with, shown to hosts only (VTI-EVT-134, VTI-EVT-153).

### Retention

The registration is kept withdrawn.

### Consent/purpose

Letting a guest change their mind. **Events confer nothing**: the community never reads an event, a registration, a check-in, a ticket or an attendance in an authorization decision (VTI-EVT-002).
