---
slug: vault/proxy-login
version: "0.2"
wireCompatibleWith: "0.1"
title: Vault — Proxy Login
summary: A vault consumer asks the vault maintainer to perform a login at the bound third-party site on the holder's behalf, returning a session blob the consumer can use without ever seeing the long-term credential.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vault
  - credentials
  - proxy-login
  - session
  - authentication
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
  rationale: Proxy-login causes the maintainer to authenticate as the holder at a third party — a high-trust, audited action. The consumer's identity MUST be verifiable so the maintainer can attribute every session it creates on the holder's behalf to a specific Companion or Service.
sideEffects:
  level: mutating
  rationale: "Performs a login at the bound site on the holder's behalf and returns a session blob; the use is logged."
consequences:
  - "Acts as you at the third-party site and issues a usable session, without exposing the long-term credential."
subjectPath: /target
exposure:
  discloses: secret
  actsAsSubject: true
  rationale: "Logs in as the holder at the bound site and returns a usable session blob, exercising the bound credential."
errorCodes:
  - code: vault/proxy-login:notFound
    meaning: No entry with this id exists in the consumer's scope.
    retryable: false
  - code: vault/proxy-login:permissionDenied
    meaning: The consumer lacks ProxyLogin capability for this entry.
    retryable: false
  - code: vault/proxy-login:stepUpRequired
    meaning: Policy demands a step-up proof before the login can proceed. Consumer retries with `stepUpProof` populated.
    retryable: true
    detailsSchema:
      $ref: "../../_shared/0.2/consumer-context.schema.json#/$defs/StepUpChallenge"
  - code: vault/proxy-login:targetUnreachable
    meaning: The maintainer attempted the login at the third-party site but the site is unreachable, rate-limiting, or returned an unexpected response. Consumer SHOULD retry with backoff.
    retryable: true
  - code: vault/proxy-login:credentialRejected
    meaning: The maintainer attempted the login but the third party rejected the credential (wrong password, expired token, revoked passkey). Consumer SHOULD prompt the user to update the entry via vault/upsert.
    retryable: false
  - code: vault/proxy-login:notProxyable
    meaning: This entry cannot be proxy-logged-in (e.g. the site requires browser-bound channel binding). Consumer falls back to vault/release for a fill flow.
    retryable: false
  - code: vault/proxy-login:policyDeny
    meaning: Policy denies proxy-login for this consumer + entry combination outright (no step-up will satisfy it).
    retryable: false
  - code: vault/proxy-login:envelopeUnsupported
    meaning: The maintainer cannot emit a `sealedSessionBlob` in any envelope kind the consumer accepts. Producers SHOULD consult `trust-task-discovery/0.1` for the maintainer's emit set.
    retryable: false
    detailsSchema:
      $ref: "../../_shared/0.2/sealed-envelope.schema.json#/$defs/EnvelopeMismatch"
---

## Abstract

The **Vault — Proxy Login** Trust Task asks the maintainer to perform an authentication at the third-party site on the holder's behalf. The maintainer uses the entry's secret material (password, OAuth refresh token, passkey, SIOP key, etc.) entirely on its side, completes the authentication, and returns a `SessionBlob` — cookies, headers, optional localStorage entries — sealed for the requesting consumer's HPKE key.

The long-term credential never leaves the maintainer. The consumer learns only the session blob, which the maintainer scopes with a short TTL and binds to a specific origin.

For sites that cannot be proxy-logged-in (rare — browser-bound channel binding, anti-bot, captcha walls), the maintainer returns `notProxyable` and the consumer falls back to `vault/release/0.1` for a fill flow.

## Conformance

A conforming **producer** **MUST**:

1. Populate `entryId`. **MAY** populate `target` to disambiguate when the entry has multiple targets and the consumer's form factor doesn't uniquely identify the right one.
2. Populate `consumerContext` truthfully — populating false UV claims to bypass policy is a violation and maintainers MAY ban consumers caught doing so.
3. Carry a `proof`.
4. On `stepUpRequired`, satisfy the demanded method and retry with `stepUpProof` carrying the proof bytes and the challenge id.
5. For SIOPv2-shaped flows where the consumer will post the resulting credential to a relying party that issued an authorization-request `nonce`, **MUST** populate `nonce` with that value. Omitting it forces the maintainer to generate its own nonce — which the RP will reject during id_token verification.

