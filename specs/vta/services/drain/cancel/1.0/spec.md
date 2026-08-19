---
slug: vta/services/drain/cancel
version: "1.0"
title: VTA Services Drain — Cancel
summary: An operator ends a drain early, accepting the loss of anything still in flight.
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
  rationale: Discards in-flight messages; the decision should be attributable to whoever made it.
sideEffects:
  level: destructive
  rationale: "Drops a mediator that was still accepting delivery."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/services/drain/cancel:notFound
    meaning: That mediator is not draining.
    retryable: false
  - code: vta/services/drain/cancel:conflict
    meaning: That mediator is the currently advertised one; cancelling its drain would strand every route the agent depends on.
    retryable: false
  - code: vta/services/drain/cancel:notAuthorized
    meaning: The caller is not a super-admin.
    retryable: false
related:
  - vta/services/drain/list
  - vta/services/disable
---

## Abstract

**VTA Services Drain — Cancel** ends a drain immediately, dropping the mediator
now rather than at `drainsUntil`.

**Messages still in flight through it are lost.** That is not a side effect to
be mitigated — it is the whole reason the drain window existed, and cancelling
is a deliberate decision to give it up, usually because the mediator is already
gone and the window is only delaying cleanup.

## Two refusals

A mediator that is not draining cannot be cancelled — there is nothing to
shorten. Neither can the **currently advertised** mediator: cancelling a drain on
the active route would strand every path the agent depends on, which is a
mistake the agent should not let an operator make by hand.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

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
  "id": "00000008-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/services/drain/cancel/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T09:10:00Z",
  "payload": {
    "mediatorDid": "did:web:old-mediator.example"
  }
}
```

## Response

```json
{
  "id": "00000008-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/services/drain/cancel/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T09:10:01Z",
  "threadId": "00000008-0000-4000-8000-000000000001",
  "payload": {
    "mediatorDid": "did:web:old-mediator.example"
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

