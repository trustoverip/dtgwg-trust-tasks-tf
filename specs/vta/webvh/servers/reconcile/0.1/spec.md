---
slug: vta/webvh/servers/reconcile
version: "0.1"
title: WebVH Servers — Reconcile
summary: A producer asks an agent to compare the DIDs a hosting server holds for it against the DIDs it has records for, and to report where the two disagree.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords:
  - webvh
  - did-hosting
  - reconcile
  - drift
  - diagnostics
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
  requirement: RECOMMENDED
  rationale: The report is a diagnostic read whose integrity is already guaranteed by the authenticated transport session. A proof matters when the report is retained as evidence of what a host was serving at a point in time — the state it describes is precisely the state that later gets repaired, so the record of it outlives the condition.
sideEffects:
  level: none
  rationale: "Read-only. The agent performs a network round trip to the hosting server and compares two listings; nothing is created or changed on either side, and no divergence is repaired."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/webvh/servers/reconcile:listingUnavailable
    meaning: The agent holds a registration for this server but cannot obtain its DID listing over the transport it has — so it cannot compare, and says so rather than reporting an empty divergence.
    retryable: true
related:
  - vta/webvh/servers/domains
  - did-management/did/list
  - did-management/did/delete
---

## Abstract

The **WebVH Servers — Reconcile** Trust Task answers "does server *X* still agree with you about which DIDs it hosts for you?"

A DID can exist on a hosting server and nowhere in the agent that controls it. The usual cause is a delete whose remote leg failed: an agent that removes the local record anyway leaves the server publishing a DID whose controller has discarded its keys. Nothing about that state is visible from either end alone — the server is serving a DID it believes is owned, and the agent simply has no record to be wrong about. It surfaces later as an update that cannot be signed, which reads to an operator like lost key material rather than an orphan.

The reverse happens too, and is milder: an agent that recorded a DID whose publish never reached the host.

This task reports both. It **repairs neither**, because the two want opposite remedies — one wants removing at the host, the other wants its publish retrying — and neither is safe to infer from a list.

## Why the agent has to answer this

Neither end can do it alone, which is the whole reason this is a task rather than something a caller assembles.

The producer holds no credentials for the hosting server; the server holds no view of the agent's records. The agent holds both. It authenticates to the server with its own credentials, reads the server's listing **scoped to itself as owner**, and compares that against its local records for the same server.

That scoping is not incidental. An agent that administers its own hosting server is an administrative caller there, and a hosting server asked for a listing by an administrator who names no owner may reasonably answer with every DID it holds. Compared against one agent's records, every other tenant's DID would then appear as a divergence. A conforming agent therefore always scopes the listing to itself.

## Why this is not a `list` task with a filter

[SPEC.md](../../../../../../SPEC.md) asks new families to ship a `list`/`get` pair, or to say why a filter suffices. Neither shape fits, because the response is not an enumeration of a collection.

A divergence is a property of **two listings compared at one instant**, not a record with an identity that persists. There is nothing for a sibling `get` to fetch by id: asking "show me divergence *D*" is asking a question about a comparison that has to be re-run to be answered, and re-running it may legitimately return nothing because the operator repaired it. The identifiers that *do* persist — the slot on the host, the DID in the agent — already have their own read tasks in the [`did-management`](../../../../../did-management) family and in the agent's own surface.

