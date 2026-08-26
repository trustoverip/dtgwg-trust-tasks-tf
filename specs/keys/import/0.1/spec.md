---
slug: keys/import
version: "0.1"
title: Keys — Import
summary: A producer hands an externally-created private key to a custodian, which stores it and thereafter exercises it like any key it generated.
status: draft
targetFrameworkVersion: "0.5"
category: key-management
keywords:
  - keys
  - import
  - custody
  - private-key
  - migration
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer (the party holding the key today)
    requirement: REQUIRED
    member: issuer
  - role: Key custodian
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: This request installs signing material the custodian will afterwards exercise on request. An unauthenticated import lets anyone who can reach the endpoint plant a key and then ask for signatures under it, so the producer's authority must be established before the material is stored — not after.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Importing key material places a caller-supplied key under the agent's control. A replayed import reinstates material an operator has deliberately removed from the store.
sideEffects:
  level: mutating
  rationale: "Creates a durable key record the custodian will sign with. Not destructive: an import never replaces existing material — a collision is refused."
exposure:
  discloses: none
  ingests: secret
  actsAsSubject: false
  rationale: "The request body is a private key. Exactly one of `privateKeySealed`, `privateKeyJwe`, or `privateKeyMultibase` is present, and the last of those is cleartext — the custodian receives material it must protect on the producer's behalf from the moment the document arrives. Nothing is disclosed back: the response carries only the public half."
retention:
  class: durable
  rationale: The custodian holds the imported material for the key's whole lifetime and holds the record beyond it, since `keys/revoke` retires rather than deletes so historic signatures stay attributable. Imported keys are additionally the family's retention edge case — they have no `derivationPath` and no `seedId`, so a seed restore does not reproduce them and deleting the record destroys the key outright.
errorCodes:
  - code: keys:alreadyExists
    meaning: A key record already carries the target identifier; the custodian refuses rather than overwrite it. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys:invalidArgument
    meaning: A payload member is well-formed against the schema but unusable for this request. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
related:
  - keys/create
  - keys/show
  - keys/revoke
  - keys/sign
---

## Abstract

The **Keys — Import** Trust Task moves an existing private key into a *key custodian*, so that a key generated elsewhere — by an operator, an HSM export, or a system being migrated — can be used through the same [`keys/sign`](../../sign/0.1/spec.md) surface as keys the custodian derived itself.

Import is the counterpart to [`keys/create`](../../create/0.1/spec.md): create asks the custodian to generate material, import supplies it. The resulting records differ in one operationally important way, recorded in `origin`: a **derived** key can be reproduced from a seed the custodian holds, an **imported** key exists only as stored material and is gone if that storage is lost.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/import/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Supply the key material in **exactly one** carrier: `privateKeySealed`, `privateKeyJwe`, or `privateKeyMultibase`.
3. Prefer `privateKeySealed` (or `privateKeyJwe`), which encrypt the material *to the custodian*. `privateKeyMultibase` carries the key in the clear and is admissible only under the transport condition below.
4. Populate `keyType` with the algorithm of the supplied material.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements).
2. Establish the producer's authority to install key material before storing anything, refusing with `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)) otherwise.
3. **Refuse `privateKeyMultibase` unless the transport provides end-to-end confidentiality between producer and custodian.** A transport that terminates at an intermediary — TLS ending at a load balancer or gateway — does not qualify, because the cleartext key exists on that intermediary.
4. Verify the supplied material is a well-formed private key of the declared `keyType`, and refuse with `keys:invalidArgument` where it is not. A custodian that stores unvalidated material discovers the problem at first use, which is to say at the moment something depends on the signature.
5. Refuse, with `keys:alreadyExists`, an import that would collide with an existing key record. Import **MUST NOT** overwrite material: silently replacing a key invalidates every signature relying party has already verified against the old one.
6. Record the result with `origin: "imported"` and no `derivationPath`, and return it under the `#response` variant.
7. **Not** echo the supplied private key in the response, in an error, or in a log.

A custodian **SHOULD** treat imported material as unrecoverable and say so to operators: a seed restore reconstitutes derived keys and cannot reconstitute this one.

## Definitions

* **Producer.** The party supplying the key; identified by `issuer`.
* **Key custodian.** The party that will store and exercise the key; identified by `recipient`.
* **Carrier.** The member conveying the private key — sealed, JWE, or multibase.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/import/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Sealed import (preferred)

```json
{
  "id": "c3f0e1d2-1a2b-4c3d-8e4f-5a6b7c8d9e0f",
  "type": "https://trusttasks.org/spec/keys/import/0.1",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:10:00Z",
  "payload": {
    "keyType": "ed25519",
    "privateKeySealed": "-----BEGIN SEALED TRANSFER-----\nBundle-Id: 7a1c...\n...\n-----END SEALED TRANSFER-----",
    "label": "legacy release-signing key",
    "contextId": "release"
  }
}
```

### Cleartext import over an end-to-end-confidential transport

