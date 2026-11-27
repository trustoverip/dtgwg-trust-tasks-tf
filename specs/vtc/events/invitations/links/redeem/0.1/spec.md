---
slug: vtc/events/invitations/links/redeem
version: '0.1'
title: VTC Events — Invitations — Links — Redeem
summary: 'An active member redeems an invitation link''s code and is invited to its event by name, as a host would invite them.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- invitation
- link
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Redeeming admits the redeemer to an event''s audience; it must be attributable to them.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed redemption would count twice.'
sideEffects:
  level: mutating
  rationale: 'Records an invitation for the caller and counts the redemption. Recoverable: a host revokes the invitation.'
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: 'The request carries a code that invites whoever holds it.'
retention:
  class: durable
  rationale: 'The invitation stands until revoked or the event ends.'
errorCodes:
- code: vtc/events/invitations/links/redeem:linkInvalid
  meaning: 'The code redeems nothing: unknown, expired, used up, revoked, or already redeemed by the caller. One answer for all (VTI-EVT-107).'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/invitations/links/create
- vtc/events/invitations/respond
---

## Abstract

The **VTC Events — Invitations — Links — Redeem** Trust Task turns an invitation link's code into an invitation by name (VTI-EVT-107): the caller, an active member of the community, is invited to the link's event, admitted to its audience while the invitation stands (VTI-EVT-103), and sent its `event` notification. Every code that does not redeem gets the same answer, so a refusal says nothing about which codes exist.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Look the code up by its SHA-256 digest only. Refuse with `linkInvalid` — one answer, whatever the cause — a code that is unknown, past its expiry, at its most redemptions, revoked, or already redeemed by the caller, or whose event is cancelled or ended (VTI-EVT-107).
3. Record an invitation for the caller with `invitedBy: link`, or answer the one that already stands; count the redemption once.
4. Send the caller the event's `event` notification as any invitation's (VTI-EVT-102).
5. Answer the invitation and the event as `EventRef`.
6. Audit the redemption by reference — the link and the caller's entry — never the code.

## Authorization

**Standing in the community.** The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and the community reads their standing from its own records at execution time. A member's act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `EventInvitation` — see [`event.schema.json`](../../../../_shared/0.2/event.schema.json).
- `EventRef` — see [`event.schema.json`](../../../../_shared/0.2/event.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Redeem a link

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000551",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/links/redeem/0.1#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "code": "q8Zx2mN4vB7tR1yK9pL3wQ6sD0fH5jG8aC2eV7nM4bX"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-20T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/invitations/links/redeem/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Redeem a link — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000552",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/links/redeem/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000551",
  "payload": {
    "invitation": {
      "invitationId": "inv_W5k2N8xRb3Qz7TpL",
      "eventId": "evt_IntroSoldering2611",
      "entryId": "dir_AdaLovelace0000000",
      "invitedBy": "link",
      "replyExpected": false,
      "answer": "none",
      "createdAt": "2026-11-20T10:00:01Z"
    },
    "event": {
      "eventId": "evt_IntroSoldering2611",
      "title": "Intro to Soldering",
      "when": {
        "start": "2026-11-30T18:30:00",
        "timeZone": "Europe/London",
        "duration": "PT1H30M"
      },
      "status": "published",
      "sequence": 0
    }
  }
}
```

## Security & Privacy

### Data carried

A code; the caller's invitation and a summary of its event.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

**Durable** for the invitation; the code is never stored, only matched by digest.

### Consent/purpose

Joining an event from a shared link, as a member of the community, without a host naming them.
