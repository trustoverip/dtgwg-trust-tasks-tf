---
slug: vta/events/invitee/withdraw
version: "0.1"
title: "VTA Events — Invitee — Withdraw"
summary: "An invitee withdraws their registration for an event a person invited them to, at any time, even after the event is cancelled or their invitation revoked."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - events
  - invitations
  - vta
parties:
  - role: Invitee
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Event host (the principal's VTA, as the persona)
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The host accepts invitee operations only from the identifier an invitation names, so the
    request must be bound to that identifier by its proof, whatever transport carried it
    (VTI-PEV-020); the host's signed answer is a receipt only if the request it answers is.
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
    In: an event identifier. Out: the registration as withdrawn and a summary of the event.
retention:
  class: durable
  rationale: >-
    Changes the registration's status, kept with the event.
errorCodes:
  - code: vta/events/invitee/withdraw:notRegistered
    meaning: "The invitee holds no registration for the event that can be withdrawn — none, or one already withdrawn, declined or removed."
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - vta/events/invitee/show
  - vta/events/invitee/register
  - vta/events/invitee/access/show
  - vta/events/invitee/mine/list
---

## Abstract

The **VTA Events — Invitee — Withdraw** Trust Task: an invitee withdraws their registration for an event a person invited them to, at any time, even after the event is cancelled or their invitation revoked. It is part of the events a VTA holds for its principal outside any community ([Events Hosted by a Person](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-PEV-*).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`, decision 24, phase 8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the host's VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Accept the request only as a signed Trust Task addressed to the host (`recipient`), whose proven signer (`issuer`) is the identifier an invitation in force names for the event the request names; answer every other request with `notFound`, exactly as for an event that does not exist (VTI-PEV-020, VTI-PEV-040).
3. Charge the request to the budget the VTA keeps for senders without standing, never to a budget kept for the principal's clients, whatever the invitee's relationship with the principal (VTI-PEV-021, VTI-TRN-029).
4. Accept a withdrawal from the registrant even where their invitation has since been revoked (VTI-PEV-023) or the event cancelled, and never park or hold it (VTI-EVT-037); from any other sender answer `notFound`.
5. Free any place the registration held and move the waitlist in order (VTI-EVT-033), and answer signed as the host; the answer is the invitee's receipt (VTI-PEV-022).

**Retry class** (VTI-OPS-060): the second — durable effect; a repeat is refused with `notRegistered`, which a retry reads as done.

## Authorization

**An invitation in force.** The proven signer must be the identifier an `active` invitation of the event names (VTI-PEV-020); for vta/events/invitee/withdraw, the registrant, whatever became of the invitation (VTI-PEV-023). The invitation confers nothing else: no standing in any trust context of the VTA, and nothing beyond this one event. An invitee's VTA signs as the invited identifier only for its principal, and never on its own account (VTI-PEV-042).

## Definitions

- `VtaEvent`, `VtaEventInviteeView`, `VtaEventInvitation`, `VtaEventRegistration`, `VtaEventRosterEntry` and the other shapes — see [`events.schema.json`](../../../_shared/0.1/events.schema.json).
- `EventId`, `When`, `RegistrationSettings`, `AccessDetails`, `RegistrationStatus`, `EventRef` — the vtc/events family's shared definitions, reused unchanged.
- **Host** — the persona identifier the event is held under (VTI-PEV-001).

## Request

The invitee — or the invitee's VTA, signing as the invited identifier — sends the host a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0001",
  "type": "https://trusttasks.org/spec/vta/events/invitee/withdraw/0.1",
  "issuer": "did:key:z6MkGraceHopperPairwise",
  "recipient": "did:webvh:QmAda:ada.example:dinner",
  "issuedAt": "2026-11-27T12:00:00Z",
  "payload": {
    "eventId": "evt_FridayDinner2611000"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkGraceHopperPairwise#z6MkGraceHopperPairwise",
    "created": "2026-11-27T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The host answers with a document of type `https://trusttasks.org/spec/vta/events/invitee/withdraw/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

```json
{
  "id": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0002",
  "type": "https://trusttasks.org/spec/vta/events/invitee/withdraw/0.1#response",
  "issuer": "did:webvh:QmAda:ada.example:dinner",
  "recipient": "did:key:z6MkGraceHopperPairwise",
  "issuedAt": "2026-11-27T12:00:01Z",
  "threadId": "urn:uuid:7a1e4c2d-9b3f-4e6a-8d0c-0000000e0001",
  "payload": {
    "registration": {
      "registrationId": "reg_Gr4ceDinner000001",
      "eventId": "evt_FridayDinner2611000",
      "registrantDid": "did:key:z6MkGraceHopperPairwise",
      "displayName": "Grace",
      "status": "withdrawn",
      "guests": 1,
      "answers": {
        "dietary": "No shellfish"
      },
      "roster": "visible",
      "eventSequence": 1,
      "createdAt": "2026-11-27T12:00:00Z",
      "updatedAt": "2026-12-01T08:00:00Z"
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

In: an event identifier. Out: the registration as withdrawn and a summary of the event.

### Correlation

The invitee addresses the host by the persona identifier the invitation named, and signs as the identifier the host invited — normally one the invitee holds with the host alone. The host shows that identifier to no one but its principal, and other registrants see a display name the invitee chose, or none (VTI-PEV-024), so registering links the invitee to nobody else. A party not invited learns nothing, not even that the event exists (VTI-PEV-020).

### Retention

Changes the registration's status, kept with the event.

### Consent/purpose

A gathering the principal hosts for people they know. The VTA invites only contacts the principal names (VTI-PEV-010), never acts on its own account (VTI-PEV-013, VTI-PEV-042), and discloses the guest list only as the roster setting fixed at publication allows (VTI-PEV-041). **An event confers nothing**: neither an invitation nor a registration is read in any authorization decision of the VTA's.