Admissible only where producer and custodian are cryptographically joined end to end — an authenticated-and-encrypted messaging envelope, for instance — so no intermediary sees the key:

```json
{
  "id": "d4e1f2a3-2b3c-4d5e-9f60-6b7c8d9e0f11",
  "type": "https://trusttasks.org/spec/keys/import/0.1",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:12:00Z",
  "payload": {
    "keyType": "ed25519",
    "privateKeyMultibase": "z3u2Wv...",
    "label": "migrated signer"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/import/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "e5f2a3b4-3c4d-5e6f-a071-7c8d9e0f1122",
  "type": "https://trusttasks.org/spec/keys/import/0.1#response",
  "threadId": "c3f0e1d2-1a2b-4c3d-8e4f-5a6b7c8d9e0f",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-07-31T09:10:02Z",
  "payload": {
    "key": {
      "keyId": "legacy-release-signer",
      "keyType": "ed25519",
      "status": "active",
      "publicKey": "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "origin": "imported",
      "label": "legacy release-signing key",
      "contextId": "release",
      "createdAt": "2026-07-31T09:10:02Z",
      "updatedAt": "2026-07-31T09:10:02Z"
    }
  }
}
```

Note the absent `derivationPath` — the shape itself records that this key is not reproducible.

Failures (`permissionDenied`, `keys:invalidArgument`, `keys:alreadyExists`) use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

### Data carried

**This is the one task in the family that carries a private key on the wire**, and
every other consideration follows from that. The schema's `oneOf` means exactly one
carrier is present, and *which* one is a confidentiality decision rather than a
formatting preference.

*Choose the carrier by who can see it, not by convenience.* `privateKeySealed` and
`privateKeyJwe` encrypt to the custodian, so the material is opaque to every
intermediary and to the transport itself. `privateKeyMultibase` does not: it is
cleartext, and its safety rests entirely on the transport being end-to-end
confidential. The distinction is easy to lose because all three are "just a string
in a JSON body" — hence the explicit consumer rule refusing the cleartext carrier
where the transport terminates early. A custodian behind TLS that terminates at a
load balancer has an intermediary that can read the key, and the schema cannot
detect that; only the deployment can.

The rest of the payload is thin and non-secret: `keyType`, which the custodian
**MUST** verify against the supplied material rather than trust, and the optional
`label` and `contextId`. `label` is operator-facing and carries no authorization
meaning, which makes it the field a deployment quietly fills with a person's name
or an account reference; a producer **SHOULD** name the key after its purpose. The
response returns a `KeyRecord` carrying `publicKey` only — no `keys/*` response
ever carries private material back.

### Correlation

An imported key is the family's least joinable record, and unusually that is a
consequence of how it arrived rather than a privacy choice. `origin` is `imported`,
`derivationPath` is absent, and `seedId` is absent, so nothing places the key in a
family of sibling keys the way a derived key's path does. What remains joinable is
`contextId`, `label`, and — far more durably — everything the key subsequently
signs, since `publicKey` is public by nature and links every signature made under
it.

The correlation risk that is specific to import is on the *other* side of the wire.
The producer held this key before the custodian did, which means at least two
parties can now produce signatures that verify identically, and nothing in a
signature distinguishes them. Where that matters, importing is the wrong operation.

### Retention

Durable in the custodian, and the retention question splits in two.

The *record* is kept for the key's operating life and beyond it:
[`keys/revoke`](../../revoke/0.1/spec.md) sets `status` to `revoked` rather than
deleting, and a revoked record **MUST NOT** be reactivated, because a signature made
last year is only attributable while the key that made it is still known.

The *material* is where import differs from every sibling. *Imported keys are
outside the seed-recovery story* — they have no derivation path, so a restore from a
seed backup does not reproduce them. Operators reasoning about disaster recovery
from a seed backup will be wrong about exactly these keys unless the custodian
tells them so, and a custodian that deletes an imported record has destroyed the
key rather than retired it.

The producer's own retention is the part this specification cannot reach. *A key
that has been on a wire has been on a wire*: even a correct import means the
material existed outside the custodian, in the producer's memory, its logs, and
possibly its backups. Where the key's value justifies it, producers **SHOULD**
generate in place with [`keys/create`](../../create/0.1/spec.md) instead and
reserve import for material that already exists elsewhere and cannot be
regenerated.

### Consent/purpose

The purpose is custody transfer: the producer is handing over material it already
holds so that a custodian can exercise it from now on. The producer's `proof` is
the record of the basis — an unauthenticated import lets anyone who can reach the
endpoint plant a key and then ask for signatures under it, so authority is
established before the material is stored, not after.

*Refuse collisions rather than resolving them.* Overwriting an existing record
would silently invalidate signatures already verified against the old key, and
would do so without any party observing an error. `keys:alreadyExists` is the safe
answer even when the producer plainly meant to replace — a replacement is a
decision that deserves to be made explicitly, under a new identifier, rather than
inferred from a repeated request.
