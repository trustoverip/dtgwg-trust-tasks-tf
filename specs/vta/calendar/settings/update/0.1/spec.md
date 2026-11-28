---
slug: vta/calendar/settings/update
version: "0.1"
title: "VTA Calendar — Settings — Update"
summary: "A client changes a trust context's calendar settings: turns syncing on or off, sets the default reminders, or sets or clears quiet hours."
status: draft
targetFrameworkVersion: "0.6.0"
category: data-exchange
keywords:
  - calendar
  - events
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
    A change to the principal's own calendar, normally carried by an authenticated transport
    between a client and its own VTA. Nothing outside the VTA relies on the document.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes a record, so the VTA must be able to place the request in a bounded window to
    absorb a repeat (SPEC §7.2 item 11).
sideEffects:
  level: mutating
  rationale: >-
    The settings are kept until changed.
subjectPath: /contextId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context and the settings to change. Out: the settings.
retention:
  class: durable
  rationale: >-
    The settings are kept until changed.
errorCodes:
  - code: vta/calendar/settings/update:quietHoursInvalid
    meaning: "`quietHours` starts and ends at the same time, or names a time zone the VTA's time zone database does not know."
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/calendar/list
  - vta/calendar/sync
  - vta/calendar/settings/show
  - vta/calendar/alerts/set
---

## Abstract

The **VTA Calendar — Settings — Update** Trust Task: a client changes a trust context's calendar settings: turns syncing on or off, sets the default reminders, or sets or clears quiet hours. It is part of the member's calendar a VTA keeps for its principal across the communities it belongs to ([The Member's Calendar](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-CAL-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, phase 3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer nothing about any context the caller lacks standing in (VTI-CAL-001).
3. Change only the members the request names, record the caller and the time (VTI-CAL-070), and audit the change before answering (VTI-CAL-071).
4. Turning `sync` off stops reading the context's communities at once and keeps what was read; turning it on reads them at the next opportunity.
5. Recompute every reminder of the context the change affects (VTI-CAL-041, VTI-CAL-042).

**Retry class** (VTI-OPS-060): the second — safe to repeat: setting a value already held changes nothing.

## Authorization

**Standing in the trust context**: an access control entry whose act scope covers the context the request names — or, where it names none, each context the VTA answers for ([VTI-ACL](https://trustoverip.github.io/dtgwg-vti-spec/)). The calendar is the principal's, held per context (VTI-CAL-001), so a party without that standing learns nothing of it.

## Definitions

- `VtaCalendarEntry`, `VtaCalendarSettings`, `VtaCalendarFeed`, `VtaCalendarReceipt` and the other shapes — see [`calendar.schema.json`](../../../_shared/0.1/calendar.schema.json).
- `EventId`, `EventRef`, `RegistrationStatus` — see the vtc/events family's shared definitions.

## Request

The client sends its VTA a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0001",
  "type": "https://trusttasks.org/spec/vta/calendar/settings/update/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T09:00:00Z",
  "payload": {
    "contextId": "personal",
    "sync": true,
    "defaultReminderMinutes": [
      60
    ],
    "quietHours": {
      "startsAt": "22:00",
      "endsAt": "07:00",
      "timeZone": "Europe/London"
    }
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/calendar/settings/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0002",
  "type": "https://trusttasks.org/spec/vta/calendar/settings/update/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2026-11-27T09:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0001",
  "payload": {
    "settings": {
      "contextId": "personal",
      "sync": true,
      "defaultReminderMinutes": [
        60
      ],
      "quietHours": {
        "startsAt": "22:00",
        "endsAt": "07:00",
        "timeZone": "Europe/London"
      },
      "updatedAt": "2026-11-27T09:00:00Z",
      "updatedBy": "did:key:z6MkWalletClient"
    }
  }
}
```

## Security & Privacy

### Data carried

In: a context and the settings to change. Out: the settings.

### Correlation

The calendar joins the principal's communities in one view, which is its purpose; it is answered only to a party with standing in each context it covers and never across contexts (VTI-CAL-001, VTI-CAL-002), so a principal who keeps memberships in separate contexts keeps them apart here too. The VTA reads a community only as the membership identifier held there (VTI-CAL-011), so the calendar adds no link between the principal's identifiers that a community could see.

### Retention

The settings are kept until changed.

### Consent/purpose

The principal's own calendar, kept by their own agent. The VTA reads a community only where the principal chose to sync that context (VTI-CAL-010), never opens a notified event by itself (VTI-CAL-021), and fires reminders itself so no community learns who wanted one (VTI-CAL-040 – 043).
