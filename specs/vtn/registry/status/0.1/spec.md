---
slug: vtn/registry/status
version: "0.1"
title: "VTN Registry — Status"
summary: "An administrator reads how the network's membership stands in its trust registry: the writer, the last reconcile, and every record compared with what the entries project."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, trust-registry, trqp, administration, reconcile]
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the administrator's authentication: the network answers only a signer in its administrator list or a key one of them delegated, because the answer includes how its writes to the registry are failing.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects; a replay discloses only what the administrator was already entitled to see.
sideEffects:
  level: none
  rationale: >-
    The network reads its trust registry and its own records, and changes nothing. It does not reconcile.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: the network's registry configuration, its last reconcile, and its records in the registry, naming organisations by public DIDs. In: nothing.
retention:
  class: transient
  rationale: >-
    Nothing in the request is kept.
errorCodes:
  - code: vtn/registry/status:noRegistry
    meaning: "The network publishes its membership to no trust registry."
    retryable: false
related:
  - vtn/registry/check
  - vtn/listings/list
  - registry/record/query
---

## Abstract

A Verifiable Trust Network that publishes its membership to a trust registry keeps the registry in line with its entries by reconciling: it puts each record its entries project and deletes each record under its authority that they do not. **VTN Registry — Status** lets an administrator see how that stands: which registry, how the network writes to it and whether that transport is up, the last reconcile's outcome, and — asking the registry now — each record compared with what the entries project (`ok`, `missing`, `stale` or `wrong`).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** refuse a requester that is not an administrator (or a key one delegated) with `permissionDenied`.
2. **MUST** answer `noRegistry` when it publishes to no trust registry.
3. **MUST** report its last reconcile as it ended, including writes that failed and why the reconcile could not write at all, when so.
4. **MUST**, when the registry answers, list every party its entries project a record for and every record the registry holds under its authority and `resource`, each with its `result`:
   - `ok` — the registry holds the projected record;
   - `missing` — the entries project a record the registry does not hold;
   - `stale` — the registry holds a record the entries project none for;
   - `wrong` — the registry holds a record for the party under another action than projected.
5. **MUST** answer `reachable: false`, with no `records` and a `reason`, when the registry could not be asked. The last reconcile is still reported.
6. **MUST NOT** reconcile or write anything to answer.

## Definitions

- **Projected record** — the record an entry projects: acknowledged entries under the member action, and, where the network publishes them, listed entries under the listed action.
- **`writer`** — how the network writes to the registry, for an operator to read (for example, the transport and the mediator).
- **`connected`** — whether that transport is up right now.
- **`lastSync`** — the last reconcile's outcome.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b01",
  "type": "https://trusttasks.org/spec/vtn/registry/status/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-05T10:00:00Z",
  "payload": {}
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b02",
  "type": "https://trusttasks.org/spec/vtn/registry/status/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-05T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b01",
  "payload": {
    "registry": "did:example:registry",
    "authority": "did:example:whn",
    "resource": "vtn",
    "actions": { "member": "member", "listed": "lists" },
    "publishes": "members",
    "writer": "DIDComm to did:example:registry through did:example:mediator",
    "connected": true,
    "lastSync": {
      "reconciledAt": "2026-11-05T09:58:00Z",
      "expected": 2,
      "put": ["did:example:msf"],
      "deleted": [],
      "failed": []
    },
    "reachable": true,
    "records": [
      { "did": "did:example:msf", "projected": "member", "held": "member", "result": "ok" },
      { "did": "did:example:health-au", "projected": "member", "result": "missing" }
    ]
  }
}
```

## Security & Privacy

### Data carried

The network's own configuration and records, which name organisations by public identifiers.

### Correlation

None beyond what the administrator already holds.

### Retention

Nothing is kept.

### Consent/purpose

For the network's administrators, to see whether what the registry publishes matches the network.
