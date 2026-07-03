---
slug: messaging/ping
version: "0.1"
title: Messaging — Ping
summary: A liveness and capability check against a messaging endpoint — a mediator or an agent; the requester asks "are you there?" and the endpoint answers with its server time, health status, and the transport protocols it supports.
status: draft
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - agent
  - ping
  - health
  - liveness
  - capability
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Requester
    requirement: REQUIRED
    member: issuer
  - role: Responder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: A ping is a transient liveness probe, not an evidentiary record. Over a transport that authenticates the parties end-to-end (a TSP message, a DIDComm authcrypt envelope, mutually-authenticated TLS) the in-band proof adds nothing and MAY be omitted; a proof SHOULD be included only where the requester needs a transport-independent, retainable attestation of the responder's response.
related:
  - did-management/server/health
  - device/heartbeat
---

## Abstract

The **Messaging — Ping** Trust Task is the liveness and capability probe of the `messaging/*` family. A *requester* — a client, a peer mediator, or an operator tool — sends a ping to a *responder* (a messaging endpoint: a mediator, or an agent such as a Verifiable Trust Agent) and receives a response carrying the responder's current server time, a coarse health status, and the set of transport protocols the responder currently serves (for example `didcomm`, `tsp`). It lets a requester decide, at runtime, both **whether** an endpoint is reachable and **which transport** to use with it, without parsing the endpoint's DID document.

The task is **side-effect-free**: a ping changes no responder state and is safe to issue repeatedly. It is the transport-agnostic analogue of a protocol-specific health check, so the same task answers identically whether it arrives over the [TSP binding](../../../../bindings/tsp/0.1/spec.md), the [DIDComm binding](../../../../bindings/didcomm/0.1/spec.md), or the [HTTPS binding](../../../../bindings/https/0.1/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the requester) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/ping/0.1`, with itself as `issuer` and the responder as `recipient`.
2. Populate `payload` per `payload.schema.json`. The request payload **MAY** be empty; a requester that wants to correlate the response to a specific probe **MAY** include an opaque `nonce`.

A conforming **consumer** (the responder) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Respond with a `#response`-variant document whose `payload` carries `serverTime` and `status`, echoing the request `nonce` when one was supplied.
3. Report `status: "ok"` only when it is able to accept and process messages; report `status: "degraded"` when it is reachable but operating with reduced function (for example a backend in a circuit-broken state); and decline with the framework's `unavailable` error ([SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)) when it cannot serve at all.
4. Populate `protocols` with the transport protocols it is currently prepared to accept, so the requester can select a transport.

A ping **MUST NOT** require the requester to hold any capability beyond reachability; a responder **MAY**, however, answer an unauthenticated ping with a reduced response (status and time only, omitting `protocols`) where exposing its capability surface to anonymous callers is undesirable.

## Definitions

* **Requester.** The party probing the responder; identified by `issuer`.
* **Responder.** The messaging endpoint being probed — a mediator, or an agent (such as a Verifiable Trust Agent); identified by `recipient`.
* **Protocol token.** A short lowerCamelCase identifier for a transport the responder serves — `didcomm`, `tsp`. The set is open; consumers ignore tokens they do not recognize.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/ping/0.1` with a payload that validates against the top-level schema in `payload.schema.json`. The `recipient` is the responder being probed — here a mediator, but an agent's DID is equally valid.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/ping/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T09:31:00Z",
  "payload": {
    "nonce": "f3a1c9"
  }
}
```

`proof` is omitted here because the example assumes a transport that conveys producer identity end-to-end (per [SPEC.md §4.7.1](../../../../SPEC.md#471-when-to-include-a-proof)).

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/ping/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/ping/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-06-22T09:31:00Z",
  "payload": {
    "serverTime": "2026-06-22T09:31:00Z",
    "status": "ok",
    "protocols": ["didcomm", "tsp"],
    "nonce": "f3a1c9"
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)); a responder that cannot serve responds with `unavailable` (retryable), not a `#response` variant.

## Security & Privacy

A ping is side-effect-free and carries no personal data. The principal privacy consideration is **capability disclosure**: the `protocols` list reveals which transports an endpoint serves. A responder that treats its transport surface as sensitive **MAY** withhold `protocols` from unauthenticated requesters (see [Conformance](#conformance)).

`serverTime` lets a requester estimate clock skew against the responder but is not a trusted time source; a requester **MUST NOT** treat it as authoritative for any security decision. Where a requester needs a retainable, transport-independent attestation that a responder was live at a given time, it **SHOULD** require an in-band `proof` per the spec's `RECOMMENDED` proof policy.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member)) is available on both the request and response payloads under the usual namespacing and ignore-unknown rules.
