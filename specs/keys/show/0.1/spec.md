---
slug: keys/show
version: "0.1"
title: Keys — Show
summary: A producer asks a key custodian for the record it holds for one key, receiving the public half and lifecycle state.
status: draft
targetFrameworkVersion: "0.1"
category: key-management
keywords:
  - keys
  - show
  - lookup
  - public-key
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
  requirement: RECOMMENDED
  rationale: A single-key read is typically consumed over an authenticated transport; a proof becomes valuable when the answer is retained or relied upon by a third party — for example when the public key is pinned.
sideEffects:
  level: none
  rationale: "Read-only read of a single key record."
subjectPath: /keyId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - keys/list
  - keys/create
  - keys/sign
  - keys/revoke
---

## Abstract

The **Keys — Show** Trust Task returns the record a *key custodian* holds for one key: its public half, type, lifecycle status, origin, and — where the key was derived — the path and seed that reproduce it. It never returns private material.

A key the custodian does not hold is reported as `key: null`, not as an error: "no such key" is a legitimate answer to a legitimate question.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/show/0.1`, with itself as `issuer`, the custodian as `recipient`, and `payload.keyId` naming the key.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Apply its policy to decide whether this producer may read this key's record, refusing with `permission_denied` ([SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)) where it may not.
3. Respond with `key` set to the record, or `null` where no such key exists.
4. **Not** include private key material in the record.

Where policy denies the read, a custodian **SHOULD** answer `permission_denied` rather than `key: null`. The two answers mean different things, and collapsing them tells a caller that a key it is simply not allowed to see does not exist.

## Definitions

* **Producer.** The party reading the record; identified by `issuer`.
* **Key custodian.** The party holding the key; identified by `recipient`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/show/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "3d4e5f60-7182-4930-a4b5-c6d7e8f90112",
  "type": "https://trusttasks.org/spec/keys/show/0.1",
  "issuer": "did:web:app.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:25:00Z",
  "payload": {
    "keyId": "app-signing-key"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/show/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "4e5f6071-8293-4a41-b5c6-d7e8f9011223",
  "type": "https://trusttasks.org/spec/keys/show/0.1#response",
  "threadId": "3d4e5f60-7182-4930-a4b5-c6d7e8f90112",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:app.example",
  "issuedAt": "2026-07-31T09:25:01Z",
  "payload": {
    "key": {
      "keyId": "app-signing-key",
      "keyType": "ed25519",
      "status": "active",
      "publicKey": "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "derivationPath": "m/26'/2'/0'/1'",
      "origin": "derived",
      "contextId": "app",
      "createdAt": "2026-07-31T09:20:01Z",
      "updatedAt": "2026-07-31T09:20:01Z"
    }
  }
}
```

### No such key

```json
{
  "id": "5f607182-93a4-4b52-c6d7-e8f901122334",
  "type": "https://trusttasks.org/spec/keys/show/0.1#response",
  "threadId": "4a5b6c7d-8e9f-40a1-b2c3-d4e5f6071829",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:app.example",
  "issuedAt": "2026-07-31T09:26:01Z",
  "payload": {
    "key": null
  }
}
```

## Security & Privacy

The record is metadata about key material, not the material itself. It nonetheless tells a reader that a given key exists, what it is for, and whether it is still usable — enough for an attacker to map a custodian's key set and pick a target. Custodians **SHOULD** scope reads the same way they scope signing.

`status` is the member relying parties care about most: a signature verified against a key whose record now reads `revoked` was still valid when made, and a consumer that conflates "revoked now" with "was never valid" will reach the wrong conclusion about historic artefacts.
