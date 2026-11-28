---
slug: vta/events/update
version: "0.1"
title: "VTA Events — Update"
summary: "A client changes an event the VTA holds for its principal. A change of time, access details or kind after publication raises the event's sequence and revises every invitation's notification."
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
    Changes the event record; the previous values are not kept.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    In and out: the event's changed fields and the event as it now stands.
retention:
  class: durable
  rationale: >-
    Changes the event record; the previous values are not kept.
errorCodes:
  - code: vta/events/update:eventCancelled
    meaning: "The event is cancelled and changes no more. Nothing was changed."
    retryable: false
  - code: vta/events/update:rosterLocked
    meaning: "The event is published, so its roster setting is fixed (VTI-EVT-051). Nothing was changed."
    retryable: false
  - code: vta/events/update:timeZoneUnknown
    meaning: "`when.timeZone` is not a zone the VTA's time zone database knows (VTI-EVT-005). Nothing was stored."
    retryable: false
  - code: vta/events/update:whenInvalid
    meaning: "`when.duration` is zero or longer than the VTA allows, or `registration.closesAt` is not after `opensAt`. Nothing was stored."
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/events/create
  - vta/events/publish
  - vta/events/cancel
  - vta/events/delete
---

## Abstract

The **VTA Events — Update** Trust Task: a client changes an event the VTA holds for its principal. A change of time, access details or kind after publication raises the event's sequence and revises every invitation's notification. It is part of the events a VTA holds for its principal outside any community ([Events Hosted by a Person](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-PEV-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, decision 24, phase 8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer `notFound` for an event in a context the caller has no standing in (VTI-PEV-001, VTI-PEV-003).
3. Refuse a change to a cancelled event with `eventCancelled`, and a change of `roster` on a published event with `rosterLocked` (VTI-EVT-051).
4. Raise `sequence` when the time, access details or kind of a published event changes (VTI-EVT-006, VTI-EVT-063), and then issue a new revision of the `event` notification of every invitation in force (VTI-PEV-012, VTI-NTF-065).
5. Never remove a registration because `capacity` was lowered: registrations already `going` stay `going`, and the waitlist moves only as places free.

**Retry class** (VTI-OPS-060): the second — durable effect; applying the same change again leaves the event as it is.

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
  "type": "https://trusttasks.org/spec/vta/events/update/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T12:00:00Z",
  "payload": {
    "eventId": "evt_FridayDinner2611000",
    "when": {
      "start": "2026-12-04T19:30:00",
      "timeZone": "Europe/London",
      "duration": "PT3H"
    }
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/events/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0002",
  "type": "https://trusttasks.org/spec/vta/events/update/0.1#response",
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
      "status": "published",
      "title": "Friday dinner",
      "when": {
        "start": "2026-12-04T19:30:00",
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
      "updatedAt": "2026-11-28T08:00:00Z"
    },
    "notificationsRevised": 6
  }
}
```

## Security & Privacy

### Data carried

In and out: the event's changed fields and the event as it now stands.

### Correlation

Events are held per trust context, under the persona identifier the principal uses with the people invited (VTI-PEV-001), so an event in one context is invisible to a party with standing only in another, and invitees see only the persona they already know.

### Retention

Changes the event record; the previous values are not kept.

### Consent/purpose

A gathering the principal hosts for people they know. The VTA invites only contacts the principal names (VTI-PEV-010), never acts on its own account (VTI-PEV-013, VTI-PEV-042), and discloses the guest list only as the roster setting fixed at publication allows (VTI-PEV-041). **An event confers nothing**: neither an invitation nor a registration is read in any authorization decision of the VTA's.
