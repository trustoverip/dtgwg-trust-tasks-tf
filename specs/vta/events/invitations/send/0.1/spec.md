---
slug: vta/events/invitations/send
version: "0.1"
title: "VTA Events — Invitations — Send"
summary: "A client invites contacts of the event's host the principal names: the VTA records each invitation and sends the invitee an event notification naming the host, and nothing else of the event."
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
    Records invitations, kept with the event; each sends a notification the invitee's agent holds until it lapses or is withdrawn.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    In: identifiers of the principal's contacts. Out: the invitations recorded. The notification sent carries the host, the event identifier and its period only (VTI-NTF-003).
retention:
  class: durable
  rationale: >-
    Records invitations, kept with the event; each sends a notification the invitee's agent holds until it lapses or is withdrawn.
errorCodes:
  - code: vta/events/invitations/send:notContact
    meaning: "An invitee is not a contact of the host in its trust context — no verifiable relationship credential under the host and no recorded contact (VTI-PEV-010). Nothing was sent to anyone."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["invitees"], "properties": {"invitees": {"type": "array", "maxItems": 50, "items": {"type": "string", "maxLength": 2048}}}}
  - code: vta/events/invitations/send:notPublished
    meaning: "The event is a draft or is cancelled; invite to a published event. Nothing was sent."
    retryable: false
  - code: vta/events/invitations/send:eventEnded
    meaning: "The event has ended. Nothing was sent."
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

The **VTA Events — Invitations — Send** Trust Task: a client invites contacts of the event's host the principal names: the VTA records each invitation and sends the invitee an event notification naming the host, and nothing else of the event. It is part of the events a VTA holds for its principal outside any community ([Events Hosted by a Person](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-PEV-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, decision 24, phase 8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization), and answer `notFound` for an event in a context the caller has no standing in (VTI-PEV-001, VTI-PEV-003).
3. Check every invitee before inviting any: each **MUST** be a contact of the host in the event's trust context (VTI-PEV-010); otherwise refuse the whole request with `notContact`, naming the ones that are not.
4. Record each invitation before sending its notification (VTI-PEV-011), and send it as an `event` notification signed as the host, with the host as the thing's holder, the invitee's identifier as the audience, `vta/events/invitee/show` as the opening operation and the event's period — nothing else of the event (VTI-NTF-064, VTI-NTF-068).
5. Treat an invitee already invited (`active`) as invited: answer the invitation held and send nothing new. Re-activate a `revoked` invitation and send a fresh notification.
6. Invite only on this request: never on the VTA's own account, never to a party the request does not name (VTI-PEV-013, VTI-PEV-042).
7. Audit each invitation by reference — event, invitation — never the invitee's relationship records (VTI-PEV-043).

**Retry class** (VTI-OPS-060): the second — durable effect; a repeat finds each invitee already invited and sends nothing new.

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
  "type": "https://trusttasks.org/spec/vta/events/invitations/send/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-11-27T12:00:00Z",
  "payload": {
    "eventId": "evt_FridayDinner2611000",
    "invitees": [
      "did:key:z6MkGraceHopperPairwise"
    ]
  }
}
```

## Response

The VTA answers with a document of type `https://trusttasks.org/spec/vta/events/invitations/send/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0002",
  "type": "https://trusttasks.org/spec/vta/events/invitations/send/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2026-11-27T12:00:01Z",
  "threadId": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0001",
  "payload": {
    "invitations": [
      {
        "invitationId": "inv_Gr4ceDinner000001",
        "eventId": "evt_FridayDinner2611000",
        "invitee": "did:key:z6MkGraceHopperPairwise",
        "status": "active",
        "createdAt": "2026-11-27T10:01:00Z",
        "updatedAt": "2026-11-27T10:01:00Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

In: identifiers of the principal's contacts. Out: the invitations recorded. The notification sent carries the host, the event identifier and its period only (VTI-NTF-003).

### Correlation

Events are held per trust context, under the persona identifier the principal uses with the people invited (VTI-PEV-001), so an event in one context is invisible to a party with standing only in another, and invitees see only the persona they already know.

### Retention

Records invitations, kept with the event; each sends a notification the invitee's agent holds until it lapses or is withdrawn.

### Consent/purpose

A gathering the principal hosts for people they know. The VTA invites only contacts the principal names (VTI-PEV-010), never acts on its own account (VTI-PEV-013, VTI-PEV-042), and discloses the guest list only as the roster setting fixed at publication allows (VTI-PEV-041). **An event confers nothing**: neither an invitation nor a registration is read in any authorization decision of the VTA's.
