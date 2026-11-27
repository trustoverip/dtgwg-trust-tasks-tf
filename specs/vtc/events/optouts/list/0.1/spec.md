---
slug: vtc/events/optouts/list
version: '0.1'
title: VTC Events — Opt-outs — List
summary: 'A member lists their own opt-outs of invitations.'
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
  rationale: 'A member''s opt-outs are their own; an unattributable caller gets nothing.'
sideEffects:
  level: none
  rationale: 'Reads the caller''s opt-outs; persists nothing.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'The caller''s own opt-outs.'
retention:
  class: transient
  rationale: 'A read.'
errorCodes: []
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/optouts/set
---

## Abstract

The **VTC Events — Opt-outs — List** Trust Task answers the caller's own opt-outs (VTI-EVT-110), newest first. Nobody else's are ever answered: an opt-out is the member's choice about what they hear, and no host or administrator lists them.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer the caller's own opt-outs, newest first, and no one else's.
2. Answer an empty list, not an error, to a caller with none, and while events are turned off.

## Authorization

**Standing in the community.** The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), and the community reads their standing from its own records at execution time. A member's act (VTI-EVT-095): a member portal holding a key the member authorized for `vtc/events` **MAY** sign it.

## Definitions

- `EventOptOut` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### List my opt-outs

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000571",
  "type": "https://trusttasks.org/spec/vtc/events/optouts/list/0.1#request",
  "issuer": "did:example:ada",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-20T10:00:00Z",
  "payload": {},
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/optouts/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### List my opt-outs — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000572",
  "type": "https://trusttasks.org/spec/vtc/events/optouts/list/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:ada",
  "issuedAt": "2026-11-20T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000571",
  "payload": {
    "items": [
      {
        "scope": "calendar",
        "calendarId": "cal_ElectronicsGroup00",
        "createdAt": "2026-11-20T10:00:01Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The caller's opt-outs.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID. 

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

A member checking what they have stopped hearing about.
