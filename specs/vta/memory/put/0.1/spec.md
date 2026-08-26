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
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A memory write mutates durable per-context state the VTA audits and may replay; transport-independent integrity is required.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A put overwrites what the agent remembers, and the agent acts on it. A stale copy therefore restores a superseded belief rather than merely an old value.
sideEffects:
  level: mutating
  rationale: "Stores a key/value memory item in a context."
subjectPath: /contextId
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: "`value` is unstructured agent-authored text about whatever the agent was asked to remember — this specification's own example stores a working email address and a correspondence preference — and `key` is an agent-chosen label that is itself descriptive. Nothing is disclosed back to the caller, but the VTA reads and stores both in the clear."
retention:
  class: durable
  rationale: Surviving the session that wrote it is the entire purpose of the task; an item persists until an agent overwrites it with the same `key` or removes it with `vta/memory/delete`. A VTA that expired items on session close would not implement this specification.
errorCodes:
  - code: vta/memory/put:contextForbidden
    meaning: The caller is not permitted to write memory in the named context.
    retryable: false
  - code: vta/memory/put:valueTooLarge
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

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the agent) **MUST**:

1. Emit a *Trust Task document* of type `https://trusttasks.org/spec/vta/memory/put/0.1` with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId`, `payload.key`, and `payload.value`.
3. Include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate + verify the `proof`.
2. Refuse with `vta/memory/put:contextForbidden` when the caller lacks access to `payload.contextId` (the same context ACL that gates keys).
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

## Security & Privacy

### Data carried

`value` is the sensitive member, and it is unstructured: the schema calls it
"opaque memory text" and constrains nothing, so whatever the agent decided to
remember about a person lands there verbatim. The example above stores a working
email address and a correspondence preference, which is the ordinary case rather
than a worst case. `key` is content too — `invoice-contact` says something about
the subject even to a party that cannot read the value, and a key naming a person,
a condition, or a dispute says a great deal. `contextId` is a scoping label rather
than a secret, but it names which part of a principal's life the item belongs to.
Nothing here is sealed: unlike [`vault/upsert`](../../../../vault/upsert/0.3/spec.md),
this task carries no HPKE envelope, and the VTA necessarily reads `value` in
cleartext in order to store it.

All three members are REQUIRED, so minimisation is not a matter of trimming the
document — it is a matter of what an agent chooses to write. A producer **SHOULD**
store the fact it needs to recall rather than the conversation it learned the fact
from, and **MUST NOT** use `value` as an overflow store for credential material
that belongs behind a sealed vault write. `ext` is agent-authored and unvalidated
by this specification; a producer **MUST NOT** place there anything it would not
put in `value`.

### Correlation

Everything under one `contextId` is joinable by construction, and that is the
point of the store. The isolation boundary is the **context**, never the item: an
agent that may read one memory in `finance` may read them all, and the accumulated
set is a far sharper profile of a principal than any single `put`. Across contexts
nothing joins automatically, but keys collide by convention — `user-timezone`,
`invoice-contact` — so a VTA operator holding several contexts for one principal
can align them on `key` alone, without ever reading a `value`.

The `issuer` DID is deliberately stable: cross-session recall is only possible if
the same agent is recognised on its next connection, so a per-session identifier
would defeat the task outright. That stability is scoped to the agent's
relationship with its own VTA and nothing in this task asks a third party to
recognise the agent, which is why the Agent party declares
`identifierScope: pairwise`.

### Retention

Durable, and by design. An item outlives the session that wrote it, the process
that ran the agent, and typically the reason it was stored; it ends only when the
same `key` is overwritten or [`vta/memory/delete`](../../delete/0.1/spec.md)
removes it. There is no expiry member in this payload and no default lifetime in
this specification, which means a VTA accumulates memory indefinitely unless its
operator imposes a policy of its own. Implementers **SHOULD** expose that policy
to the principal, because the absence of a TTL here is a silence this task cannot
fill: a memory store whose oldest entries are the ones nobody remembers writing is
the predictable failure mode.

### Consent/purpose

The purpose is recall inside one context: an agent stores what it needs to do the
work that context exists for. The context ACL is the record of the basis on which
the write happens — a caller writes only where it already holds access, and
`vta/memory/put:contextForbidden` is the refusal when it does not. That basis does
not travel with the item: material an agent was given for one context's work is
not thereby available for another, and a VTA **MUST NOT** surface an item written
in one context to an agent scoped to a different one. Whether a human is asked
before an agent commits something to long-term memory is a consumer policy
question and this specification takes no position on it.
