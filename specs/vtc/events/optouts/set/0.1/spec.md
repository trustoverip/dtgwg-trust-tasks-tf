---
slug: vtc/events/optouts/set
version: '0.1'
title: VTC Events — Opt-outs — Set
summary: 'A member opts out of invitations to one event or from one calendar, or withdraws such an opt-out. An opt-out removes them from no audience and stops no host''s invitation by name.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- invitation
- opt-out
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
  rationale: 'An opt-out changes what the community sends the member; it must be the member''s own.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed opt-out would undo the member''s later change.'
sideEffects:
  level: mutating
  rationale: 'Records or removes one opt-out. Recoverable by setting it again.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'An event or calendar, and whether the member opts out.'
retention:
  class: durable
  rationale: 'An opt-out stands until withdrawn.'
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/optouts/list
- vtc/events/invitations/send
---

## Abstract

The **VTC Events — Opt-outs — Set** Trust Task lets a member stop invitations their audience would bring them (VTI-EVT-110): to one event, or from one calendar. The audience rule never overrides an opt-out. It does not remove the member from any audience — they can still find and register for the event — and does not stop a host's invitation by name, which the member may decline.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event or calendar that does not exist and one the caller may not see (VTI-EVT-023).
2. With `optedOut` true, record the opt-out (once; a repeat changes nothing); with false, remove it.
3. From then on, invite the caller under VTI-EVT-101 to no event the opt-out covers.
4. Never remove the caller from an audience, and never stop a host's invitation by name (VTI-EVT-110).
5. Answer whether the caller is now opted out and, if so, the opt-out.

## Authorization

**Standing in the community.** The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and the community reads their standing from its own records at execution time. A member's act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `EventOptOut` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventOptOutScope` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `EventId` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).
- `CalendarId` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Stop invitations from a calendar

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000561",
  "type": "https://trusttasks.org/spec/vtc/events/optouts/set/0.1#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {
    "scope": "calendar",
    "calendarId": "cal_ElectronicsGroup00",
    "optedOut": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:ada#key-1",
    "created": "2026-11-20T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/optouts/set/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Stop invitations from a calendar — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000562",
  "type": "https://trusttasks.org/spec/vtc/events/optouts/set/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000561",
  "payload": {
    "optedOut": true,
    "optOut": {
      "scope": "calendar",
      "calendarId": "cal_ElectronicsGroup00",
      "createdAt": "2026-11-20T10:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

A scope, an event or calendar, a yes or no.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

**Durable** until withdrawn; it goes when the member leaves the community.

### Consent/purpose

Hearing less. The member, not the rule, decides whether they hear about every event a group runs.
