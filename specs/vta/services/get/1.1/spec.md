---
slug: vta/services/get
version: "1.1"
title: VTA Services — Get
summary: An operator reads the current state of one transport.
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - services
  - transport
  - did-document
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
  requirement: OPTIONAL
  rationale: A read with no durable state change.
sideEffects:
  level: none
  rationale: "Read only."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/services/get:notFound
    meaning: The agent has never held this transport. Distinct from holding it disabled, which returns a state with enabled:false.
    retryable: false
related:
  - vta/services/list
  - vta/services/update
---

## Abstract

**VTA Services — Get** returns one transport's state: whether it is advertised,
its kind-specific settings, and — for a draining DIDComm mediator — when the
drain ends.

Use it over `list` when you need to distinguish *"this transport is not
configured"* from *"I cannot see it"*: `get` answers `notFound` for a kind the
agent has never held, where `list` would simply omit it and leave the caller
guessing.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Changes from 1.0

* Response `state.drainsUntil` → **`drainEndsAt`** (`ServiceState`). Same type: an RFC 3339 `date-time`, present only while a `didcomm` mediator is draining.

The registry's naming-consistency pass gives one concept one name: `vta/_shared/0.1/service` called the instant a drain ends both `drainUntil` and `drainsUntil`. The shared `$ref`s move to `vta/_shared/0.8/service`, which carries the renamed members; nothing else in the payload changes.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `1.0` remains published.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** answer only for transports the caller is authorized to see, and **MUST NOT** distinguish 'not configured' from 'not permitted' by latency or message.

## Authorization

Authority is **super-admin**. The advertised half of this answer is public in the
agent's DID document, but the unadvertised half — a transport configured and
disabled — is not, and it discloses operational intent.

A `proof`, where present, establishes that the producer authored the request, not
that it may read.

## Request

```json
{
  "id": "00000002-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/services/get/1.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": {
    "service": "didcomm"
  }
}
```

## Response

```json
{
  "id": "00000002-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/get/1.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "00000002-0000-4000-8000-000000000001",
  "payload": {
    "state": {
      "kind": "didcomm",
      "enabled": true,
      "mediatorDid": "did:web:mediator.example"
    }
  }
}
```

## Security & Privacy

### Data carried

The request names one transport kind. The response carries that transport's state: whether it is advertised, its endpoint URL or mediator DID, and — for a draining DIDComm mediator — `drainEndsAt`.

### Correlation

The advertised half of the answer is public by construction: anyone may fetch the agent's DID document. The unadvertised half — a transport configured and disabled — is not, and discloses operational intent; the spec confines it to super-admins (see Authorization) and imposes no further rule.

### Retention

The task is read only and writes nothing; the spec imposes no retention rule on the exchange.

### Consent/purpose

The operator reads how its own agent is reached. No other party's data is involved, and the spec imposes no further rule.

### Threats

The advertised surface is public by construction — it lives in the agent's DID
document, which anyone may fetch. So the disclosure risk here is not the
*content* of a service entry but the *ability to change it*: an attacker who can
enable a transport can point the agent's traffic at infrastructure they control,
and every client that resolves the DID afterwards will believe it.

That is why every mutation in this family is super-admin only, and why each one
writes a signed log entry rather than flipping a runtime flag. The log is what
makes a change attributable after the fact.

Two failure modes are worth stating plainly:

- **`serverless: true` means nobody else can see the change yet.** The entry is
  written locally but not published; a consumer that reports success without
  surfacing this tells the operator a change is live when no verifier can
  observe it.
- **A drain is not a completed disable.** While `drainEndsAt` is in the future the
  old mediator is still accepting delivery. Reporting "disabled" at that point is
  wrong in the direction that loses messages.
