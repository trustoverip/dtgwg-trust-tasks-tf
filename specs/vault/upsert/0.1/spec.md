---
slug: vault/upsert
version: "0.1"
title: Vault — Upsert
summary: A vault consumer creates a new vault entry or updates an existing one; secret material rides inside an HPKE-sealed envelope so the Trust Task itself carries only ciphertext.
status: draft
targetFrameworkVersion: "0.1"
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
errorCodes:
  - code: vault/upsert:context_not_found
    meaning: The supplied `contextId` does not exist.
    retryable: false
  - code: vault/upsert:permission_denied
    meaning: The consumer lacks VaultWrite on the target context.
    retryable: false
  - code: vault/upsert:not_found
    meaning: An `id` was supplied (update path) but no entry with that id exists in the consumer's visible scope.
    retryable: false
  - code: vault/upsert:version_conflict
    meaning: An `expectedVersion` was supplied and does not match the current version. The consumer SHOULD re-read the entry (vault/get) and retry the upsert with the up-to-date version.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        currentVersion: { type: "integer", minimum: 0 }
  - code: vault/upsert:sealed_secret_invalid
    meaning: The sealedSecret envelope failed verification (digest mismatch, signature invalid, recipient key unknown, or armor malformed).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: "string", enum: ["armor_malformed", "recipient_key_unknown", "signature_invalid", "digest_mismatch"] }
  - code: vault/upsert:secret_required
    meaning: A create or rotation was attempted without `sealedSecret` for a secretKind that requires one.
    retryable: false
  - code: vault/upsert:envelope_unsupported
    meaning: The `sealedSecret.envelope` kind is not one the maintainer implements (e.g. a TSP message arriving at a maintainer that only speaks `didcomm-authcrypt`). Producers SHOULD consult `trust-task-discovery/0.1` to learn which envelope kinds the maintainer accepts.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        receivedEnvelope: { type: "string" }
        supportedEnvelopes: { type: "array", items: { "type": "string" } }
  - code: vault/upsert:context_change_forbidden
    meaning: The consumer attempted to change `contextId` on an existing entry. Move-between-contexts MUST be done as delete + recreate.
    retryable: false
---

## Abstract

The **Vault — Upsert** Trust Task creates a new vault entry or updates an existing one. The secret material — when present — is wrapped in an HPKE-sealed envelope (`SealedSecret`) so the Trust Task wire form carries ciphertext + an authenticator only.

Updates may be partial: populated metadata fields replace current values; omitted fields are left untouched; fields listed in `clearFields` are explicitly cleared. Optimistic concurrency via `expectedVersion`.

## Conformance

A conforming **producer** **MUST**:

1. Populate `contextId`, `targets`, `label`, `secretKind`.
2. On create with a kind that carries secret bytes (everything except `did-self-issued` and `didcomm-peer`), populate `sealedSecret` with an HPKE envelope sealed to the maintainer's published recipient key.
3. On update where the secret is unchanged, omit `sealedSecret`. On rotation, populate it.
4. Supply `expectedVersion` on every update; omit on create.
5. **MUST NOT** include the cleartext `VaultSecret` anywhere in the payload — secrets travel only inside `sealedSecret`.
6. **MUST NOT** attempt to change `contextId` on an existing entry; delete and recreate instead.
7. Carry a `proof` over the document; the proof's verificationMethod MUST resolve via the producer's authenticated DID.

A conforming **consumer** **MUST**:

1. Verify the document `proof` and the consumer's `VaultWrite` capability on `contextId`.
2. On create: assign or accept an entry id, set `version = 1`, set `createdAt = createdBy = updatedAt = updatedBy = now / issuer`. Unseal `sealedSecret`, validate its inner `VaultSecret` against the `vault-secret.schema.json` schema for the declared `secretKind`. Persist.
3. On update: verify `expectedVersion` matches the stored version (atomic check-and-set); reject with `version_conflict` on mismatch with `details.currentVersion`. Apply field changes. Apply `clearFields`. If `sealedSecret` is populated, unseal, validate, replace stored secret, set `passwordChangedAt = now` (for `password` kind) or `lastRotatedAt` equivalent. Increment `version`.
4. Return the resulting `entry` (metadata view) and `created: bool`.
5. Emit a `sync/event/0.1` of kind `vault.upserted` to all other registered consumers in the maintainer's ACL with VaultRead on the entry's context. The triggering consumer SHOULD also receive the event for echo, so its local cache merges via the same code path.
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
  "type": "https://trusttasks.org/spec/vault/upsert/0.1",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T11:00:00Z",
  "payload": {
    "contextId": "ctx_personal",
    "targets": [
      { "kind": "web-origin", "origin": "https://news.ycombinator.com" }
    ],
    "label": "Hacker News",
    "secretKind": "password",
    "tags": ["personal", "social"],
    "sealedSecret": {
      "armored": "-----BEGIN TRUST-TASKS SEALED ENVELOPE-----\nBundle-Id: vault-upsert-…\nDigest-Algo: SHA-256\n\n…base64…\n=Z4eP\n-----END TRUST-TASKS SEALED ENVELOPE-----",
      "recipientKeyId": "did:key:z6LSj…",
      "producerAssertion": "did-signed"
    }
  },
  "proof": { "…": "…" }
}
```

### Update label and tags without rotating the secret

```json
{
  "id": "vupsert-2345",
  "type": "https://trusttasks.org/spec/vault/upsert/0.1",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T11:01:00Z",
  "payload": {
    "id": "vault_01HZX2QY…",
    "expectedVersion": 7,
    "contextId": "ctx_personal",
    "targets": [{ "kind": "web-origin", "origin": "https://news.ycombinator.com" }],
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

**No cleartext on the wire.** The Trust Task wire form carries only HPKE-sealed bytes for the secret. Maintainers MUST refuse documents that smuggle plaintext via `ext` or by abusing optional metadata fields to carry credential material.

**Optimistic concurrency.** `expectedVersion` is the only safe way to update — without it, two Companions racing on the same entry can silently overwrite each other. Maintainers MAY accept blind updates (omit `expectedVersion`) only when explicitly opted in via ACL capability; this is a foot-gun and should not be default.

**Rotation audit.** Every secret rotation is an audit event; the maintainer MUST record `who, when, kind` (NOT the new secret bytes). For `password` kind, the maintainer MUST set `passwordChangedAt` to the upsert time.

**Sealed envelope verification.** The maintainer MUST verify the envelope's authenticator (the `producerAssertion` mode) before persisting. A failed verification MUST result in `sealed_secret_invalid` with `details.reason` and MUST NOT cause any stored state to mutate.

**Permission scope.** `VaultWrite` granted to a Service consumer (AI agent, sync daemon) is a high-risk capability — that consumer can introduce or rotate credentials silently. Maintainers SHOULD prefer narrow per-site Capability grants over blanket VaultWrite for Service consumers.

**Replay.** The `id` is used for idempotency: a maintainer SHOULD treat a retry of the same document id as a no-op once the upsert has been applied (within an idempotency window — recommended 24h).
