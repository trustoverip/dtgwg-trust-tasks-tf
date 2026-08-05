---
slug: keys/derive-and-sign
version: "0.1"
title: Keys — Derive and Sign
summary: A producer asks a custodian to derive a key at a given path and sign with it in one step, without the key being added to the custodian's stored key set.
status: draft
targetFrameworkVersion: "0.1"
category: key-management
keywords:
  - keys
  - signing
  - derivation
  - hierarchical-deterministic
  - custody
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
  rationale: As with keys/sign, the custodian is exercising a private key on the producer's say-so and must be able to attribute the request. The derived key is reachable by path rather than by stored identifier, so the custodian's policy has one less handle to check against — making producer authentication more important, not less, and leaving the proof as the load-bearing control. Forgery of the request is the threat, and the resulting signature is a durable artefact relied on by parties beyond this exchange.
sideEffects:
  level: none
  rationale: "No key record is created and no stored state changes; the derived key is used and discarded. The signature is a durable artefact."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: The custodian exercises the private half of the key derived at the requested path, producing a signature attributable to that derived identity — an identity the response names, since nothing was stored for the caller to look it up afterwards.
errorCodes: []
related:
  - keys/sign
  - keys/derive-and-sign-document
  - keys/create
---

## Abstract

The **Keys — Derive and Sign** Trust Task combines derivation and signing: the *key custodian* derives a key at the requested path, signs the supplied bytes with it, returns the signature and the derived public key, and adds nothing to its stored key set.

This exists for identities that are **addressed by path rather than by name**. A producer that needs to sign as a particular derived identity — one per context, per tenant, per document — would otherwise have to create and store a key record for each, and then manage the lifecycle of records it only ever uses once. Derivation is deterministic, so the same path yields the same identity every time without anything being stored.

The response carries `publicKey` for a reason that is easy to miss: the producer generally does **not** know which identity it just signed as. Nothing was stored, so there is no record to read it from afterwards.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/derive-and-sign/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Populate `keyType`, `derivationPath`, the base64url-encoded `payload`, and an `algorithm` the derived key can perform.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Decide, from its own policy, whether this producer may derive and sign **at this path**, refusing with `permission_denied` ([SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)) otherwise. A path is not a stored record, so there is no per-key ACL to fall back on — the policy has to be expressed over paths.
3. Refuse an `algorithm` the derived key type cannot perform, with `invalid_argument`.
4. Sign the decoded bytes verbatim.
5. Return the derived `publicKey` alongside the signature, and **not** the derived private key.
6. **Not** create a stored key record as a side effect. A producer expecting a durable key uses [`keys/create`](../../create/0.1/spec.md).

## Definitions

* **Producer.** The party requesting the signature; identified by `issuer`.
* **Key custodian.** The party holding the seed and deriving; identified by `recipient`.
* **Derivation path.** The hierarchical-deterministic path selecting the key; deterministic against the custodian's seed.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/derive-and-sign/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "c6d7e8f9-0112-4237-3445-566778899001",
  "type": "https://trusttasks.org/spec/keys/derive-and-sign/0.1",
  "issuer": "did:web:app.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:45:00Z",
  "payload": {
    "keyType": "ed25519",
    "derivationPath": "m/26'/9'/0'",
    "payload": "dGVuYW50LWNoYWxsZW5nZQ",
    "algorithm": "EdDSA"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/derive-and-sign/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "d7e8f901-1223-4348-4556-677889900112",
  "type": "https://trusttasks.org/spec/keys/derive-and-sign/0.1#response",
  "threadId": "c6d7e8f9-0112-4237-3445-566778899001",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:app.example",
  "issuedAt": "2026-07-31T09:45:01Z",
  "payload": {
    "publicKey": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "signature": "vB2sT9c1h2G5nP8aQ3q2-7wRLBt0FVsn6dR3ZLC8s0Xh0zvR3sVQ1qWvS0m8xJ7bB4kEwT",
    "algorithm": "EdDSA"
  }
}
```

Failures (`permission_denied`, `invalid_argument`) use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Everything the [`keys/sign`](../../sign/0.1/spec.md) Security section says about opaque bytes applies here, with one addition that makes this task sharper.

**Authority is over paths, not over records.** With `keys/sign`, a custodian can consult a stored record and whatever scope it carries. Here there is no record: the producer names a path, and the custodian either derives it or does not. A policy that authorizes a path *prefix* therefore authorizes every identity beneath it — which may be exactly what an operator wants for a tenant subtree, and is a serious over-grant if the prefix is chosen carelessly. Custodians **SHOULD** express these policies over the narrowest prefix that satisfies the use case, and **SHOULD NOT** default to authorizing the seed root.

Because the same path always yields the same key, a producer that can obtain one signature at a path can obtain any number — this is a stable identity, not an ephemeral one, despite nothing being stored.
