---
slug: vta/memory/delete
version: "0.1"
title: VTA Memory — Delete
summary: An agent forgets a memory item from a VTA context by key.
status: draft
targetFrameworkVersion: "0.1"
category: ai-agents
keywords:
  - memory
  - agent
  - context
  - forget
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
  requirement: REQUIRED
  rationale: A delete mutates durable per-context state the VTA audits; transport-independent integrity is required.
errorCodes:
  - code: vta/memory/delete:context_forbidden
    meaning: The caller is not permitted to delete memory in the named context.
    retryable: false
  - code: vta/memory/delete:not_found
    meaning: No item with that key exists in the context.
    retryable: false
related:
  - vta/memory/put
  - vta/memory/list
---

## Abstract

The **VTA Memory — Delete** Trust Task removes a key/value memory item from a VTA
**context** — the agent *forgets* it. Context-isolated and ACL-gated like
[`vta/memory/put`](../../put/0.1/spec.md): a caller may only delete in a context
it can access, and an unknown key yields `vta/memory/delete:not_found`.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply. The VTA **MUST** refuse with `context_forbidden` when the caller lacks access to `payload.contextId`, with `not_found` when the key is absent, and otherwise remove the item.

## Request

```json
{
  "id": "1f8a...",
  "type": "https://trusttasks.org/spec/vta/memory/delete/0.1",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-24T12:10:00Z",
  "payload": { "contextId": "finance", "key": "invoice-contact" }
}
```

## Response

```json
{
  "id": "3b6d...",
  "type": "https://trusttasks.org/spec/vta/memory/delete/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkFinanceAgent",
  "issuedAt": "2026-06-24T12:10:01Z",
  "threadId": "1f8a...",
  "payload": { "key": "invoice-contact" }
}
```
