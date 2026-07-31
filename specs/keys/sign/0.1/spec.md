---
slug: keys/sign
version: "0.1"
title: Keys — Sign
summary: A producer asks a key custodian to sign a supplied byte string with a named key it holds, without the private key ever leaving the custodian.
status: draft
targetFrameworkVersion: "0.1"
category: key-management
keywords:
  - keys
  - signing
  - signing-oracle
  - custody
  - eddsa
  - es256
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer (the party wanting a signature)
    requirement: REQUIRED
    member: issuer
  - role: Key custodian
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: The request asks a custodian to exercise a private key on the producer's say-so, so the custodian must be able to attribute the request. Where the transport does not already authenticate the producer, a proof is the only thing standing between the key and anyone who can reach the endpoint.
sideEffects:
  level: none
  rationale: "No stored state changes. The signature itself is a durable, externally-verifiable artefact, so the operation is not repeatable-without-consequence in the way a read is."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: The custodian exercises the named key's private half, producing a signature that verifies as that key's identity. Nothing is disclosed to the caller beyond the signature, but the artefact is attributable to the key's holder.
errorCodes: []
related:
  - keys/derive-and-sign
  - keys/show
  - keys/revoke
  - vault/sign-trust-task
---

## Abstract

The **Keys — Sign** Trust Task lets a *producer* obtain a signature over bytes it supplies, from a key held by a *key custodian*. The private key never leaves the custodian; the producer learns only the signature.

This is the signing-oracle pattern: the custodian is an oracle that will sign anything a sufficiently authorized producer asks it to. That is the whole point and also the whole risk, which is why the conformance rules below say more about authorization than about signing.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/sign/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Populate `payload.keyId` with the key it wants used, `payload.payload` with the base64url encoding of the exact bytes to be signed, and `payload.algorithm` with an algorithm the key supports.
3. **Not** assume the custodian will canonicalize, wrap or otherwise transform the bytes. A producer that needs a canonical form (JCS, for instance) computes it before encoding.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Decide, from its own policy, whether **this producer may use this key** — and refuse with `permission_denied` ([SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)) where it may not. The custodian signs bytes it does not interpret, so this decision is the only limit on what the producer can obtain a signature for.
3. Refuse a key whose `status` is not `active`, with `failed_precondition`.
4. Refuse an `algorithm` the named key's `keyType` cannot perform, with `invalid_argument`. An `x25519` key signs nothing and **MUST** be refused whatever algorithm is named.
5. Sign the decoded bytes **verbatim** and return the signature under the `#response` variant.
6. **Not** return the private key, or any encoding of it, under any circumstances.

A custodian **SHOULD** record the request — producer, key, and a digest of the bytes — in an audit trail. A signature is durable and externally verifiable; a custodian that cannot say afterwards who asked for one has lost the only record that would distinguish authorized use from compromise.

## Definitions

* **Producer.** The party requesting the signature; identified by `issuer`.
* **Key custodian.** The party holding the private key and performing the signing; identified by `recipient`.
* **Key.** The material named by `payload.keyId`, as described by [`keys/show`](../../show/0.1/spec.md).

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/sign/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Signing a challenge with an Ed25519 key

```json
{
  "id": "8f14e45f-ceea-467a-9f0a-2d0e6f7c5b31",
  "type": "https://trusttasks.org/spec/keys/sign/0.1",
  "issuer": "did:web:app.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:00:00Z",
  "payload": {
    "keyId": "signing-key-1",
    "payload": "aGVsbG8tY2hhbGxlbmdl",
    "algorithm": "EdDSA"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/sign/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "9a24e45f-ceea-467a-9f0a-2d0e6f7c5b32",
  "type": "https://trusttasks.org/spec/keys/sign/0.1#response",
  "threadId": "8f14e45f-ceea-467a-9f0a-2d0e6f7c5b31",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:app.example",
  "issuedAt": "2026-07-31T09:00:01Z",
  "payload": {
    "keyId": "signing-key-1",
    "signature": "3q2-7wRLBt0FVsn6dR3ZLC8s0Xh0zvR3sVQ1qWvS0m8xJ7bB4kEwT9c1h2G5nP8aQ",
    "algorithm": "EdDSA"
  }
}
```

Failures (`permission_denied`, `failed_precondition`, `invalid_argument`) use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

**The bytes are opaque to the custodian, so naming a key is equivalent to using it.** A custodian cannot tell a login challenge from a payment authorization from a software release manifest — it sees base64url. Every constraint therefore has to live in *which producers may name which keys*, and a deployment that authorizes broadly has, in effect, handed out the key.

Two consequences worth stating plainly:

* **Scope keys to purposes, not to convenience.** A producer that signs for two unrelated purposes with one key gives anyone who can obtain one signature the ability to obtain the other. Separate keys are the only enforceable boundary, because the custodian cannot enforce a boundary it cannot see.
* **A revoked key must stay revoked.** Records for revoked keys are retained so historic signatures remain attributable; a custodian that allowed reactivation would make the audit trail unfalsifiable in the wrong direction.

The request and response are individually low-disclosure — neither carries key material — but a signature is durable and verifiable by anyone. Confidentiality of the *payload bytes* is the producer's concern and **SHOULD** be enforced at the transport layer where those bytes are sensitive.
