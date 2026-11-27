---
slug: vtc/events/invitations/links/revoke
version: '0.1'
title: VTC Events — Invitations — Links — Revoke
summary: 'A host revokes an invitation link: its code redeems nothing from then on. Invitations already made by it stand.'
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
- role: event host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Revoking a link is a host''s decision about who may still be invited; it must be attributable.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor.'
sideEffects:
  level: mutating
  rationale: 'Marks the link revoked. Not recoverable: a host makes a new link.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'The link, never its code.'
retention:
  class: durable
  rationale: 'The revoked link is kept with its redemptions until the event''s records go.'
errorCodes:
- code: vtc/events/invitations/links/revoke:linkNotFound
  meaning: 'The event has no link with this identifier.'
  retryable: false
- code: vtc/events/invitations/links/revoke:alreadyRevoked
  meaning: 'The link was revoked already; nothing changed.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/invitations/links/create
- vtc/events/invitations/links/redeem
---

## Abstract

The **VTC Events — Invitations — Links — Revoke** Trust Task stops an invitation link (VTI-EVT-107). The code redeems nothing afterwards, answered as any code that does not redeem is. Invitations the link already made stand; a host revokes those one by one.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller does not host and may not see (VTI-EVT-023).
2. Refuse a link the event does not have with `linkNotFound`, and one already revoked with `alreadyRevoked`.
3. Record `revokedAt`; from then on refuse its code at redemption as every code that does not redeem is refused (VTI-EVT-107).
4. Leave the invitations it made as they stand.

## Authorization

**Authority: `vtc.events.manage` covering the event** — a host's grant qualified `event:<eventId>`, or the authority over the event's calendar (VTI-EVT-013, VTI-EVT-070) — read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A host's act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-EVT-095).

## Definitions

- `EventId` — see [`event.schema.json`](../../../../_shared/0.2/event.schema.json).
- `EventInvitationLink` — see [`event.schema.json`](../../../../_shared/0.2/event.schema.json).
- `EventInvitationLinkId` — see [`event.schema.json`](../../../../_shared/0.2/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Revoke a link

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000541",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/links/revoke/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "linkId": "lnk_Fl7yerLibrary0000"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-20T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/invitations/links/revoke/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Revoke a link — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000542",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/links/revoke/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000541",
  "payload": {
    "link": {
      "linkId": "lnk_Fl7yerLibrary0000",
      "eventId": "evt_IntroSoldering2611",
      "label": "Flyer at the library",
      "maxRedemptions": 20,
      "redemptions": 7,
      "expiresAt": "2026-11-29T23:59:59Z",
      "createdAt": "2026-11-20T10:00:01Z",
      "revokedAt": "2026-11-20T10:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

An event and a link identifier; the link.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

**Durable**: the revocation is kept with the link.

### Consent/purpose

Stopping a code that went further than meant.
