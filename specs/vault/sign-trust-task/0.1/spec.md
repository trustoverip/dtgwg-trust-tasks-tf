---
slug: vault/sign-trust-task
version: "0.1"
title: Vault — Sign Trust Task
summary: A vault consumer asks the vault maintainer to attach a Data Integrity proof to a Trust Task envelope, signing as the principal DID of a `did-self-issued` (or `didcomm-peer`) vault entry. The long-term signing key never leaves the maintainer.
status: draft
targetFrameworkVersion: "0.1"
category: credentials
keywords:
  - vault
  - credentials
  - signing
  - data-integrity
  - siop
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
  rationale: Sign-trust-task causes the maintainer to sign an arbitrary Trust Task envelope as the holder's principal DID — equivalent in power to a session-mint, performed inline whenever the consumer needs to issue a follow-up task in an already-authenticated session. The consumer's identity MUST be verifiable so every signature the maintainer produces is attributable to a specific Companion or Service.
errorCodes:
  - code: vault/sign-trust-task:not_found
    meaning: No entry with this id exists in the consumer's scope.
    retryable: false
  - code: vault/sign-trust-task:permission_denied
    meaning: The consumer lacks `SignTrustTask` capability for this entry.
    retryable: false
  - code: vault/sign-trust-task:not_signable
    meaning: The entry's `secretKind` has no DID-based signing identity (`password`, `passkey`, `oauth-tokens`, `bearer-token`, `ssh-key`, `custom`). Only `did-self-issued` and `didcomm-peer` entries can sign Trust Tasks.
    retryable: false
  - code: vault/sign-trust-task:envelope_invalid
    meaning: The supplied `unsignedEnvelope` is missing a framework-required field (`id`, `type`, `issuer`, `recipient`, `issuedAt`, `payload`) or carries fields the maintainer cannot canonicalise.
    retryable: false
  - code: vault/sign-trust-task:envelope_issuer_mismatch
    meaning: The supplied envelope's `issuer` does not match the entry's `principalDid`. The maintainer refuses to sign — the consumer MUST set `issuer = principalDid` for the entry being used. This guards against the consumer accidentally requesting a signature for an issuer the maintainer can't actually authenticate as.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        envelopeIssuer: { type: "string" }
        expectedIssuer: { type: "string" }
  - code: vault/sign-trust-task:envelope_already_proofed
    meaning: The supplied envelope already carries a `proof`. The maintainer refuses to re-sign — strip the existing proof and resubmit.
    retryable: false
  - code: vault/sign-trust-task:envelope_expired
    meaning: The supplied envelope's `expiresAt` is in the past. Signing it would produce a stale credential.
    retryable: false
  - code: vault/sign-trust-task:step_up_required
    meaning: Policy demands a step-up proof before the signature can be issued. Consumer retries with `stepUpProof` populated. Same shape as `vault/proxy-login:step_up_required`.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["method", "challengeId"]
      properties:
        method: { type: "string", enum: ["webauthn-uv", "push-approval", "totp"] }
        challengeId: { type: "string" }
        ttlSeconds: { type: "integer", minimum: 1 }
  - code: vault/sign-trust-task:policy_deny
    meaning: Policy denies sign-trust-task for this consumer + entry combination outright (no step-up will satisfy it).
    retryable: false
---

## Abstract

The **Vault — Sign Trust Task** Trust Task asks the maintainer to attach a Data Integrity proof to a Trust Task envelope, signing as the principal DID of a `did-self-issued` or `didcomm-peer` vault entry. The long-term signing key never leaves the maintainer.

This task complements `vault/proxy-login/0.1`: proxy-login mints a *session credential* (a SIOPv2 id_token, an OAuth bearer, etc.) at session-start time; sign-trust-task signs *individual Trust Tasks* the consumer needs to issue *during* an already-authenticated session.

### Motivating use case

