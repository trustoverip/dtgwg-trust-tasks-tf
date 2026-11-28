---
slug: vta/events/cancel
version: "0.1"
title: "VTA Events — Cancel"
summary: "A client cancels a published event: it takes no more registrations, its registrants' receipts stand, and every invitation's notification is withdrawn."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - events
  - invitations
  - vta
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    An operation of the principal's own client on its own VTA, normally carried by an
    authenticated transport between them.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor; a replay would undo a later
    change.
sideEffects:
  level: mutating
  rationale: >-
    Changes the event's status; the event and its registrations are kept.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    In: an event identifier and a reason. Out: the event as cancelled.
retention:
  class: durable
  rationale: >-
    Changes the event's status; the event and its registrations are kept.
errorCodes:
  - code: vta/events/cancel:notPublished
    meaning: "The event is a draft (delete it with vta/events/delete) or is already cancelled."
    retryable: false
  - code: vta/events/cancel:eventEnded
    meaning: "The event has already ended."
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/events/create
  - vta/events/update
  - vta/events/publish
  - vta/events/delete
---

## Abstract

The **VTA Events — Cancel** Trust Task: a client cancels a published event: it takes no more registrations, its registrants' receipts stand, and every invitation's notification is withdrawn. It is part of the events a VTA holds for its principal outside any community ([Events Hosted by a Person](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-PEV-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, decision 24, phase 8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer `notFound` for an event in a context the caller has no standing in (VTI-PEV-001, VTI-PEV-003).
3. Cancel only a published event that has not ended, raise its `sequence`, keep it and its registrations (VTI-EVT-004), and withdraw the `event` notification of every invitation (VTI-PEV-012, VTI-NTF-065).
4. Keep accepting withdrawals from registrants after cancellation (VTI-EVT-037).
5. Audit the cancellation by reference (VTI-PEV-043).

**Retry class** (VTI-OPS-060): the second — durable effect; a repeat is refused with `notPublished`, which a retry reads as done.

## Authorization

**Standing in the trust context**: an access control entry whose act scope covers the context the event is held in ([VTI-ACL](https://trustoverip.github.io/dtgwg-vti-spec/)). An invitee never has it by being invited (VTI-PEV-003).

## Definitions

- `VtaEvent`, `VtaEventInviteeView`, `VtaEventInvitation`, `VtaEventRegistration`, `VtaEventRosterEntry` and the other shapes — see [`events.schema.json`](../../_shared/0.1/events.schema.json).
- `EventId`, `When`, `RegistrationSettings`, `AccessDetails`, `RegistrationStatus`, `EventRef` — the vtc/events family's shared definitions, reused unchanged.
- **Host** — the persona identifier the event is held under (VTI-PEV-001).

## Request

The client sends its VTA a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0001",
  "type": "https://trusttasks.org/spec/vta/events/cancel/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T12:00:00Z",
  "payload": {
    "eventId": "evt_FridayDinner2611000",
    "reason": "Ada is unwell — another week."
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/events/cancel/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0002",
  "type": "https://trusttasks.org/spec/vta/events/cancel/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2026-11-27T12:00:01Z",
  "threadId": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0001",
  "payload": {
    "event": {
      "eventId": "evt_FridayDinner2611000",
      "contextId": "friends",
      "host": "did:webvh:QmAda:ada.example:dinner",
      "kind": "event",
      "status": "cancelled",
      "title": "Friday dinner",
      "when": {
        "start": "2026-12-04T19:00:00",
        "timeZone": "Europe/London",
        "duration": "PT3H"
      },
      "placeSummary": "Leeds",
      "registration": {
        "mode": "open",
        "capacity": 6,
        "waitlist": true,
        "plusOnes": 1,
        "questions": [
          {
            "questionId": "dietary",
            "label": "Anything you don't eat?",
            "kind": "text",
            "required": false
          }
        ]
      },
      "roster": "visible",
      "suggestedReminderMinutes": [
        1440
      ],
      "goingCount": 3,
      "invitationCount": 6,
      "full": false,
      "sequence": 2,
      "publishedAt": "2026-11-27T10:00:00Z",
      "createdAt": "2026-11-27T09:30:00Z",
      "updatedAt": "2026-12-03T17:00:00Z",
      "cancelReason": "Ada is unwell — another week.",
      "cancelledAt": "2026-12-03T17:00:00Z"
    },
    "notificationsWithdrawn": 6
  }
}
```

## Security & Privacy

### Data carried

In: an event identifier and a reason. Out: the event as cancelled.

### Correlation

Events are held per trust context, under the persona identifier the principal uses with the people invited (VTI-PEV-001), so an event in one context is invisible to a party with standing only in another, and invitees see only the persona they already know.

### Retention

Changes the event's status; the event and its registrations are kept.

### Consent/purpose

A gathering the principal hosts for people they know. The VTA invites only contacts the principal names (VTI-PEV-010), never acts on its own account (VTI-PEV-013, VTI-PEV-042), and discloses the guest list only as the roster setting fixed at publication allows (VTI-PEV-041). **An event confers nothing**: neither an invitation nor a registration is read in any authorization decision of the VTA's.
