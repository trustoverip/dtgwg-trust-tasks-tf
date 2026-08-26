---
slug: vault/upsert
version: "0.3"
wireCompatibleWith: "0.1"
title: Vault — Upsert
summary: A vault consumer creates a new vault entry or updates an existing one; secret material rides inside an HPKE-sealed envelope so the Trust Task itself carries only ciphertext.
status: draft
targetFrameworkVersion: "0.4"
category: credentials
keywords:
  - vault
  - credentials
  - create
  - update
  - rotation
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Upsert is the canonical state-changing task on the vault; it can introduce credentials that other Companions will later trust and use. The producer's identity MUST be verifiable so the maintainer can attribute the change to a specific consumer and so the audit log records who introduced or rotated the credential.
sideEffects:
  level: mutating
  rationale: "Creates or updates a vault entry; secret material rides sealed inside the envelope."
subjectPath: /id
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: "The secret rides sealed and the maintainer cannot read it, but everything wrapped around it arrives in cleartext and is about a person: `targets` names the sites, apps, and relying parties the principal holds an account with, `label` is a human display name of the account's own naming (\"Personal bank — checking\"), `tags` group entries by area of life, `notes` is 4096 characters of free text, and `customFieldNames` describes the shape of a credential the maintainer is not permitted to read. Nothing is disclosed back to the caller beyond the entry it just wrote."
retention:
  class: durable
  rationale: "An entry is the vault's whole product — it is stored until an explicit `vault/delete`, survives every session, and is what later `vault/release` and `vault/proxy-login` calls resolve against. The cleartext metadata is retained on exactly the same terms as the sealed secret, which is the asymmetry worth naming: deleting it would leave the maintainer holding ciphertext it could no longer match to a site or render to a person."
errorCodes:
  - code: vault/upsert:contextNotFound
    meaning: The supplied `contextId` does not exist.
    retryable: false
  - code: vault/upsert:notFound
    meaning: An `id` was supplied (update path) but no entry with that id exists in the consumer's visible scope.
    retryable: false
  - code: vault/upsert:versionConflict
    meaning: An `expectedVersion` was supplied and does not match the current version. The consumer SHOULD re-read the entry (vault/get) and retry the upsert with the up-to-date version.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        currentVersion: { type: "integer", minimum: 0 }
  - code: vault/upsert:sealedSecretInvalid
    meaning: The sealedSecret envelope failed verification (digest mismatch, signature invalid, recipient key unknown, or armor malformed).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: "string", enum: ["armor_malformed", "recipient_key_unknown", "signature_invalid", "digest_mismatch"] }
  - code: vault/upsert:secretRequired
    meaning: A create or rotation was attempted without `sealedSecret` for a secretKind that requires one.
    retryable: false
  - code: vault/upsert:envelopeUnsupported
    meaning: The `sealedSecret.envelope` kind is not one the maintainer implements (e.g. a TSP message arriving at a maintainer that only speaks `didcommAuthcrypt`). Producers SHOULD consult `trust-task-discovery/0.1` to learn which envelope kinds the maintainer accepts.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        receivedEnvelope: { type: "string" }
        supportedEnvelopes: { type: "array", items: { "type": "string" } }
  - code: vault/upsert:contextChangeForbidden
    meaning: The consumer attempted to change `contextId` on an existing entry. Move-between-contexts MUST be done as delete + recreate.
    retryable: false
---

## Abstract

The **Vault — Upsert** Trust Task creates a new vault entry or updates an existing one. The secret material — when present — is wrapped in an HPKE-sealed envelope (`SealedSecret`) so the Trust Task wire form carries ciphertext + an authenticator only.

Updates may be partial: populated metadata fields replace current values; omitted fields are left untouched; fields listed in `clearFields` are explicitly cleared. Optimistic concurrency via `expectedVersion`.

## Changes from 0.2

The attachment digest carried in `VaultEntry.attachments[]` moves from
**`sha256`** — a bare lowercase-hex SHA-256 — to **`digestMultibase`**, the
framework's
[`DigestMultibase`](../../../_framework/0.3/framework.schema.json): a
multibase-encoded multihash.

A bare hex string hard-codes one algorithm into the wire contract, so moving off
SHA-256 later would need a schema revision rather than a different multihash
prefix, and it names no base encoding, leaving a verifier to infer base16 from
context. The digest is taken over the encrypted blob **bytes**, not over a
canonicalization — the blob is an opaque artifact rather than a JSON document —
so unlike the credential digests converged in the same sweep, no reproducibility
defect is being fixed here. This is the encoding argument alone.

