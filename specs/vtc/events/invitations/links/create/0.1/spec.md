---
slug: vtc/events/invitations/links/create
version: '0.1'
title: VTC Events — Invitations — Links — Create
summary: 'A host makes an invitation link for an event: a code an active member redeems to be invited by name, kept by the community only as a digest, with an expiry and a most number of redemptions.'
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
  rationale: 'A link invites whoever redeems it as a host''s invitation would; it must be attributable to the host who made it.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed creation would mint a second code.'
sideEffects:
  level: mutating
  rationale: 'Stores a link and the digest of its code. Recoverable with vtc/events/invitations/links/revoke.'
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: 'The response carries the code once; whoever holds it can be invited. The community keeps only its digest (VTI-EVT-107).'
retention:
  class: durable
  rationale: 'The link stands until it expires, is used up, or is revoked.'
errorCodes:
- code: vtc/events/invitations/links/create:eventCancelled
  meaning: 'The event is cancelled. Nothing was stored.'
  retryable: false
- code: vtc/events/invitations/links/create:expiryInvalid
  meaning: '`expiresAt` is not in the future, or is after the event ends. Nothing was stored.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/invitations/links/revoke
- vtc/events/invitations/links/redeem
---

## Abstract

The **VTC Events — Invitations — Links — Create** Trust Task makes an invitation link (VTI-EVT-107): a trigger link (VTI-LNK) whose code, redeemed by an active member of the community, invites them by name as a host's invitation does — admitting them to the event's audience (VTI-EVT-103). The community answers the code once and keeps only its digest, so neither its records nor its hosts can produce it again.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller does not host and may not see (VTI-EVT-023); answer `permissionDenied` to a caller who may see but does not host it.
2. Refuse a cancelled event with `eventCancelled`, and an `expiresAt` that is past or after the event ends with `expiryInvalid`.
3. Make a code of at least 256 random bits, answer it once in `code`, and keep only its SHA-256 digest (VTI-EVT-107).
4. Answer the link as its hosts will list it, never with the code again.
5. Audit the creation by reference, never the code.

## Authorization

**Authority: `vtc.events.manage` covering the event** — a host's grant qualified `event:<eventId>`, or the authority over the event's calendar (VTI-EVT-013, VTI-EVT-070) — read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A host's act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-EVT-095).

## Definitions

- `EventId` — see [`event.schema.json`](../../../../_shared/0.2/event.schema.json).
- `EventInvitationLink` — see [`event.schema.json`](../../../../_shared/0.2/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A link for a flyer

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000531",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/links/create/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "label": "Flyer at the library",
    "maxRedemptions": 20,
    "expiresAt": "2026-11-29T23:59:59Z"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/invitations/links/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A link for a flyer — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000532",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/links/create/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000531",
  "payload": {
    "link": {
      "linkId": "lnk_Fl7yerLibrary0000",
      "eventId": "evt_IntroSoldering2611",
      "label": "Flyer at the library",
      "maxRedemptions": 20,
      "redemptions": 0,
      "expiresAt": "2026-11-29T23:59:59Z",
      "createdAt": "2026-11-20T10:00:01Z"
    },
    "code": "q8Zx2mN4vB7tR1yK9pL3wQ6sD0fH5jG8aC2eV7nM4bX"
  }
}
```

## Security & Privacy

### Data carried

An event, a label, a limit and an expiry; the link and, once, its code.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

**Durable** for the link and the code's digest, until the link lapses or is revoked; the code itself is never stored.

### Consent/purpose

Inviting members the hosts cannot name — everyone who picks up a flyer. Whoever holds the code is invited; only active members of the community can redeem it.
