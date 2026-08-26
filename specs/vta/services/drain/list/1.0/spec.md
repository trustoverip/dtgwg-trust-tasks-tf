---
slug: vta/services/drain/list
version: "1.0"
title: VTA Services Drain — List
summary: An operator sees which DIDComm mediators are draining and until when.
status: draft
targetFrameworkVersion: "0.2"
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
errorCodes: []
related:
  - vta/services/disable
  - vta/services/drain/cancel
---

## Abstract

When a DIDComm mediator is disabled or replaced it enters a **drain**: no longer
advertised, still accepting delivery, until a deadline. **VTA Services Drain —
List** reports which mediators are in that state and when each one ends.

This is the call an operator needs before decommissioning mediator
infrastructure. An empty array means nothing is draining — the ordinary state,
not an error.

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
  "id": "00000007-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/services/drain/list/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": {}
}
```

## Response

```json
{
  "id": "00000007-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/drain/list/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "00000007-0000-4000-8000-000000000001",
  "payload": {
    "entries": [
      {
        "mediatorDid": "did:web:old-mediator.example",
        "endpoint": "https://old-mediator.example/didcomm",
        "drainsUntil": "2026-08-19T21:10:01Z"
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

