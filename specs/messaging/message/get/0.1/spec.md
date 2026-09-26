---
slug: messaging/message/get
version: "0.1"
title: "Messaging — Get Message"
summary: "Fetch one stored message from a mediator queue, verbatim and undecrypted, together with its metadata — an account's own message, or, for a rootAdmin, any account's."
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
parties:
  - role: Requester (the account owner, or a rootAdmin)
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request releases a stored message — for a rootAdmin, one addressed to another account — so the record of who asked must survive the transport and be attributable after the fact; a cross-account read is audited against this proof. The response carries secret-class material (SPEC §7.3 item 8), so it too is signed, binding the released bytes to the mediator that released them.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task discloses secret-class material, which makes it consequential (SPEC §7.3 item 17); a replayed request must fall outside a bounded acceptance window rather than release the same message again to whoever captured it.
sideEffects:
  level: none
  rationale: >-
    A read. Retrieval does not delete the message, does not change its delivery state, and is not a pickup.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The response carries the stored message verbatim. Most messages are encrypted to their recipient, but a DIDComm signed-only or plaintext message is readable without the recipient's key, and a TSP envelope exposes its sender and receiver identifiers and payload framing — so the body must be treated as potentially confidential content, not as metadata.
retention:
  class: transient
  rationale: >-
    The mediator keeps nothing of the request beyond its audit record of a cross-account read; the requester decides how long to hold the message it received.
errorCodes:
  - code: messaging/message/get:unknownAccount
    meaning: The named account (`did`) has no account at this mediator.
    retryable: false
  - code: messaging/message/get:unknownMessage
    meaning: No message with this `msgId` is held in the named account's queues — it never existed, was deleted, or expired.
    retryable: false
  - code: messaging/message/get:rootAdminRequired
    meaning: The request names another account's message, and the requester is not a rootAdmin. An admin may see another account's message metadata through messaging/message/list, but not its body.
    retryable: false
  - code: messaging/message/get:messageTooLarge
    meaning: The stored message exceeds the 10 MiB this task can carry. It is refused rather than truncated; it remains in the queue and can still be listed or deleted.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [size]
      properties:
        size: { type: integer, minimum: 0 }
related:
  - messaging/message/list
  - messaging/message/delete
  - messaging/queue/status
---

## Abstract

The **Messaging — Get Message** Trust Task fetches one stored message from a mediator queue: its [`MessageMeta`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/MessageMeta) and the stored message **exactly as the mediator holds it** — a DIDComm JWE/JWS as JSON text, or a TSP message as CESR qb64 text. The mediator never decrypts. A client that holds the recipient's key-agreement secret may unpack the message locally, which makes this the troubleshooting path for "what is actually sitting in my queue". The plaintext never travels back to the mediator.

This is the read-one half of the split pair with [`messaging/message/list`](../../list/0.1/spec.md). `list` enumerates metadata. `get` returns one message and answers `unknownMessage` definitively when it is absent.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the requester) **MUST**:

