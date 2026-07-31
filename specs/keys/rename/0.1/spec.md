---
slug: keys/rename
version: "0.1"
title: Keys — Rename
summary: A producer changes the identifier a custodian addresses a key by, leaving the key material untouched.
status: draft
targetFrameworkVersion: "0.1"
category: key-management
keywords:
  - keys
  - rename
  - identifier
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
  rationale: The identifier is what signing requests name, so changing it changes which requests reach this key. That is an authorization-relevant edit and must be attributable.
sideEffects:
  level: mutating
  rationale: "Changes the stored identifier. The key material and its public half are unchanged."
subjectPath: /keyId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
related:
  - keys/show
  - keys/list
  - keys/revoke
---

## Abstract

The **Keys — Rename** Trust Task changes the identifier a *key custodian* addresses a key by. Nothing cryptographic changes: the same private key, the same public half, the same signatures verify afterwards.

What does change is **which requests reach it**. `keyId` is what [`keys/sign`](../../sign/0.1/spec.md) names, so a rename silently redirects every producer still using the old identifier — they will get "no such key", not the key they meant.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/rename/0.1`, with itself as `issuer`, the custodian as `recipient`, and both `keyId` and `newKeyId` populated.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Establish the producer's authority over the key, refusing with `permission_denied` ([SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)) otherwise.
3. Refuse with `not_found` where no record carries `keyId`.
4. Refuse with `already_exists` where a record already carries `newKeyId`. A rename **MUST NOT** overwrite another key's record — doing so would silently repoint every signing request naming that identifier at different material.
5. Leave the key material, `publicKey`, `createdAt` and `origin` unchanged, and return the new identifier with the time of the change.

## Definitions

* **Producer.** The party renaming; identified by `issuer`.
* **Key custodian.** The party holding the key; identified by `recipient`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/rename/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "8293a4b5-c6d7-4e83-f901-122334455667",
  "type": "https://trusttasks.org/spec/keys/rename/0.1",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:35:00Z",
  "payload": {
    "keyId": "app-signing-key",
    "newKeyId": "app-signing-key-2026"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/rename/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "93a4b5c6-d7e8-4f94-0112-233445566778",
  "type": "https://trusttasks.org/spec/keys/rename/0.1#response",
  "threadId": "8293a4b5-c6d7-4e83-f901-122334455667",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-07-31T09:35:01Z",
  "payload": {
    "keyId": "app-signing-key-2026",
    "updatedAt": "2026-07-31T09:35:01Z"
  }
}
```

Failures (`permission_denied`, `not_found`, `already_exists`) use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

A rename is cryptographically inert and operationally sharp. Every producer configured with the old identifier breaks at its next signing request, and breaks by *failing*, which is the safe direction — but a custodian that resolved the old name to the renamed key "helpfully" would turn a rename into an alias, and an alias into an ambiguity about which key a request meant.

Refusing a colliding `newKeyId` matters for the same reason in reverse: allowing the collision would repoint requests at material the producer never named.
