---
slug: vtc/networks/deliver
version: "0.1"
title: "VTC Networks — Deliver"
summary: "A Verifiable Trust Network delivers a signed broadcast envelope to the community's group registered for one of its roles, and the community passes it on to the group's members."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, broadcast, relay, roles]
parties:
  - role: network
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the network's authentication: the community passes on only what a network it belongs to delivers, so an unattributed delivery would let anyone message the members of its registered groups in a network's name.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Duplicate messages are caught by message id regardless; the document's own freshness bounds how long a captured delivery can be replayed at all.
sideEffects:
  level: mutating
  rationale: >-
    Records the message as delivered and sends vtc/networks/notice to every active member of the group registered for the role. Notices sent cannot be recalled.
consequences:
  - "Every active member of the community's group registered for `role` receives the envelope, unchanged, in a vtc/networks/notice signed by the community."
  - "The message id is kept, so a later copy of the message — by any path — is answered as a duplicate and sent to nobody."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a broadcast between organisations, signed by its origin and every relaying network. Out: whether the community passed it on — never who received it, nor how many.
retention:
  class: durable
  rationale: >-
    The community keeps the message id to answer later copies as duplicates, and the delivery as the record of what it passed on to its members on which network's behalf.
errorCodes:
  - code: vtc/networks/deliver:notMember
    meaning: "The issuer is not a network the community holds an acknowledged membership in."
    retryable: false
  - code: vtc/networks/deliver:roleUnregistered
    meaning: "The community has no group registered with the issuer for `role` whose acceptance is in force."
    retryable: false
  - code: vtc/networks/deliver:envelopeInvalid
    meaning: "The envelope does not verify: an origin or hop signature fails, the message has expired, the hop chain is broken or out of order, it exceeds the hop limit, its last hop is not addressed to the community's group for `role`, or its last hop was not signed by the issuer. `details.message` says which."
    retryable: false
    detailsSchema:
      type: object
      required: [message]
      properties:
        message:
          type: string
related:
  - vtc/networks/notice
  - vtc/networks/groups/register
  - vtc/networks/join
  - vtc/networks/leave
  - vtn/relay/deliver
  - vtn/broadcasts/send
---

## Abstract

A Verifiable Trust Network broadcasts to a role — every member's group that serves `admins`, say ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063), VTI-VTN-032, VTI-VTN-033). A community serves a role with one of its own groups, registered with the network through [`vtc/networks/groups/register`](../../groups/register/0.2/spec.md), which names the community's Trust Task endpoint as the group's `endpoint`. A member network passes the broadcast on to its own members' groups with [`vtn/relay/deliver`](../../../../vtn/relay/deliver/0.1/spec.md); a community's group is the end of the line, and the network hands the broadcast to the community with this task.

**VTC Networks — Deliver** is that hand-over. The network sends the **envelope** — the origin's signed message, and one signed **hop** per delivery step, the last addressed to the community's group — naming the role. The community verifies the envelope end to end and passes it on, unchanged, to each active member of the group with [`vtc/networks/notice`](../../notice/0.1/spec.md). It answers only whether it did: never who received the message, nor how many, so the network does not learn the group's size.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`issuer`):

1. **MUST** deliver only to a group registered with it, for the role being broadcast, by a community whose entry is `acknowledged`, at the `endpoint` that registration names, and **MUST** set `role` to that role.
2. **MUST** append to the envelope a hop it signs for `assertionMethod`, with `index` the hop's position, `from` its own DID, `to` the group's DID, and `message` the message's `id`, and **MUST** leave the message and the earlier hops exactly as it received or made them.
3. **MUST** sign the document for `authentication` as itself, so that the delivery is attributable to the network the last hop names.

A conforming **community** (`recipient`):

1. **MUST** verify that the document's proof is the issuer's, made for `authentication` by a key of `issuer`.
2. **MUST** refuse with `vtc/networks/deliver:notMember` unless `issuer` is a network the community holds an acknowledged membership in ([`vtc/networks/join`](../../join/0.2/spec.md)). A network the community left or repudiated is not one.
3. **MUST** refuse with `vtc/networks/deliver:roleUnregistered` when it has no group registered with `issuer` for `role` ([`vtc/networks/groups/register`](../../groups/register/0.2/spec.md)) whose acceptance is in force.
4. **MUST** verify the envelope as a VTN envelope ([`Envelope`](../../../../vtn/_shared/0.1/relay.schema.json)), as [`vtn/relay/deliver`](../../../../vtn/relay/deliver/0.1/spec.md) network item 3 does, and refuse with `vtc/networks/deliver:envelopeInvalid` unless:
   - the message is a broadcast whose `origin` signed it for `assertionMethod`, its `expiresAt` has not passed, and its `expiresAt` is no more than 7 days after its `sentAt`;
   - there is at least one hop and no more than the hop limit, 8;
   - each hop is for the message's `id`, carries its position as `index`, and is signed for `assertionMethod` by its `from`, where the first hop's `from` is the origin and each later hop's `from` is the network that owns the previous hop's `to` (for a network group `<did>#role-<name>`, that `<did>`);
   - the last hop's `to` is the DID of the community's group registered with `issuer` for `role`; and
   - the last hop's `from` is `issuer`.
