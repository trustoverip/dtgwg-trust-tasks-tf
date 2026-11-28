---
slug: vtc/events/meeting/actions/update
version: '0.1'
title: VTC Events — Meeting — Actions — Update
summary: 'Change an action item: its assignee may set its status; the chair, the secretary or a host may change anything.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: assignee, chair, secretary or host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'What the caller may do or see depends on who asks; the request must be attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would repeat a change.'
sideEffects:
  level: mutating
  rationale: 'Changes the action item.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'Changed members of an action item; the item.'
retention:
  class: durable
  rationale: 'As the action item.'
errorCodes:
- code: vtc/events/meeting/actions/update:actionNotFound
  meaning: 'No action item the caller may see has that identifier.'
  retryable: false
- code: vtc/events/meeting/actions/update:assigneeNotEligible
  meaning: 'A new assignee is not an active member of the community.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/meeting/actions/add
- vtc/events/meeting/actions/list
---

## Abstract

The **VTC Events — Meeting — Actions — Update** Trust Task changes an action item (VTI-EVT-266). Its assignee marks it done or dropped; the meeting's chair, secretary and hosts may change its title, assignee and due date too.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `actionNotFound` alike for an action item that does not exist and one the caller may not see.
2. Let the assignee change only `status`; refuse their changing anything else with `permissionDenied`. Let the meeting's chair, secretary and hosts change any member (VTI-EVT-266).
3. Refuse a new assignee who is not an active member with `assigneeNotEligible`.
4. Audit the change by reference (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**The assignee for `status`; the meeting's chair, secretary or a host for anything** (VTI-EVT-266). The assignee's status change is their own act: a portal holding a key the member authorized for `vtc/events` **MAY** sign it (VTI-EVT-095); any other change never from a portal-held key. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The assignee, chair, secretary or host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/actions/update/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "actionId": "act_ShedQuotes0000001",
    "status": "done"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:mary#key-1",
    "created": "2026-12-05T10:40:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/meeting/actions/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/actions/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "action": {
      "actionId": "act_ShedQuotes0000001",
      "eventId": "evt_AGM2026HarbourRowing",
      "title": "Get three quotes for the boat shed",
      "assignee": "dir_KwameOsei000000000",
      "dueAt": "2027-01-15T17:00:00Z",
      "status": "done",
      "createdAt": "2026-12-05T12:30:00Z",
      "updatedAt": "2027-01-10T09:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

An action item's changes; the item.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

As the action item.

### Consent/purpose

Keeping an action item current.
