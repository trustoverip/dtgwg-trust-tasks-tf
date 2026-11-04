---
slug: vtn/broadcasts/send
version: "0.1"
title: "VTN Broadcasts — Send"
summary: "An administrator sends a message to a role: the network signs it and delivers it to every current member's group for that role, network groups relay it further, and receipts come back."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, broadcast, relay, messaging]
parties:
  - role: administrator
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
    The proof is the administrator's authentication: the network signs the message as its own and sends it to every member's group, so an unattributed request would let anyone speak for the network to all of its members.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would send the same message again under a new message id, which no recipient could drop as a repeat, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Signs a new message, delivers it to groups outside the network, queues failed deliveries for retry, and records the broadcast. A delivered message cannot be recalled; it expires at its `expiresAt`.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a role, a subject, a body and an optional reply address written by an administrator. Out: the broadcast record, with per-group receipts naming groups and the members that own them, never who is in a group.
retention:
  class: durable
  rationale: >-
    The broadcast record is kept so administrators can see what the network said, to whom, and how it was received; queued deliveries are kept until they succeed, fail or expire.
errorCodes:
  - code: vtn/broadcasts/send:unknownRole
    meaning: "The network defines no role named `role`."
    retryable: false
    detailsSchema:
      type: object
      required: [role]
      properties:
        role:
          type: string
related:
  - vtn/broadcasts/list
  - vtn/broadcasts/show
  - vtn/outbox/list
  - vtn/outbox/retry
  - vtn/relay/deliver
  - vtn/groups/register
  - vtn/roles/list
  - auth/signing-key/enroll
---

## Abstract

A network reaches its members through groups. Each member registers, per role the network defines, a group that receives messages for that role ([`vtn/groups/register`](../../../groups/register/0.1/spec.md)). **VTN Broadcasts — Send** lets an administrator address a role. The network signs a message as its origin and delivers it to every current member's group for the role, each delivery carrying a hop the network signs. A group that belongs to a member network relays the message on to that network's own members' groups ([`vtn/relay/deliver`](../../../relay/deliver/0.1/spec.md)), and so on down the nesting. A copy that comes back round is dropped by message id, and a hop limit applies. Each network expands only its own groups, so the sender learns which groups accepted, counted through relays, and never who is in them. The chain of signed hops shows who relayed what to whom, so nesting does not flatten into one global claim (VTI-VTN-032, VTI-VTN-033).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **SHOULD** set `replyTo` when it wants answers, so that recipients reply to one place and not to the role.
2. **MUST NOT** put in `subject` or `body` anything that every group for the role, and every group they relay to, may not read.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** refuse with `vtn/broadcasts/send:unknownRole`, with `details.role`, when it defines no role named `role`.
3. **MUST** build the message: a document of type `VtnBroadcast` with a fresh `id`, `origin` set to the network's DID, `role`, `class` (`notice` when the request omits it), `subject`, `body`, `replyTo` when given, `sentAt`, and `expiresAt` set by the network's message lifetime, which **MUST NOT** exceed 7 days after `sentAt` and **SHOULD** default to 24 hours: a broadcast is a notice, not a record, and an undelivered one stops being retried when it expires. It **MUST** sign the message for `assertionMethod`, and **MUST** record its `id` as seen, so that a copy routed back to it is a duplicate.
4. **MUST** deliver the message to every group registered for `role` whose member's entry is `acknowledged`, and to no other. A suspended member's groups receive nothing (VTI-VTN-013).
5. **MUST** wrap each delivery in an envelope `{ message, hops }` whose last hop it signs, naming the message, the group, the role and the hop's index. When the delivery would exceed its hop limit, it **MUST NOT** deliver and **MUST** record the receipt `refused` with the detail `hop limit reached`.
6. **MUST** record a receipt per group: `delivered` when the group accepted; for a network group, the status and the relayed receipts its relay returned; `refused` when the group refused, which is final; `queued` when delivery failed in transport and the message has not expired, in which case it **MUST** put the delivery in its outbox for retry with backoff ([`vtn/outbox/retry`](../../../outbox/retry/0.1/spec.md)); and `failed` otherwise.
7. **MUST** store the broadcast record with direction `sent`, `actor` set to `issuer`, and `reached` set to the number of groups delivered to, counting relays. It **MUST** answer with the record as it stands. Queued deliveries update the stored record as they settle.
8. **MUST NOT** disclose to any group the membership of another group, and **MUST NOT** expand a member network's groups itself. Each network expands only its own.
9. **MUST NOT** change any member's own membership, trust contexts or keys (VTI-VTN-060 – VTI-VTN-063). A broadcast carries a message and grants nothing.

Each request is a new broadcast with a new message id: the same text sent twice is two broadcasts. A repeat of the same request document is caught by duplicate detection ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)).

## Authorization

