---
slug: vtc/events/hosts/remove
version: '0.1'
title: VTC Events — Hosts — Remove
summary: A host of an event, or a holder of authority over its calendar, ends a member's hosting of that event — withdraws the vtc.events.manage grant qualified to it. A host may step down; a published event is never left with no host.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- host
- appointment
parties:
- role: event host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Takes authority over an event away from a member. Must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed removal would remove a host appointed again since.
sideEffects:
  level: mutating
  rationale: Withdraws a resource grant from the host's access-control entry. Recoverable with vtc/events/hosts/appoint.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Names the host by entry and returns the event's hosts by entry, never by DID.
retention:
  class: durable
  rationale: The withdrawal is recorded in the access-control history and the audit.
errorCodes:
- code: vtc/events/hosts/remove:notHost
  meaning: The entry holds no grant qualified to this event. Authority over the whole calendar is not removed here.
  retryable: false
- code: vtc/events/hosts/remove:lastHost
  meaning: The entry is the published event's only host; removing it would leave the event with nobody to run it. Appoint another host first, or cancel the event.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/hosts/appoint
- vtc/events/hosts/list
- vtc/events/cancel
---

## Abstract

The **VTC Events — Hosts — Remove** Trust Task ends a member's hosting of one event: it withdraws the access-control resource grant `vtc.events.manage` qualified `event:<eventId>` that [`vtc/events/hosts/appoint`](../../appoint/0.1/spec.md) wrote (VTI-EVT-070). A host may remove another host, or step down themselves. A **published** event is never left with no host (VTI-EVT-072): its last host is refused until another is appointed, or the event is cancelled.

Authority over the event's whole calendar is not a host's grant and is not removed here; it is the community's to change in its access-control list.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 7.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller neither hosts nor may see, and `permissionDenied` to a caller who may see the event but does not cover `vtc.events.manage` for it. Answer `notFound` while the community has events turned off (VTI-EVT-007).
2. Refuse an entry that holds no grant qualified to the event with `notHost`.
3. Refuse with `lastHost` the removal of the only remaining host of a published event, counting hosts as entries holding a grant qualified to the event (VTI-EVT-072). A draft or cancelled event may be left with no host; authority over its calendar still runs it.
4. Withdraw the grant from the host's access-control entry and answer the event's hosts after the removal.
5. Leave the removed host's own registration for the event, if any, as it stands: hosting and registering are separate.
6. Audit the removal by reference — the event, the entry and the remover (VTI-EVT-091).

## Authorization

**Authority: `vtc.events.manage` covering the event**, as for [`vtc/events/hosts/appoint`](../../appoint/0.1/spec.md): a host's grant qualified to the event, or the authority over the event's calendar (VTI-EVT-013), read from the signer's access-control entry at execution time. Any host may remove any other host of the same event, the creator included: the hosts are peers. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A removal is the host's own act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-EVT-095).

## Definitions

- `EventId` — see [`event.schema.json`](../../../_shared/0.1/event.schema.json).
- `EntryId` — see [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Step down as a co-host

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000481",
  "type": "https://trusttasks.org/spec/vtc/events/hosts/remove/0.1#request",
  "issuer": "did:example:tom",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-26T09:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "entryId": "dir_TomAshworth000000",
    "reason": "Away that week."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:tom#key-1",
    "created": "2026-11-26T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/hosts/remove/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Step down as a co-host — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000482",
  "type": "https://trusttasks.org/spec/vtc/events/hosts/remove/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:tom",
  "issuedAt": "2026-11-26T09:00:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000481",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "hosts": ["dir_PriyaNatarajan000"]
  }
}
```

### Refused: the last host

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000484",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-26T09:10:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000483",
  "payload": {
    "code": "vtc/events/hosts/remove:lastHost",
    "message": "Intro to Soldering is published and you are its only host. Appoint another host first, or cancel it.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

An event, an entry and a reason; the event's hosts by entry.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Hosts are named by their community-scoped `entryId`, never by DID.

### Retention

The withdrawal is recorded in the access-control history and the audit.

### Consent/purpose

Ending a delegation of an event's running. A removed host stops reading the roster and the answers at once, since every gate reads the grant at execution time.
