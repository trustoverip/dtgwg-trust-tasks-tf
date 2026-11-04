---
slug: vtn/relay/deliver
version: "0.1"
title: "VTN Relay — Deliver"
summary: "A parent network delivers a signed broadcast envelope to this network's group for a role, and this network relays it on to its own groups."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, broadcast, relay, roles, nested-membership]
parties:
  - role: parent network
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the parent network's authentication: this network relays only what a network it has joined delivers, so an unattributed delivery would let anyone fan a message out to its members' groups.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Duplicate messages are caught by message id regardless; the document's own freshness bounds how long a captured delivery can be replayed at all.
sideEffects:
  level: mutating
  rationale: >-
    Records the received broadcast and delivers it onward to every group registered for the role, queueing deliveries that fail in transport. Deliveries made cannot be recalled.
consequences:
  - "The message is delivered onward, under a new hop signed by this network, to every group its current members registered for the role."
  - "Deliveries that fail in transport are queued and retried until they succeed, the message expires, or the network's retry limit is reached."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a broadcast between organisations, signed by its origin and every relaying network. Out: delivery receipts naming the groups reached and the members that own them, never who is in a group.
retention:
  class: durable
  rationale: >-
    The network keeps the message id to answer later copies as duplicates, and the broadcast with its receipts as the record of what it relayed on whose behalf; queued deliveries are kept until they settle.
errorCodes:
  - code: vtn/relay/deliver:notMember
    meaning: "The issuer is not a network this one is an acknowledged member of, or this network's group for `role` is not registered with it."
    retryable: false
  - code: vtn/relay/deliver:unknownRole
    meaning: "This network defines no role named `role`."
    retryable: false
  - code: vtn/relay/deliver:envelopeInvalid
    meaning: "The envelope does not verify: an origin or hop signature fails, the message has expired, the hop chain is broken or out of order, it exceeds the hop limit, its last hop is not addressed to this network's group for `role`, or its last hop was not signed by the issuer. `details.message` says which."
    retryable: false
    detailsSchema:
      type: object
      required: [message]
      properties:
        message:
          type: string
related:
  - vtn/broadcasts/send
  - vtn/broadcasts/list
  - vtn/groups/register
  - vtn/memberships/join
  - vtn/memberships/groups/add
  - vtn/outbox/list
---

## Abstract

A Verifiable Trust Network broadcasts to a role — every member's group that serves `admins`, say. A member that is itself a network registered its **network group** for that role ([`vtn/groups/register`](../../../groups/register/0.1/spec.md)), so the parent does not reach that network's members directly: it delivers to the member network, which relays the message to its own members' groups, one level at a time. No network ever holds another's roster, and the message keeps the trail of who relayed it to whom — *WHN → Health-AU → Health-NSW → St Vincent's admins* — rather than one flattened claim ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-032, VTI-VTN-033).

**VTN Relay — Deliver** is that hand-over. The parent network sends the **envelope** — the origin's signed message, and one signed **hop** per delivery step — naming the local role whose network group is addressed. This network verifies the envelope end to end, relays it to its own groups under a hop of its own, and answers with the receipts.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **parent network** (`issuer`):

1. **MUST** deliver only to a group registered with it, for the role being broadcast, by a member whose entry is `acknowledged`, and **MUST** set `role` to the local role in that network group's identifier (`<recipient>#role-<role>`).
2. **MUST** append to the envelope a hop it signs for `assertionMethod`, with `index` the hop's position, `from` its own DID, `to` the group, and `message` the message's `id`, and **MUST** leave the message and the earlier hops exactly as it received or made them.
3. **MUST NOT** deliver an envelope whose hops would exceed the hop limit; it records that delivery as `failed` instead.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` when it does not accept relayed broadcasts at all.
2. **MUST** refuse with `vtn/relay/deliver:unknownRole` when it defines no role named `role`.
3. **MUST** verify the envelope and refuse with `vtn/relay/deliver:envelopeInvalid` unless:
   - the message is a broadcast whose `origin` signed it for `assertionMethod`, its `expiresAt` has not passed, and its `expiresAt` is no more than 7 days after its `sentAt`;
   - there is at least one hop and no more than its hop limit (by default 8, matching the DTG attenuation depth);
   - each hop is for the message's `id`, carries its position as `index`, and is signed for `assertionMethod` by its `from`, where the first hop's `from` is the origin and each later hop's `from` is the network that owns the previous hop's `to` (for a network group `<did>#role-<name>`, that `<did>`);
   - the last hop's `to` is this network's own group for `role`, `<recipient>#role-<role>`; and
   - the last hop's `from` is `issuer`.
4. **MUST** refuse with `vtn/relay/deliver:notMember` unless `issuer` is a network this one has joined and is an acknowledged member of, and this network's group for `role` is registered with it ([`vtn/memberships/groups/add`](../../../memberships/groups/add/0.1/spec.md)).
5. **MUST** answer a message whose `id` it has seen before — sent by itself or delivered by any network — with `status` `duplicate` and no receipts, and relay nothing. A duplicate is an outcome, not an error: another path got there first.
6. **MUST** otherwise deliver the message to every group registered for `role` by a member whose entry here is `acknowledged`, each under a new hop it signs as in parent item 2: to a member network's group by sending it this task, and to a community's group at the endpoint it registered. A delivery whose hop would exceed the limit is recorded `failed` without being attempted; one that fails in transport is queued and retried.
7. **MUST** answer `status` `delivered`, with one receipt per group it delivered to, and record the broadcast as received from `issuer`.
8. **MUST NOT** expand another network's group, deliver to anyone but its own members' registered groups, or reveal who is in any group: a receipt names a group and the member that owns it, never its members (VTI-VTN-060 – VTI-VTN-063).