The authority this task presupposes is **administrator standing on the network**. The `issuer` is one of the administrators named in the network's configuration, or a key that one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md). The network reads that standing when it executes the request (network item 1), so an administrator removed from the configuration, or a delegation revoked, loses the authority at once. The proof establishes who signed and that the document is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The match between the signer and the configured administrators is what entitles it to have the network speak. The message then carries the network's own signature, not the administrator's, and each relaying network signs only its own hop. No hop carries authority over the groups it reaches beyond delivery. A consumer **MAY** require an operation-bound step-up before it sends; that is its own policy.

## Definitions

- **`role`**: a role the network defines ([`RoleName`](../../../_shared/0.1/groups.schema.json)); the groups registered for it are the addressees.
- **`class`**: what kind of message it is (`notice`, `request-for-action`, `room-invite`, …), for the receiving group to sort on. Defaults to `notice`.
- **`subject`**, **`body`**: the message.
- **`replyTo`**: where replies go.
- **Message**: the signed `VtnBroadcast` document; its `id` is the broadcast's id.
- **Hop**: a signed `VtnRelayHop` document by which one network delivers the message to one group ([`Envelope`](../../../_shared/0.1/relay.schema.json)).
- **`broadcast`**: the [`BroadcastRecord`](../../../_shared/0.1/relay.schema.json), with one [`Receipt`](../../../_shared/0.1/relay.schema.json) per group.

## Request

An administrator (`issuer`) asks the network (`recipient`) to send a message to a role. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator sends a notice to every member's admins group

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b01",
  "type": "https://trusttasks.org/spec/vtn/broadcasts/send/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T16:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000bff",
  "payload": {
    "role": "admins",
    "subject": "Directory maintenance on Saturday",
    "body": "The directory is read-only from 02:00 to 04:00 UTC on Saturday.",
    "replyTo": "https://whn.example/rooms/ops"
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the broadcast record with its receipts as they stand. A refusal is a `trust-task-error`.

### Delivered to a community, relayed by a network, one delivery queued

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b02",
  "type": "https://trusttasks.org/spec/vtn/broadcasts/send/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T16:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000bff",
  "payload": {
    "broadcast": {
      "id": "urn:uuid:00000000-0000-4000-8000-0000000b0001",
      "direction": "sent",
      "role": "admins",
      "message": { "id": "urn:uuid:00000000-0000-4000-8000-0000000b0001" },
      "actor": "did:example:admin",
      "recordedAt": "2026-11-03T16:00:02Z",
      "receipts": [
        {
          "group": "did:example:community#admins",
          "member": "did:example:community",
          "status": "delivered",
          "network": false
        },
        {
          "group": "did:example:health-au#role-admins",
          "member": "did:example:health-au",
          "status": "delivered",
          "network": true,
          "relayed": [
            {
              "group": "did:example:clinic#admins",
              "member": "did:example:clinic",
              "status": "delivered",
              "network": false
            }
          ]
        },
        {
          "group": "did:example:hospital#admins",
          "member": "did:example:hospital",
          "status": "queued",
          "detail": "connection refused",
          "network": false
        }
      ],
      "reached": 3
    }
  }
}
```

The signed message is abbreviated to its `id`; the network returns it in full.

### The role is not defined

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T16:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000bff",
  "payload": {
    "code": "vtn/broadcasts/send:unknownRole",
    "message": "this network defines no role \"admin\"",
    "retryable": false,
    "details": { "role": "admin" }
  }
}
```

## Security & Privacy

### Data carried

The request carries a role, a subject, a body and an optional reply address. The response carries the signed message and per-group receipts, which name each group and the member that owns it. The message is delivered to organisations' groups and relayed onward, so its readers are everyone in every group for the role down the nesting. An administrator **MUST NOT** put personal data or anything confidential in `subject`, `body` or `ext`. `replyTo` should name a place, not a person.

### Correlation

The message carries the network's DID as `origin`, and each hop carries the relaying network's DID and the group it delivered to, so a recipient can see the path the message took. That is deliberate (VTI-VTN-032, VTI-VTN-033). The parties declare `identifierScope: public`. The network signs the message under the public DID that its members verify it against, and the administrator is recorded in the broadcast record under the identifier the network's configuration knows it by. The administrator's DID is not in the message, so recipients do not learn which administrator sent it. Receipts show the sender which groups and members accepted, but never the people in a group. A member network reports only its own groups' outcomes.

### Retention

The network keeps the broadcast record, including the signed message and receipts, so its administrators can see what the network said and how it was received; how long is the network's policy. Queued deliveries are kept in the outbox until they are delivered, refused, expire or reach the retry limit, and then removed. Message ids are kept as seen for at least the message lifetime, so that repeats are dropped.

### Consent/purpose

The message is collected to be delivered to the role's groups and is used for nothing else. Members agreed to receive messages for a role by registering a group for it. A member that no longer wants them can stop being addressed by leaving the role's group registration, or the network. Recipients use `replyTo` to answer, not the role.
