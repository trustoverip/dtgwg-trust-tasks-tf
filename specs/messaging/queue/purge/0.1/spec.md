---
slug: messaging/queue/purge
version: "0.1"
title: "Messaging — Purge Queue"
summary: "An account controller, or an administrator, removes the messages in one account queue — optionally only those exchanged with one counterparty or older than an age — with a dry run to preview the count first."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
parties:
  - role: Account controller or administrator
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    A purge irreversibly destroys undelivered messages, possibly in another party's account; the record of who
    ordered it must survive the transport and be attributable to the requester's key, not to a session. The response
    is RECOMMENDED so the requester can retain a signed statement of how many messages were removed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A purge is destructive, and a replayed request would destroy messages that arrived after the original one; the
    mediator can only reject a duplicate inside a bounded freshness window, which requires issuedAt.
sideEffects:
  level: destructive
  rationale: >-
    Removes queued messages permanently. Undelivered messages are gone — not returned to their senders, not
    recoverable by the mediator or the recipient.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns only counts and byte totals of the matched and removed messages; no identifiers of individual messages
    and no content.
retention:
  class: durable
  rationale: >-
    A non-dry-run purge is an irreversible deletion that may be disputed by the account's controller or its
    counterparties; the mediator keeps its audit record of who purged what.
errorCodes:
  - code: messaging/queue/purge:unknownAccount
    meaning: The named account does not exist at this mediator.
    retryable: false
  - code: messaging/queue/purge:rootAdminRequired
    meaning: The target account holds the admin, rootAdmin, or mediator role, and purging a privileged account's queue requires the requester to be a rootAdmin.
    retryable: false
related:
  - messaging/queue/status
  - messaging/queue/list
  - messaging/message/delete
  - messaging/message/list
---

## Abstract

The **Messaging — Purge Queue** Trust Task removes messages from one of an account's queues in a single operation. The selection is the whole queue, narrowed optionally to messages exchanged with one `peer` and to messages older than `olderThanSeconds`. A `dryRun` reports what would be removed and removes nothing.

It exists for recovery. Because a message is held against its sender's send queue until its recipient deletes it, one stalled recipient can fill a sender's queue to its limit, after which the sender cannot send at all. The account's own controller can clear the backlog addressed to that one peer; an administrator can do the same for an account that cannot help itself — offline, misconfigured, or wedged — without deleting and rebuilding the account. A mediator's own-queue purge endpoints deliberately cannot reach another account's queue, because that would hand a denial-of-service primitive to any local account; this task makes the cross-account form available only behind an administrative gate, and only behind a root-administrative gate for privileged accounts.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/queue/purge/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`. An interactive producer — an operator console, a CLI — **SHOULD** first send the same selection with `dryRun: true`, show the `matched` count to its user, and send the real purge only after the user confirms that count.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window or already seen within it.
2. Resolve the target account: `payload.did` when present, otherwise the requester's own account.
3. Where the target is the requester's own account, respond with `permissionDenied` unless the account holds the `local` capability — the same gate the mediator applies to its own-queue deletes.
4. Where the target is another account, respond with `permissionDenied` unless the requester's account type is `admin`, `rootAdmin`, or `mediator`.
5. Respond with `messaging/queue/purge:unknownAccount` where the target account does not exist. Rules 3–4 apply first, so a requester without standing learns nothing about another account's existence.
6. Where the target account's type is `admin`, `rootAdmin`, or `mediator` and is not the requester's own, respond with `messaging/queue/purge:rootAdminRequired` unless the requester is a `rootAdmin`.
7. Select the messages in the named `queue` of the target account, narrowed by `peer` (in a send queue, the recipient; in a receive queue, the sender) and by `olderThanSeconds` (time since the message was stored) when present.
8. On `dryRun: true`, remove **nothing** and return `matched`, `matchedBytes`, `purged: 0`, `dryRun: true`.
9. Otherwise remove every selected message, return the counts, and **MUST** record an audit entry naming the requester, the target account, the queue, the `peer` and `olderThanSeconds` filters if any, and the number of messages removed. A purge that removes nothing is still recorded.

