---
slug: vta/memory/put
version: "0.1"
title: VTA Memory — Put
summary: An agent stores a key/value memory item in a VTA context, so it persists across sessions while staying scoped to that context.
status: draft
targetFrameworkVersion: "0.1"
category: ai-agents
keywords:
  - memory
  - agent
  - context
  - key-value
  - cross-session
  - recall
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
  rationale: A memory write mutates durable per-context state the VTA audits and may replay; transport-independent integrity is required.
sideEffects:
  level: mutating
  rationale: "Stores a key/value memory item in a context."
subjectPath: /contextId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/memory/put:context_forbidden
    meaning: The caller is not permitted to write memory in the named context.
    retryable: false
  - code: vta/memory/put:value_too_large
    meaning: The value exceeds the VTA's per-item limit.
    retryable: false
related:
  - vta/memory/list
  - vta/memory/delete
---

## Abstract

The **VTA Memory — Put** Trust Task stores a free-form **key/value memory item**
in a VTA **context**, so an AI agent can *remember across sessions* while its
memory stays **scoped to one context**. It is a generic per-context key/value
store distinct from the password vault (`vault/*`, secret-bearing, structured)
and the credential store (`vault/credentials/*`): memory is opaque agent text —
facts, summaries, preferences — keyed by an agent-chosen `key`.

Memory is **context-isolated**: an item written to context A is invisible to an
agent scoped to context B. The VTA enforces this with the same context ACL that
gates keys and contexts — a caller may only `put` in a context it has access to.
Re-putting an existing `key` replaces the value (upsert).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the agent) **MUST**:

1. Emit a *Trust Task document* of type `https://trusttasks.org/spec/vta/memory/put/0.1` with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`, `payload.key`, and `payload.value`.
3. Include a `proof` per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate + verify the `proof`.
2. Refuse with `vta/memory/put:context_forbidden` when the caller lacks access to `payload.contextId` (the same context ACL that gates keys).
3. Upsert the item under `(contextId, key)` and return the `#response`.

## Definitions

* **Agent.** The party storing the memory; identified by `issuer`.
* **Context.** The VTA context the memory is scoped to; the isolation boundary.
* **Key / value.** An agent-chosen identifier and its opaque text payload.

## Request

```json
{
  "id": "5b1d2c8a-0e44-4f21-9c10-3a7e2b6d4f90",
  "type": "https://trusttasks.org/spec/vta/memory/put/0.1",
  "issuer": "did:key:z6MkFinanceAgent",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-24T12:00:00Z",
  "payload": {
    "contextId": "finance",
    "key": "invoice-contact",
    "value": "billing@acme.example — prefers PDF"
  }
}
```

## Response

```json
{
  "id": "a2c9...",
  "type": "https://trusttasks.org/spec/vta/memory/put/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkFinanceAgent",
  "issuedAt": "2026-06-24T12:00:01Z",
  "threadId": "5b1d2c8a-0e44-4f21-9c10-3a7e2b6d4f90",
  "payload": { "key": "invoice-contact" }
}
```
