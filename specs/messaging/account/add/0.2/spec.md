---
slug: messaging/account/add
version: "0.2"
title: Messaging — Add Account
summary: An administrator registers a new served account at the mediator, declaring the account's DID and optional role, access-control capabilities, and queue limits.
status: draft
targetFrameworkVersion: "0.5.0"
category: messaging
keywords:
  - messaging
  - mediator
  - account
  - admin
  - provisioning
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
  requirement: REQUIRED
  rationale: Registering an account is an administrative mutation that provisions standing at the mediator; its record may be replayed by an auditor or relied on after the original transport has closed, so transport-independent integrity and non-repudiation of the change are required.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Adding an account attaches a mailbox the agent will send and receive on. Replayed, it re-attaches an account that was detached for cause.
sideEffects:
  level: mutating
  rationale: "Registers a new served account at the mediator; removable."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: messaging/account/add:alreadyExists
    meaning: The target DID already has an account at this mediator.
    retryable: false
related:
  - messaging/account/get
  - messaging/account/remove
  - messaging/account/update
---

## Abstract

The **Messaging — Add Account** Trust Task registers a new served account at the mediator. The *administrator* declares the target account's [`Vid`](../../../_shared/0.2/messaging.schema.json#/$defs/Vid) and, optionally, its [`AccountType`](../../../_shared/0.2/messaging.schema.json#/$defs/AccountType), its initial [`MediatorAcl`](../../../_shared/0.2/messaging.schema.json#/$defs/MediatorAcl) capabilities, and its [`QueueLimits`](../../../_shared/0.2/messaging.schema.json#/$defs/QueueLimits). The mediator creates the account and returns its realized [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) view.

When `accountType` is omitted the mediator provisions a `standard` account. Members of `acl` and `queueLimits` that are omitted take the mediator's configured defaults. The mediator applies its own authorization policy: assigning a non-standard `accountType` requires administrative standing, and only a `rootAdmin` may assign the `rootAdmin` role.

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

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/account/add/0.2`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.did` with the new account's DID and **MAY** include `payload.accountType`, `payload.acl`, and `payload.queueLimits`.
3. Include a `proof` member per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the mediator runs an explicit-allow policy, accept the request only from an `admin`/`rootAdmin` requester; respond with the framework's `permissionDenied` where the requester has no standing.
3. Where assigning a non-standard `accountType` (`admin`, `rootAdmin`, or `mediator`) without sufficient standing — only a `rootAdmin` may assign `rootAdmin` — respond with the framework's `permissionDenied`.
4. Where the target DID already has an account, respond with `messaging/account/add:alreadyExists`.
5. Create the account, applying `standard` for an omitted `accountType` and its configured defaults for omitted `acl` / `queueLimits` members, and return the full realized [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) in the response.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/account/add/0.2` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/messaging/account/add/0.2",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "did": "did:web:alice.example",
    "accountType": "standard",
    "acl": {
      "sendMessages": true,
      "receiveMessages": true
    },
    "queueLimits": {
      "sendQueueLimit": 1000,
      "receiveQueueLimit": 1000
    }
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

This example registers a `standard` account for Alice with messaging enabled and a thousand-message send/receive queue.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/account/add/0.2#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ account }`, the full realized mediator view of the newly created account.

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/messaging/account/add/0.2#response",
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
      "sendQueueCount": 0,
      "receiveQueueCount": 0,
      "accessListCount": 0
    }
  }
}
```

Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Adding an account is an administrative mutation that grants standing at the mediator: the **REQUIRED** `proof` makes the change non-repudiable and tamper-evident. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not that the requester may provision an account, nor that they may assign the requested role.

Assigning a privileged `accountType` is a privilege grant; a mediator **MUST** refuse to elevate beyond the requester's own standing and **MUST** restrict the `rootAdmin` role to assignment by an existing `rootAdmin`.

### Data carried

The request carries the new account's DID and, optionally, its role, initial capabilities, and queue limits. The response carries the realized [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) view: the same members as applied, plus the account's current queue and access-list counts. The mediator **SHOULD** return the full realized [`Account`](../../../_shared/0.2/messaging.schema.json#/$defs/Account) so the administrator can confirm the defaults the mediator applied.

The optional `ext` extension (see [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload. This specification imposes no further data-minimisation rule.

### Correlation

The account DID (`subjectPath: /did`) appears in both request and response and is the identifier every other `messaging/account/*` task accepts, so it joins this registration to every later read or change of the same account; `threadId` joins the request to its response. This specification imposes no further rule on correlation.

### Retention

The signed request is the evidence that the account was provisioned, and by whom, with what role; the proof rationale anticipates an auditor relying on it after the transport has closed. This specification sets no retention period for it.

### Consent/purpose

The data is supplied to provision a served account at the mediator. This specification places no further limit on its reuse.