1. Emit a document whose `type` is `https://trusttasks.org/spec/messaging/message/get/0.1`, with itself as `issuer`, the mediator as `recipient`, an `issuedAt`, and a `proof`.
2. Omit `did` to fetch from its own account, or set it to the account whose queue holds the message.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Resolve the target account: `did` when present, otherwise the requester. Where it has no account, respond `messaging/message/get:unknownAccount`.
3. Apply the [Authorization](#authorization) rules below before looking the message up, so that an unauthorized requester cannot learn whether a given `msgId` exists.
4. Where no message with `msgId` is held in either of the target account's queues, respond `messaging/message/get:unknownMessage`.
5. Where the stored message exceeds the schema's 10 MiB bound on `message`, respond `messaging/message/get:messageTooLarge` with its size; never truncate. Otherwise return the stored message byte-for-byte as held, without decrypting, re-encoding, or unwrapping it, together with its metadata.
6. **Not** change the message in any way: not delete it, not mark it `delivered`, not reset its expiry. A get is not a pickup.
7. Record an audit entry naming the requester, the target account, and the `msgId` whenever the target account is not the requester's own.

## Authorization

The authority is **ownership of the queue**, with one privileged exception.

- **Own message.** An account may fetch any message held in its own queues. The mediator **SHOULD** apply the same capability gate it applies to that account's own pickup. For a mediator that exposes the `local` capability, this means the account must hold it. A requester without it receives `permissionDenied`.
- **Another account's message.** Only a `rootAdmin` may fetch one (`rootAdminRequired`). An `admin` can already see who sent what to whom, when, and at what size through [`messaging/message/list`](../../list/0.1/spec.md). Releasing the body is a larger disclosure, so it is reserved to the highest role. A standard account naming another account receives `rootAdminRequired` too: the answer is the same whether the requester is an admin or not.

The `proof` establishes **who** asked, not that they may. The mediator derives the requester's role from its own account records, never from the document. The proof is what makes the audit record of a cross-account read non-repudiable.

## Definitions

- **`did`** — OPTIONAL. The account whose queue holds the message, as a [`Vid`](../../../_shared/0.1/messaging.schema.json#/$defs/Vid) in the form the mediator uses for accounts (a DID or a stable hash of one). Omitted means the requester's own account.
- **`msgId`** — REQUIRED. The mediator's identifier for the stored message, as returned in `MessageMeta.msgId` by `messaging/message/list` or in a `messaging/monitor/event`.
- **`meta`** (response) — the message's [`MessageMeta`](../../../_shared/0.1/mediator-ops.schema.json#/$defs/MessageMeta): queue, size, timestamps, counterparties where known, detected protocol, and delivery state.
- **`message`** (response) — the stored message as text, exactly as held. A client identifies the encoding from `meta.protocol`.

## Request

The requester sends the request to its mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A rootAdmin fetches a message stuck in another account's receive queue

```json
{
  "id": "urn:uuid:7c1f0e52-3d0a-4b8e-9a51-2f1c6e7d9a01",
  "type": "https://trusttasks.org/spec/messaging/message/get/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-09-21T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "msgId": "1726912345000-0"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-21T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg..."
  }
}
```

## Response

The mediator responds with the message and its metadata, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### The stored DIDComm envelope, undecrypted

```json
{
  "id": "urn:uuid:7c1f0e52-3d0a-4b8e-9a51-2f1c6e7d9a02",
  "type": "https://trusttasks.org/spec/messaging/message/get/0.1#response",
  "threadId": "urn:uuid:7c1f0e52-3d0a-4b8e-9a51-2f1c6e7d9a01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-21T10:00:01Z",
  "payload": {
    "meta": {
      "msgId": "1726912345000-0",
      "queue": "receive",
      "size": 1843,
      "receivedAt": "2026-09-21T09:12:25Z",
      "expiresAt": "2026-09-28T09:12:25Z",
      "to": "did:web:alice.example",
      "protocol": "didcomm",
      "deliveryState": "delivered",
      "deliveredAt": "2026-09-21T09:12:26Z"
    },
    "message": "{\"protected\":\"eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIn0\",\"recipients\":[{\"header\":{\"kid\":\"did:web:alice.example#key-2\"},\"encrypted_key\":\"...\"}],\"iv\":\"...\",\"ciphertext\":\"...\",\"tag\":\"...\"}"
  }
}
```

`deliveryState: delivered` with an old `deliveredAt` is the classic stuck-message signature: the recipient picked the message up but never deleted it. The `from` member is absent because the envelope is anonymous to the mediator.

## Security & Privacy

### Data carried

The request carries an account identifier and a message handle. The response carries the stored message itself, which is why this task declares `discloses: secret`. An authcrypted DIDComm message is opaque without the recipient's key, but a signed-only or plaintext DIDComm message is readable by whoever holds the response, and a TSP envelope names its sender and receiver in the clear. The mediator **MUST NOT** add anything to the stored message, and **MUST NOT** decrypt it even where it could, such as a message addressed to the mediator itself. A client that decrypts a message it fetched **MUST NOT** send the plaintext back to the mediator.

### Correlation

`meta` joins a message to its counterparties, size and timing. Combined with `messaging/message/list` and `messaging/monitor/event`, `msgId` is a stable handle across all three. This is unavoidable, since `msgId` is the handle the task exists to resolve. The ciphertext of an encrypted message adds nothing a mediator operator could not already see in the store.

### Retention

The mediator retains the audit record of a cross-account read for as long as it retains its other privileged-change audit entries. It retains nothing for an own-account read. The requester **SHOULD** hold a fetched message only as long as the troubleshooting session that needed it, since the mediator copy remains authoritative until it is deleted.

### Consent/purpose

The purpose is operational: diagnosing why a message is stuck, malformed, or undeliverable, and letting an account inspect its own queue. A rootAdmin's access to another account's message is an operator capability for that purpose. Reusing fetched messages for anything else is outside this task's purpose. Whether a cross-account read needs additional approval is the mediator operator's policy, not this specification's.
