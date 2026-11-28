---
slug: vtc/events/polls/answer
version: '0.1'
title: VTC Events — Polls — Answer
summary: 'A member of a poll''s audience answers it: yes, if need be or no for each candidate time, replacing their earlier answers.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
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
  rationale: 'What the caller may do or see depends on who asks; the request must be attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would repeat a change.'
sideEffects:
  level: mutating
  rationale: 'Stores the member''s answers, replacing earlier ones.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'Answers per candidate time; the caller''s answers.'
retention:
  class: durable
  rationale: 'Answers are kept until the community''s retention for closed polls removes them.'
errorCodes:
- code: vtc/events/polls/answer:pollClosed
  meaning: 'The poll takes no more answers.'
  retryable: false
- code: vtc/events/polls/answer:slotUnknown
  meaning: 'An answer names a time the poll does not hold.'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"slotId": {"type": "string", "maxLength": 128}}}
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/polls/show
---

## Abstract

The **VTC Events — Polls — Answer** Trust Task answers a poll: yes, if need be or no for each candidate time. A later answer replaces an earlier one.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for a poll that does not exist and one the caller may not see — a poll is seen by its creator, the hosts of its calendar, and the members its audience admits (VTI-EVT-270). A member who may see the poll but whom its audience does not admit is refused with `permissionDenied`.
2. Refuse a closed poll, or one past `closesAt`, with `pollClosed`, and an answer for a time the poll does not hold with `slotUnknown`.
3. Replace the caller's earlier answers with these.

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**A member the poll's audience admits** (VTI-EVT-270). A member's act: a portal holding a key the member authorized for `vtc/events` **MAY** sign it (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/polls/answer/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "pollId": "pol_AGM2026TimesPoll01",
    "answers": {
      "sat-5": "yes",
      "sun-6": "ifNeedBe"
    }
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/polls/answer/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/polls/answer/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "pollId": "pol_AGM2026TimesPoll01",
    "myAnswers": {
      "sat-5": "yes",
      "sun-6": "ifNeedBe"
    }
  }
}
```

## Security & Privacy

### Data carried

A poll and the caller's answers.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable** until the community's retention for closed polls removes them.

### Consent/purpose

Saying which times suit.
