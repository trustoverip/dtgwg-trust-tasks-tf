---
slug: keys/revoke
version: "0.1"
title: Keys — Revoke
summary: A producer retires a key from further use; the record is kept so signatures it already made remain attributable.
status: draft
targetFrameworkVersion: "0.1"
category: key-management
keywords:
  - keys
  - revoke
  - retire
  - lifecycle
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
  rationale: Revocation withdraws signing capacity and is irreversible. An unattributable revocation is a denial-of-service against whoever depended on the key, with no record of who caused it.
sideEffects:
  level: destructive
  rationale: "Irreversible: a revoked key MUST NOT be reactivated. The record survives, but the key's usefulness does not."
subjectPath: /keyId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
related:
  - keys/show
  - keys/list
  - keys/sign
  - keys/create
---

## Abstract

The **Keys — Revoke** Trust Task retires a key: the *key custodian* refuses further signing requests naming it, and the record survives with `status: "revoked"`.

**Revocation is not deletion, and the distinction is load-bearing.** Signatures the key made before revocation remain verifiable artefacts in the world, and a relying party that encounters one needs to be able to ask the custodian what that key was and when it was retired. A custodian that deleted the record would leave those signatures unattributable — the audit trail would say nothing rather than saying "retired at 09:40".

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/revoke/0.1`, with itself as `issuer`, the custodian as `recipient`, and `payload.keyId` naming the key.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Establish the producer's authority over the key, refusing with `permission_denied` ([SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)) otherwise.
3. Refuse with `not_found` where no record carries `keyId`.
4. Set the record's status to `revoked`, **retain the record**, and refuse every subsequent [`keys/sign`](../../sign/0.1/spec.md) naming it.
5. **Never** return a revoked key to `active`. Reactivation would make the audit trail unfalsifiable in the wrong direction: a signature made during the revoked window would afterwards look as though it were made by a valid key.
6. Return the realized `status` and the `updatedAt` boundary under the `#response` variant.

Revoking an already-revoked key **SHOULD** succeed idempotently rather than erroring — the caller's intent is satisfied, and failing invites a retry loop against a key that is already retired.

## Definitions

* **Producer.** The party revoking; identified by `issuer`.
* **Key custodian.** The party holding the key; identified by `recipient`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/revoke/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "a4b5c6d7-e8f9-4015-1223-344556677889",
  "type": "https://trusttasks.org/spec/keys/revoke/0.1",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:40:00Z",
  "payload": {
    "keyId": "app-signing-key-2026",
    "reason": "superseded by the 2027 signer"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/revoke/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "b5c6d7e8-f901-4126-2334-455667788990",
  "type": "https://trusttasks.org/spec/keys/revoke/0.1#response",
  "threadId": "a4b5c6d7-e8f9-4015-1223-344556677889",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-07-31T09:40:01Z",
  "payload": {
    "keyId": "app-signing-key-2026",
    "status": "revoked",
    "updatedAt": "2026-07-31T09:40:01Z"
  }
}
```

Failures (`permission_denied`, `not_found`) use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Revocation is the fastest lever a custodian has when a key is suspected compromised, and it is one-way by design. Producers **SHOULD** treat it as such: there is no unrevoke, and the correct recovery from an unnecessary revocation is a new key, not a restored one.

`updatedAt` is the member relying parties reason with. It marks the boundary between signatures made while the key was in good standing and requests the custodian refused afterwards — which is what lets a verifier judge a historic artefact rather than dismissing everything the key ever signed.

Revocation does **not** invalidate past signatures, and consumers **MUST NOT** treat it as though it does unless their own policy says so. Conflating "retired" with "was never trustworthy" retroactively repudiates work that was legitimate when it happened.