A purge acts on the selection at the moment it executes. A message that arrives between a dry run and the real purge and matches the selection is removed too; `purged` may therefore exceed the dry run's `matched`, and a producer confirming a count **SHOULD** narrow with `olderThanSeconds` when that matters.

## Authorization

The entitlement is **control of the account, or administrative standing proportionate to the target**:

- The account's own controller may purge its own queues if the account holds the `local` capability (Consumer rule 3). An account that the mediator does not store messages for locally has nothing here to purge.
- An administrator (`admin`, `rootAdmin`, or `mediator`) may purge any `standard` account's queues (rule 4).
- Only a `rootAdmin` may purge the queues of another privileged account (rule 6), so an `admin` cannot silence another administrator or the mediator's own account.

The `proof` establishes *who* ordered the purge and makes the order non-repudiable; the authorization is the comparison between that identity, the target account, and the requester's role ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- `did` — the target account ([`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid)). Omitted = the requester's own account.
- `queue` — the [`Queue`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/Queue) to purge. Required: there is no "both queues" form, so one request never empties more than the requester named.
- `peer` — only messages exchanged with this counterparty.
- `olderThanSeconds` — only messages stored at least this long ago.
- `dryRun` — preview only. Default `false`.
- `matched`, `matchedBytes` — the size of the selection. `purged` — messages actually removed (`0` on a dry run).

## Request

The requester names the queue and any narrowing; see the top-level schema in [`payload.schema.json`](payload.schema.json).

### Previewing a purge of the backlog addressed to one stalled peer

```json
{
  "id": "urn:uuid:c1e8a3f4-5b7d-4c29-9e0a-6f2b8d1c4e01",
  "type": "https://trusttasks.org/spec/messaging/queue/purge/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T10:10:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "queue": "send",
    "peer": "did:web:carol.example",
    "olderThanSeconds": 3600,
    "dryRun": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-21T10:10:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3Wq..."
  }
}
```

After the operator confirms the count, the producer sends the same payload with `dryRun: false`.

## Response

The mediator responds with `https://trusttasks.org/spec/messaging/queue/purge/0.1#response`, whose payload validates against the `$anchor: "response"` sub-schema. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### The dry run's result

```json
{
  "id": "urn:uuid:c1e8a3f4-5b7d-4c29-9e0a-6f2b8d1c4e02",
  "type": "https://trusttasks.org/spec/messaging/queue/purge/0.1#response",
  "threadId": "urn:uuid:c1e8a3f4-5b7d-4c29-9e0a-6f2b8d1c4e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T10:10:00Z",
  "payload": {
    "matched": 912,
    "matchedBytes": 1771204,
    "purged": 0,
    "dryRun": true
  }
}
```

## Security & Privacy

### Data carried

The request carries an account identifier, a queue, and optionally a counterparty identifier and an age. The response carries counts and byte totals only — no message identifiers and no content. The counterparty named in `peer` is the one piece of relationship data the request moves; it is already known to the requester. A producer **MUST NOT** place message content or credentials in `ext`.

### Correlation

The request names the target account and, optionally, one of its counterparties; the mediator's audit record joins them to the requester. That record is the point of the task and is visible only to administrators. The response is sized by the purge and reveals nothing beyond the counts.

### Retention

A dry run is transient. A real purge is durable on the mediator's side: its audit entry **SHOULD** be kept for as long as the mediator keeps any audit record, because the deletion is irreversible and may be disputed by the account's controller or by a sender whose messages were never delivered. The request document, with its proof, is the strongest evidence of who ordered the purge and **MAY** be retained alongside the audit entry.

### Consent/purpose

The task exists to recover an account whose queue is blocking it. It is not a moderation or retention tool, and an operator should not reuse it to remove messages for reasons other than relieving back-pressure — a message removed here is lost to its intended recipient without notice.
