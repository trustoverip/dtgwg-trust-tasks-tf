---
slug: vta/memory/list
version: "0.1"
title: VTA Memory — List
summary: An agent recalls all memory items stored in a VTA context — cross-session recall, scoped to that context.
status: draft
targetFrameworkVersion: "0.1"
category: ai-agents
keywords:
  - memory
  - agent
  - context
  - recall
  - cross-session
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Agent
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: A read returns no durable state change; the transport's authenticated sender is sufficient. A proof MAY be included where the response is retained for audit.
sideEffects:
  level: none
  rationale: "Read-only recall of a context's memory items."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/memory/list:context_forbidden
    meaning: The caller is not permitted to read memory in the named context.
    retryable: false
related:
  - vta/memory/put
  - vta/memory/delete
---

## Abstract

The **VTA Memory — List** Trust Task returns every key/value memory item stored
in a VTA **context** ([`vta/memory/put`](../../put/0.1/spec.md)) — the agent's
**cross-session recall**, scoped to one context. Memory is context-isolated: an
agent scoped to context A lists only A's items; the VTA's context ACL (the same
one that gates keys) refuses a context the caller can't access.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** refuse with `vta/memory/list:context_forbidden` when the caller lacks access to `payload.contextId`, and otherwise return all items under that context.

## Request

```json
{
  "id": "7c2e...",
  "type": "https://trusttasks.org/spec/vta/memory/list/0.1",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-24T12:05:00Z",
  "payload": { "contextId": "finance" }
}
```

## Response

```json
{
  "id": "9d4f...",
  "type": "https://trusttasks.org/spec/vta/memory/list/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkFinanceAgent",
  "issuedAt": "2026-06-24T12:05:01Z",
  "threadId": "7c2e...",
  "payload": {
    "items": [
      { "key": "invoice-contact", "value": "billing@acme.example — prefers PDF" }
    ]
  }
}
```
