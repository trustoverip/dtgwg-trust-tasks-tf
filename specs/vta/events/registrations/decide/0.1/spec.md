---
slug: vta/events/registrations/decide
version: "0.1"
title: "VTA Events — Registrations — Decide"
summary: "A client accepts or declines a pending registration, or removes one, for an event the VTA holds; the registrant's next read shows the decision."
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
    Changes the registration's status, kept with the event.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    In: a registration and a decision. Out: the registration and a summary of the event.
retention:
  class: durable
  rationale: >-
    Changes the registration's status, kept with the event.
errorCodes:
  - code: vta/events/registrations/decide:notPending
    meaning: "`accept` or `decline` of a registration that is not pending, or `remove` of one already withdrawn, declined or removed. Nothing changed."
    retryable: false
  - code: vta/events/registrations/decide:full
    meaning: "Accepting would exceed the event's capacity and it keeps no waitlist. The registration stays pending."
    retryable: true
  - code: vta/events/registrations/decide:eventCancelled
    meaning: "The event is cancelled; its registrations are not decided. Nothing changed."
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/events/create
  - vta/events/update
  - vta/events/publish
  - vta/events/cancel
---

## Abstract

The **VTA Events — Registrations — Decide** Trust Task: a client accepts or declines a pending registration, or removes one, for an event the VTA holds; the registrant's next read shows the decision. It is part of the events a VTA holds for its principal outside any community ([Events Hosted by a Person](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-PEV-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, decision 24, phase 8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer `notFound` for an event in a context the caller has no standing in (VTI-PEV-001, VTI-PEV-003).
3. Decide as VTI-EVT-031 – 033 do: `accept` makes a pending registration `going` where it fits, else `waitlisted` where the event keeps a waitlist, else refuses with `full`; `decline` and `remove` free any place it held and move the waitlist in order.
4. Answer the registration and the event as they stand; the registrant reads the decision with vta/events/invitee/show or vta/events/invitee/mine/list, and the VTA sends them nothing of its own beyond its signed answer to their own requests.
5. Audit the decision by reference — event, registration, decision — never its note (VTI-PEV-043).

**Retry class** (VTI-OPS-060): the second — durable effect; a repeated decision is refused with `notPending`, which a retry reads as done.

## Authorization

**Standing in the trust context**: an access control entry whose act scope covers the context the event is held in ([VTI-ACL](https://trustoverip.github.io/dtgwg-vti-spec/)). An invitee never has it by being invited (VTI-PEV-003).

## Definitions

- `VtaEvent`, `VtaEventInviteeView`, `VtaEventInvitation`, `VtaEventRegistration`, `VtaEventRosterEntry` and the other shapes — see [`events.schema.json`](../../../_shared/0.1/events.schema.json).
- `EventId`, `When`, `RegistrationSettings`, `AccessDetails`, `RegistrationStatus`, `EventRef` — the vtc/events family's shared definitions, reused unchanged.
- **Host** — the persona identifier the event is held under (VTI-PEV-001).

## Request

The client sends its VTA a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0001",
  "type": "https://trusttasks.org/spec/vta/events/registrations/decide/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T12:00:00Z",
  "payload": {
    "eventId": "evt_FridayDinner2611000",
    "registrationId": "reg_Gr4ceDinner000001",
    "decision": "accept"
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/events/registrations/decide/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0002",
  "type": "https://trusttasks.org/spec/vta/events/registrations/decide/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2026-11-27T12:00:01Z",
  "threadId": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0001",
  "payload": {
    "registration": {
      "registrationId": "reg_Gr4ceDinner000001",
      "eventId": "evt_FridayDinner2611000",
      "registrantDid": "did:key:z6MkGraceHopperPairwise",
      "displayName": "Grace",
      "status": "going",
      "guests": 1,
      "answers": {
        "dietary": "No shellfish"
      },
      "roster": "visible",
      "eventSequence": 1,
      "createdAt": "2026-11-27T12:00:00Z",
      "updatedAt": "2026-11-27T12:00:00Z"
    },
    "event": {
      "eventId": "evt_FridayDinner2611000",
      "title": "Friday dinner",
      "when": {
        "start": "2026-12-04T19:00:00",
        "timeZone": "Europe/London",
        "duration": "PT3H"
      },
      "status": "published",
      "sequence": 1,
      "suggestedReminderMinutes": [
        1440
      ],
      "placeSummary": "Leeds"
    }
  }
}
```

## Security & Privacy

### Data carried

In: a registration and a decision. Out: the registration and a summary of the event.

### Correlation

Events are held per trust context, under the persona identifier the principal uses with the people invited (VTI-PEV-001), so an event in one context is invisible to a party with standing only in another, and invitees see only the persona they already know.

### Retention

Changes the registration's status, kept with the event.

### Consent/purpose

A gathering the principal hosts for people they know. The VTA invites only contacts the principal names (VTI-PEV-010), never acts on its own account (VTI-PEV-013, VTI-PEV-042), and discloses the guest list only as the roster setting fixed at publication allows (VTI-PEV-041). **An event confers nothing**: neither an invitation nor a registration is read in any authorization decision of the VTA's.
