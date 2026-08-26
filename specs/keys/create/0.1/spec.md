---
slug: keys/create
version: "0.1"
title: Keys — Create
summary: A producer asks a key custodian to generate a new key and hold it, receiving only the public half.
status: draft
targetFrameworkVersion: "0.5"
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Creating a key adds material the agent will sign with. A replayed create leaves a second, unaccounted-for key in the store, and an operator auditing the store cannot tell it from one they asked for.
sideEffects:
  level: mutating
  rationale: "Generates and stores a new key record."
exposure:
  discloses: none
  ingests: secret
  actsAsSubject: false
  rationale: "Ordinarily the request carries only naming and scoping data and the custodian generates the material itself. The `mnemonic` member breaks that: a BIP-39 phrase reconstitutes the key anywhere, so a request supplying one hands the custodian externally-generated seed material and is an import wearing create's clothes. `label` and `contextId` are additionally operator-authored free text. Nothing is disclosed back — the response carries only the public half."
retention:
  class: durable
  rationale: The custodian holds the resulting KeyRecord for the key's whole lifetime and beyond it — `keys/revoke` sets `status` to `revoked` rather than deleting, precisely so signatures the key already made stay attributable. A custodian that deleted revoked records would leave historic signatures unverifiable against any known key, which is the loss this class is recording.
errorCodes:
  - code: keys:alreadyExists
    meaning: A key record already carries the target identifier; the custodian refuses rather than overwrite it. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys:invalidArgument
    meaning: A payload member is well-formed against the schema but unusable for this request. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
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

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/create/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Populate `payload.keyType` with the algorithm to generate.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements).
2. Establish the producer's authority to add keys, refusing with `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)) otherwise.
2a. **Refuse `mnemonic` on any transport that is not end-to-end confidential**, and never log or echo it. A BIP-39 phrase reconstitutes the key anywhere, so it is secret-bearing in exactly the way the rest of this payload is not — the same reasoning that makes [`keys/import`](../../import/0.1/spec.md) refuse its cleartext carrier.
3. Refuse, with `keys:alreadyExists`, a request that would collide with an existing key record rather than replacing it.
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

Failures (`permissionDenied`, `keys:alreadyExists`, `keys:invalidArgument`) use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

### Data carried

For an ordinary create the request is thin by design: `keyType`, and optionally a
`derivationPath`, a `keyId`, a `label`, and a `contextId`. None of that is secret,
and the whole point of the task is that the secret — the private half — is
generated inside the custodian and never travels. The response returns a
`KeyRecord` carrying `publicKey` and nothing more; no `keys/*` response ever
carries private material.

`mnemonic` is the exception that changes the task's character, and a producer
should understand what supplying it does. A BIP-39 phrase reconstitutes the key
anywhere, forever, from twelve or twenty-four words — so a create carrying a
`mnemonic` is not a create at all but an import of externally-generated seed
material, and the confidentiality rules of [`keys/import`](../../import/0.1/spec.md)
apply to it unchanged. A custodian **MUST** refuse it on a transport that is not
end-to-end confidential, and **MUST NOT** log or echo it. Worse than a private
key, a seed phrase may derive an entire tree of keys, so a phrase that leaks
compromises material this custodian never saw and cannot revoke.

`label` and `contextId` are the free-text members and the ones a deployment fills
without thinking. `label` is documented as operator-facing and carries no
authorization meaning, which makes it exactly the field into which a person's
name, an account number, or a customer identifier gets typed for convenience.
The smallest conforming request is `keyType` alone; everything else is the
producer's choice, and a producer **SHOULD** name a key after what it is *for*
rather than after whom it belongs to.

### Correlation

The custodian-side joins are the ones that matter, and they exist by
construction. `derivationPath` and `seedId` on the returned record group every
derived key under the seed it came from: a custodian holding a key's path knows
which other keys share its ancestry, whether or not it was asked. `contextId`
groups keys by scope, and `label` — being human-authored and habitually
descriptive — will in practice group them by whatever the operator was actually
thinking about. An `internal: true` key is the only one that joins to nothing:
generated from a CSPRNG, derived from no seed, reproducible from nothing, it has
no `derivationPath` and no `seedId` to align on. Producers wanting a key that
cannot be placed in a family have exactly that lever, and `origin` on the
returned record is the only reliable confirmation the custodian honoured it.

Externally, the joinable artefact is not the request but everything the key later
signs, since `publicKey` is by nature public and links every signature made under
it. A producer that wants unlinkable signing surfaces creates separate keys; a
custodian cannot manufacture that separation after the fact.

`contextId` is the scoping member, and its absence means *unscoped*, not *all
scopes*. A consumer that reads an absent context as a wildcard would make every
unscoped key reachable by every caller — the inverse of the intended reading, and
a correlation failure as much as an authorization one.

### Retention

Durable, and longer-lived than the key itself. The custodian keeps the
`KeyRecord` for the key's operating life and then keeps it after revocation:
[`keys/revoke`](../../revoke/0.1/spec.md) moves `status` to `revoked` rather than
deleting the row, and the record **MUST NOT** be reactivated. That is deliberate —
a signature made last year is only attributable if the key that made it is still
known — but it means the descriptive members travel with it. A `label` naming a
person outlives the key's usefulness by design, and there is no member in this
payload that expires it.

The `mnemonic`, uniquely, must not be retained at all: it is a transport carrier
for material the custodian converts into a stored key, and a custodian that
retained the phrase would hold a recovery capability the producer never asked it
to keep.

### Consent/purpose

Creating a key is granting future signing capacity: whoever may later name this
key in [`keys/sign`](../../sign/0.1/spec.md) can sign with it, and the custodian
signs the bytes it is handed without inspecting them. Custodians **SHOULD**
therefore treat "who may create keys, and in which scope" as an authorization
decision of the same weight as the signing decision itself — the purpose a key
was created for is not recorded anywhere in this payload, so the scope it was
created in is the only durable expression of it.

The corollary is a minimisation rule rather than a gate: a producer creating one
key for two unrelated purposes has, as [`keys/sign`](../../sign/0.1/spec.md)
explains, made either purpose's authorization sufficient for the other. Separate
keys per purpose is the only boundary a custodian can actually enforce, because
it is the only one it can see.