A consumer logs in to a relying party via `vault/proxy-login/0.1`. The RP authenticates the resulting session as the vault entry's `principalDid` (the `iss`/`sub` of the SIOP id_token). The consumer then needs to send the RP a follow-up Trust Task — for example `acl/grant/0.1` to add a new admin. The RP's verifier requires the task's `proof.verificationMethod` to match the authenticated session's DID. If the consumer signs the task with its own holder DID (the wallet's general identity), the RP rejects with `proof_invalid`: "proof verificationMethod DID does not match the authenticated caller."

`vault/sign-trust-task/0.1` closes that gap: the consumer hands the unsigned task to the maintainer, the maintainer signs as `principalDid`, the resulting envelope is acceptable to the RP.

### Difference from `keys/sign` and the maintainer's signing oracle

A maintainer's generic signing oracle (e.g. `POST /keys/{key_id}/sign` in a typical VTA) signs *arbitrary bytes*. `vault/sign-trust-task/0.1` is a higher-level operation: the maintainer parses the envelope, validates it conforms to the framework, validates `issuer == principalDid`, and emits a *canonical* `eddsa-jcs-2022` Data Integrity proof. The consumer doesn't have to know how to canonicalise the envelope or how to format the proof block. This is to vault what `proxy-login` is to the underlying SIOP id_token signing — a vault-scoped, audited, policy-gated, capability-checked wrapper.

## Conformance

A conforming **producer** **MUST**:

1. Populate `entryId` with a vault entry the producer knows is `did-self-issued` or `didcomm-peer`. Other `secretKind`s are rejected with `not_signable`.
2. Populate `unsignedEnvelope` with a complete Trust Task envelope per the framework (§4.x of SPEC.md): `id`, `type`, `issuer`, `recipient`, `issuedAt`, `payload` are REQUIRED; `threadId`, `expiresAt`, `ext` are OPTIONAL. The envelope MUST NOT carry a `proof`.
3. Set `unsignedEnvelope.issuer` to the entry's `principalDid`. The producer learns `principalDid` from the entry metadata (`vault/list/0.1` / `vault/get/0.1` returns it; the proxy-login response also carries it implicitly via the minted id_token's `iss`).
4. Carry a `proof`.
5. On `step_up_required`, satisfy the demanded method and retry with `stepUpProof`.

A conforming **consumer** (the vault maintainer) **MUST**:

1. Verify proof and the consumer's `SignTrustTask` capability on the entry.
2. Verify `entry.secretKind` is `did-self-issued` or `didcomm-peer`. Reject with `not_signable` otherwise.
3. Verify `unsignedEnvelope.issuer == entry.principalDid`. Reject with `envelope_issuer_mismatch` on mismatch.
4. Verify `unsignedEnvelope` has no `proof` member. Reject with `envelope_already_proofed` if present.
5. Verify the envelope's framework-required fields (`id`, `type`, `issuer`, `recipient`, `issuedAt`, `payload`) are all present and well-typed. Reject with `envelope_invalid` otherwise. The maintainer is NOT obligated to validate the inner `payload` against the task type's schema — that's the recipient's job. The maintainer signs the envelope as it stands.
6. If `expiresAt` is present and in the past, reject with `envelope_expired`.
7. Evaluate the policy engine against `{ entry, consumer, envelope: { type, recipient }, request: { kind: "sign_trust_task" } }`. Possible outcomes: `allow`, `require_step_up`, `deny`.
8. On allow, JCS-canonicalise the envelope (proof slot first set to the proof's metadata without `proofValue`, per the eddsa-jcs-2022 Data Integrity rules), sign with the entry's signing key, attach `proof` to the envelope, return.
9. The proof's `verificationMethod` MUST be `<principalDid>#<signingKeyId>` (same shape the proxy-login id_tokens use). `proof.proofPurpose` MUST be `assertionMethod`. `proof.cryptosuite` MUST be `eddsa-jcs-2022`.
10. Audit-log the sign with `{ who, when, entryId, envelope: { id, type, recipient }, outcome }`. The audit MUST NOT include the envelope's `payload` (which may carry sensitive RP-side data).

## Payload

`payload.entryId` (REQUIRED).

`payload.unsignedEnvelope` (REQUIRED) — the Trust Task envelope to sign. MUST NOT carry a `proof`. MUST set `issuer = principalDid`.

`payload.consumerContext` (optional).

`payload.stepUpProof` (REQUIRED on retry after `step_up_required`).

## Response

`payload.signedEnvelope` — the supplied envelope with a `proof` attached.

## Examples

### Initial request

```json
{
  "id": "sttsk-1234",
  "type": "https://trusttasks.org/spec/vault/sign-trust-task/0.1",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-27T08:48:00Z",
  "payload": {
    "entryId": "vault_01HZX2_did_self_issued",
    "unsignedEnvelope": {
      "id": "urn:uuid:b11a487f-e98c-4929-9ab9-c5dd210499d6",
      "type": "https://trusttasks.org/spec/acl/grant/0.1",
      "issuer": "did:webvh:QmTenant…:host.example:work-persona",
      "recipient": "did:webvh:QmRp…:host.example",
      "issuedAt": "2026-05-27T08:48:11.163Z",
      "payload": {
        "entry": {
          "subject": "did:example:test",
          "role": "admin"
        }
      }
    }
  },
  "proof": { "…": "…" }
}
```

The `unsignedEnvelope.issuer` (`did:webvh:…:work-persona`) matches the principalDid of the referenced vault entry. The maintainer signs and returns.

### Response

```json
{
  "id": "sttsk-resp-1234",
  "type": "https://trusttasks.org/spec/vault/sign-trust-task/0.1",
  "threadId": "sttsk-1234",
  "issuer": "did:web:vta.example",
  "recipient": "did:peer:2.Ez6LSc…",
  "issuedAt": "2026-05-27T08:48:00Z",
  "payload": {
    "signedEnvelope": {
      "id": "urn:uuid:b11a487f-e98c-4929-9ab9-c5dd210499d6",
      "type": "https://trusttasks.org/spec/acl/grant/0.1",
      "issuer": "did:webvh:QmTenant…:host.example:work-persona",
      "recipient": "did:webvh:QmRp…:host.example",
      "issuedAt": "2026-05-27T08:48:11.163Z",
      "payload": { "entry": { "subject": "did:example:test", "role": "admin" } },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:QmTenant…:host.example:work-persona#z6Mk…",
        "created": "2026-05-27T08:48:00.123Z",
        "proofPurpose": "assertionMethod",
        "proofValue": "z47…"
      }
    }
  },
  "proof": { "…": "…" }
}
```

The consumer extracts `payload.signedEnvelope` and forwards it to the RP. The RP's verifier sees `proof.verificationMethod` = principalDid = authenticated session DID → accepts.

### Issuer mismatch → error

Consumer request with wrong issuer:

```json
{
  "payload": {
    "entryId": "vault_01HZX2_did_self_issued",
    "unsignedEnvelope": {
      "issuer": "did:key:z6MkHolder…",
      "…": "…"
    }
  }
}
```

Maintainer's first response:

```json
{
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "sttsk-1234-bad",
  "payload": {
    "code": "vault/sign-trust-task:envelope_issuer_mismatch",
    "message": "envelope.issuer must equal the entry's principalDid",
    "details": {
      "envelopeIssuer": "did:key:z6MkHolder…",
      "expectedIssuer": "did:webvh:QmTenant…:host.example:work-persona"
    }
  }
}
```

The producer is expected to set `unsignedEnvelope.issuer` correctly on retry. The maintainer does not silently fix this — see Security & Privacy.

## Security & Privacy

**Key containment.** The principal's long-term signing key MUST stay on the maintainer side throughout. Maintainers MUST NOT log the key, MUST NOT include it in any response field, MUST NOT export it to the consumer. The maintainer's signing oracle (per implementation) wipes the key from memory immediately after signing.

**Strict issuer match.** The maintainer MUST refuse to sign envelopes whose `issuer` differs from the entry's `principalDid`, rather than silently overwriting. Silent overwrite would mask consumer bugs (e.g. the consumer thinks it's asking for a signature as DID X but the maintainer signs as Y) and could surface as cryptic verification failures at the recipient. Explicit mismatch failure forces the consumer to make the issuer choice deliberately.

