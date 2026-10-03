---
slug: vtc/admin/actions/acknowledge
version: "0.2"
title: "VTC Admin Actions — Acknowledge"
summary: "An administrator of a Verifiable Trust Community acknowledges an operation that took effect outside the approval path — an operator's offline or break-glass write — so that every administrator demonstrably saw it."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords: [vtc, admin, actions, acknowledge, break-glass, offline, audit]
parties:
  - role: Administrator (acknowledger)
    requirement: REQUIRED
    member: issuer
  - role: Community maintainer (VTC)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    An acknowledgement is the community's evidence that a named administrator saw an operation nobody approved. Its
    value is entirely in its attribution: one recorded from a session rather than a signature could be produced by
    whoever holds that session — including the operator whose write is being acknowledged — and would make the record
    say that the other administrators were told when they were not.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The acknowledgement records that an administrator had seen the operation by a point in time. A document that cannot
    be placed in time cannot say when, and gives the consumer no window over which to absorb a duplicate (SPEC §7.2
    item 11).
sideEffects:
  level: mutating
  rationale: >-
    Records the caller's acknowledgement on the action and, when it is the last one expected, closes the action as
    `completed`. It neither reverses nor ratifies the operation acknowledged, which has already taken effect.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns the action with the acknowledgement recorded — the same administrative metadata `show` discloses: the
    operation, who performed it, and which administrators have acknowledged it so far.
retention:
  class: durable
  rationale: >-
    A break-glass or offline write bypasses the approval path by design; the signed acknowledgements are what keep it
    accountable afterwards, by showing that every administrator was told and when. They are kept with the action's
    audit record.
errorCodes:
  - code: vtc/admin/actions/acknowledge:notFound
    meaning: No action with this `actionId` exists that the caller may see. Answered identically for both cases.
    retryable: false
  - code: vtc/admin/actions/acknowledge:notAcknowledgeable
    meaning: >-
      The action cannot be acknowledged by this caller — it is not of category `acknowledge`, it is no longer `open`, or
      the caller is not among the administrators expected to acknowledge it (for example, its requester).
    retryable: false
  - code: vtc/admin/actions/acknowledge:alreadyAcknowledged
    meaning: >-
      The caller has already acknowledged this action. The earlier acknowledgement stands; the state the caller wanted is
      already true.
    retryable: false
related:
  - vtc/admin/actions/list
  - vtc/admin/actions/show
  - vtc/admin/actions/cancel
  - vtc/operator/offline-write
---

## Abstract

Some administrative operations take effect without other administrators' approval — an operator's offline write against the community's store while the service is stopped, or a break-glass recovery. A Verifiable Trust Community records each such operation as an action of category `acknowledge` and asks every remaining administrator to acknowledge it. **VTC Admin Actions — Acknowledge** is that acknowledgement. There is nothing to approve or decline: the operation has happened. What the acknowledgements establish is that nobody was left unaware of it. The record is the shared [`Action`](../../_shared/0.2/action.schema.json); the model is described in [`vtc/admin/actions/list/0.2`](../../list/0.2/spec.md).

### Naming an offline write

An action's `typeUri` names the operation it concerns. An operator's offline write — `vtc admin emergency-bootstrap`, or one of the other commands run against the community's store on its host — is not a Trust Task: nothing was sent, so there is no request type to name. Such an action names the record type [`vtc/operator/offline-write/0.1`](../../../../operator/offline-write/0.1/spec.md) instead, and its `payload` is that record — which command ran, the DIDs it changed, on which host, and when — so the summary fields locate their values in it like any other payload and `payloadDigest` covers it. An operation that *was* a Trust Task (a break-glass write the community executed itself) keeps naming its own type.

### Changes from 0.1

This version differs from [`vtc/admin/actions/acknowledge/0.1`](../../acknowledge/0.1/spec.md) only in the record it carries: the shared `Action` moved to [`_shared/0.2`](../../_shared/0.2/action.schema.json), which adds the `coolingOff` category (refused here as `notAcknowledgeable`: nobody acknowledges a cooling-off action), the `landsAt` and `cancellableBy` members, the `landedAfterCoolingOff` closed reason, the `subject` caller role, and the offline-write record type above. [`vtc/admin/actions/list/0.2`](../../list/0.2/spec.md) describes the changes in full. The request payload and the error codes are unchanged.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Take the caller from the verified `proof`. An action the caller may not see is `notFound`, answered exactly as an unknown `actionId`.
2. Refuse with `notAcknowledgeable` an action whose `category` is not `acknowledge`, one that is not `open`, or a caller who is not among the administrators expected to acknowledge it. The expected set is the administrators who held an administrative role when the operation was recorded, less its requester and less anyone who has since lost every administrative role.
3. Record at most one acknowledgement per administrator; a second is `alreadyAcknowledged`, and leaves the first unchanged.
4. Record the acknowledgement in the action's `approvals` with the proven signer as `subject`, audit it, and — when no expected administrator remains — close the action with `status: completed`, `closedReason: acknowledged` and `closedAt`.
5. **MUST NOT** treat an acknowledgement as approval of the operation, nor as authority for any further operation.

A producer that receives `alreadyAcknowledged` **MAY** treat it as success.

## Authorization