A conforming **consumer** (the vault maintainer) **MUST**:

1. Verify proof and the consumer's `ProxyLogin` capability on the entry.
2. Cross-check `consumerContext.deviceId` against the authenticated transport identity. Discard any consumer-supplied field the maintainer can verify independently (e.g. the maintainer trusts its own record of the device's `last_seen_at`, not the consumer's `lastUserVerificationAt` blindly).
3. Evaluate the policy engine (Rego per the policy/* family) against `{ site, contextId, consumer, request: { kind: "proxyLogin" } }`. Possible outcomes: `allow + proxy`, `allow + fill` (returns `notProxyable` to nudge the consumer to use `vault/release` instead), `requireStepUp`, or `deny`.
4. On `requireStepUp`, mint a challenge id, return `stepUpRequired` with `details.method` and `details.challengeId`. On the consumer's retry, verify the proof against the challenge id; on success, proceed.
5. Perform the login at the third-party site using the entry's secret. The mechanism depends on `secretKind`:
   - `password`: HTTP form post / API call with username + password, then TOTP if seed present.
   - `passkey`: WebAuthn assertion using the stored credential — only viable when the third party accepts a non-browser WebAuthn flow (rare; usually falls through to `notProxyable`).
   - `oauthTokens`: refresh the access token if needed, return it as a header.
   - `didSelfIssued`: issue a SIOP id_token signed by the referenced key. If the consumer supplied `nonce`, embed it verbatim as the id_token's `nonce` claim; otherwise generate a fresh nonce. Drivers MUST treat the consumer's nonce as opaque (no canonicalisation, trimming, or re-encoding) so the RP's exact-match check succeeds.
   - `didcommPeer`: complete the DIDComm authentication handshake to the relying party.
   - `bearerToken`: simply return the token in the configured header.
6. Construct a `SessionBlob` with `expiresAt` set conservatively (RECOMMENDED ≤ the third party's session TTL, and never more than 1 hour for high-value sites). Seal with HPKE to the consumer's published recipient X25519 key.
7. Record `lastUsedAt = now` on the entry; emit a `sync/event/0.1` of kind `vaultUpserted` so other consumers see the update.
8. Audit-log the proxy-login with `{ who, when, entryId, sessionId, outcome }`.
9. On third-party failure: distinguish reachability (`targetUnreachable`, retryable) from credential rejection (`credentialRejected`, not retryable).
10. **MUST NOT** include the long-term credential in any field of the response.

## Payload

`payload.entryId` (REQUIRED).

`payload.target` (optional) — disambiguate multi-target entries.

`payload.consumerContext` (optional, RECOMMENDED).

`payload.stepUpProof` (REQUIRED on retry after step_up_required).

`payload.nonce` (optional) — caller-supplied nonce the maintainer embeds verbatim in the session credential when the driver has a nonce concept. The canonical use is SIOPv2: the RP's authorization-request `nonce` MUST appear as the `nonce` claim in the SIOP id_token the maintainer mints, or the RP rejects the token. Drivers without a nonce concept (Password POST, OAuth refresh, cookie-injection drivers) ignore the field. When omitted, the maintainer generates its own nonce; appropriate for push-mode flows that don't pre-fetch a challenge.

`payload.ttlSecondsHint` (optional) — caller-preferred session TTL in seconds; capped server-side. A higher hint MUST silently truncate, not reject. Drivers issuing fixed-TTL bearers (e.g. SIOP id_tokens with their own `exp`) MUST NOT extend beyond the underlying credential's lifetime regardless of hint.

## Response

`payload.sealedSessionBlob` — HPKE-sealed SessionBlob.

## Examples

### Initial request

```json
{
  "id": "plogin-1234",
  "type": "https://trusttasks.org/spec/vault/proxy-login/0.2",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T13:00:00Z",
  "payload": {
    "entryId": "vault_01HZX2QY…",
    "consumerContext": {
      "deviceId": "dev_01HZX3…",
      "lastUserVerificationAt": "2026-05-26T12:59:30Z",
      "networkClass": "home"
    }
  },
  "proof": { "…": "…" }
}
```

### SIOP-bound login with RP-supplied nonce

```json
{
  "id": "plogin-siop-1",
  "type": "https://trusttasks.org/spec/vault/proxy-login/0.2",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T13:00:00Z",
  "payload": {
    "entryId": "vault_01HZX2_did_self_issued",
    "target": { "kind": "did", "did": "did:web:rp.example" },
    "nonce": "5e3f… (the value returned by GET /auth/challenge at the RP)",
    "consumerContext": {
      "deviceId": "dev_01HZX3…",
      "lastUserVerificationAt": "2026-05-26T12:59:30Z"
    }
  },
  "proof": { "…": "…" }
}
```

The maintainer mints a SIOP id_token whose `nonce` claim is exactly the value supplied. The consumer posts the id_token (extracted from the returned `SessionBlob.headers[].Authorization`) to the RP's `/auth/` endpoint; the RP's nonce check succeeds.

### Step-up required → retry

Maintainer's first response:
```json
{
  "id": "plogin-resp-1234-err",
  "type": "https://trusttasks.org/spec/trust-task-error/0.2",
  "threadId": "plogin-1234",
  "issuer": "did:web:vta.example",
  "recipient": "did:peer:2.Ez6LSc…",
  "issuedAt": "2026-05-26T13:00:01Z",
  "payload": {
    "code": "vault/proxy-login:stepUpRequired",
    "details": { "method": "pushApproval", "challengeId": "ch_abc123", "ttlSeconds": 60 }
  }
}
```

Consumer retry with proof:
```json
{
  "id": "plogin-1234-retry",
  "type": "https://trusttasks.org/spec/vault/proxy-login/0.2",
  "issuer": "did:peer:2.Ez6LSc…",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T13:00:15Z",
  "payload": {
    "entryId": "vault_01HZX2QY…",
    "stepUpProof": {
      "kind": "pushApproval",
      "proof": "didcomm-msg-id-of-approval-grant",
      "challengeId": "ch_abc123"
    }
  },
  "proof": { "…": "…" }
}
```

## Security & Privacy

**Credential containment.** The long-term credential MUST stay on the maintainer side throughout. Maintainers MUST NOT log the credential, MUST NOT include it in any response field, MUST NOT cache it in memory beyond the duration of the login attempt. Defense in depth: implementations SHOULD wipe the credential from memory immediately after use.

**Session TTL discipline.** Short TTL is the primary defence against a compromised consumer replaying the session indefinitely. Maintainers SHOULD scope `expiresAt` to minutes for high-value sites, hours for low-value. The maintainer's policy is the source of truth, not the third party's intrinsic session TTL.

**Step-up trust.** When policy requires step-up, the maintainer MUST verify the proof on its own — not trust the consumer's claim that a UV happened. For WebAuthn UV, the consumer signs a maintainer-issued challenge with a key the maintainer can verify. For push-approval, the maintainer issues a DIDComm challenge to the holder's mobile Companion and waits for a signed approval.

**Audit reach.** Every proxy-login is logged. This is non-negotiable: a Service consumer (AI agent) with `ProxyLogin` can authenticate as the holder; without audit, it would be impossible to detect misuse.

**Replay.** The `id` is the maintainer's idempotency key. A retry of the same proxy-login `id` within the maintainer's idempotency window MUST return the same `sealedSessionBlob` (re-sealing to the same key), not perform a second login at the third party. Different `id` = new login.

**Policy hot-reload.** If policy is updated mid-session-window, the existing session blob remains valid until `expiresAt`. New proxy-login requests are evaluated against the updated policy.
