---
slug: messaging/acl/get
version: "0.1"
title: Messaging — Get ACL
summary: An administrator reads the mediator's access-control capability flags for one or more served accounts, retrieving the full realized MediatorAcl capability set for each known account in a single batch query.
status: draft
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - acl
  - access-control
  - admin
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
  rationale: A query is a read-only operation, not an evidentiary mutation; over an authenticating transport the in-band proof adds nothing to integrity or non-repudiation, so the proof is RECOMMENDED rather than REQUIRED.
sideEffects:
  level: none
  rationale: "Read-only read of an account's mediator capability flags."
exposure:
  discloses: metadata
  actsAsSubject: false
related:
  - messaging/account/update
  - messaging/account/get
  - messaging/access-list/list
---

## Abstract

The **Messaging — Get ACL** Trust Task reads the mediator's per-account access-control capabilities for one or more served accounts. The *administrator* names the accounts to query in `payload.dids`, and the mediator returns, for each known account, the full realized [`MediatorAcl`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) capability set it currently holds. This is the read counterpart of the `acl` member of [`messaging/account/update`](../../../account/update/0.1/spec.md): `update` declares a partial update of capabilities; `get` retrieves the complete realized set.

The query is a **batch** read: any number of accounts may be requested in one document. A queried DID that has no account at the mediator is simply omitted from the response `entries`; the response **MAY** additionally list such DIDs in `unknown` so the administrator can distinguish an omitted account from a transmission loss. The mediator applies its own authorization policy — an ACL discloses an account's privileges, so the mediator **MUST** authorize the reader before returning any entry.

The mediator's internal capability representation (a packed flag set) is opaque to the framework; this task carries the capabilities as the named-boolean [`MediatorAcl`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl) object so the document is human-meaningful and transport-agnostic.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/acl/get/0.1`, with itself as `issuer` and the mediator as `recipient`.
2. Populate `payload.dids` with one or more target account DIDs to query.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof) where the transport does not already authenticate the request (proof is **RECOMMENDED**).

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof` where present.
2. Authorize the reader before disclosing any account's ACL; where the requester has no standing to read a queried account, respond with the framework's `permissionDenied`.
3. Return one `entries` member per queried DID that has an account at the mediator, each carrying the full realized [`MediatorAcl`](../../../_shared/0.1/messaging.schema.json#/$defs/MediatorAcl).
4. Omit from `entries` any queried DID that has no account at the mediator, and **MAY** list such DIDs in `unknown`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/messaging/acl/get/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "6a3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f4",
  "type": "https://trusttasks.org/spec/messaging/acl/get/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-06-22T10:00:00Z",
  "payload": {
    "dids": [
      "did:web:alice.example",
      "did:web:bob.example"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-22T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

This example queries the ACLs of two accounts, Alice and Bob, in a single request.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/messaging/acl/get/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is `{ entries }`, where each entry is `{ did, acl }` and `acl` is the full realized capability set the mediator holds for the account. Queried DIDs with no account are omitted from `entries` and **MAY** be listed in `unknown`.

```json
{
  "id": "7a3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f5",
  "type": "https://trusttasks.org/spec/messaging/acl/get/0.1#response",
  "threadId": "6a3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f4",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-22T10:00:01Z",
  "payload": {
    "entries": [
      {
        "did": "did:web:alice.example",
        "acl": {
          "accessListMode": "explicitDeny",
          "blocked": false,
          "local": true,
          "sendMessages": true,
          "receiveMessages": true,
          "sendForwarded": true,
          "receiveForwarded": true,
          "createInvites": false,
          "anonReceive": false,
          "selfManageList": false,
          "selfManageSendQueueLimit": false,
          "selfManageReceiveQueueLimit": false,
          "didcommEnabled": true,
          "tspEnabled": false
        }
      }
    ],
    "unknown": [
      "did:web:bob.example"
    ]
  }
}
```

In this example Alice's realized ACL is returned, while Bob has no account at the mediator and is therefore omitted from `entries` and listed in `unknown`.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

A get-ACL document is a read-only query: the **RECOMMENDED** `proof` authenticates the requester where the transport does not, but introduces no mutation to be made non-repudiable. A mediator **MUST** enforce its own authorization independent of the document — `proof` establishes who asked, not who is entitled to read.

An ACL is **capability-disclosing**: it reveals which privileges an account holds at the mediator (what it may send, receive, forward, self-manage, and whether it is blocked). Returning an account's ACL to an unauthorized reader leaks that account's standing, so a mediator **MUST** authorize the reader against each queried account and **MUST NOT** return an entry the reader is not entitled to see. Omitting an entry for a queried account **SHOULD NOT** by itself confirm or deny the account's existence to a reader lacking standing; the optional `unknown` list is intended for authorized administrators.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload where a `proof` is present.
