---
slug: vta/services/disable
version: "1.1"
title: VTA Services — Disable
summary: An operator stops advertising a transport; for DIDComm this begins a drain rather than cutting delivery.
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
  requirement: REQUIRED
  rationale: Withdraws a route others depend on; attributable by the same argument as enable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Disabling a service stops it. Replayed after an operator restored it, it stops it again, an availability effect with no cause visible anywhere in the payload.
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
immediately, but keeps accepting delivery until `drainEndsAt`, so messages
already in flight toward it are not stranded. The result carries `drainEndsAt`
and `drainingMediator` when that happens.

## A drain is not a finished disable

While `drainEndsAt` is in the future the old mediator is still live. A consumer
that reports "disabled" at that moment is wrong in the direction that loses
messages — and the operator's next action, often decommissioning the mediator,
is exactly the thing that must wait.

Its absence means no drain was scheduled. It does **not** mean a drain completed
instantly.

## The caller proposes the window; the agent decides it

`drainTtlSeconds` is a request, not an instruction. `0` asks for immediate
teardown, and the agent enforces a floor when the request arrived **through the
mediator being torn down** — cutting it mid-request would discard the reply to
the very task asking for it. So a caller cannot rely on `0` being obeyed: read
`drainEndsAt` in the result to learn what the agent actually did.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Changes from 1.0

* Request `drainTtlSecs` → **`drainTtlSeconds`**. Same type and bounds: an integer number of seconds, minimum 0.
* Response `result.drainUntil` → **`drainEndsAt`** (`ServiceMutationResult`). Same type: an RFC 3339 `date-time`, present only when a drain was scheduled.

The registry's naming-consistency pass gives one concept one name and spells every duration's unit out: `vta/_shared/0.1/service` called the instant a drain ends both `drainUntil` and `drainsUntil`, and `…Secs` is not a spelling the registry uses. The shared `$ref`s move to `vta/_shared/0.8/service`, which carries the renamed members; nothing else in the payload changes.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `1.0` remains published.

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
  "type": "https://trusttasks.org/spec/vta/services/disable/1.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": {
    "service": "didcomm",
    "drainTtlSeconds": 43200
  }
}
```

## Response

```json
{
  "id": "00000005-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/disable/1.1#response",
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
      "drainEndsAt": "2026-08-19T21:10:01Z",
      "drainingMediator": "did:web:old-mediator.example"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries the transport to stop advertising and, for `didcomm`, the drain window. The response carries the log entry that made the change and, when a drain was scheduled, the draining mediator and `drainEndsAt`. Everything here ends up in, or describes, the agent's public DID document.

### Correlation

The advertised surface is public by construction: anyone may fetch the agent's DID document. The change introduces no identifier that was not about to be published in it.

### Retention

The signed log entry is the durable record, kept for as long as the DID's log is; the spec imposes no further retention rule on the exchange.

### Consent/purpose

The operator changes how its own agent is reached. No other party's data is involved, and the spec imposes no further rule.

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
