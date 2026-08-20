---
slug: vta/webvh/servers/retire-orphan
version: "0.1"
title: WebVH Servers — Retire Orphan
summary: A producer asks an agent to remove a slot from a hosting server after the agent has confirmed for itself that the slot is orphaned — served by the host, with no record in the agent that controls it.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords:
  - webvh
  - did-hosting
  - orphan
  - reconcile
  - cleanup
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer (operator or tooling)
    requirement: REQUIRED
    member: issuer
  - role: Agent holding the server registration and the DID records
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The act is destructive, has no undo, and removes an identifier from the public record. The document is the only durable evidence of who asked for it — the slot it names ceases to exist, so the request cannot be reconstructed from the result.
sideEffects:
  level: destructive
  rationale: "Irreversible at the hosting server. A retired slot stops being served, and any DID it published stops resolving. Nothing recreates it; a re-created slot is a new allocation with a new history."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/webvh/servers/retire-orphan:notOrphaned
    meaning: The agent holds a record for this slot, so it is not an orphan and this task is the wrong instrument. Deleting a DID the agent still controls is an ordinary delete, with its own authorization.
    retryable: false
  - code: vta/webvh/servers/retire-orphan:didMismatch
    meaning: The slot does not serve the DID the producer named. The report the producer acted on is stale, and proceeding would retire something it never saw.
    retryable: false
  - code: vta/webvh/servers/retire-orphan:listingUnavailable
    meaning: The agent cannot obtain the server's listing, so it cannot confirm the slot is orphaned — and will not act on the producer's word for it.
    retryable: true
related:
  - vta/webvh/servers/reconcile
  - did-management/did/delete
  - did-management/did/list
---

## Abstract

The **WebVH Servers — Retire Orphan** Trust Task removes a slot from a hosting server in the one case where no ordinary delete can reach it: when the agent that controls the server registration has **no record of the slot at all**.

[`vta/webvh/servers/reconcile`](../../reconcile/0.1) names these. It repairs nothing, deliberately, because its two divergences want opposite remedies. This task is the remedy for one of them — and only that one.

An orphan cannot be deleted the ordinary way, and the reason is structural rather than incidental. Every delete addresses a DID the agent holds a record for; it looks the record up to find which server to talk to and which keys to sign with. An orphan is defined by the absence of that record, so the lookup fails before any request reaches the host. The slot is visible, unreachable, and permanent.

It is also undeletable by anyone else. The producer holds no credentials for the hosting server — the agent does. So a slot that both parties can see, and neither can remove, accumulates.

## The precondition is checked, not asserted

The safety of this task rests on one rule: **the agent decides whether the slot is orphaned. The producer does not get to claim it.**

A conforming agent re-derives the orphan status at the moment of the request, exactly as [`reconcile`](../../reconcile/0.1) derives it, and refuses if it holds any record for the slot. A producer therefore cannot use this task to remove a live DID by mislabelling it: a live DID has a record, and the record is what makes the refusal automatic.

That inversion is what makes an undo-less operation safe to expose at all. Without it, this task would be "delete anything on the host, unaudited by the agent's own state" wearing a narrower name.

It follows that a producer cannot rely on a report to stay true. A reconcile response is a comparison at an instant, and a slot may be published to between the report and the request. That is what `expectedDid` is for: naming the DID the producer saw turns a stale report into a refusal rather than a surprise.

## It is not automatic, and MUST NOT be made so

A conforming agent **MUST NOT** retire a slot on a schedule, on a sweep, or as a consequence of any other task.

The signal a sweeper would act on is an *absence* — no local record. Absences are produced by bugs as readily as by orphaning: a storage read that fails open, a record written under the wrong server id, a migration half-applied. Every one of those presents to a sweeper as an orphan, and the sweeper's response is to make a published identifier stop resolving. The blast radius of a false positive is unbounded and silent, and the operation has no undo.

