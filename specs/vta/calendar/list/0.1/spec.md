---
slug: vta/calendar/list
version: "0.1"
title: "VTA Calendar — List"
summary: "A client reads the principal's calendar across the trust contexts it has standing in: the events the principal registered for, as their communities answered them, and the events it has only been told of, as their notifications state them."
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
    A read that changes nothing, normally carried by an authenticated transport between a client
    and its own VTA.
sideEffects:
  level: none
  rationale: "Reads the principal's calendar; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: the principal's events in the contexts the caller may act in — titles, times and place summaries of `mine` entries, communities and periods of `notified` ones. In: filters.
retention:
  class: exchange
  rationale: >-
    The VTA remains the record; the page is the client's working copy for the view it renders.
errorCodes:
  - code: vta/calendar/list:rangeInvalid
    meaning: "`to` is not after `from`, or more than 400 days after it."
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/calendar/sync
  - vta/calendar/settings/show
  - vta/calendar/settings/update
  - vta/calendar/alerts/set
---

## Abstract

The **VTA Calendar — List** Trust Task: a client reads the principal's calendar across the trust contexts it has standing in: the events the principal registered for, as their communities answered them, and the events it has only been told of, as their notifications state them. It is part of the member's calendar a VTA keeps for its principal across the communities it belongs to ([The Member's Calendar](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-CAL-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, phase 3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer nothing about any context the caller lacks standing in (VTI-CAL-001).
3. Answer only the contexts the caller has standing in, and of those only the ones `contextIds` names where given; never indicate that other contexts hold anything (VTI-CAL-002).
4. Return `mine` entries as their communities last answered them (VTI-CAL-013) and `notified` entries only as their held `event` notification states them — community, event identifier, addressee and period, never a title (VTI-CAL-020, VTI-NTF-067).
5. Mark a withdrawn notification `withdrawn` and leave a `mine` entry whose event was cancelled with its `cancelled` status (VTI-CAL-022).
6. Set `reminders` and `reminderSource` as VTI-CAL-040 – 041 compute them, and `overlaps` only among entries in the same answer (VTI-CAL-023).
7. **MUST NOT** read an event from its community, open a notified event, or send anything to any community because a client listed the calendar (VTI-CAL-021).
8. Page with `cursor` and `limit`, refusing with `malformedRequest` a `cursor` it did not issue for the same caller and filters.

**Retry class** (VTI-OPS-060): the first — no durable effect.

## Authorization

**Standing in the trust context**: an access control entry whose act scope covers the context the request names — or, where it names none, each context the VTA answers for ([VTI-ACL](https://trustoverip.github.io/dtgwg-vti-spec/)). The calendar is the principal's, held per context (VTI-CAL-001), so a party without that standing learns nothing of it.

## Definitions

- `VtaCalendarEntry`, `VtaCalendarSettings`, `VtaCalendarFeed`, `VtaCalendarReceipt` and the other shapes — see [`calendar.schema.json`](../../_shared/0.1/calendar.schema.json).
- `EventId`, `EventRef`, `RegistrationStatus` — see the vtc/events family's shared definitions.

## Request

The client sends its VTA a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0001",
  "type": "https://trusttasks.org/spec/vta/calendar/list/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T09:00:00Z",
  "payload": {
    "from": "2026-11-30T00:00:00Z",
    "to": "2026-12-07T00:00:00Z"
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/calendar/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0002",
  "type": "https://trusttasks.org/spec/vta/calendar/list/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2026-11-27T09:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0001",
  "payload": {
    "items": [
      {
        "contextId": "personal",
        "community": "did:webvh:QmRiverside:riverside-makers.example",
        "eventId": "evt_Kq7N2qLm4Rt8Vx1ZsP0a",
        "tier": "mine",
        "period": {
          "startsAt": "2026-11-30T18:30:00Z",
          "endsAt": "2026-11-30T20:00:00Z"
        },
        "identifier": "did:webvh:QmMember:riverside-makers.example:members:ada",
        "event": {
          "eventId": "evt_Kq7N2qLm4Rt8Vx1ZsP0a",
          "title": "Intro to Soldering",
          "when": {
            "start": "2026-11-30T18:30:00",
            "timeZone": "Europe/London",
            "duration": "PT1H30M"
          },
          "status": "published",
          "sequence": 2
        },
        "placeSummary": "The Makerspace",
        "registrationStatus": "going",
        "reminders": [
          1440,
          60
        ],
        "reminderSource": "event",
        "readAt": "2026-11-27T09:00:00Z"
      },
      {
        "contextId": "personal",
        "community": "did:web:openledger.example",
        "eventId": "evt_Op3nL3dg3rGu1ld0001",
        "tier": "notified",
        "period": {
          "startsAt": "2026-12-02T19:00:00Z",
          "endsAt": "2026-12-02T21:00:00Z"
        },
        "identifier": "did:key:z6MkLedgerMember",
        "notificationRecordId": "hn-4Tq8Zr2Lw9Ks6Pm1",
        "reminders": [],
        "reminderSource": "none"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Out: the principal's events in the contexts the caller may act in — titles, times and place summaries of `mine` entries, communities and periods of `notified` ones. In: filters.

### Correlation

The calendar joins the principal's communities in one view, which is its purpose; it is answered only to a party with standing in each context it covers and never across contexts (VTI-CAL-001, VTI-CAL-002), so a principal who keeps memberships in separate contexts keeps them apart here too. The VTA reads a community only as the membership identifier held there (VTI-CAL-011), so the calendar adds no link between the principal's identifiers that a community could see.

### Retention

The VTA remains the record; the page is the client's working copy for the view it renders.

### Consent/purpose

The principal's own calendar, kept by their own agent. The VTA reads a community only where the principal chose to sync that context (VTI-CAL-010), never opens a notified event by itself (VTI-CAL-021), and fires reminders itself so no community learns who wanted one (VTI-CAL-040 – 043).