## Authorization

The authority this task presupposes is **being a joined parent network**: this network relays only for a network it has joined — its own acknowledged upstream membership, recorded when it joined ([`vtn/memberships/join`](../../../memberships/join/0.1/spec.md)) — and only to the group it registered there for `role` (network item 4). The request's proof establishes which network delivers; the envelope's hops establish that the same network relayed the message, and the origin's signature who sent it. The decision to join the parent, and to map the parent's role to `role`, was made earlier by this network's administrators and is read from its records at execution time.

## Definitions

- **`role`** — this network's local role whose network group (`<recipient>#role-<role>`) the delivery addresses. It is the role this network's administrators mapped to the parent's role when they registered the group.
- **`envelope`** — a broadcast in transit ([`Envelope`](../../../_shared/0.1/relay.schema.json)): `message`, signed by the origin network, carrying its `id`, `origin`, `role`, `class`, `subject`, `body`, `sentAt`, `expiresAt` and optionally `replyTo`; and `hops`, each signed by the network that made it, carrying `message`, `index`, `from`, `to`, `role` and `at`.
- **Hop limit** — the most hops an envelope may carry; a network's own configuration, by default 8.
- **`status`** — `delivered` when this network accepted the message and relayed it, `duplicate` when it had seen the message before ([`DeliveryStatus`](../../../_shared/0.1/relay.schema.json)).
- **`relayed`** — one [`Receipt`](../../../_shared/0.1/relay.schema.json) per group this network delivered to; a receipt for a member network's group carries that network's own receipts in turn.

## Request

The parent network (`issuer`) delivers an envelope to this network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A parent relays a broadcast to a member network's `admins` group

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000901",
  "type": "https://trusttasks.org/spec/vtn/relay/deliver/0.1#request",
  "issuer": "did:example:whn",
  "recipient": "did:example:health-au",
  "issuedAt": "2026-11-03T12:00:00Z",
  "payload": {
    "role": "admins",
    "envelope": {
      "message": { "id": "urn:uuid:00000000-0000-4000-8000-0000000b0001" },
      "hops": [
        { "message": "urn:uuid:00000000-0000-4000-8000-0000000b0001", "index": 0 }
      ]
    }
  }
}
```

The message and hop are abbreviated; the network sends them in full, with their proofs.

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: what happened to the message, and the receipts of its own deliveries. A refusal is a `trust-task-error`.

### Relayed to two groups

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000902",
  "type": "https://trusttasks.org/spec/vtn/relay/deliver/0.1#response",
  "issuer": "did:example:health-au",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T12:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000901",
  "payload": {
    "status": "delivered",
    "relayed": [
      {
        "group": "did:example:health-nsw#role-admins",
        "member": "did:example:health-nsw",
        "status": "delivered",
        "network": true,
        "relayed": [
          { "group": "did:example:st-vincents-admins", "member": "did:example:st-vincents", "status": "delivered" }
        ]
      },
      {
        "group": "did:example:rpa-admins",
        "member": "did:example:rpa",
        "status": "queued",
        "detail": "endpoint unreachable; will retry"
      }
    ]
  }
}
```

### Already seen

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000904",
  "type": "https://trusttasks.org/spec/vtn/relay/deliver/0.1#response",
  "issuer": "did:example:health-au",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T12:00:05Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000903",
  "payload": {
    "status": "duplicate",
    "relayed": []
  }
}
```

## Security & Privacy

### Data carried

The request carries a broadcast between organisations — its subject and body are whatever the origin network sent to a role — and the signed hops that show its route. The response carries receipts naming groups and the members that own them. Neither carries a group's membership. An origin network **MUST NOT** put personal data in a broadcast's body that it would not hand to every group serving the role, since every relaying network can read it.

### Correlation

The hops record which network relayed the message to which group, so each network on the path, and each recipient group, learns the route from the origin down to it; that trail is the point (VTI-VTN-032). Both parties declare `identifierScope: public` because they are networks found by their public DIDs: each hop must be checkable against a network named the same way by every party on the path, and a network group's identifier is built on its network's DID. Receipts tell the parent which of this network's groups were reached — the same groups its members registered with this network — and nothing about who is in them.

### Retention

The network keeps the message id for as long as it answers duplicates, the broadcast and its receipts as the record of what it relayed for whom, and queued deliveries until they are delivered, expire with the message, or exhaust their retries.

### Consent/purpose

Each member consented to receive the network's broadcasts for a role by registering a group for it, and this network consented to receive its parent's by registering its network group there. The message is relayed for that purpose only. Whether the content of a message is acceptable is the origin network's governance, not the relaying network's; a relaying network **MAY** decline relays altogether (network item 1).