5. **MUST** answer a message whose `id` it has already delivered — by this network or by any other path — with `status` `duplicate`, and send nothing. A duplicate is an outcome, not an error: another path got there first.
6. **MUST** otherwise send [`vtc/networks/notice`](../../notice/0.1/spec.md) to each active member of the registered group, carrying the envelope unchanged, with `network` the issuer, `role` and the group's `groupId`; record the message id as delivered; and answer `status` `delivered`. A notice that fails in transport is the community's to retry; it does not change the answer.
7. **MUST NOT** tell the network who is in the group, how many members it has, or which of them a notice reached: the answer is `status` alone (VTI-VTN-060 – VTI-VTN-063).

## Authorization

The authority this task presupposes is **being a network the community belongs to**: the community passes on broadcasts only from a network it holds an acknowledged membership in — recorded when it joined — and only to the group it registered there for `role` (community items 2 and 3). The request's proof establishes which network delivers; the envelope's hops establish that the same network relayed the message, and the origin's signature who sent it. The decision to join the network, and which group serves each role, was made earlier by the community's administrators and is read from its records at execution time.

## Definitions

- **`role`** — the network's role whose registered community group the delivery addresses. It is the role the community named when it registered the group.
- **`envelope`** — a broadcast in transit ([`Envelope`](../../../../vtn/_shared/0.1/relay.schema.json)): `message`, signed by the origin network, carrying its `id`, `origin`, `role`, `class`, `subject`, `body`, `sentAt`, `expiresAt` and optionally `replyTo`; and `hops`, each signed by the network that made it, carrying `message`, `index`, `from`, `to`, `role` and `at`.
- **Hop limit** — the most hops an envelope may carry: 8, the bound [`vtc/networks/notice`](../../notice/0.1/spec.md) carries on to the members.
- **`status`** — `delivered` when the community accepted the message and passed it on, `duplicate` when it had delivered the message before.

## Request

The network (`issuer`) delivers an envelope to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A network delivers a broadcast to the community's `admins` group

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000381",
  "type": "https://trusttasks.org/spec/vtc/networks/deliver/0.1#request",
  "issuer": "did:example:whn",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T09:59:01Z",
  "payload": {
    "role": "admins",
    "envelope": {
      "message": {
        "type": "VtnBroadcast",
        "id": "urn:uuid:00000000-0000-4000-8000-00000000b001",
        "origin": "did:example:whn",
        "role": "admins",
        "class": "notice",
        "subject": "Regional exercise",
        "body": "A coordination exercise runs on 12 November at 09:00 UTC.",
        "sentAt": "2026-11-03T09:59:00Z",
        "expiresAt": "2026-11-06T09:59:00Z",
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "proofPurpose": "assertionMethod",
          "verificationMethod": "did:example:whn#key-0",
          "proofValue": "z3abc"
        }
      },
      "hops": [
        {
          "type": "VtnRelayHop",
          "message": "urn:uuid:00000000-0000-4000-8000-00000000b001",
          "index": 0,
          "from": "did:example:whn",
          "to": "did:example:community-admins-group",
          "role": "admins",
          "at": "2026-11-03T09:59:01Z",
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "proofPurpose": "assertionMethod",
            "verificationMethod": "did:example:whn#key-0",
            "proofValue": "z3def"
          }
        }
      ]
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:whn#key-0",
    "created": "2026-11-03T09:59:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ghi"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Passed on to the group's members

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000382",
  "type": "https://trusttasks.org/spec/vtc/networks/deliver/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T09:59:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000381",
  "payload": {
    "status": "delivered"
  }
}
```

### Already delivered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000384",
  "type": "https://trusttasks.org/spec/vtc/networks/deliver/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T09:59:07Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000383",
  "payload": {
    "status": "duplicate"
  }
}
```

## Security & Privacy

### Data carried

The request carries a broadcast between organisations — its subject and body are whatever the origin network sent to a role — and the signed hops that show its route. The response carries only `status`. Neither carries the group's membership, its size, or anything about the members a notice reaches.

### Correlation

The hops record which network relayed the message to which group, so the community learns the route from the origin down to its group; that trail is the point (VTI-VTN-032). Both parties declare `identifierScope: public` because they are found by their public DIDs: the network's hop must be checkable against the DID the community joined, and the community is the member the network lists. The answer is the same whether the group has one member or a thousand, so the network learns that its broadcast arrived and nothing about who is in the group.

### Retention

The community keeps the message id for as long as it answers duplicates — at least until the message's `expiresAt` — and a record of the delivery as evidence of what it passed on to its members on which network's behalf. It keeps no record of individual notices beyond what its delivery queue needs.

### Consent/purpose

The community consented to receive the network's broadcasts for a role by registering a group for it, and the group's members by being in it. The message is passed on for that purpose only. Whether the content of a message is acceptable is the origin network's governance; the community carries it unchanged ([`vtc/networks/notice`](../../notice/0.1/spec.md) community item 3), and withdraws its consent by registering another group or by leaving the network.