**Proof-already-present rejection.** The maintainer MUST refuse to sign an envelope that already has a `proof`. Re-signing a proofed envelope would either replace existing crypto material the consumer didn't author or chain proofs (unsupported by the framework). The consumer strips and resubmits if it needs to re-sign.

**Payload privacy in audit logs.** The envelope's `payload` is omitted from the maintainer's audit record because it can carry sensitive task-specific content (an ACL grant subject, a vault entry update, etc.). The audit records only the *fact* of the signature plus the envelope's `id`, `type`, and `recipient` — enough to correlate with other side-channel observations.

**Step-up disposition.** Sign-trust-task SHOULD require step-up at parity with `vault/proxy-login` — both produce credentials capable of authorising further actions at the RP. The default policy SHOULD demand recent UV ≤ 60 seconds.

**Replay.** The maintainer's `id` is the idempotency key. A retry of the same sign-trust-task request id within the maintainer's idempotency window returns the same signed envelope (with the same `proof.created` timestamp). Different `id` = new signature (which will differ in `proof.created` and `proof.proofValue`).

**Issuer-DID exfiltration.** A misconfigured consumer that submits arbitrary envelopes risks the maintainer signing tasks the operator did not intend. Defence in depth:
* `SignTrustTask` is a separate capability from `ProxyLogin`. Operators can grant proxy-login without sign-trust-task to limit blast radius.
* Policy MAY restrict signing to a configured list of `envelope.type` values per consumer (e.g. an AI Agent Service might be limited to `vault/upsert/0.1` and `auth/refresh/0.1` only).
* Every signature is audited with `{ envelope.id, envelope.type, recipient }` so unexpected `type` values are detectable post-hoc.

**Relationship to `proxy-login` nonce.** `proxy-login` carries an optional `nonce` that the maintainer embeds in the minted SIOP id_token. Sign-trust-task does not have a `nonce` field at the envelope level — if the recipient needs an anti-replay nonce, it lives in the task's `payload` (defined by the recipient's task type) and the consumer is responsible for populating it before submission. The maintainer just signs what's there.
