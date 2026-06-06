---
slug: vault/release
version: "0.1"
title: Vault — Release
summary: A vault consumer requests the cleartext secret material of an entry; the maintainer returns it inside an HPKE-sealed envelope with a strict cache TTL. The fallback when proxy-login is not viable.
status: draft
targetFrameworkVersion: "0.1"
category: credentials
keywords:
  - vault
  - credentials
  - release
  - autofill
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
  rationale: Release transfers long-term secret material to the consumer. Even though wrapped in HPKE, the consumer becomes the secret's custodian for the TTL window. The producer's identity MUST be verifiable for audit and so policy can enforce per-consumer release rules.
errorCodes:
  - code: vault/release:not_found
    meaning: No entry with this id exists in the consumer's scope.
    retryable: false
  - code: vault/release:permission_denied
    meaning: The consumer lacks FillRelease capability for this entry.
    retryable: false
  - code: vault/release:step_up_required
    meaning: Policy demands a step-up proof. Same shape as vault/proxy-login:step_up_required.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["method", "challengeId"]
      properties:
        method: { type: "string", enum: ["webauthn-uv", "push-approval", "totp"] }
        challengeId: { type: "string" }
        ttlSeconds: { type: "integer", minimum: 1 }
  - code: vault/release:policy_deny
    meaning: Policy refuses to release this secret to this consumer.
    retryable: false
  - code: vault/release:envelope_unsupported
    meaning: The consumer's published recipient key advertises envelope kinds the maintainer does not implement (e.g. consumer requests `tsp-message` against a maintainer that only emits `didcomm-authcrypt`). Producers SHOULD consult `trust-task-discovery/0.1` for the maintainer's emit set.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        requestedEnvelope: { type: "string" }
        supportedEnvelopes: { type: "array", items: { "type": "string" } }
---

## Abstract

The **Vault — Release** Trust Task is the fallback when `vault/proxy-login/0.1` cannot be used: the consumer needs the raw secret bytes (autofill into a desktop app, browser-bound channel binding at the third party, copy-to-clipboard, etc.). The maintainer returns the secret in an HPKE-sealed envelope with a short TTL the consumer is required to enforce.

Release is the higher-risk path. Use proxy-login when possible.

## Conformance

A conforming **producer** **MUST**:

1. Populate `entryId`. **MAY** populate `target` and `consumerContext`.
2. Carry a `proof`.
3. **MUST** enforce the maintainer's returned `ttlSeconds` — wipe the cleartext from memory after that window.
4. **MUST NOT** persist the cleartext beyond the TTL — not to disk, not to logs, not to syncing storage.
5. On `step_up_required`, satisfy the demanded method and retry with `stepUpProof`.

A conforming **consumer** (the vault maintainer) **MUST**:

1. Verify proof and `FillRelease` capability on the entry.
2. Evaluate policy. Possible outcomes: `allow`, `require_step_up`, `deny`.
3. On allow: unseal the stored secret, validate against `vault-secret.schema.json` for the entry's `secretKind`, re-seal under HPKE to the consumer's published recipient key.
4. Cap `ttlSeconds` at the maintainer's policy ceiling (RECOMMENDED ≤ 300 seconds for `password`, ≤ 60 for `bearer-token` and `oauth-tokens`, ≤ 30 for `ssh-key`).
5. Record `lastUsedAt = now` on the entry; emit a `sync/event/0.1` of kind `vault.upserted`.
6. Audit-log the release with `{ who, when, entryId, ttlSeconds, outcome }` — NOT the secret bytes.

## Payload

`payload.entryId` (REQUIRED).

`payload.target` (optional).

`payload.consumerContext` (optional).

`payload.stepUpProof` (REQUIRED on retry).

`payload.ttlSecondsHint` (optional) — caller's preferred TTL; maintainer caps.

## Response

`payload.sealedSecret` — HPKE-sealed VaultSecret.

`payload.secretKind` — discriminator.

`payload.ttlSeconds` — enforced cache TTL.

## Security & Privacy

**TTL is contractual.** The consumer is bound to wipe within `ttlSeconds`. Maintainers SHOULD assume a non-compliant consumer cannot be detected directly — defense is in the TTL ceiling itself (short windows minimise exposure) plus device attestation and ACL revocation when misuse is suspected.

**Prefer proxy-login.** Whenever the maintainer can do the login itself, it should — release is the last resort. Consumers SHOULD attempt `vault/proxy-login` first and only fall back to `release` on `not_proxyable`.

**Audit reach.** Every release is logged. AI Agent consumers with `FillRelease` are high-risk; maintainers SHOULD prefer narrow per-site capability grants for them.

**Step-up disposition.** Release SHOULD require step-up more aggressively than proxy-login does, because the consumer holds the secret. The default policy SHOULD demand recent UV ≤ 30 seconds for any release.

**Replay.** The `id` is the maintainer's idempotency key (same semantics as proxy-login). A retry within the idempotency window returns the same sealed material with the same TTL; the TTL does not reset on retry.
