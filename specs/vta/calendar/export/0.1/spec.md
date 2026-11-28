---
slug: vta/calendar/export
version: "0.1"
title: "VTA Calendar — Export"
summary: "A client exports the principal's calendar as iCalendar, together with the receipts the VTA keeps — what the member takes with them, whether or not any community still answers."
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
    Out: the principal's events and the communities' signed replies about their registrations, answers to questions included where a reply carried them. In: filters.
retention:
  class: exchange
  rationale: >-
    The export is the member's own copy; the VTA keeps its records as before.
errorCodes:
  - code: vta/calendar/export:rangeInvalid
    meaning: "`to` is not after `from`."
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/calendar/list
  - vta/calendar/sync
  - vta/calendar/settings/show
  - vta/calendar/settings/update
---

## Abstract

The **VTA Calendar — Export** Trust Task: a client exports the principal's calendar as iCalendar, together with the receipts the VTA keeps — what the member takes with them, whether or not any community still answers. It is part of the member's calendar a VTA keeps for its principal across the communities it belongs to ([The Member's Calendar](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-CAL-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, phase 3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer nothing about any context the caller lacks standing in (VTI-CAL-001).
3. Answer only the contexts the caller has standing in (VTI-CAL-002).
4. Build `ics` from what the VTA holds, without reading any community, carrying no access detail, registrant, answer or identifier of the principal (VTI-CAL-053).
5. Page the receipts with `cursor` and `limit`, each exactly as received (VTI-CAL-030, VTI-CAL-032).
6. Page with `cursor` and `limit`, refusing with `malformedRequest` a `cursor` it did not issue for the same caller and filters.

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
  "type": "https://trusttasks.org/spec/vta/calendar/export/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T09:00:00Z",
  "payload": {
    "contextIds": [
      "personal"
    ]
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/calendar/export/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0002",
  "type": "https://trusttasks.org/spec/vta/calendar/export/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2026-11-27T09:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0001",
  "payload": {
    "ics": "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//vta.example//calendar//EN\r\nBEGIN:VEVENT\r\nUID:7c1e…@vta.example\r\nDTSTART;TZID=Europe/London:20261130T183000\r\nDURATION:PT1H30M\r\nSUMMARY:Intro to Soldering\r\nLOCATION:The Makerspace\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
    "receipts": [
      {
        "receiptId": "rcpt_R1vers1deReg000001",
        "contextId": "personal",
        "community": "did:webvh:QmRiverside:riverside-makers.example",
        "eventId": "evt_Kq7N2qLm4Rt8Vx1ZsP0a",
        "replyType": "https://trusttasks.org/spec/vtc/events/registrations/register/0.2#response",
        "receivedAt": "2026-11-20T10:00:01Z",
        "document": {
          "id": "urn:uuid:0d9a8b7c-6e5f-4a3b-9c2d-1e0f9a8b7c6d",
          "type": "https://trusttasks.org/spec/vtc/events/registrations/register/0.2#response",
          "issuer": "did:webvh:QmRiverside:riverside-makers.example",
          "recipient": "did:webvh:QmMember:riverside-makers.example:members:ada",
          "issuedAt": "2026-11-20T10:00:01Z",
          "payload": {
            "registration": {
              "status": "going"
            }
          },
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "proofValue": "z3ExampleNotReal"
          }
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Out: the principal's events and the communities' signed replies about their registrations, answers to questions included where a reply carried them. In: filters.

### Correlation

The calendar joins the principal's communities in one view, which is its purpose; it is answered only to a party with standing in each context it covers and never across contexts (VTI-CAL-001, VTI-CAL-002), so a principal who keeps memberships in separate contexts keeps them apart here too. The VTA reads a community only as the membership identifier held there (VTI-CAL-011), so the calendar adds no link between the principal's identifiers that a community could see.

### Retention

The export is the member's own copy; the VTA keeps its records as before.

### Consent/purpose

The principal's own calendar, kept by their own agent. The VTA reads a community only where the principal chose to sync that context (VTI-CAL-010), never opens a notified event by itself (VTI-CAL-021), and fires reminders itself so no community learns who wanted one (VTI-CAL-040 – 043).
