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
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: >-
    The request changes no durable state, so the transport's authenticated sender
    is sufficient for it. The response is not equivalent: it hands the caller the
    entire contents of a context's memory, and the SPEC §7.3 item 8 floor for a
    `discloses: secret` response applies — what was released has to stay
    attributable to the VTA that released it, on any transport, including one that
    authenticates only hop by hop.
sideEffects:
  level: none
  rationale: "Read-only recall of a context's memory items."
subjectPath: /contextId
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response returns every `items[].value` in the context — free-text agent
    memory written by the producer, which routinely holds counterparty contact
    details, notes about people, and whatever else the agent chose to remember.
    That is the stored content itself rather than descriptive data about it, and
    the caller keeps it, so the bulk read is a secret disclosure and not an
    enumeration. Inbound, the request carries only `contextId` — a scoping label
    the VTA already holds — so nothing personal travels toward the recipient;
    the asymmetry between the two directions is the whole shape of this task.
retention:
  class: exchange
  rationale: >-
    The VTA remains the record of the memory; this response is a working copy
    handed to an agent for the session that asked for it, and an agent that keeps
    it past that session has created a second copy no `vta/memory/delete` can
    reach. Read-only at the maintainer — the request itself persists nothing.
errorCodes:
  - code: vta/memory/list:contextForbidden
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

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** refuse with `vta/memory/list:contextForbidden` when the caller lacks access to `payload.contextId`, and otherwise return all items under that context.

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

## Security & Privacy

### Data carried

The two directions are lopsided. The request is one member, `contextId`, naming
a scope the VTA already knows about. The response is the context's entire memory:
`items` is described in the schema as "every memory item in the context", each
entry carrying both `key` and `value` in cleartext.

There is no paging member, no filter, no key prefix, and — the omission worth
naming — no projection. Compare
[`vta/app-state/list/1.0`](../../../app-state/list/1.0/spec.md), which carries
`includeValues` defaulting to false precisely so a caller can ask *what is
stored* without also receiving *what it says*. This task has no such member, so
there is no conforming way to enumerate keys without disclosing every value
alongside them. An agent that only needs to know whether it has seen a
counterparty before must nonetheless receive that counterparty's notes, and
everyone else's. A VTA **MAY** bound `items` by its own per-context limits, but
this specification gives it no member to express that it did, so a caller cannot
distinguish a complete answer from a truncated one.

Minimisation therefore cannot happen in this document; it happens at
[`vta/memory/put`](../../put/0.1/spec.md), in what the agent chose to write down
in the first place, and in how narrowly contexts are cut.

### Correlation

A single call yields the whole profile, and the set is worth considerably more
than its members. Watching every individual `put` over a month reveals the same
facts one at a time and in the order they were learned; one `list` returns them
assembled, deduplicated by `key`, and ready to read as a description of a
principal. Whoever holds the response holds the correlation — no further joining
is required of them.

Nothing joins *across* contexts through this task: the response is scoped to the
`contextId` the caller named and the VTA's context ACL refuses anything else. The
join that remains available to the VTA operator is the one described in
[`vta/memory/put`](../../put/0.1/spec.md) — keys collide by convention across
contexts — and `list` makes exercising it cheap, since the operator now has every
key in one place.

The `issuer` DID is stable across sessions for the same reason recall itself is:
the VTA must recognise the returning agent to know which context it may read.
That recognition is needed only by the VTA, not by any third party, so the Agent
declares `identifierScope: pairwise`.

### Retention

The exchange, not beyond it. The durable copy lives at the VTA; what this task
hands back is a working copy for the session that asked. That distinction is the
one a consumer is most likely to erode, and it matters because
[`vta/memory/delete`](../../delete/0.1/spec.md) reaches only the VTA's copy. Every
`list` response an agent writes to a transcript, a log line, a vector index, or a
model's context window is a fork of the store that forgetting cannot follow —
the principal deletes an item, the VTA honours it, and the copy handed out last
Tuesday is untouched and unaccounted for. Consumers **SHOULD** hold the response
for the working life of the session and drop it, and **SHOULD NOT** persist it
into any store whose deletion path is not wired to the same context ACL.

### Consent/purpose

The purpose is recall for the work the context exists to do, and the context ACL
is the record of the basis on which the read happens — a caller reads only where
it already holds access, with `vta/memory/list:contextForbidden` as the refusal
when it does not. That basis is *this* read, for *this* context, and does not
extend by volume: obtaining every item in one call authorises using them for the
context's work, not aggregating them into a profile for another purpose,
exporting them, or training on them. Whether a principal is told which of their
agents recall their memory, and how often, is a consumer policy question this
specification takes no position on.
