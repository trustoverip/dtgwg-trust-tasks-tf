---
slug: vta/calendar/feeds/revoke
version: "0.1"
title: "VTA Calendar — Feeds — Revoke"
summary: "A client revokes one of the principal's calendar feeds; its address stops working at once."
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
    The feed is kept as revoked, for its listing.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a feed identifier. Out: the feed as revoked.
retention:
  class: durable
  rationale: >-
    The feed is kept as revoked, for its listing.
errorCodes:
  - code: vta/calendar/feeds/revoke:feedNotFound
    meaning: "No feed with this identifier that the caller has standing in every context of."
    retryable: false
  - code: vta/calendar/feeds/revoke:alreadyRevoked
    meaning: "The feed is already revoked or expired."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"revokedAt": {"type": "string", "format": "date-time"}}}
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/calendar/list
  - vta/calendar/sync
  - vta/calendar/settings/show
  - vta/calendar/settings/update
---

## Abstract

The **VTA Calendar — Feeds — Revoke** Trust Task: a client revokes one of the principal's calendar feeds; its address stops working at once. It is part of the member's calendar a VTA keeps for its principal across the communities it belongs to ([The Member's Calendar](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-CAL-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, phase 3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer nothing about any context the caller lacks standing in (VTI-CAL-001).
3. Answer `feedNotFound` alike for a feed that does not exist and one the caller lacks standing in any context of (VTI-CAL-052).
4. Stop serving the address before answering, and answer it afterwards as a feed that never existed (VTI-CAL-052).
5. Audit the revocation (VTI-CAL-071).

**Retry class** (VTI-OPS-060): the second — safe to repeat: a repeat is `alreadyRevoked` and changes nothing.

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
  "type": "https://trusttasks.org/spec/vta/calendar/feeds/revoke/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T09:00:00Z",
  "payload": {
    "feedId": "feed_Ph0n3Ca1endar00001"
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/calendar/feeds/revoke/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-0000000c0002",
  "type": "https://trusttasks.org/spec/vta/calendar/feeds/revoke/0.1#response",
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
      "lastFetchedAt": "2026-11-27T09:14:00Z",
      "fetchCount": 3,
      "revokedAt": "2026-11-28T08:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

In: a feed identifier. Out: the feed as revoked.

### Correlation

The calendar joins the principal's communities in one view, which is its purpose; it is answered only to a party with standing in each context it covers and never across contexts (VTI-CAL-001, VTI-CAL-002), so a principal who keeps memberships in separate contexts keeps them apart here too. The VTA reads a community only as the membership identifier held there (VTI-CAL-011), so the calendar adds no link between the principal's identifiers that a community could see.

### Retention

The feed is kept as revoked, for its listing.

### Consent/purpose

The principal's own calendar, kept by their own agent. The VTA reads a community only where the principal chose to sync that context (VTI-CAL-010), never opens a notified event by itself (VTI-CAL-021), and fires reminders itself so no community learns who wanted one (VTI-CAL-040 – 043).
