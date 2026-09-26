---
slug: vta/services/report
version: "0.1"
title: VTA Services — Report
summary: An operator asks an agent which mediators its inbound traffic arrived through, and through which mediator each sender last reached it — to see who is still on the old route after a migration.
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - services
  - mediator
  - migration
  - telemetry
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Operator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The response names every DID that reached the agent in the window and when, which is a record of other parties' contact with it. Only the agent's operator may read that, and the recipient decides who the operator is from the request's signer; an unsigned request leaves it nothing to decide on. The response is RECOMMENDED rather than REQUIRED because the operator is asking its own agent over a channel it already authenticated, and acts on the answer only to decide whether a drain can end.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: >-
    A read with no effect. A replayed request returns the report for its own window to the party entitled to it.
sideEffects:
  level: none
  rationale: "Reads the agent's inbound telemetry."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Sender DIDs, mediator DIDs and timestamps of contact. Nothing about what was said.
retention:
  class: transient
  rationale: The operator consults the report to decide whether a mediator drain can end; the agent's telemetry, not this exchange, is the record.
errorCodes:
  - code: vta/services/report:invalidWindow
    meaning: "`since` is after `until`."
    retryable: false
related:
  - vta/services/list
  - vta/services/update
  - vta/services/drain/list
---

## Abstract

When an agent moves to a new DIDComm mediator, its DID document changes at once, but its correspondents do not all notice at once: a sender that cached the old document keeps delivering through the old mediator, which the agent keeps listening on for a drain period. Before the operator ends that period, it needs to know whether anyone is still using the old route.

**VTA Services — Report** answers that from the agent's inbound telemetry: for a time window, how many messages arrived through each mediator, and for each sender the mediator it most recently arrived through.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **operation of the agent as a whole**: the same standing that may change which transports it advertises ([`vta/services/update`](../../update/1.0/)). A recipient **MUST** refuse a producer without it, including one that administers only some of the agent's contexts — the report spans every sender, whatever context it dealt with.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), the proof establishes who asked; entitlement is the recipient's own decision.

## Request

The request payload is the top-level schema of [`payload.schema.json`](payload.schema.json). Both bounds are optional RFC 3339 timestamps; an absent `until` means now, and an absent `since` means as far back as the agent's telemetry goes.

```json
{
  "id": "urn:uuid:3e4f5a6b-7c8d-4e9f-8a0b-1c2d3e4f5a6b",
  "type": "https://trusttasks.org/spec/vta/services/report/0.1",
  "issuer": "did:key:z6MkOperatorExampleKey",
  "recipient": "did:webvh:QmExampleScid:vta.example.com",
  "issuedAt": "2026-09-26T12:00:00Z",
  "payload": {
    "since": "2026-09-19T00:00:00Z"
  }
}
```

A recipient **MUST** refuse a window whose `since` is after its `until` with `vta/services/report:invalidWindow`.

## Response

```json
{
  "id": "urn:uuid:8f9a0b1c-2d3e-4f4a-8b5c-6d7e8f9a0b1c",
  "threadId": "urn:uuid:3e4f5a6b-7c8d-4e9f-8a0b-1c2d3e4f5a6b",
  "type": "https://trusttasks.org/spec/vta/services/report/0.1#response",
  "issuer": "did:webvh:QmExampleScid:vta.example.com",
  "issuedAt": "2026-09-26T12:00:01Z",
  "payload": {
    "since": "2026-09-19T00:00:00Z",
    "until": "2026-09-26T12:00:01Z",
    "mediators": [
      {
        "mediatorDid": "did:web:old-mediator.example.com",
        "inboundCount": 12,
        "firstSeen": "2026-09-19T08:12:00Z",
        "lastSeen": "2026-09-24T17:40:00Z"
      },
      {
        "mediatorDid": "did:web:new-mediator.example.com",
        "inboundCount": 431,
        "firstSeen": "2026-09-20T09:00:00Z",
        "lastSeen": "2026-09-26T11:59:30Z"
      }
    ],
    "senders": [
      {
        "senderDid": "did:key:z6MkLaggingSenderKey",
        "lastSeenMediator": "did:web:old-mediator.example.com",
        "lastSeenAt": "2026-09-24T17:40:00Z"
      }
    ]
  }
}
```

- `until` is the upper bound the recipient applied, so an open-ended request learns what "now" was.
- A sender whose `lastSeenMediator` is the old mediator is one the drain is still serving.

## Security & Privacy

### Data carried

The request carries a time window. The response carries mediator DIDs, sender DIDs, message counts and the times of first and last contact — never content.

### Correlation

The response is a contact log: which parties reached the agent, through which route, and when. That is correlation data about third parties, which is why only the agent's operator may read it and why the request must be signed. The report introduces no identifier those parties did not already present to the agent.

### Retention

The exchange is consulted and discarded. The agent's inbound telemetry is the record, kept under the agent's own retention policy; this task neither extends nor copies it.

### Consent/purpose

The purpose is operational: deciding when a mediator migration's drain can end without stranding a correspondent. A recipient **SHOULD NOT** retain the telemetry this report reads for longer than that purpose needs.
