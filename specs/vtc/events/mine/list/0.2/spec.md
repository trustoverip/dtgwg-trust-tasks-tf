---
slug: vtc/events/mine/list
version: '0.2'
title: VTC Events — Mine — List
summary: 'List the caller''s own registrations, each with the event it belongs to as the community now states it: what a member''s agent reads to keep the member''s calendar, and reads again with updatedSince to learn what changed.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- registration
- sync
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
  rationale: The answer is the caller's own registrations, answers included; an unattributable caller gets nothing.
sideEffects:
  level: none
  rationale: Reads the caller's registrations and the events they belong to; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses the caller's own registrations — statuses, guests and their own answers — and a summary of each event. Nothing of any other registrant.
retention:
  class: transient
  rationale: A read.
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/show
- vtc/events/list
- vtc/events/registrations/register
- vtc/events/registrations/withdraw
---

## Abstract

The **VTC Events — Mine — List** Trust Task lists the caller's own registrations in a community, each with the event it belongs to as the community states it now: its title, its time, its status and its `sequence`. It is what a member's agent reads to keep the member's calendar across communities, signed as the member's identity at this community, so the community learns nothing it did not already hold: who its members are and what they registered for. Read again with `updatedSince`, it answers only what changed since — a registration a host decided, an event rescheduled or cancelled.

The single-record read is [`vtc/events/show`](../../../show/0.3/spec.md), which answers the caller's registration for one event as `myRegistration`.

## Status of this Document

Version 0.2 answers the Event and Registration of `_shared/0.2`, which carry invitation settings and partners' members (VTI Community Events chapter, VTI-EVT-100 – 135). Its Event and Registration are `vtc/events/_shared/0.2`'s.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 9.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Answer an empty list while the community has events turned off (VTI-EVT-007).
3. Answer only the caller's own registrations — every status, `withdrawn`, `declined` and `removed` included, so an agent learns a registration ended — and **MUST NOT** answer any other registrant's (VTI-EVT-053).
4. Pair each registration with an `EventRef` of its event as the event stands now: `title`, `when`, `status` and `sequence` (VTI-EVT-006). An event that was cancelled is answered `cancelled`; an event that was deleted had no registrations (VTI-EVT-093) and is never answered.
5. With `updatedSince`, answer only the items in which the registration or its event changed at or after that instant. With `statuses`, answer only registrations in one of them.
6. Include the caller's own `answers` and `codeOfConductAcknowledged` until they are deleted on the event's retention schedule (VTI-EVT-090), and never another registrant's.
7. Page as [`vtc/badges/list`](../../../../badges/list/0.1/spec.md) does: an opaque cursor bound to the caller and the filters, 50 items when `limit` is absent.

## Authorization

**Being the registrant.** The answer is the caller's own registrations, selected by the identity the proof establishes; no capability is involved and none is read. The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and who asked is the whole of the selection.

Reading one's own registrations is an act of the member (VTI-EVT-095): a member portal that holds a key the member authorized to sign for them for `vtc/events` (`auth/oob` `signingScope`) **MAY** sign it. A member's agent signs it as the member's identity at this community.

## Definitions

- `Registration`, `RegistrationStatus`, `EventRef`, `EventInstant` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### What changed since the last sync

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000401",
  "type": "https://trusttasks.org/spec/vtc/events/mine/list/0.2#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-28T08:00:00Z",
  "payload": {
    "updatedSince": "2026-11-27T08:00:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-28T08:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/mine/list/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### What changed since the last sync — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000402",
  "type": "https://trusttasks.org/spec/vtc/events/mine/list/0.2#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-28T08:00:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000401",
  "payload": {
    "items": [
      {
        "registration": {
          "registrationId": "reg_Kf3nP8qLs2Vx7Bd1",
          "eventId": "evt_IntroSoldering2611",
          "entryId": "dir_AdaLovelace000000",
          "status": "going",
          "guests": 1,
          "answers": { "dietary": "Vegetarian" },
          "roster": "hidden",
          "eventSequence": 2,
          "createdAt": "2026-11-20T19:02:11Z",
          "updatedAt": "2026-11-27T09:15:40Z"
        },
        "event": {
          "eventId": "evt_IntroSoldering2611",
          "title": "Intro to Soldering",
          "when": { "start": "2026-11-30T18:30:00", "timeZone": "Europe/London", "duration": "PT1H30M" },
          "status": "published",
          "sequence": 2
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

A time filter and statuses in; the caller's own registrations and a summary of each event out. Access details are never in an `EventRef`: they are released by [`vtc/events/access/show`](../../../access/show/0.1/spec.md) alone (VTI-EVT-061).

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. The member is identified by their identity at this community, which the community already holds; a member's agent asks each community separately, so no community learns of the member's other communities, and the agent assembles the calendar itself.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Keeping a member's own calendar, in their own agent. **An event confers nothing**: the community **MUST NOT** read a registration or an event in any authorization decision of its own (VTI-EVT-002).