The authority is **being one of the administrators the community expects to acknowledge this operation** — an administrative role at the community, held when the operation was recorded and still held now, by someone other than the operation's requester. The community reads both from its own records at execution time. Verifying the `proof` establishes who is acknowledging ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the entitlement is membership of the expected set. An acknowledgement confers nothing: it changes what the community can show about who knew, never what anyone may do.

## Definitions

- **`actionId`** (REQUIRED) — the open `acknowledge` action.
- **`action`** (response) — the action with the caller's acknowledgement recorded.

## Request

Bob acknowledges an emergency bootstrap an operator ran on the community's host while the service was stopped. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Acknowledging an emergency bootstrap

```json
{
  "id": "urn:uuid:2f7a9c3e-5b1d-4e68-a0f4-6c2e8b1d3a57",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/acknowledge/0.2",
  "issuer": "did:web:bob.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T11:05:00Z",
  "threadId": "urn:uuid:2f7a9c3e-5b1d-4e68-a0f4-6c2e8b1d3a57",
  "payload": {
    "actionId": "act_Qw3nB8vK1sYd6Pm2"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T11:05:00Z",
    "verificationMethod": "did:web:bob.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3t5T7vzcSkiuDycV34FJnAcWVJhZ8fwiL4E97Aapox1nBP78Qu4uoQNwBgua8Rbiru8GkeYstFzmA7aRdaoLqQMx"
  }
}
```

## Response

The community answers with the action, per the `$anchor: "response"` sub-schema. Failures use `trust-task-error`.

### Recorded; one administrator still to acknowledge

```json
{
  "id": "urn:uuid:7c4e1a9b-3d6f-4b28-8e5a-0f1c7d3b9e64",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/acknowledge/0.2#response",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-02T11:05:01Z",
  "threadId": "urn:uuid:2f7a9c3e-5b1d-4e68-a0f4-6c2e8b1d3a57",
  "payload": {
    "action": {
      "actionId": "act_Qw3nB8vK1sYd6Pm2",
      "category": "acknowledge",
      "kind": "operator.offlineWrite",
      "typeUri": "https://trusttasks.org/spec/vtc/operator/offline-write/0.1",
      "requester": "did:web:ops.community.example",
      "status": "open",
      "createdAt": "2026-10-02T06:12:44Z",
      "approvals": [
        { "subject": "did:web:dana.example", "at": "2026-10-02T09:01:30Z" },
        { "subject": "did:web:bob.example", "at": "2026-10-02T11:05:01Z" }
      ],
      "approversRemaining": 1,
      "summary": {
        "title": "Operator changed access control offline",
        "effect": "This change was written directly to the community's store on its host while the service was stopped. It is already in effect.",
        "fields": {
          "command": { "pointer": "/command", "format": "text", "value": "emergencyBootstrap" },
          "affected": { "pointer": "/dids/0", "format": "did", "value": "did:web:erin.example" },
          "host": { "pointer": "/host", "format": "text", "value": "vtc-01.community.example" },
          "at": { "pointer": "/at", "format": "datetime", "value": "2026-10-02T06:12:44Z" }
        },
        "templateDigest": "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX"
      },
      "payload": {
        "command": "emergencyBootstrap",
        "dids": ["did:web:erin.example"],
        "host": "vtc-01.community.example",
        "at": "2026-10-02T06:12:44Z"
      },
      "payloadDigest": "zQmbay7SyLLJyRieYo81yjde3YuF19D9h9kCNGwGBDxE5z1",
      "callerRole": "acknowledger"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T11:05:01Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z4W4W9KN3Gh2k6T4Jo31XZdFPRLyxg9nLYvNQtLS9cvCB2T9hqQT9m1upFcFXDJxFRoJeweN2Hu4gjfAd57ZDqLe6"
  }
}
```

### Acknowledging twice

```json
{
  "id": "urn:uuid:0b8d2f6a-9e4c-4a17-b3d5-6e1f8c2a4d90",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-02T11:05:01Z",
  "threadId": "urn:uuid:2f7a9c3e-5b1d-4e68-a0f4-6c2e8b1d3a57",
  "payload": {
    "code": "vtc/admin/actions/acknowledge:alreadyAcknowledged",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries one identifier. The response carries the action: the operation that was performed outside the approval path, including its payload — which names whoever it concerns — the operator who performed it, and every administrator who has acknowledged it so far. Nothing in the request is free text, deliberately: an acknowledgement says "I saw this", and a note would invite it to be read as something more.

### Correlation

The acknowledgement joins a named administrator to a named operation by signature and `actionId`, which is its whole purpose. The `approvals` list shows each administrator who acknowledged, and when, to every other administrator who can see the action; that visibility is how an administrator who was not told can notice.

### Retention

Acknowledgements are kept with the action's audit record for as long as the community retains that record. The operation they concern bypassed approval; deleting the acknowledgements would remove the only evidence that the other administrators were made aware of it.

### Consent/purpose

The data serves one purpose: to make an operation nobody approved visible to, and verifiably seen by, every administrator. An acknowledgement **MUST NOT** be repurposed as approval, as consent to the operation, or as consent to anything that follows from it. This specification describes the acknowledgement only; whether an operation is recorded for acknowledgement is the community's policy.