So this task returns the whole comparison, and the count of what matched, in one document.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/webvh/servers/reconcile/0.1`, with itself as `issuer`, the agent as `recipient`, and `payload.serverId` naming a server the agent has registered.

A conforming **consumer** (the agent) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements).
2. Refuse with `notFound` where it holds no registration under `serverId`. An unknown server is not an empty comparison — a producer told "nothing diverges" about a server the agent has never heard of has been given a false assurance.
3. Obtain the hosting server's listing **scoped to the agent itself as owner**, and compare it only against local records that name that same server. Records naming another server, or no server at all, are not missing from this one.
4. Compare on the **slot identifier**, not the DID. A slot the server reserved but never published to carries no DID, and omitting it would hide exactly the entries an operator is least likely to find another way.
5. Report `inBoth` even when both divergence arrays are empty. It is what separates "compared them, all matched" from "compared nothing" — see [§ Security & Privacy](#security--privacy).
6. Refuse with `vta/webvh/servers/reconcile:listingUnavailable` where it holds the registration but cannot obtain the server's listing over the transport available to it. **An agent MUST NOT answer an unobtainable listing with an empty comparison.**

A conforming consumer **MUST NOT** repair a divergence it reports, and **MUST NOT** treat this task as authorization to do so. Removing a DID from a host and re-publishing one are separate acts with their own tasks, their own authorization, and — in the first case — no undo.

An agent **SHOULD** return both arrays in a stable order, so that two reports of an unchanged estate compare equal.

## Definitions

* **Producer.** The party asking; identified by `issuer`.
* **Agent.** The party holding both the server registration and the DID records, and performing the comparison; identified by `recipient`.
* **Hosting server.** The DID-hosting service being compared against; not a party to this document.
* **Slot.** A hosting server's unit of allocation for one DID — the identifier its own management API addresses, present from reservation onward and therefore before any DID exists to name. Implementations spell this differently in their own APIs; `slotId` is the spec's name for it, chosen because the term one implementation uses (`mnemonic`) means an unrelated thing (a BIP-39 recovery phrase) elsewhere in the same deployments.
* **Divergence.** A slot present on one side of the comparison and absent from the other.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/webvh/servers/reconcile/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "8c1f2d90-4a7b-4c19-9f3e-2b6d5c8a71e4",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/reconcile/0.1",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:agent.example",
  "issuedAt": "2026-08-14T09:00:00Z",
  "payload": {
    "serverId": "primary-host"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/webvh/servers/reconcile/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

`hostOnly` and `agentOnly` are the two divergences; `inBoth` is the count that matched. Failures use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Divergence in both directions

An orphan the host still serves, and a DID whose publish never landed.

```json
{
  "id": "b6a4e3f1-2c58-4d0a-8e17-3f9c2a5b6d70",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/reconcile/0.1#response",
  "threadId": "8c1f2d90-4a7b-4c19-9f3e-2b6d5c8a71e4",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-08-14T09:00:03Z",
  "payload": {
    "serverId": "primary-host",
    "hostOnly": [
      {
        "slotId": "attract-case",
        "did": "did:webvh:QmZ4rT9xK2mN8vB5cD1sA7wE3fH6jL0pQ:did.example.com:attract-case",
        "domain": "did.example.com",
        "disabled": false
      },
      {
        "slotId": "quiet-harbour",
        "disabled": false
      }
    ],
    "agentOnly": [
      {
        "did": "did:webvh:QmY8nP3bV6xC1kM4hS9dF2gJ5tR7wL0zQ:did.example.com:never-landed",
        "slotId": "never-landed",
        "contextId": "production"
      }
    ],
    "inBoth": 14
  }
}
```

The second `hostOnly` entry is a reserved slot that was never published to: it has no `did`, and it is exactly as unreconciled as the first.

### Agreement

An estate where nothing diverges. The count is what makes this readable as an answer rather than as an empty screen.

```json
{
  "id": "c7b5f4a2-3d69-4e1b-9f28-4a0d3b6c7e81",
  "type": "https://trusttasks.org/spec/vta/webvh/servers/reconcile/0.1#response",
  "threadId": "8c1f2d90-4a7b-4c19-9f3e-2b6d5c8a71e4",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-08-14T09:04:03Z",
  "payload": {
    "serverId": "primary-host",
    "hostOnly": [],
    "agentOnly": [],
    "inBoth": 15
  }
}
```

### Listing unobtainable

The agent holds the registration but cannot read the server's listing over the transport it has. It says so rather than reporting agreement it did not observe.

```json
{
  "id": "d8c6a5b3-4e70-4f2c-a039-5b1e4c7d8f92",
  "type": "https://trusttasks.org/spec/trust-task-error/0.3",
  "threadId": "8c1f2d90-4a7b-4c19-9f3e-2b6d5c8a71e4",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-08-14T09:06:03Z",
  "payload": {
    "code": "vta/webvh/servers/reconcile:listingUnavailable",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/webvh/servers/reconcile/0.1",
      "id": "8c1f2d90-4a7b-4c19-9f3e-2b6d5c8a71e4"
    },
    "message": "server `primary-host` is reachable, but its DID listing is not available over the transport this agent has registered for it",
    "retryable": true
  }
}
```

`inResponseTo` is populated here deliberately. [SPEC.md §8.2](../../../../../../SPEC.md#82-error-payload) makes it **MUST** where the error will be relied upon beyond the original producer, and this one will be: a report that could not be produced is itself a finding an operator may keep, and `threadId` alone means nothing to a reader who never saw the request.

## Security & Privacy

**An empty answer is the dangerous answer.** Every other failure in this task announces itself; a comparison that silently ran against nothing looks exactly like a healthy estate, and is the one result an operator stops investigating after. That is why `inBoth` is required rather than optional, why an unobtainable listing is an error rather than an empty report, and why an unregistered `serverId` is `notFound` rather than a comparison with no divergences. Each of those is a place where the reassuring answer and the truthful one differ.

**The report is a map of what cannot be defended.** A `hostOnly` entry names a DID that its own controller can no longer sign for — useful to the operator repairing it, and equally useful to anyone considering whether a DID is worth attacking, since it identifies identifiers whose controller has already lost the ability to rotate them. Agents **SHOULD** answer only for producers entitled to act through them, and producers **SHOULD** treat a stored report as sensitive for as long as the divergence it describes is unrepaired.

**The comparison is scoped, and the scoping is load-bearing.** An agent that requested an unscoped listing from a server it administers would receive other tenants' DIDs and report them as its own divergences — disclosing the server's tenancy to a producer who asked only about one agent, and burying the real divergences in noise.

**It reads across whatever internal grouping the agent applies.** A divergence has no such grouping by construction — a DID absent from the agent belongs to no context there — so an agent cannot scope this report the way it scopes its own listings, and **SHOULD** require correspondingly broad authority to answer it at all.
