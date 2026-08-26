---
slug: vta/services/enable
version: "1.0"
title: VTA Services — Enable
summary: An operator advertises a transport, writing it into the agent's signed DID document.
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
  rationale: A change to what the world believes about how to reach this agent. The log entry it writes is signed, and the request that caused it should be attributable too.
sideEffects:
  level: mutating
  rationale: "Edits the agent's DID document and republishes its signed log; other parties resolve the result."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/services/enable:conflict
    meaning: The transport is already advertised. Use update to change its settings.
    retryable: false
  - code: vta/services/enable:validationFailed
    meaning: The config is not valid for the named service — wrong member, or a URL that is not https:// with no fragment and no userinfo.
    retryable: false
  - code: vta/services/enable:notAuthorized
    meaning: The caller is not a super-admin.
    retryable: false
related:
  - vta/services/update
  - vta/services/disable
  - vta/services/rollback
---

## Abstract

**VTA Services — Enable** adds a transport to the agent's DID document and
republishes the signed log.

`service` selects the transport and `config` carries its settings. The two must
agree: a `rest` request carrying `mediatorDid` is **malformed**, not a request
with a harmlessly ignored field. Treating a mismatch as ignorable is how an
operator ends up believing they configured something they did not.

## Enable is not idempotent, on purpose

Enabling a transport that is already enabled is a **conflict**, not a silent
success. The operation that changes settings on a live transport is `update`,
and keeping them separate means a typo cannot quietly re-point a working
transport while the operator believes they were setting up a new one.

## `force` skips the proof that it works

For `didcomm`, the handshake is what establishes that the mediator is reachable
and willing to route. `force` skips it — DID resolution still runs, but nothing
else. That is occasionally necessary and always a risk: the agent will advertise
a mediator it has not confirmed can deliver, and the failure surfaces later, to
whoever tries to send a message.

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
  "id": "00000003-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/services/enable/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": {
    "service": "didcomm",
    "config": {
      "mediatorDid": "did:web:mediator.example",
      "force": false
    }
  }
}
```

## Response

```json
{
  "id": "00000003-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/enable/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "00000003-0000-4000-8000-000000000001",
  "payload": {
    "result": {
      "logEntryVersionId": "4-zQmLogEntry",
      "effectiveAt": "2026-08-19T09:10:01Z",
      "vtaDid": "did:webvh:QmAgent:vta.example",
      "serverless": false
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

