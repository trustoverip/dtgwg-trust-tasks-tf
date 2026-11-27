---
slug: vtc/events/hosts/appoint
version: '0.1'
title: VTC Events — Hosts — Appoint
summary: A host of an event, or a holder of authority over its calendar, appoints a community member a co-host of that event — an access-control resource grant of vtc.events.manage qualified to the event.
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
  rationale: Gives a member authority over an event — its settings, its roster and its registrants' answers. Must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed appointment would restore a host since removed.
sideEffects:
  level: mutating
  rationale: Writes a resource grant onto the appointee's access-control entry. Recoverable with vtc/events/hosts/remove.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Names the appointee by entry and returns the event's hosts by entry, never by DID.
retention:
  class: durable
  rationale: The grant is the appointee's authority over the event for as long as it stands.
errorCodes:
- code: vtc/events/hosts/appoint:notEligible
  meaning: The entry is not an active member of the community.
  retryable: false
- code: vtc/events/hosts/appoint:alreadyHost
  meaning: The member already hosts the event — by a grant qualified to it, or by holding authority over its calendar.
  retryable: false
- code: vtc/events/hosts/appoint:eventCancelled
  meaning: The event is cancelled; it takes no new hosts. Nothing changed.
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/hosts/remove
- vtc/events/hosts/list
- vtc/groups/managers/add
---

## Abstract

The **VTC Events — Hosts — Appoint** Trust Task appoints a community member a co-host of one event (VTI-EVT-070). A host changes the event's settings, publishes and cancels it, reads its whole roster with its registrants' answers, decides pending registrations, removes registrations, and appoints and removes other hosts (VTI-EVT-073). The member who created the event is its first host.

The appointment is not a field of the event. It is an **access-control resource grant** on the appointee's entry — `vtc.events.manage` qualified `event:<eventId>` — written through the community's appointment mechanism, as a group's managers and a board's facilitators are ([`vtc/groups/managers/add`](../../../../groups/managers/add/0.1/spec.md)), so the community's access-control list stays the one place its authority is recorded. A member with no administrative role may hold it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 7.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller neither hosts nor may see, and `permissionDenied` to a caller who may see the event but does not cover `vtc.events.manage` for it (VTI-EVT-071). Answer `notFound` while the community has events turned off (VTI-EVT-007).
2. Refuse a cancelled event with `eventCancelled`, an entry that is not an active member with `notEligible` (VTI-EVT-071), and one that already hosts the event — by a grant qualified to it, or by authority over its calendar (VTI-EVT-013) — with `alreadyHost`.
3. Write the appointment as a resource grant on the appointee's access-control entry: capability `vtc.events.manage`, qualifier `event:<eventId>`, delegated by the signer, beside the entry's administrative role and never inside its ceiling (VTI-EVT-070).
4. Bound the grant by the signer's own holding: the signer covers `vtc.events.manage` for this event at the moment of writing, and a grant the signer does not cover is never written.
5. Treat the grant as any other delegated grant when its granter departs or no longer covers it: it is reviewed on the community's terms for a departed granter's grants, never left standing silently (VTI-EVT-074).
6. Withdraw the grant when the event is deleted, and when the appointee leaves the community.
7. Answer the event's hosts after the appointment: entries holding a grant qualified to the event, its creator first.
8. Audit the appointment by reference — the event, the appointee's entry and the granter (VTI-EVT-091) — and **MUST NOT** treat the event's record as the source of who hosts it.

A host's authority is over the event alone, and confers nothing else: hosting an event is never read by any other authorization decision (VTI-EVT-002).

## Authorization

**Authority: `vtc.events.manage` covering the event** — a host's grant qualified `event:<eventId>`, or the authority over the event's calendar: unqualified for the community's calendar, qualified `group:<groupId>` or `vtc.groups.manage` for that group for a group's calendar (VTI-EVT-013) — read from the signer's access-control entry at execution time. A host may appoint a co-host of the event they host; authority over the calendar appoints for every event in it. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

An appointment is the host's own act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-EVT-095).

## Definitions

- `EventId` — see [`event.schema.json`](../../../_shared/0.1/event.schema.json).
- `EntryId` — see [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Appoint a co-host

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000471",
  "type": "https://trusttasks.org/spec/vtc/events/hosts/appoint/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-18T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "entryId": "dir_TomAshworth000000",
    "reason": "Tom runs the soldering stations."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-18T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/hosts/appoint/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Appoint a co-host — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000472",
  "type": "https://trusttasks.org/spec/vtc/events/hosts/appoint/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-18T10:00:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000471",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "hosts": ["dir_PriyaNatarajan000", "dir_TomAshworth000000"]
  }
}
```

## Security & Privacy

### Data carried

An event, an entry and a reason; the event's hosts by entry.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Hosts are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable** for the grant, until it is removed, its event is deleted, or its holder leaves the community.

### Consent/purpose

Sharing the running of an event. A host reads registrants' answers, which are about members as people; the grant is authority at the community, so it lives in the access-control list and is reviewed with every other delegated grant.
