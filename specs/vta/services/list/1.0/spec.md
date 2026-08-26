---
slug: vta/services/list
version: "1.0"
title: VTA Services — List
summary: An operator enumerates the transports an agent advertises in its DID document.
status: draft
targetFrameworkVersion: "0.5"
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
  rationale: A read with no durable state change. A proof MAY be included where the response is retained for audit.
sideEffects:
  level: none
  rationale: "Enumeration only."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vta/services/get
  - vta/services/enable
  - vta/services/disable
---

## Abstract

An agent reaches the world through *transports* — DIDComm mediation, a REST
endpoint, TSP mediation, a WebAuthn origin — and it advertises them in the
`service` array of its own did:webvh document. **VTA Services — List** returns
what it currently advertises, plus what it knows about and does not.

That second part is the reason this task exists rather than telling callers to
read the DID document. The document says what is advertised; it cannot say that
a transport was configured and then disabled, and an operator deciding whether
to `enable` or `update` needs exactly that distinction.

## The distinction that matters

A kind **absent** from `services` has never been configured. A kind **present
with `enabled: false`** is known and deliberately not advertised. Those are
different states with different next steps — `enable` for the first, and for the
second, `enable` again but with the knowledge that previous settings exist and a
`rollback` may be the better move.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

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
  "id": "00000001-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/services/list/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": {}
}
```

## Response

```json
{
  "id": "00000001-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/list/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "00000001-0000-4000-8000-000000000001",
  "payload": {
    "services": [
      {
        "kind": "didcomm",
        "enabled": true,
        "mediatorDid": "did:web:mediator.example"
      },
      {
        "kind": "rest",
        "enabled": true,
        "url": "https://vta.example/api"
      },
      {
        "kind": "tsp",
        "enabled": false
      }
    ]
  }
}
```

## Security & Privacy

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
- **A drain is not a completed disable.** While `drainUntil` is in the future the
  old mediator is still accepting delivery. Reporting "disabled" at that point is
  wrong in the direction that loses messages.

