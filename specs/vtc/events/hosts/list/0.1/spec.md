---
slug: vtc/events/hosts/list
version: '0.1'
title: VTC Events — Hosts — List
summary: 'List an event''s hosts by entry — whom to ask about it — answered to every member who may see the event.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- host
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Whether the event is answered at all depends on who asks; an unattributable caller gets nothing.
sideEffects:
  level: none
  rationale: Reads the access-control grants qualified to the event; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses the event's hosts by community-scoped entry identifier, never by DID.
retention:
  class: transient
  rationale: A read.
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/hosts/appoint
- vtc/events/hosts/remove
- vtc/events/show
---

## Abstract

The **VTC Events — Hosts — List** Trust Task reads who hosts an event: the entries holding `vtc.events.manage` qualified to it (VTI-EVT-070), the member who created it first while they still host it. It is whom a member asks about an event — a question, an access need — and is answered to everyone who may see the event, as [`vtc/events/show`](../../../show/0.4/spec.md)'s `hosts` is. It is its own read because who hosts an event is read from the access-control list, never from the event's record.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 7.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Answer `notFound` alike for an event that does not exist and one the caller may not see — a draft to anyone but its hosts, or an event whose audience does not admit the caller (VTI-EVT-020, VTI-EVT-023) — and while the community has events turned off (VTI-EVT-007).
3. Answer the entries holding a `vtc.events.manage` grant qualified `event:<eventId>`, read from the access-control list at execution time, the event's creator first while they hold one, then the rest in order of appointment. **MUST NOT** list holders of authority over the whole calendar, which is the community's administration rather than this event's hosting.

## Authorization

**Seeing the event.** Any active member the event's audience admits, and its hosts, may read who hosts it (VTI-EVT-020); no capability is involved. The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

Reading the hosts is an act of the member (VTI-EVT-095): a member portal that holds a key the member authorized to sign for them for `vtc/events` (`auth/oob` `signingScope`) **MAY** sign it.

## Definitions

- `EventId` — see [`event.schema.json`](../../../_shared/0.1/event.schema.json).
- `EntryId` — see [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Who runs it?

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000491",
  "type": "https://trusttasks.org/spec/vtc/events/hosts/list/0.1#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T18:50:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-20T18:50:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/hosts/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Who runs it? — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000492",
  "type": "https://trusttasks.org/spec/vtc/events/hosts/list/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T18:50:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000491",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "hosts": ["dir_PriyaNatarajan000", "dir_TomAshworth000000"]
  }
}
```

## Security & Privacy

### Data carried

An event identifier in; its hosts by entry out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Hosts are named by their community-scoped `entryId`, never by DID; what a reader may see *of* a host is the directory's projection ([`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md)), not this task's.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Knowing whom to ask about an event. Hosting is public to the event's audience: a member deciding whether to come, or needing an accommodation, is entitled to know who runs it. **An event confers nothing** (VTI-EVT-002).
