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
  requirement: REQUIRED
  rationale: The request asks a custodian to exercise a private key on the producer's say-so, so the custodian must be able to attribute the request. Where the transport does not already authenticate the producer, a proof is the only thing standing between the key and anyone who can reach the endpoint — which is why it is REQUIRED rather than left to the transport. Forgery of the request is the threat, and the resulting signature is a durable artefact that outlives the exchange and will be relied on by parties who never saw it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Signing acts with the subject's authority and produces evidence that outlives the exchange. A signing request that cannot be placed in an acceptance window can be re-presented indefinitely to obtain further signatures over the same material.
sideEffects:
  level: none
  rationale: "No stored state changes. The signature itself is a durable, externally-verifiable artefact, so the operation is not repeatable-without-consequence in the way a read is."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: The custodian exercises the named key's private half, producing a signature that verifies as that key's identity. Nothing is disclosed to the caller beyond the signature, but the artefact is attributable to the key's holder. What the request carries in is `payload` — bytes the custodian signs verbatim without parsing them, so it cannot classify their contents; `metadata` records what this specification can honestly claim about them, and a producer whose bytes are personal or secret is responsible for the transport that carries them.
retention:
  class: transient
  rationale: The custodian changes no stored state — it reads a key record, produces a signature, and returns it. Nothing about the request needs to survive the exchange except whatever audit line the deployment chooses to keep, and the `payload` bytes in particular are handled in flight and not stored. The signature itself is durable, but it is durable in the hands of whoever relies on it rather than in the custodian's.
errorCodes:
  - code: keys:invalidArgument
    meaning: A payload member is well-formed against the schema but unusable for this request. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys/sign:failedPrecondition
    meaning: The named key exists but its `status` is not `active`, so it cannot sign.
    retryable: false
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

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/sign/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Populate `payload.keyId` with the key it wants used, `payload.payload` with the base64url encoding of the exact bytes to be signed, and `payload.algorithm` with an algorithm the key supports.
3. **Not** assume the custodian will canonicalize, wrap or otherwise transform the bytes. A producer that needs a canonical form (JCS, for instance) computes it before encoding.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements).
2. Decide, from its own policy, whether **this producer may use this key** — and refuse with `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)) where it may not. The custodian signs bytes it does not interpret, so this decision is the only limit on what the producer can obtain a signature for.
3. Refuse a key whose `status` is not `active`, with `keys/sign:failedPrecondition`.
4. Refuse an `algorithm` the named key's `keyType` cannot perform, with `keys:invalidArgument`. An `x25519` key signs nothing and **MUST** be refused whatever algorithm is named.
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

Failures (`permissionDenied`, `keys/sign:failedPrecondition`, `keys:invalidArgument`) use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

### Data carried

**The bytes are opaque to the custodian, so naming a key is equivalent to using
it.** A custodian cannot tell a login challenge from a payment authorization from
a software release manifest — it sees base64url in `payload` and signs it verbatim,
without parsing, canonicalizing, or wrapping it. Every constraint therefore has to
live in *which producers may name which keys*, and a deployment that authorizes
broadly has, in effect, handed out the key.

That opacity cuts both ways, and the privacy consequence is the one usually
missed. Because the custodian cannot classify the bytes, it cannot apply a policy
to their contents, cannot redact them from a debug log, and cannot warn a producer
that it has just been handed a document containing someone's medical history.
This specification declares `ingests: metadata` because that is the most it can
honestly say about a member whose contents it never inspects; a producer signing
personal or confidential bytes is carrying personal or confidential data to the
custodian and is responsible for choosing a transport that reflects that.
Confidentiality of the `payload` bytes is the producer's concern and **SHOULD** be
enforced at the transport layer where those bytes are sensitive.

Neither leg carries key material. The request names a `keyId` and an `algorithm`;
the response returns `signature`, `algorithm`, and an echoed `keyId` — echoed
specifically so that a response separated from its request is still attributable.
The smallest conforming request is already the whole request: all three members
are required and none is padding.

### Correlation

A signature is the most durable correlator in this family, and unlike the request
it is not addressed to anybody. Anyone holding the public half can verify it, at
any time, forever, and every signature made under one key is thereby linked to
every other. The custodian's records and the producer's authorization can both be
revoked; the linkage between a key and everything it ever signed cannot.

**Scope keys to purposes, not to convenience.** A producer that signs for two
unrelated purposes with one key gives anyone who can obtain one signature the
ability to obtain the other, and gives every verifier of either the ability to
join them. Separate keys are the only enforceable boundary, because the custodian
cannot enforce a boundary it cannot see — and they are the only unlinkability
boundary for the same reason.

On the custodian's side, the sequence of requests is its own trace: which keys a
producer names, how often, and when is a behavioural record of what that producer
is doing, available to the custodian without ever reading a byte of `payload`.

### Retention

Transient at the custodian. Nothing is stored — the operation reads a key record,
produces a signature, and returns it — and a custodian **SHOULD NOT** retain
`payload` beyond the moment it has been signed, since keeping bytes it cannot
classify is keeping an unbounded liability it cannot inventory. What a deployment
*does* keep is the audit line, and there the same rule applies in reverse:
recording that key *K* signed a digest at time *T* is proportionate; recording the
bytes is not.

The retained state that does persist belongs to the key rather than to the
request. **A revoked key must stay revoked.** Records for revoked keys are kept so
historic signatures remain attributable; a custodian that allowed reactivation
would make the audit trail unfalsifiable in the wrong direction.

### Consent/purpose

The purpose is exercise of an existing custody relationship: a producer already
authorized over a key asks the custodian to use it. The `proof` is the record of
the basis and is REQUIRED rather than delegated to the transport, because where
the transport does not authenticate the producer it is the only thing standing
between the key and anyone who can reach the endpoint.

Nothing in this payload states what the signature is *for*, and no member could —
the custodian cannot read the bytes to check. The purpose limitation therefore has
to be expressed structurally, at creation time, by the scope a key was created in
and the set of producers permitted to name it. A deployment that wants a key
usable only for one purpose gets that by making it the only key that purpose can
reach, not by anything this task can assert.
