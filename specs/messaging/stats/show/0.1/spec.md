---
slug: messaging/stats/show
version: "0.1"
title: "Messaging — Show Mediator Statistics"
summary: "An administrator reads a mediator's operational telemetry — version, uptime, connections, lifetime traffic counters, forwarding and circuit-breaker state, and the aggregate of its latest queue survey."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request is gated on administrative standing, and a mediator that authorizes it from a transport session alone
    cannot show afterwards who read its telemetry. A proof binds the read to the administrator's key independent of the
    transport. The response is RECOMMENDED so an operator can retain a reading as evidence of the mediator's state at
    a point in time.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The mediator enforces a freshness window on every administrative request; a request that cannot be placed in time
    cannot be checked against it, and a captured request could otherwise be replayed indefinitely to poll telemetry.
sideEffects:
  level: none
  rationale: >-
    A read of counters and state the mediator already maintains; it persists nothing.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns operational telemetry only — software version, uptime, connection counts, lifetime traffic totals,
    forwarding and circuit-breaker state, and queue-survey aggregates. It carries no per-account data, no message
    content, and no configuration secret.
retention:
  class: transient
  rationale: >-
    A reading is superseded by the next one; a client derives rates from successive readings and has no reason to
    keep any single one.
errorCodes: []
related:
  - messaging/queue/list
  - messaging/queue/status
  - config/show
---

## Abstract

The **Messaging — Show Mediator Statistics** Trust Task returns a mediator's operational telemetry in one read: its software version and uptime, live and maximum websocket connections, lifetime traffic counters, the state of its forwarding queue and store circuit breaker, and — when one has completed — the aggregate of its most recent queue survey. It is the dashboard read an operator console polls, and replaces bespoke HTTP status endpoints so the same read works over any Trust Task binding.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/stats/show/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`.
2. Send an empty payload (or one carrying only `ext`).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window.
2. Respond with `permissionDenied` unless the requester's account type is `admin`, `rootAdmin`, or `mediator`.
3. Return `totals` as monotonic lifetime counters — never reset by the read, never windowed — so a client can derive rates from successive readings.
4. Omit `queues` when no queue survey has completed, rather than returning zeros that read as "every queue is empty".
5. **NOT** include any database locator, credential, key, or other configuration secret in the response. Configuration is read with [`config/show`](../../../../config/show/0.1/spec.md), which applies its own redaction.

## Authorization

The entitlement is **administrative standing at the mediator**: the requester's account holds the `admin`, `rootAdmin`, or `mediator` role (Consumer rule 2). A `standard` account has no standing to read mediator-wide telemetry, even though every counter is an aggregate, because connection counts and traffic totals are an operational fingerprint of the deployment.

The `proof` establishes *who* asked and that the request is unaltered; it is not itself the authorization ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The mediator resolves the verified issuer to its account and decides from the role it finds there.

## Definitions

- `version` — the mediator software's version string.
- `startedAt`, `uptimeSeconds` — when the current process started, and seconds since.
- `connections.websocketActive` / `websocketMax` — live websocket connections, and the configured ceiling (absent when unlimited).
- `totals` — lifetime counters: messages and bytes received, sent, and deleted; websocket opens and closes; sessions created and authenticated; invitations created and claimed.
- `forwarding.queueLength` / `queueLimit` — messages waiting to be relayed to other mediators, and the ceiling. `forwarding.circuitBreaker` is `closed` (normal), `open` (shedding store operations), or `halfOpen` (probing recovery).
- `queues` — the aggregate of the mediator's latest survey of account queues: when it ran (`surveyedAt`), how many accounts it covered (`accounts`), whether it stopped early (`truncated`), and summed [`QueueDepth`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/QueueDepth) views of every `receive` and `send` queue, whose `saturation` is the maximum over any single queue rather than a ratio of sums.

## Request

The administrator sends an empty payload to the mediator; see the top-level schema in [`payload.schema.json`](payload.schema.json).

### Reading the dashboard

```json
{
  "id": "urn:uuid:3b1f7c2e-6a4d-4b8e-9f10-2c5d7e8a9b01",
  "type": "https://trusttasks.org/spec/messaging/stats/show/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T10:00:00Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-21T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4xQ..."
  }
}
```

## Response

The mediator responds with a document of type `https://trusttasks.org/spec/messaging/stats/show/0.1#response`, whose payload validates against the `$anchor: "response"` sub-schema. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not a `#response` document.

### A mediator with a completed survey

```json
{
  "id": "urn:uuid:3b1f7c2e-6a4d-4b8e-9f10-2c5d7e8a9b02",
  "type": "https://trusttasks.org/spec/messaging/stats/show/0.1#response",
  "threadId": "urn:uuid:3b1f7c2e-6a4d-4b8e-9f10-2c5d7e8a9b01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T10:00:00Z",
  "payload": {
    "version": "0.21.0",
    "startedAt": "2026-09-19T08:12:40Z",
    "uptimeSeconds": 179240,
    "connections": { "websocketActive": 412, "websocketMax": 10000 },
    "totals": {
      "receivedCount": 1843201,
      "receivedBytes": 2210937411,
      "sentCount": 1839977,
      "sentBytes": 2205114093,
      "deletedCount": 1838410,
      "deletedBytes": 2203001877,
      "websocketOpened": 22817,
      "websocketClosed": 22405,
      "sessionsCreated": 30112,
      "sessionsAuthenticated": 29870,
      "invitationsCreated": 214,
      "invitationsClaimed": 198
    },
    "forwarding": { "queueLength": 3, "queueLimit": 50000, "circuitBreaker": "closed" },
    "queues": {
      "surveyedAt": "2026-09-21T09:59:12Z",
      "accounts": 1288,
      "truncated": false,
      "receive": { "count": 1904, "bytes": 3912455, "saturation": 0.41 },
      "send": { "count": 2311, "bytes": 4402918, "saturation": 0.93 }
    }
  }
}
```

A `send` saturation of `0.93` says at least one account is close to its send-queue limit; [`messaging/queue/list`](../../../queue/list/0.1/spec.md) sorted by `saturation` names it.

## Security & Privacy

### Data carried

The request carries nothing beyond the framework envelope. The response carries aggregate telemetry: counts, byte totals, timestamps, the software version, and breaker state. It names no account and carries no message content. The software version is the most sensitive member — it tells a reader which advisories may apply — which is one reason the task is restricted to administrators. A mediator **MUST NOT** place connection strings, credentials, hostnames of backing stores, or any other configuration value in the response or in `ext`.

### Correlation

The response is not about any subject, so there is nothing to join across documents beyond the mediator itself. Successive readings reveal the mediator's traffic volume over time, which is the purpose of the task; that is visible only to an administrator.

### Retention

Transient. Each reading supersedes the last. A console keeps a short rolling window to draw rates and discards it; a mediator need not retain the request beyond its replay window.

### Consent/purpose

The data is collected to operate the mediator: capacity planning, incident diagnosis, and alerting. It is not collected for, and should not be reused as, a record of any particular account's activity — this task has none to give.