The change arrives through the shared component: this version re-pins its
`$ref`s from `vault/_shared/0.2` to `vault/_shared/0.3`, which is why it is a new
version at all ([SPEC.md §5.4](/SPEC.md#54-migrating-between-versions)
couples the two). Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under
[§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2`
remains published and pinned to `vault/_shared/0.2`; `vault/proxy-login` and
`vault/release` stay on `0.2` deliberately, since they reference only
`SiteTarget` and `SecretKind` and never expose an attachment digest.

## Conformance

A conforming **producer** **MUST**:

1. Populate `contextId`, `targets`, `label`, `secretKind`.
2. On create with a kind that carries secret bytes (everything except `didSelfIssued` and `didcommPeer`), populate `sealedSecret` with an HPKE envelope sealed to the maintainer's published recipient key.
3. On update where the secret is unchanged, omit `sealedSecret`. On rotation, populate it.
4. Supply `expectedVersion` on every update; omit on create.
5. **MUST NOT** include the cleartext `VaultSecret` anywhere in the payload — secrets travel only inside `sealedSecret`.
6. **MUST NOT** attempt to change `contextId` on an existing entry; delete and recreate instead.
7. Carry a `proof` over the document; the proof's verificationMethod MUST resolve via the producer's authenticated DID.

A conforming **consumer** **MUST**:

1. Verify the document `proof` and the consumer's `VaultWrite` capability on `contextId`.
2. On create: assign or accept an entry id, set `version = 1`, set `createdAt = createdBy = updatedAt = updatedBy = now / issuer`. Unseal `sealedSecret`, validate its inner `VaultSecret` against the `vault-secret.schema.json` schema for the declared `secretKind`. Persist.
3. On update: verify `expectedVersion` matches the stored version (atomic check-and-set); reject with `versionConflict` on mismatch with `details.currentVersion`. Apply field changes. Apply `clearFields`. If `sealedSecret` is populated, unseal, validate, replace stored secret, set `passwordChangedAt = now` (for `password` kind) or `lastRotatedAt` equivalent. Increment `version`.
4. Return the resulting `entry` (metadata view) and `created: bool`.
5. Emit a `sync/event/0.1` of kind `vaultUpserted` to all other registered consumers in the maintainer's ACL with VaultRead on the entry's context. The triggering consumer SHOULD also receive the event for echo, so its local cache merges via the same code path.
6. **MUST NOT** persist or log the cleartext `VaultSecret` outside the encrypted vault store.

## Payload

`payload.id` (optional on create, REQUIRED on update) — entry id.

`payload.expectedVersion` (REQUIRED on update) — observed version for concurrency check.

`payload.contextId` (REQUIRED) — target trust context. Immutable post-create.

`payload.targets` (REQUIRED) — non-empty list of `SiteTarget` bindings.

`payload.label` (REQUIRED).

`payload.secretKind` (REQUIRED) — discriminator.

`payload.tags`, `payload.notes`, `payload.favicon`, `payload.selectors`, `payload.customFieldNames`, `payload.expiresAt` (optional metadata).

`payload.sealedSecret` (REQUIRED on create except for the two reference kinds; optional on update) — HPKE envelope.

`payload.clearFields` (optional) — explicit clears.

`payload.ext` (optional).

## Examples

### Create a new password entry with sealed secret

```json
{
  "id": "vupsert-1234",
  "type": "https://trusttasks.org/spec/vault/upsert/0.3",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T11:00:00Z",
  "payload": {
    "contextId": "ctx_personal",
    "targets": [
      { "kind": "webOrigin", "origin": "https://news.ycombinator.com" }
    ],
    "label": "Hacker News",
    "secretKind": "password",
    "tags": ["personal", "social"],
    "sealedSecret": {
      "armored": "-----BEGIN TRUST-TASKS SEALED ENVELOPE-----\nBundle-Id: vault-upsert-…\nDigest-Algo: SHA-256\n\n…base64…\n=Z4eP\n-----END TRUST-TASKS SEALED ENVELOPE-----",
      "recipientKeyId": "did:key:z6LSj…",
      "producerAssertion": "didSigned"
    }
  },
  "proof": { "…": "…" }
}
```

### Update label and tags without rotating the secret

```json
{
  "id": "vupsert-2345",
  "type": "https://trusttasks.org/spec/vault/upsert/0.3",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T11:01:00Z",
  "payload": {
    "id": "vault_01HZX2QY…",
    "expectedVersion": 7,
    "contextId": "ctx_personal",
    "targets": [{ "kind": "webOrigin", "origin": "https://news.ycombinator.com" }],
    "label": "Hacker News — main",
    "secretKind": "password",
    "tags": ["personal", "social", "daily"],
    "clearFields": ["notes"]
  },
  "proof": { "…": "…" }
}
```

## Response

`payload.entry` — the resulting VaultEntry in metadata-only view (with updated `version`).

`payload.created` — true on create, false on update.

## Security & Privacy

### Data carried

**No cleartext secret on the wire.** The Trust Task wire form carries the secret
only as HPKE-sealed bytes inside `sealedSecret`, and the maintainer verifies the
envelope's authenticator before persisting — a failed verification yields
`vault/upsert:sealedSecretInvalid` with `details.reason` and **MUST NOT** mutate any
stored state. Maintainers **MUST** refuse documents that smuggle plaintext via `ext`
or by abusing optional metadata fields to carry credential material.

**The metadata is the leak, and it is not sealed.** Everything wrapped around the
envelope arrives in cleartext, is stored in cleartext, and is about a person:

* `targets[]` names every web origin, iOS bundle id, Android package, or relying-party
  DID the entry applies to. Accumulated across a vault this is the single most
  revealing structure the maintainer holds — the list of services a principal has
  an account with — and it is available without decrypting anything.
* `label` is a display name the schema illustrates with *"Work GitHub"* and
  *"Personal bank — checking"*. Those examples are the ordinary case, and both leak
  more than the origin alone does.
* `tags[]` are user-authored groupings; the schema's own example is
  `["family", "finance"]`, which classifies an account by area of life.
* `notes` is up to 4096 characters of free text. The shared schema calls it
  "non-sensitive notes" and suggests support contacts, account numbers, and expiry
  memos — but the field enforces nothing, and a member described as non-sensitive is
  exactly the one that fills with sensitive text. Genuinely sensitive notes belong
  inside the sealed payload as `secureNotes`, where only a
  [`vault/release`](../../release/0.2/spec.md) reveals them.
* `customFieldNames[]` is the subtlest. It tells the maintainer the *shape* of a
  credential it is expressly not permitted to read — that this entry has a field
  called `securityAnswerMothersMaidenName`, or `recoveryEmail` — which is inference
  the seal was supposed to prevent.

A producer **SHOULD** treat every unsealed member as visible to the maintainer
forever and put in it only what the maintainer needs to route, match, and render the
entry. `secretKind` is the deliberate exception: it is exposed precisely so consumers
can render the right affordance and policy can route by kind without unsealing.

### Correlation

The joins here are trivial and unavoidable. `contextId` partitions entries by persona
and is immutable on an entry — the schema requires a delete-and-recreate to move one —
so a maintainer can see the boundary between a principal's personas but cannot be
asked to forget it. Within a context, `targets[]` correlates directly to the outside
world: a maintainer holding a vault knows which relying parties its principal deals
with, and a relying party's presence is disclosed by the act of storing a credential
for it, before any login ever happens.

Across entries, `tags` and `label` are user-authored and habitually consistent, so
they cluster reliably even where `targets` do not overlap. And because
[`vault/get`](../../get/0.1/spec.md) and [`vault/list`](../../list/0.1/spec.md) return
this metadata without releasing secrets, the correlation surface is reachable by any
consumer holding read scope — a strictly larger set than those holding
`FillRelease`.

The `version` counter and `updatedAt` add a temporal trace: the sequence of upserts
against one entry records when a principal changed a password, which is behavioural
data the maintainer accrues simply by doing its job.

### Retention

Durable, without qualification. An entry persists until an explicit
[`vault/delete`](../../delete/0.1/spec.md); nothing in this payload expires it, and
`expiresAt` describes the credential's own validity rather than the record's lifetime.
That is what a vault is for, and the sealed secret's durability is not the
interesting half — the cleartext metadata is retained on identical terms, so a `label`
or a `notes` line written once outlives every rotation of the secret it describes.

`clearFields` is the only member that shortens retention, and it exists because
omission had to mean "don't touch": a consumer that wants a `notes` line gone
**MUST** name it in `clearFields`, since leaving it out of the payload preserves it.
Producers correcting an over-shared `notes` or `label` should know that an update
alone does not remove it.

**Rotation audit.** Every secret rotation is an audit event and the maintainer
**MUST** record `who, when, kind` — and **NOT** the new secret bytes. For `password`
kind the maintainer **MUST** set `passwordChangedAt` to the upsert time. The audit
record therefore outlives the secret deliberately, and is scoped so that it proves a
rotation happened without reproducing what was rotated to.

**Replay.** The `id` is used for idempotency: a maintainer **SHOULD** treat a retry of
the same document id as a no-op once the upsert has been applied, within an
idempotency window (recommended 24h). That window is itself retained state, and it is
bounded for the same reason.

### Consent/purpose

The data moves so that a maintainer can hold a credential on a principal's behalf and
act on it later at a named target. The `proof` is REQUIRED so the maintainer can
attribute the change to a specific consumer and record who introduced or rotated a
credential that other Companions will subsequently trust.

The purpose limit that matters is scope creep between consumers rather than between
uses. **`VaultWrite` granted to a Service consumer** (an AI agent, a sync daemon) is a
high-risk capability: that consumer can introduce or rotate credentials silently, and
a credential it introduces is one every other Companion will treat as the principal's
own. Maintainers **SHOULD** prefer narrow per-site capability grants over blanket
`VaultWrite` for Service consumers.

**Optimistic concurrency** is a purpose-integrity control as much as a correctness
one. `expectedVersion` is the only safe way to update — without it two Companions
racing on the same entry silently overwrite each other, and the loser's rotation is
lost with no party observing an error. Maintainers **MAY** accept blind updates only
where a consumer has explicitly opted in via ACL capability; this is a foot-gun and
**SHOULD NOT** be the default.
