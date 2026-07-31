---
slug: keys/create
version: "0.1"
title: Keys — Create
summary: A producer asks a key custodian to generate a new key and hold it, receiving only the public half.
status: draft
targetFrameworkVersion: "0.1"
category: key-management
keywords:
  - keys
  - create
  - generate
  - custody
  - derivation
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer
    requirement: REQUIRED
    member: issuer
  - role: Key custodian
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Creating a key establishes material the custodian will later sign with on request, so the request has to be attributable to a party authorized to add to the custodian's key set.
sideEffects:
  level: mutating
  rationale: "Generates and stores a new key record."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
related:
  - keys/import
  - keys/show
  - keys/list
  - keys/revoke
---

## Abstract

The **Keys — Create** Trust Task asks a *key custodian* to generate a key and keep it. The producer receives the public half and an identifier; the private half never leaves the custodian, which is the entire reason to create a key this way rather than generating one locally and importing it.

Where the custodian derives from a seed it holds, a key created with an explicit `derivationPath` is **reproducible**: restoring the seed reconstitutes the key. This is the property that distinguishes create from [`keys/import`](../../import/0.1/spec.md), whose result exists only as stored material.

The optional `mnemonic` member sits between the two. Supplying a BIP-39 phrase asks the custodian to derive from *that* seed rather than its own — an import of externally-generated seed material wearing create's clothes. It carries import's confidentiality problem with it, and the conformance rules treat it accordingly.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/create/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Populate `payload.keyType` with the algorithm to generate.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Establish the producer's authority to add keys, refusing with `permission_denied` ([SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)) otherwise.
2a. **Refuse `mnemonic` on any transport that is not end-to-end confidential**, and never log or echo it. A BIP-39 phrase reconstitutes the key anywhere, so it is secret-bearing in exactly the way the rest of this payload is not — the same reasoning that makes [`keys/import`](../../import/0.1/spec.md) refuse its cleartext carrier.
3. Refuse, with `already_exists`, a request that would collide with an existing key record rather than replacing it.
4. Return the realized record — including `publicKey` and the assigned `keyId` — under the `#response` variant, with `origin: "derived"`.
5. **Not** return the private key, or any encoding of it.

A custodian that derives from a seed **SHOULD** record `derivationPath` and `seedId` on the resulting record, so an operator can later tell which keys a seed restore would bring back.

## Definitions

* **Producer.** The party requesting the key; identified by `issuer`.
* **Key custodian.** The party generating and holding it; identified by `recipient`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/create/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "1b2c3d4e-5f60-4718-8293-a4b5c6d7e8f9",
  "type": "https://trusttasks.org/spec/keys/create/0.1",
  "issuer": "did:web:app.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:20:00Z",
  "payload": {
    "keyType": "ed25519",
    "derivationPath": "m/26'/2'/0'/1'",
    "label": "app signing key",
    "contextId": "app"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/create/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "2c3d4e5f-6071-4829-93a4-b5c6d7e8f901",
  "type": "https://trusttasks.org/spec/keys/create/0.1#response",
  "threadId": "1b2c3d4e-5f60-4718-8293-a4b5c6d7e8f9",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:app.example",
  "issuedAt": "2026-07-31T09:20:01Z",
  "payload": {
    "key": {
      "keyId": "app-signing-key",
      "keyType": "ed25519",
      "status": "active",
      "publicKey": "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "derivationPath": "m/26'/2'/0'/1'",
      "seedId": 1,
      "origin": "derived",
      "label": "app signing key",
      "contextId": "app",
      "createdAt": "2026-07-31T09:20:01Z",
      "updatedAt": "2026-07-31T09:20:01Z"
    }
  }
}
```

Failures (`permission_denied`, `already_exists`, `invalid_argument`) use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Creating a key is granting future signing capacity: whoever may later name this key in [`keys/sign`](../../sign/0.1/spec.md) can sign with it. Custodians **SHOULD** therefore treat "who may create keys, and in which scope" as an authorization decision of the same weight as the signing decision itself.

`contextId` is the scoping member, and its absence means *unscoped*, not *all scopes*. A consumer that reads an absent context as a wildcard would make every unscoped key reachable by every caller — the inverse of the intended reading.
