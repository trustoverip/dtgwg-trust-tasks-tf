---
slug: vtn/broadcasts/list
version: "0.1"
title: "VTN Broadcasts — List"
summary: "An administrator reads the broadcasts the network sent or relayed, newest first, with their receipts."
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
    The proof is the administrator's authentication: the records show which member groups accepted which messages, which the network shows only to its administrators.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read changes nothing, so a replay discloses only what the same administrator could read again; issuedAt lets the network bound how long it accepts a captured request.
sideEffects:
  level: none
  rationale: >-
    Reads the stored broadcast records and changes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: broadcast records — signed messages, the administrators who sent them, the networks that relayed them, and per-group receipts naming groups and the members that own them. In: at most a count.
retention:
  class: transient
  rationale: >-
    The request carries nothing the network needs to keep once it has answered.
errorCodes: []
related:
  - vtn/broadcasts/send
  - vtn/broadcasts/show
  - vtn/relay/deliver
  - auth/signing-key/enroll
---

## Abstract

**VTN Broadcasts — List** returns the broadcasts the network holds records for, newest first. It includes those it sent ([`vtn/broadcasts/send`](../../send/0.1/spec.md)), with the administrator who sent each, and those it received from a network it belongs to and relayed to its own groups ([`vtn/relay/deliver`](../../../relay/deliver/0.1/spec.md)), with the network that delivered each. Each record carries its receipts as they stand, including deliveries settled since by retry. [`vtn/broadcasts/show`](../../show/0.1/spec.md) reads one record by id.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** return its broadcast records, both `sent` and `received`, ordered by `recordedAt` with the newest first. When `limit` is present it **MUST** return at most that many, the newest.
3. **MUST** return each record's receipts as they currently stand.

This version takes a `limit` and no `cursor`. It returns the most recent records, which is what an administrator reviewing recent traffic needs. Paging through older records is left to a later version.

## Authorization

The authority this task presupposes is **administrator standing on the network**, read from the network's configuration when it executes the request (network item 1). A key that an administrator delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md) carries the same standing. The proof establishes who is asking ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the match against the configured administrators is what entitles them to read the records.

## Definitions

- **`limit`**: the most records to return. Absent: all of them.
- **`broadcasts`**: [`BroadcastRecord`](../../../_shared/0.1/relay.schema.json)s, newest first.

## Request

An administrator (`issuer`) asks the network (`recipient`) for its recent broadcasts. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The last ten broadcasts

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b11",
  "type": "https://trusttasks.org/spec/vtn/broadcasts/list/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T17:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b1f",
  "payload": {
    "limit": 10
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One relayed, one sent

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b12",
  "type": "https://trusttasks.org/spec/vtn/broadcasts/list/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T17:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b1f",
  "payload": {
    "broadcasts": [
      {
        "id": "urn:uuid:00000000-0000-4000-8000-0000000b0002",
        "direction": "received",
        "role": "admins",
        "message": { "id": "urn:uuid:00000000-0000-4000-8000-0000000b0002" },
        "from": "did:example:global-health",
        "recordedAt": "2026-11-03T16:30:00Z",
        "receipts": [
          {
            "group": "did:example:community#admins",
            "member": "did:example:community",
            "status": "delivered",
            "network": false
          }
        ],
        "reached": 1
      },
      {
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
          }
        ],
        "reached": 1
      }
    ]
  }
}
```

The signed messages are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries at most a count. The response carries broadcast records: the signed messages, the administrator who sent each sent message, the network that delivered each received one, and receipts that name groups and the members that own them. No record names a person in a group, because the network never learns who is in one. The response is for administrators. A network **MUST NOT** answer anyone else, because receipts show which members registered which groups and which accepted.

### Correlation

The records join messages to the groups and members they reached and to the administrators who sent them; that is their purpose. Both parties declare `identifierScope: public`. The network answers under the public DID it signs messages with, and the administrator is recognised by the identifier the network's configuration names. A pairwise administrator identifier would not match the configuration. Request timing shows when an administrator reviewed traffic, and nothing more.

### Retention

The request is not kept after the answer, beyond what duplicate detection and the network's access log need. The records themselves are retained under [`vtn/broadcasts/send`](../../send/0.1/spec.md)'s policy, not this task's.

### Consent/purpose

The records are disclosed to administrators so they can see what the network said and how it was received, and for nothing else. An administrator **SHOULD NOT** pass receipts outside the network's governance, because they describe members' group registrations.
