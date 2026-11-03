---
slug: vtn/outbox/list
version: "0.1"
title: "VTN Outbox — List"
summary: "An administrator reads the broadcast deliveries waiting to be retried, with their attempts, next attempt, expiry and last error."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, broadcast, relay, retry, delivery]
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
    The proof is the administrator's authentication: the outbox holds members' group endpoints and the envelopes in flight to them, which the network shows only to its administrators.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read changes nothing, so a replay discloses only what the same administrator could read again; issuedAt lets the network bound how long it accepts a captured request.
sideEffects:
  level: none
  rationale: >-
    Reads the outbox and changes nothing; it neither attempts nor reschedules a delivery.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: queued deliveries — message id, group, the member that owns it, the group's delivery endpoint, the signed envelope, and the retry state. In: nothing.
retention:
  class: transient
  rationale: >-
    The request carries nothing the network needs to keep once it has answered.
errorCodes: []
related:
  - vtn/outbox/retry
  - vtn/broadcasts/send
  - vtn/broadcasts/show
  - auth/signing-key/enroll
---

## Abstract

When a broadcast delivery fails in transport, the network queues it in its outbox and retries it with backoff until it is delivered, refused, expires, or reaches the network's attempt limit ([`vtn/broadcasts/send`](../../../broadcasts/send/0.1/spec.md)). **VTN Outbox — List** shows an administrator what is waiting: each delivery's group and member, where it is going, how often it has been tried, when it will be tried next, when it will be given up, and why the last attempt failed. [`vtn/outbox/retry`](../../retry/0.1/spec.md) tries them all now.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** return every delivery in its outbox: one item per message and group that is queued, including items past their `nextAttemptAt` that have not yet been retried.
3. **MUST NOT** attempt, reschedule or remove a delivery while answering.

The outbox has no read-one sibling. An item is a short-lived piece of work keyed by message and group, not a record whose existence a caller acts on. The lasting outcome of each delivery is the receipt in the broadcast record, which [`vtn/broadcasts/show`](../../../broadcasts/show/0.1/spec.md) reads by message id. The queue is bounded by the attempt limit and message lifetimes, so it is returned whole, without paging.

## Authorization

The authority this task presupposes is **administrator standing on the network**, read from the network's configuration when it executes the request (network item 1). A key that an administrator delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md) carries the same standing. The proof establishes who is asking ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the match against the configured administrators is what entitles them to read the outbox.

## Definitions

- **`items`**: the queued deliveries ([`OutboxItem`](../../../_shared/0.1/relay.schema.json)). `attempts` counts the tries so far, the first included. `nextAttemptAt` is when the network will try again on its own. `expiresAt` is the message's expiry, after which the delivery is given up. `lastError` is why the last try failed.

## Request

An administrator (`issuer`) asks the network (`recipient`) for its outbox. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json) and carries nothing.

### Read the outbox

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b31",
  "type": "https://trusttasks.org/spec/vtn/outbox/list/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T16:05:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b3f",
  "payload": {}
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One delivery waiting

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b32",
  "type": "https://trusttasks.org/spec/vtn/outbox/list/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T16:05:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b3f",
  "payload": {
    "items": [
      {
        "messageId": "urn:uuid:00000000-0000-4000-8000-0000000b0001",
        "group": "did:example:hospital#admins",
        "member": "did:example:hospital",
        "endpoint": "https://hospital.example/inbox/admins",
        "envelope": {
          "message": { "id": "urn:uuid:00000000-0000-4000-8000-0000000b0001" },
          "hops": [
            { "id": "urn:uuid:00000000-0000-4000-8000-0000000b0101" }
          ]
        },
        "attempts": 2,
        "nextAttemptAt": "2026-11-03T16:06:02Z",
        "expiresAt": "2026-11-04T16:00:02Z",
        "lastError": "connection refused"
      }
    ]
  }
}
```

The signed message and hop are abbreviated to an `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries nothing. The response carries, per queued delivery, a member's group and its delivery endpoint, the signed envelope with the message text, and transport errors. Endpoints are members' delivery addresses, which the network learned when they registered their groups and does not publish. The response is for administrators only, and a network **MUST NOT** answer anyone else.

### Correlation

Items join a message to the member groups it has not yet reached and to their endpoints. That is their purpose, and it is visible only to administrators. Both parties declare `identifierScope: public`. The network answers under the public DID it signs messages with, and the administrator is recognised by the identifier the network's configuration names, which a pairwise identifier would not match. `lastError` can reveal a member's infrastructure problems, such as an endpoint that refuses connections, which is a further reason to keep the response to administrators.

### Retention

The request is not kept after the answer, beyond what duplicate detection and the network's access log need. Outbox items live only until they settle, under [`vtn/broadcasts/send`](../../../broadcasts/send/0.1/spec.md)'s policy.

### Consent/purpose

The outbox is disclosed to administrators so they can see and fix deliveries that are not getting through, and for nothing else. Members' endpoints were given to the network for delivery and **SHOULD NOT** be used or passed on for anything else.
