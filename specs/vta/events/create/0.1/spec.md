---
slug: vta/events/create
version: "0.1"
title: "VTA Events — Create"
summary: "A client creates a draft event the VTA holds for its principal under one of the principal's persona identifiers, outside any community."
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
    Creates an event record in the host's trust context, kept until deleted.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    In and out: the event — title, time, place, access details, registration settings. Nothing about any invitee.
retention:
  class: durable
  rationale: >-
    Creates an event record in the host's trust context, kept until deleted.
errorCodes:
  - code: vta/events/create:hostUnknown
    meaning: "`host` is not an identifier the VTA holds for its principal in `contextId`. Nothing was stored."
    retryable: false
  - code: vta/events/create:timeZoneUnknown
    meaning: "`when.timeZone` is not a zone the VTA's time zone database knows (VTI-EVT-005). Nothing was stored."
    retryable: false
  - code: vta/events/create:whenInvalid
    meaning: "`when.duration` is zero or longer than the VTA allows, or `registration.closesAt` is not after `opensAt`. Nothing was stored."
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/events/update
  - vta/events/publish
  - vta/events/cancel
  - vta/events/delete
---

## Abstract

The **VTA Events — Create** Trust Task: a client creates a draft event the VTA holds for its principal under one of the principal's persona identifiers, outside any community. It is part of the events a VTA holds for its principal outside any community ([Events Hosted by a Person](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-PEV-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, decision 24, phase 8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer `notFound` for an event in a context the caller has no standing in (VTI-PEV-001, VTI-PEV-003).
3. Refuse a `host` that is not an identifier the VTA holds in `contextId` with `hostUnknown`, and hold the event in that context (VTI-PEV-001).
4. Create the event as a `draft` with `sequence` 0; a draft is shown only to parties with standing in the context and takes no registrations (VTI-EVT-004).
5. Accept `roster` only as `visible` or `hidden` (VTI-PEV-002) — the schema admits nothing else.
6. Audit the creation by reference — event, context, host — never its title or access details (VTI-PEV-043).

**Retry class** (VTI-OPS-060): the second — durable effect; a repeat with the same document `id` answers the event it created.

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
  "type": "https://trusttasks.org/spec/vta/events/create/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T12:00:00Z",
  "payload": {
    "contextId": "friends",
    "host": "did:webvh:QmAda:ada.example:dinner",
    "title": "Friday dinner",
    "when": {
      "start": "2026-12-04T19:00:00",
      "timeZone": "Europe/London",
      "duration": "PT3H"
    },
    "placeSummary": "Leeds",
    "access": {
      "places": [
        {
          "name": "Ada's flat",
          "address": "14 Albion Street, Leeds LS1 6AB"
        }
      ],
      "notes": "Ring the top bell."
    },
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
    ]
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/events/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0002",
  "type": "https://trusttasks.org/spec/vta/events/create/0.1#response",
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
      "status": "draft",
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
      "goingCount": 0,
      "invitationCount": 0,
      "full": false,
      "sequence": 0,
      "createdAt": "2026-11-27T09:30:00Z",
      "updatedAt": "2026-11-27T09:30:00Z",
      "access": {
        "places": [
          {
            "name": "Ada's flat",
            "address": "14 Albion Street, Leeds LS1 6AB"
          }
        ],
        "notes": "Ring the top bell."
      }
    }
  }
}
```

## Security & Privacy

### Data carried

In and out: the event — title, time, place, access details, registration settings. Nothing about any invitee.

### Correlation

Events are held per trust context, under the persona identifier the principal uses with the people invited (VTI-PEV-001), so an event in one context is invisible to a party with standing only in another, and invitees see only the persona they already know.

### Retention

Creates an event record in the host's trust context, kept until deleted.

### Consent/purpose

A gathering the principal hosts for people they know. The VTA invites only contacts the principal names (VTI-PEV-010), never acts on its own account (VTI-PEV-013, VTI-PEV-042), and discloses the guest list only as the roster setting fixed at publication allows (VTI-PEV-041). **An event confers nothing**: neither an invitation nor a registration is read in any authorization decision of the VTA's.
