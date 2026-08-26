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
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A delete mutates durable per-context state the VTA audits; transport-independent integrity is required.
sideEffects:
  level: mutating
  rationale: "Forgets a memory item from a context by key."
subjectPath: /contextId
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: "`key` is the agent-chosen label from `vta/memory/put` and is descriptive in its own right — a key naming a person, a condition, or a dispute states its subject even though the value it addressed is never carried here. Nothing is disclosed back to the caller; what travels inbound is the name of the thing being forgotten."
retention:
  class: durable
  rationale: The item is destroyed, but the deletion is not — it mutates per-context state the VTA audits, so the record of the forgetting, including `key`, outlives the thing forgotten. A consumer that deleted that record would lose the ability to answer whether a given item was ever removed, and by whom, which is the one question a disputed erasure turns on.
errorCodes:
  - code: vta/memory/delete:contextForbidden
    meaning: The caller is not permitted to delete memory in the named context.
    retryable: false
  - code: vta/memory/delete:notFound
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
it can access, and an unknown key yields `vta/memory/delete:notFound`.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply. The VTA **MUST** refuse with `contextForbidden` when the caller lacks access to `payload.contextId`, with `notFound` when the key is absent, and otherwise remove the item.

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

## Security & Privacy

### Data carried

Two members inbound, `contextId` and `key`, and one echoed back. The `value` is
never carried in either direction — the VTA already holds it and the caller does
not need it back to destroy it, which is the right shape and worth stating
because it is easy to get wrong.

`key` is nonetheless content. It is the label the agent chose at
[`vta/memory/put`](../../put/0.1/spec.md), and labels describe: `invoice-contact`
names a business relationship, and a key naming a person, a diagnosis, or a legal
matter states its subject to anyone who sees the request, without any value
attached. A producer that treats `key` as an opaque handle is mistaken about what
it is disclosing here.

The counter-intuitive part is what this document leaves behind. The deletion is a
`mutating` operation the VTA audits, and the request carries a REQUIRED `proof`.
So the act of forgetting is itself remembered, attributably, with the key in it:
after this task succeeds, the VTA cannot tell you what `invoice-contact` said, but
it can tell you that this agent removed something called `invoice-contact` at this
moment. A consumer that needs the label gone as well as the item needs a redaction
path this specification does not define.

### Correlation

The error codes distinguish `vta/memory/delete:contextForbidden` from
`vta/memory/delete:notFound`, and that pair is an existence oracle. Inside a
context the caller may already reach, a delete against an absent key returns
`notFound` and changes nothing — so a caller can probe whether a given key exists,
repeatedly and non-destructively, without ever holding the read capability that
[`vta/memory/list`](../../list/0.1/spec.md) requires. Since keys are descriptive,
confirming that `finance` holds a key is frequently the interesting bit. The
distinction earns its place — collapsing the two codes would tell a legitimate
caller nothing about why its delete failed — but a VTA whose write capability is
granted more freely than its read capability has, through this task, granted a
narrow read as well.

Deletions also correlate over time. A sequence of removals is a record of what a
principal asked to have forgotten and when, which is a sharper signal than the
memory it erased; the timing alone marks the moment something changed. The
`issuer` DID is stable so the VTA can bind the delete to the agent that holds
context access, and only the VTA needs that recognition, so the Agent declares
`identifierScope: pairwise`.

### Retention

The item goes; the record of its going stays. This task offers no acknowledgement
that the material is unrecoverable — the `#response` echoes `key` and asserts
nothing about backups, replicas, or the VTA's own audit trail, and there is no
member through which a VTA could assert it if it wanted to. Erasure at the store
of record is therefore the *beginning* of forgetting rather than the end of it,
and a caller **MUST NOT** report to a principal that a memory is gone on the
strength of this response alone: copies handed out by earlier
[`vta/memory/list`](../../list/0.1/spec.md) calls are outside this task's reach,
as are the maintainer's own snapshots. Implementers **SHOULD** document how far
their delete actually propagates, because this specification cannot.

### Consent/purpose

The purpose is narrow and negative — to stop holding one named item — and the
context ACL is the basis, with `vta/memory/delete:contextForbidden` as the refusal
when the caller has no standing in the context. Nothing here establishes *whose*
memory is being forgotten or on whose instruction: the task authenticates the
agent, not the principal behind it, so a VTA cannot distinguish a principal
exercising erasure from an agent tidying its own workspace. Deployments that need
that distinction have to carry it outside this payload. Whether a deletion
requires a human in the loop, and whether the audit record of the deletion is
itself subject to erasure, are consumer policy questions this specification takes
no position on.
