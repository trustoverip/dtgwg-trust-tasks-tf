---
slug: vta/calendar/feeds/create
version: "0.1"
title: "VTA Calendar — Feeds — Create"
summary: "A client creates a private iCalendar feed of the principal's calendar for a calendar application, choosing the contexts, what it carries and when it expires; the feed's address is returned once."
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
  requirement: REQUIRED
  rationale: >-
    The response carries the feed's address, a bearer credential for the principal's calendar
    (SPEC §7.3 item 8), so both documents are attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes a record and mints a credential, so the VTA must be able to place the request
    in a bounded window to absorb a repeat (SPEC §7.2 item 11).
sideEffects:
  level: mutating
  rationale: >-
    The feed is kept until it expires or is revoked; its address only as a digest.
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: once, the feed's address — a bearer credential for the feed. In: contexts, detail and expiry.
retention:
  class: durable
  rationale: >-
    The feed is kept until it expires or is revoked; its address only as a digest.
errorCodes:
  - code: vta/calendar/feeds/create:expiryInvalid
    meaning: "`expiresAt` is not in the future, or is more than a year from now (VTI-CAL-052)."
    retryable: false
  - code: vta/calendar/feeds/create:limitReached
    meaning: "The principal holds as many live feeds as the VTA allows. Revoke one first."
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

The **VTA Calendar — Feeds — Create** Trust Task: a client creates a private iCalendar feed of the principal's calendar for a calendar application, choosing the contexts, what it carries and when it expires; the feed's address is returned once. It is part of the member's calendar a VTA keeps for its principal across the communities it belongs to ([The Member's Calendar](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-CAL-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, phase 3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer nothing about any context the caller lacks standing in (VTI-CAL-001).
3. Refuse with `permissionDenied` a caller without standing in every context `contextIds` names (VTI-CAL-050).
4. Mint an address with at least 256 bits of randomness, answer it in `url` once, and keep only its digest (VTI-CAL-051).
5. Serve the feed as a subscription interface (VTI-OPS-111 – 113) carrying, of each entry, only what `detail` names — `titles` by default — and never access details, registrants, answers, receipts or any identifier of the principal; derive each `UID` from the feed and the event (VTI-CAL-053).
6. Audit the creation, never the address (VTI-CAL-071, VTI-OPS-113).

**Retry class** (VTI-OPS-060): the third — not safe to repeat blindly: a repeat creates a second feed with a second address.

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
  "type": "https://trusttasks.org/spec/vta/calendar/feeds/create/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T09:00:00Z",
  "payload": {
    "label": "Phone calendar",
    "contextIds": [
      "personal"
    ],
    "expiresAt": "2027-05-27T09:00:00Z"
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/calendar/feeds/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0002",
  "type": "https://trusttasks.org/spec/vta/calendar/feeds/create/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2026-11-27T09:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0001",
  "payload": {
    "feed": {
      "feedId": "feed_Ph0n3Ca1endar00001",
      "label": "Phone calendar",
      "contextIds": [
        "personal"
      ],
      "detail": "titles",
      "tiers": [
        "mine"
      ],
      "createdAt": "2026-11-27T09:00:00Z",
      "expiresAt": "2027-05-27T09:00:00Z",
      "fetchCount": 0
    },
    "url": "https://vta.example/calendar/feeds/Xq9ExampleAddressNotReal0000000000000000000.ics"
  }
}
```

## Security & Privacy

### Data carried

Out: once, the feed's address — a bearer credential for the feed. In: contexts, detail and expiry.

### Correlation

The calendar joins the principal's communities in one view, which is its purpose; it is answered only to a party with standing in each context it covers and never across contexts (VTI-CAL-001, VTI-CAL-002), so a principal who keeps memberships in separate contexts keeps them apart here too. The VTA reads a community only as the membership identifier held there (VTI-CAL-011), so the calendar adds no link between the principal's identifiers that a community could see.

### Retention

The feed is kept until it expires or is revoked; its address only as a digest.

### Consent/purpose

The principal's own calendar, kept by their own agent. The VTA reads a community only where the principal chose to sync that context (VTI-CAL-010), never opens a notified event by itself (VTI-CAL-021), and fires reminders itself so no community learns who wanted one (VTI-CAL-040 – 043).
