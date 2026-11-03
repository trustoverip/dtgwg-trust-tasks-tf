---
slug: vtn/broadcasts/show
version: "0.1"
title: "VTN Broadcasts — Show"
summary: "An administrator reads one broadcast record by message id, with its receipts as they stand."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, broadcast, relay, receipts]
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
    The proof is the administrator's authentication: a record shows which member groups accepted a message, which the network shows only to its administrators.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read changes nothing, so a replay discloses only what the same administrator could read again; issuedAt lets the network bound how long it accepts a captured request.
sideEffects:
  level: none
  rationale: >-
    Reads one stored broadcast record and changes nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a message id. Out: the broadcast record — the signed message, who sent or delivered it, and per-group receipts naming groups and the members that own them.
retention:
  class: transient
  rationale: >-
    The request carries nothing the network needs to keep once it has answered.
errorCodes: []
related:
  - vtn/broadcasts/list
  - vtn/broadcasts/send
  - vtn/outbox/retry
  - auth/signing-key/enroll
---

## Abstract

**VTN Broadcasts — Show** returns one broadcast record, either sent ([`vtn/broadcasts/send`](../../send/0.1/spec.md)) or received and relayed, by its message id. Its receipts are as they currently stand, so an administrator can follow queued deliveries as retries settle them ([`vtn/outbox/retry`](../../../outbox/retry/0.1/spec.md)). [`vtn/broadcasts/list`](../../list/0.1/spec.md) enumerates the records.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** refuse with `notFound` when it holds no broadcast record for `id`.
3. **MUST** otherwise return the record with its receipts as they currently stand.

## Authorization

The authority this task presupposes is **administrator standing on the network**, read from the network's configuration when it executes the request (network item 1). A key that an administrator delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md) carries the same standing. The proof establishes who is asking ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the match against the configured administrators is what entitles them to read the record.

## Definitions

- **`id`**: the broadcast's message id, the `id` of the signed message.
- **`broadcast`**: the [`BroadcastRecord`](../../../_shared/0.1/relay.schema.json).

## Request

An administrator (`issuer`) asks the network (`recipient`) for one record. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Follow up a broadcast with a queued delivery

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b21",
  "type": "https://trusttasks.org/spec/vtn/broadcasts/show/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T17:10:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b2f",
  "payload": {
    "id": "urn:uuid:00000000-0000-4000-8000-0000000b0001"
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The queued delivery has since been delivered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b22",
  "type": "https://trusttasks.org/spec/vtn/broadcasts/show/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T17:10:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b2f",
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
          "group": "did:example:hospital#admins",
          "member": "did:example:hospital",
          "status": "delivered",
          "network": false
        }
      ],
      "reached": 2
    }
  }
}
```

The signed message is abbreviated to its `id`; the network returns it in full.

## Security & Privacy

### Data carried

The request carries a message id. The response carries one broadcast record: the signed message, the administrator who sent it or the network that delivered it, and receipts naming groups and the members that own them. No record names a person in a group. A network **MUST NOT** answer anyone but its administrators.

### Correlation

The record joins a message to the groups and members it reached, which is its purpose. Both parties declare `identifierScope: public`. The network answers under the public DID it signs messages with, and the administrator is recognised by the identifier the network's configuration names, which a pairwise identifier would not match. A refusal with `notFound` tells only an administrator, who could list the records anyway, that an id is unknown.

### Retention

The request is not kept after the answer, beyond what duplicate detection and the network's access log need. The record is retained under [`vtn/broadcasts/send`](../../send/0.1/spec.md)'s policy.

### Consent/purpose

The record is disclosed to administrators so they can see how a message was received, and for nothing else. An administrator **SHOULD NOT** pass receipts outside the network's governance, because they describe members' group registrations.
