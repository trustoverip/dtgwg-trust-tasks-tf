---
slug: vta/services/disable
version: "1.0"
title: VTA Services — Disable
summary: An operator stops advertising a transport; for DIDComm this begins a drain rather than cutting delivery.
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
  requirement: REQUIRED
  rationale: Withdraws a route others depend on; attributable by the same argument as enable.
sideEffects:
  level: mutating
  rationale: "Removes the transport from the agent's DID document and republishes the signed log."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/services/disable:notFound
    meaning: No such transport is configured on this agent.
    retryable: false
  - code: vta/services/disable:notAuthorized
    meaning: The caller is not a super-admin.
    retryable: false
related:
  - vta/services/enable
  - vta/services/drain/list
  - vta/services/drain/cancel
---

## Abstract

**VTA Services — Disable** stops advertising a transport and republishes the
log.

For DIDComm it does not cut delivery. The mediator stops being advertised
immediately, but keeps accepting delivery until `drainUntil`, so messages
already in flight toward it are not stranded. The result carries `drainUntil`
and `drainingMediator` when that happens.

## A drain is not a finished disable

While `drainUntil` is in the future the old mediator is still live. A consumer
that reports "disabled" at that moment is wrong in the direction that loses
messages — and the operator's next action, often decommissioning the mediator,
is exactly the thing that must wait.

Its absence means no drain was scheduled. It does **not** mean a drain completed
instantly.

## The caller proposes the window; the agent decides it

`drainTtlSecs` is a request, not an instruction. `0` asks for immediate
teardown, and the agent enforces a floor when the request arrived **through the
mediator being torn down** — cutting it mid-request would discard the reply to
the very task asking for it. So a caller cannot rely on `0` being obeyed: read
`drainUntil` in the result to learn what the agent actually did.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** reject a payload whose `config` members do not belong to the named `service`, rather than ignoring the surplus member. It **MUST** write a did:webvh log entry for every accepted change, and **MUST** report `serverless: true` when that entry was not published.

## Authorization

Authority is **super-admin**. Every task in this family edits — or reads — what
the agent tells the world about reaching it, and the ACL has no finer capability
for a subset of transports.

A `proof`, where present, establishes that the producer authored the request. It
is not the authorization; the caller's super-admin role is.

## Request

```json
{
  "id": "00000005-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/services/disable/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": {
    "service": "didcomm",
    "drainTtlSecs": 43200
  }
}
```

## Response

```json
{
  "id": "00000005-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/disable/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "00000005-0000-4000-8000-000000000001",
  "payload": {
    "result": {
      "logEntryVersionId": "4-zQmLogEntry",
      "effectiveAt": "2026-08-19T09:10:01Z",
      "vtaDid": "did:webvh:QmAgent:vta.example",
      "serverless": false,
      "drainUntil": "2026-08-19T21:10:01Z",
      "drainingMediator": "did:web:old-mediator.example"
    }
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

