---
slug: vtn/outbox/retry
version: "0.1"
title: "VTN Outbox — Retry"
summary: "An administrator has the network retry every queued broadcast delivery now, without waiting for its backoff; the answer says how many were attempted."
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
    The proof is the administrator's authentication: a retry sends traffic to every queued member endpoint at once and spends attempts against the network's limit, so an unattributed request could be used to hammer members' endpoints or exhaust deliveries.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed retry spends another attempt on every queued delivery, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Attempts every queued delivery: each one is delivered, refused or failed again, the outbox is updated or emptied, and the broadcast records' receipts change. A delivered message cannot be recalled.
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: >-
    In: nothing. Out: a count of the deliveries attempted. The outcomes are read with vtn/broadcasts/show.
retention:
  class: transient
  rationale: >-
    The request carries nothing the network needs to keep once it has answered; the outcomes are kept in the broadcast records.
errorCodes: []
related:
  - vtn/outbox/list
  - vtn/broadcasts/send
  - vtn/broadcasts/show
  - auth/signing-key/enroll
---

## Abstract

The network retries queued broadcast deliveries on its own schedule, with exponential backoff ([`vtn/broadcasts/send`](../../../broadcasts/send/0.1/spec.md)). **VTN Outbox — Retry** lets an administrator retry them now, typically after a member reports that its endpoint is back. Every queued delivery is attempted regardless of when it was next due. Each outcome is recorded in the broadcast's receipt as the scheduled retry would record it, and the answer is the number attempted.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** take every item in its outbox, whether or not its `nextAttemptAt` has come.
3. **MUST** settle an item whose `expiresAt` has passed as `failed` with the detail `expired before delivery`, remove it from the outbox, and **MUST NOT** attempt or count it.
4. **MUST** otherwise deliver the item's envelope, unchanged, to its endpoint, and count it in `attempted`:
   - when the group accepts or refuses, **MUST** remove the item and record that status, with any relayed receipts, on the broadcast's receipt for the group;
   - when it fails in transport again, **MUST** increment `attempts` and record the error. When the network's attempt limit is reached, it **MUST** remove the item and record the receipt `failed`. Otherwise it **MUST** reschedule `nextAttemptAt` with backoff and leave the receipt `queued`.
5. **MUST** update each affected broadcast record's `reached`, and answer with `attempted`.
6. **MUST NOT** re-sign the message or its hops, and **MUST NOT** deliver to a group that is not in the outbox. A retry repeats a delivery already decided, and addresses nobody new.

Retrying an empty outbox succeeds with `attempted` `0`.

## Authorization

The authority this task presupposes is **administrator standing on the network**. The `issuer` is one of the administrators named in the network's configuration, or a key that one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md). The network reads that standing when it executes the request (network item 1). The proof establishes who signed and that the document is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the match against the configured administrators is what entitles it. The deliveries themselves were authorised when the broadcast was sent or relayed, and this task does not widen them (network item 6). A consumer **MAY** require an operation-bound step-up; that is its own policy.

## Definitions

- **`attempted`**: the number of queued deliveries this request attempted. Expired items, which were settled without an attempt, are not counted.

## Request

An administrator (`issuer`) asks the network (`recipient`) to retry its outbox. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json) and carries nothing.

### Retry now

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b41",
  "type": "https://trusttasks.org/spec/vtn/outbox/retry/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T16:10:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b4f",
  "payload": {}
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One delivery attempted

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000b42",
  "type": "https://trusttasks.org/spec/vtn/outbox/retry/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T16:10:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000b4f",
  "payload": {
    "attempted": 1
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing and the response carries a count. The deliveries the request causes carry the envelopes already queued, to the endpoints already queued, and nothing new.

### Correlation

The count reveals how many deliveries were waiting, and only to an administrator. Both parties declare `identifierScope: public`. The network answers under the public DID it signs messages with, and the administrator is recognised by the identifier the network's configuration names, which a pairwise identifier would not match. To the members' endpoints, a forced retry looks like any scheduled retry.

### Retention

The request is not kept after the answer, beyond what duplicate detection and the network's access log need. Outcomes are kept in the broadcast records under [`vtn/broadcasts/send`](../../../broadcasts/send/0.1/spec.md)'s policy.

### Consent/purpose

This task delivers messages that members' groups were already addressed with, for the purpose they were sent, and nothing else. A member that no longer wants a role's messages stops them through its group registration. A forced retry does not override that, because a refusal is final and is not queued.
