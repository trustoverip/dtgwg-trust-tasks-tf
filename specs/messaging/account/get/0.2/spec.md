---
slug: messaging/account/get
version: "0.2"
title: Messaging — Get Account
summary: A requester fetches the mediator's view of one served account — its role, access-control capabilities, queue limits, and current queue state.
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
keywords:
  - messaging
  - mediator
  - account
  - admin
  - read
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Reading an account is a non-mutating query; a proof is RECOMMENDED to bind the request to its requester for authorization and audit, but is not required for integrity of any change since none is made.
sideEffects:
  level: none
  rationale: "Read-only read of one served account."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: messaging/account/get:unknownAccount
    meaning: The target DID has no account at this mediator.
    retryable: false
related:
  - messaging/account/add
  - messaging/account/list
  - messaging/acl/get
---

## Abstract

The **Messaging — Get Account** Trust Task fetches the mediator's view of a single served account. The requester names the target [`Vid`](../../../_shared/0.2/messaging.schema.json#/$defs/Vid); the mediator returns the full [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) — its [`AccountType`](../../../_shared/0.2/messaging.schema.json#/$defs/AccountType), its realized [`MediatorAcl`](../../../_shared/0.2/messaging.schema.json#/$defs/MediatorAcl) capabilities, its [`QueueLimits`](../../../_shared/0.2/messaging.schema.json#/$defs/QueueLimits), and its current queue state.

The task is read-only; it makes no change to the account. The mediator applies its own authorization policy — typically an administrator may read any account and a `standard` account may read its own.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.1

The two activity instants on [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) — the `account` returned in the response — change type; their names do not:

- **`lastReceivedAt`**: integer Unix seconds → RFC 3339 string (`type: "string"`, `format: "date-time"`), e.g. `"2026-10-23T09:30:00Z"`.
- **`lastAuthenticatedAt`**: integer Unix seconds → RFC 3339 string (`type: "string"`, `format: "date-time"`).

Both remain optional, and present only under the same conditions as in `0.1`. The registry-wide convention is that an instant is an RFC 3339 string whose member name ends in `At`; these names already did, so only the type moves.

The change arrives through the shared component: this version re-pins its `$ref`s from `messaging/_shared/0.1` to `messaging/_shared/0.2`, which is why it is a new version at all ([SPEC.md §5.4](/SPEC.md#54-migrating-between-versions) couples the two). Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published and pinned to `messaging/_shared/0.1`.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the requester) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/account/get/0.2`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the target account's DID — or, for privacy (and for mediators that key accounts by a one-way hash and never hold the full DID), a stable hash of that DID. Either form is a valid [`Vid`](../../../_shared/0.2/messaging.schema.json#/$defs/Vid); the same form is used across `account/*`, `acl/*`, and `access-list/*` and compared by exact string equality.
3. **SHOULD** include a `proof` member per [SPEC.md §4.7](/SPEC.md#47-proof).
4. Set `payload.includeActivity` to `true` only to receive the activity timestamps (`lastReceivedAt`, `lastAuthenticatedAt`). A mediator that predates this member rejects the request as a schema violation, so a requester **SHOULD** set it only where it knows the mediator supports it.
5. Set `payload.includeStats` to `true` only to receive the account's lifetime counters (`stats`). A mediator that predates this member rejects the request as a schema violation, so a requester **SHOULD** set it only where it knows the mediator supports it.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and, where a `proof` is present, verify it.
2. Enforce its own authorization policy and respond with the framework's `permissionDenied` where the requester may not read the target account.
3. Where the target DID has no account, respond with `messaging/account/get:unknownAccount`.
4. Return the full [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) view in the response.
5. Include `lastReceivedAt` and `lastAuthenticatedAt` in the returned account when, and only when, `payload.includeActivity` is `true`, and then only the timestamps it has recorded. It **MUST NOT** include them otherwise: a requester that did not ask may validate the response against a schema without them.
6. Include `stats` in the returned account when, and only when, `payload.includeStats` is `true`, and then only the counters it keeps. It **MUST NOT** include them otherwise: a requester that did not ask may validate the response against a schema without them.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/account/get/0.2` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/account/get/0.2",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-22T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg..."
  }
}
```

This example fetches the mediator's view of Alice's account.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/account/get/0.2#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ account }`, the full mediator view of the requested account.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/account/get/0.2#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "account": {
      "did": "did:web:alice.example",
      "accountType": "standard",
      "acl": {
        "accessListMode": "explicitDeny",
        "blocked": false,
        "local": true,
        "sendMessages": true,
        "receiveMessages": true,
        "sendForwarded": false,
        "receiveForwarded": true,
        "createInvites": false,
        "anonReceive": false,
        "selfManageList": false,
        "selfManageSendQueueLimit": false,
        "selfManageReceiveQueueLimit": false,
        "didcommEnabled": true,
        "tspEnabled": true
      },
      "queueLimits": {
        "sendQueueLimit": 1000,
        "receiveQueueLimit": 1000
      },
      "sendQueueCount": 3,
      "sendQueueBytes": 8192,
      "receiveQueueCount": 0,
      "receiveQueueBytes": 0,
      "accessListCount": 2
    }
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

A mediator **MUST** enforce its own authorization independent of the document, returning the framework's `permissionDenied` where the requester may not read the target account, even though `proof` is only **RECOMMENDED** for this read-only task. A `proof`, when present, binds the request to its requester for authorization and audit.

### Data carried

The request carries the target account's DID and two opt-in flags. An account view discloses the account's role, capabilities, limits, and current queue depth — operational metadata about the served party. The queue-state counts it returns are point-in-time and **MAY** be stale by the time the response is read.

The activity timestamps (`lastReceivedAt`, `lastAuthenticatedAt`) are returned only when `includeActivity` is set, and the lifetime counters (`stats`) only when `includeStats` is set. The lifetime counters are operational metadata about the served party: how much it has sent and received, and over which protocols. They disclose volume rather than content, and a mediator **SHOULD** apply the same authorization to them as to the rest of the account view.

The optional `ext` extension (see [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload.

### Correlation

The activity timestamps disclose when a party last received traffic and last connected — a behavioural signal beyond the account's configuration. A mediator **SHOULD** apply the same authorization to them as to the rest of the account view, and **MAY** record `lastReceivedAt` coarsely (it may lag by up to 60 seconds) so recording it does not add a write to every message.

The account DID (`subjectPath: /did`) is the identifier every other `messaging/account/*` task accepts, so repeated reads of the same account join to each other and to its changes; `threadId` joins the request to its response.

### Retention

The response is a point-in-time snapshot of operational state. This specification sets no retention rule for it.

### Consent/purpose

The data is returned so an authorized requester can inspect one served account. This specification places no further limit on its reuse.