The population this addresses is also finite and historical. Agents that hold an idempotency key across retries do not create fresh orphans of the create-retried kind, and an agent whose delete confirms its remote leg before dropping the local record does not create them of the delete-failed kind. Automating the cleanup of a set that stops growing trades a permanent risk for a temporary saving.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/webvh/servers/retire-orphan/0.1`, with itself as `issuer`, the agent as `recipient`, `payload.serverId` naming a server the agent has registered, and `payload.slotId` naming a slot on it. A producer **SHOULD** obtain both from a [`reconcile`](../../reconcile/0.1) response rather than constructing them, and **SHOULD** send `expectedDid` whenever that response carried one.

A conforming **consumer** (the agent) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements).
2. Refuse with `notFound` where it holds no registration under `serverId`.
3. Determine for itself whether the slot is orphaned, by the same comparison [`reconcile`](../../reconcile/0.1) performs — the host's listing **scoped to the agent as owner**, against its own records naming that server. It **MUST NOT** treat the producer's request as evidence of orphanhood.
4. Refuse with `vta/webvh/servers/retire-orphan:notOrphaned` where it holds any record for the slot, whatever that record's state. A record that is disabled, archived, or superseded is still a record; the ordinary delete path exists for those, and it is the path that carries the right authorization.
5. Refuse with `vta/webvh/servers/retire-orphan:listingUnavailable` where it cannot obtain the listing. An agent that cannot confirm orphanhood **MUST NOT** proceed on the producer's word.
6. Refuse with `vta/webvh/servers/retire-orphan:didMismatch` where `expectedDid` is present and the slot does not serve it.
7. Report `retired: false` where the host did not confirm removal, rather than reporting success it did not observe.
8. Record the act — including `reason` where supplied — in whatever audit trail it keeps. The slot it names will not exist afterwards, so the request is the only account of what happened.

A conforming consumer **MUST NOT** perform this operation other than on an explicit request, and **MUST NOT** derive one from a `reconcile` result, a schedule, or a sweep.

An agent **SHOULD** require authority at least as broad as [`reconcile`](../../reconcile/0.1) demands. A slot absent from the agent belongs to no internal grouping there, so an agent cannot scope this the way it scopes its own listings — the same reasoning, reaching a stricter conclusion, because this one writes.

## Definitions

* **Producer.** The party asking; identified by `issuer`.
* **Agent.** The party holding both the server registration and the DID records, and performing the removal; identified by `recipient`.
* **Hosting server.** The DID-hosting service holding the slot; not a party to this document.
* **Slot.** A hosting server's unit of allocation for one DID — present from reservation onward, and therefore before any DID exists to name it. See [`reconcile`](../../reconcile/0.1) for why the spec does not call this a mnemonic.
* **Orphan.** A slot the hosting server serves for this agent, for which the agent holds no record. Established by the agent's own comparison, never by assertion.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/webvh/servers/retire-orphan/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "5f2c7a91-8b3d-4e60-9c14-7a2f5d8e3b06",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/retire-orphan/0.1",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:agent.example",
  "issuedAt": "2026-08-20T10:15:00Z",
  "payload": {
    "serverId": "primary-host",
    "slotId": "attract-case",
    "expectedDid": "did:webvh:QmZ4rT9xK2mN8vB5cD1sA7wE3fH6jL0pQ:did.example.com:attract-case",
    "reason": "orphaned by a create whose reply was lost; confirmed by reconcile 2026-08-19"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/webvh/servers/retire-orphan/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. Failures use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Retired

The DID is echoed because the slot that named it no longer exists.

```json
{
  "id": "6a3d8b02-9c4e-4f71-a025-8b3f6e9c4d17",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/retire-orphan/0.1#response",
  "threadId": "5f2c7a91-8b3d-4e60-9c14-7a2f5d8e3b06",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-08-20T10:15:04Z",
  "payload": {
    "serverId": "primary-host",
    "slotId": "attract-case",
    "retired": true,
    "did": "did:webvh:QmZ4rT9xK2mN8vB5cD1sA7wE3fH6jL0pQ:did.example.com:attract-case"
  }
}
```

### Not an orphan

The agent holds a record, so it refuses and names the instrument that fits.

```json
{
  "id": "7b4e9c13-0d5f-4a82-b136-9c4a7f0d5e28",
  "type": "https://trusttasks.org/spec/trust-task-error/0.3",
  "threadId": "5f2c7a91-8b3d-4e60-9c14-7a2f5d8e3b06",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-08-20T10:16:04Z",
  "payload": {
    "code": "vta/webvh/servers/retire-orphan:notOrphaned",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/webvh/servers/retire-orphan/0.1",
      "id": "5f2c7a91-8b3d-4e60-9c14-7a2f5d8e3b06"
    },
    "message": "slot `attract-case` on server `primary-host` has a record in this agent; retire-orphan applies only to slots this agent has no record of. Use did-management/did/delete.",
    "retryable": false
  }
}
```

### Stale report

The slot has been published to since the producer read it. The producer named what it saw, so the agent can tell the difference between a stale request and a wrong one.

```json
{
  "id": "8c5f0d24-1e60-4b93-c247-0d5b8a1e6f39",
  "type": "https://trusttasks.org/spec/trust-task-error/0.3",
  "threadId": "5f2c7a91-8b3d-4e60-9c14-7a2f5d8e3b06",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-08-20T10:17:04Z",
  "payload": {
    "code": "vta/webvh/servers/retire-orphan:didMismatch",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/webvh/servers/retire-orphan/0.1",
      "id": "5f2c7a91-8b3d-4e60-9c14-7a2f5d8e3b06"
    },
    "message": "slot `attract-case` serves a different DID than the one named; re-run reconcile before retiring it",
    "retryable": false
  }
}
```

`inResponseTo` is populated on these deliberately. [SPEC.md §8.2](../../../../../../SPEC.md#82-error-payload) makes it **MUST** where the error will be relied upon beyond the original producer, and a refusal to destroy something is exactly the kind of record an operator keeps.

## Security & Privacy

**The producer's claim is not evidence.** Every safety property here reduces to the agent checking orphanhood itself. An implementation that trusted `slotId` as a statement of fact would have built an unaudited "delete anything on the host" primitive with a reassuring name — the one failure mode that matters, and the one that is invisible until it is used.

**A refusal is the safe default and must stay that way.** Three conditions refuse: a record exists, the listing is unobtainable, the DID does not match. Two of them are *uncertainty* rather than *contradiction* — the agent does not know that the slot is safe to remove. An implementation tempted to proceed under uncertainty should note that the alternative to a false refusal is a retry, and the alternative to a false removal is nothing.

**It is a write against an identifier the agent cannot prove it owns.** By construction the agent holds no key for an orphan; its authority comes from the server registration, not from control of the DID. That is a broader authority than any other task in this family exercises, and it argues for correspondingly narrow authorization — an agent **SHOULD** treat this as an administrative operation rather than an ordinary DID lifecycle one.

**Retirement is publicly observable and irreversible.** A DID that stops resolving is visible to every relying party that held it, and they cannot distinguish retirement from compromise, revocation, or an outage. Producers **SHOULD** retire only what a reconcile report has shown to be orphaned, and **SHOULD** record `reason` — for a reader who finds the audit entry long after the slot, the rationale is the only thing left that explains it.
