---
slug: auth/refresh
version: "0.2"
title: Auth — Refresh
summary: Exchange a refresh token for a new access token, without re-running the challenge-response handshake. 0.2 accepts a proof by the session key bound at login as an additional binding check, and never extends a session past its absolute lifetime.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - refresh
  - token
  - session
  - jwt
  - session-key
  - absolute-lifetime
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: "The refreshToken itself remains the authority; a proof on the document is redundant when the transport binds the producer's identity end-to-end, or when the consumer never bound a session key to begin with. This declaration is a floor, not the whole rule: per Conformance and Authorization, a consumer whose located session carries a bound `sessionKey` (auth/authenticate/0.2 or 0.3) MUST itself require a proof made by exactly that key — a per-session elevation the framework's single front-matter declaration cannot express, because it depends on server-side state the document alone does not carry."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A refresh trades a long-lived credential for a fresh session, so a captured refresh is a session-minting oracle for as long as it stays acceptable. The acceptance window is what closes it.
sideEffects:
  level: mutating
  rationale: "Exchanges a refresh token for a new access token; rotates session token state. Every rotation is bounded by the session's absolute lifetime (new in 0.2): no sequence of refreshes, however frequent, mutates a session past the ceiling its authenticate response established."
exposure:
  discloses: none
  actsAsSubject: false
  rationale: "Refresh mints a fresh access token for a session the subject already holds; it exercises no authority beyond continuing that session's existing scope and never widens it (scopeWideningRefused)."
errorCodes:
  - code: auth/refresh:tokenNotFound
    meaning: The refreshToken does not refer to any session the auth service issued.
    retryable: false
  - code: auth/refresh:tokenExpired
    meaning: The refreshToken's refreshExpiresIn has elapsed.
    retryable: false
  - code: auth/refresh:tokenRevoked
    meaning: The refreshToken was explicitly invalidated (typically via auth/revoke-session, or by a step-up event that rotated all session refresh tokens).
    retryable: false
  - code: auth/refresh:scopeWideningRefused
    meaning: The requested scope exceeds the original session's scope. Refresh MUST NOT broaden privilege.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        offending:
          type: array
          items: { type: string }
  - code: auth/refresh:sessionKeyProofRequired
    meaning: "The located session carries a bound `sessionKey` (established at auth/authenticate/0.2 or 0.3), but the request's `proof` is absent, or made by a key other than that bound `sessionKey` — including the subject's own long-term signing key. A session that bound a key MUST refresh with that key's proof; no other proof, and no absence of one, substitutes."
    retryable: false
  - code: auth/refresh:sessionLifetimeExceeded
    meaning: "The session has already reached its `absoluteExpiresAt`, or this refresh's ordinary rotation would need to move `expiresAt` past it. Refresh MUST NOT extend a session beyond its absolute lifetime by any increment; the producer MUST re-authenticate via auth/authenticate to obtain a new session."
    retryable: false
related:
  - auth/authenticate
  - auth/revoke-session
  - auth/whoami
---

## Abstract

The **Auth — Refresh** Trust Task exchanges a long-lived *refresh token* for a fresh short-lived *access token*, without re-running the challenge-response handshake. The refresh token serves as bearer authentication for this exchange; the consumer verifies it against its own session state.

This task does not change the session's *AAL* — refresh preserves whatever `amr` and `acr` the original authentication established. To elevate AAL, use [`auth/passkey/login/finish/0.1`](../../passkey/login/finish/0.1/spec.md) against the existing session, or run a [`auth/step-up/approve-request`](../../step-up/approve-request/0.1/spec.md) handshake.

**New in 0.2, two independent additions:**

1. **A session-key binding check.** [`auth/authenticate/0.2`](../../authenticate/0.2/spec.md) and [`0.3`](../../authenticate/0.3/spec.md) let a producer bind an ephemeral `did:key` `sessionKey` to the session at login. When the session that owns a presented `refreshToken` carries one, this task now requires a `proof` made by that key alongside the token — proof that the party presenting the token is the same browser session that logged in, not merely someone who came to hold the token. The bearer token remains the authority (see Authorization); the session-key proof is a binding check layered on top of it, never a substitute for it, and it is never itself sufficient to obtain anything.
2. **An absolute session lifetime.** A session established with `Session.absoluteExpiresAt` set (a consumer policy choice, not required by this specification) can no longer be refreshed indefinitely: this task's Conformance now requires that no refresh advance `Session.expiresAt` past `Session.absoluteExpiresAt`, and that a session which has already reached it refuse refresh outright with `sessionLifetimeExceeded` rather than silently capping the extension.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the subject) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/refresh/0.2`, with itself as `issuer` and the auth service as `recipient`.
2. Populate `payload.refreshToken` with the value previously received in a `TokenBundle`.
3. **MAY** include a `payload.scope` request, which MUST be a (non-strict) subset of the session's current scope. A consumer that detects widening MUST respond with `auth/refresh:scopeWideningRefused`.
4. **MUST** include a `proof` made by the session's bound `sessionKey`, with `proofPurpose: authentication`, whenever the producer holds one for this session — in practice, whenever a prior `auth/authenticate/0.2` or `/0.3` response echoed `session.sessionKey` back to it. A producer that never bound a session key omits `proof` exactly as in 0.1.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements). A `proof`, if present, is verified per [SPEC.md §4.7](/SPEC.md#47-proof) before it is relied on for step 3.
2. Look up the session associated with `payload.refreshToken`. Unknown → `auth/refresh:tokenNotFound`. Expired → `auth/refresh:tokenExpired`. Revoked → `auth/refresh:tokenRevoked`. None of these three consume the token — a lookup failure leaves whatever server-side state exists untouched.
3. Determine whether the located session carries a bound `Session.sessionKey`.
   - If it does **not**, proceed exactly as [0.1](../0.1/spec.md#conformance): a `proof`, if present, is informational; its absence is not an error.
   - If it **does**, require that `proof` be present and its `verificationMethod` resolve to exactly that `sessionKey`. Absent, or resolving to any other key — including the session's own `subject`'s ordinary long-term signing key — **MUST** be refused with `auth/refresh:sessionKeyProofRequired`, and **MUST NOT** consume the presented `refreshToken`: a producer that retries with the correct proof must still be able to succeed. This check happens before the token's single-use claim (step 5), so a rejected attempt never burns the token on the way to refusing it.
4. Compute the candidate new `expiresAt` under the consumer's ordinary rotation policy, then cap it: **MUST NOT** set the resulting `Session.expiresAt` later than `Session.absoluteExpiresAt`, when the session declares one. When the session's *current* `expiresAt` has already reached `absoluteExpiresAt` — so no cap-respecting extension is available at all — **MUST** refuse the refresh outright with `auth/refresh:sessionLifetimeExceeded` rather than silently returning a token whose window is zero or negative. There is no partial-credit response: a session past its absolute lifetime is refreshed by re-authentication, never by this task.
5. Issue a fresh access token. The consumer's policy decides whether to also rotate the refresh token: rotation is RECOMMENDED for tokens older than 24 h or after any suspicious-activity signal.
6. Preserve `Session.amr`, `Session.acr`, `Session.actor` (when present) and `Session.absoluteExpiresAt` (when declared) across the refresh unchanged — refresh does not elevate or downgrade AAL, does not change who the session's acting delegate is, and never itself moves the absolute ceiling.
7. Refuse with `auth/refresh:scopeWideningRefused` when `payload.scope` ⊄ session scope.
8. **MUST NOT** treat a verified `sessionKey` proof as authorization for anything beyond satisfying step 3 of *this* exchange — in particular MUST NOT accept it in place of an `assertionMethod` attestation anywhere one is required ([`auth/authenticate`](../../authenticate/0.3/spec.md#conformance) item 13 states the same rule for the key's other uses), and MUST NOT treat its presence as consent, approval, or a step-up. A session key never gains approval authority by appearing on a refresh document, exactly as it never gains it by appearing on any other.

## Authorization

*Per [SPEC.md §7.3 item 15](/SPEC.md#73-specification-requirements). This specification's `targetFrameworkVersion` is 0.4 or later, so the section is required; 0.1's equivalent note, written before that requirement existed, is superseded by this one.*

The primary authorization evidence for this task is unchanged from 0.1: **possession of the `refreshToken`**. The token is bearer material — the consumer looks up the session it names, and any party presenting a live, unrevoked token within the session's absolute lifetime is authorized to obtain a fresh access token for that session's subject and scope. There is no separate check of *who* is asking beyond that, in the ordinary case.

**0.2 adds a second, narrower piece of evidence for the session-key-bound case: possession of the session's `sessionKey` private half.** Where a session was established with one, holding the private key that signs `proof.proofPurpose: authentication` over this document proves the presenter is the same browser context that logged in — not a party who merely came to hold a copied or stolen `refreshToken`. This is a *binding* check, not an independent authorization: it answers "is this the same session's own client", never "is this request otherwise permitted." The `refreshToken` remains the thing that carries the authority to refresh at all; the `sessionKey` proof can only add a further requirement on top of it, never substitute for it, and — per Conformance item 8 — it confers no authority beyond this one check. **The session key MUST NOT be accepted anywhere as evidence of an approval, a consent, or a step-up**; that restriction is absolute and carries over unchanged from every other task in this family that touches a session key.

**The absolute session lifetime is not itself authorization evidence — it is a ceiling on what the `refreshToken`'s own authority can extend to.** A valid token, and (where required) a valid session-key proof, together authorize exactly one thing: a fresh access token whose window does not cross `Session.absoluteExpiresAt`. Neither credential, singly or combined, authorizes crossing it; there is no proof strong enough to extend a session past the ceiling its own `auth/authenticate` response set, because extending past it is not a thing this task is capable of authorizing at all — the producer's only path past the ceiling is a fresh `auth/authenticate`.

Where a consumer requires a `proof` on every refresh for its own audit reasons, per 0.1's own note, that proof still resolves through the subject's own DID document exactly as `auth/authenticate` requires — it is a stronger local policy, not a change to this specification's baseline.

## Definitions

* **Refresh token.** A long-lived opaque string issued in a prior `TokenBundle`. Consumer-internal correlation to a session.
* **Session.** The `Session` object the consumer holds for the subject; see [`_shared/0.3/session.schema.json`](../../_shared/0.3/session.schema.json).
* **Session key.** The optional `did:key` VID bound to a session at [`auth/authenticate/0.2` or `0.3`](../../authenticate/0.3/spec.md). Governed entirely by whichever authenticate version established it; this specification only adds one more place its proof is checked.
* **Absolute session lifetime.** `Session.absoluteExpiresAt` — the instant past which no refresh may advance `Session.expiresAt`, however it was set by the consumer at authentication time.
* **Access token.** The short-lived bearer credential. Consumers SHOULD pick lifetimes between 5 min and 1 h.

## Payload

`payload.refreshToken` (REQUIRED) — the refresh token value. Unchanged from 0.1.

`payload.scope` (optional) — narrower scope request; MUST NOT broaden. Unchanged from 0.1.

`payload.ext` (optional) — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

0.2 adds no new payload member. The session-key check operates on the framework `proof` member, which SPEC.md already defines on every Trust Task document; see Conformance.

## Examples

### Standard refresh (no bound session key)

```json
{
  "id": "5e6f7890-1234-5678-9012-3456abcdef78",
  "type": "https://trusttasks.org/spec/auth/refresh/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:14:00Z",
  "payload": {
    "refreshToken": "rt_8f2c1d4e9a7b3056"
  }
}
```

### Refresh with narrower scope

```json
{
  "id": "6f789012-3456-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/refresh/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:14:00Z",
  "payload": {
    "refreshToken": "rt_8f2c1d4e9a7b3056",
    "scope": ["acl:read"]
  }
}
```

### Refresh of a session-key-bound session (valid)

The browser's console logged in with `auth/authenticate/0.2` and was handed back `session.sessionKey`. It now refreshes, proving possession of that same key alongside the token.

```json
{
  "id": "7a8b9c0d-2345-6789-0123-4567890abcde",
  "type": "https://trusttasks.org/spec/auth/refresh/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:14:00Z",
  "payload": {
    "refreshToken": "rt_8f2c1d4e9a7b3056"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "created": "2026-05-23T10:14:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7mn…"
  }
}
```

### Refresh of a session-key-bound session rejected — wrong key (invalid)

Same session and token as above, but the `proof` resolves to the subject's own long-term `did:web` signing key rather than the bound `sessionKey`. This is refused even though the key genuinely belongs to the subject — the point of the check is that it is not *this session's* bound key:

```json
{
  "id": "8b9c0d1e-3456-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/refresh/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:14:00Z",
  "payload": {
    "refreshToken": "rt_8f2c1d4e9a7b3056"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T10:14:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

The auth service refuses with `auth/refresh:sessionKeyProofRequired` — the correct remediation is to sign with the session's own `sessionKey`, not with any other key the subject happens to control.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/refresh/0.2#response`. The payload is `{ tokens, session? }`. Consumers SHOULD include the `session` snapshot so the client can reconcile state — including `absoluteExpiresAt`, when the consumer declares one — without a separate `auth/whoami` call.

### Successful refresh with rotated refresh token

```json
{
  "id": "78901234-5678-9012-3456-7890abcdef12",
  "type": "https://trusttasks.org/spec/auth/refresh/0.2#response",
  "threadId": "5e6f7890-1234-5678-9012-3456abcdef78",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:14:01Z",
  "payload": {
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T10:29:01Z",
      "absoluteExpiresAt": "2026-05-24T10:00:31Z",
      "amr": ["did"],
      "acr": "aal1"
    },
    "tokens": {
      "accessToken": "eyJhbGciOiJFZERTQSI…",
      "refreshToken": "rt_9b3e2c5fa8d41067",
      "tokenType": "Bearer",
      "expiresIn": 900,
      "refreshExpiresIn": 86400
    }
  }
}
```

### Refused — absolute session lifetime reached

The same session, refreshed one more time after `expiresAt` has already caught up to `absoluteExpiresAt`:

```json
{
  "id": "9c0d1e2f-4567-8901-2345-67890abcdef1",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "a1b2c3d4-5678-9012-3456-7890abcdef34",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-24T10:00:31Z",
  "payload": {
    "code": "auth/refresh:sessionLifetimeExceeded",
    "message": "Session ec5d3c89… has reached its absolute lifetime (2026-05-24T10:00:31Z); re-authenticate to continue."
  }
}
```

## Security & Privacy

### Data carried

The `refreshToken` is bearer material — both in the request and, when rotated, in the response — and is the entire trust burden of the ordinary case. The `proof` this version adds carries no new secret: a `sessionKey` is public key material, exactly as in `auth/authenticate`, and its presence on the wire discloses nothing beyond "this request was signed by the session's bound key." Consumers MUST require transport-level confidentiality (TLS, DIDComm authcrypt) for the exchange regardless, because the token itself is the thing an eavesdropper would want.

### Correlation

A refresh carries the same `refreshToken` on every use until rotation, so a consumer that logs raw tokens (rather than a hash or the session id they resolve to) creates an unnecessary long-lived correlation handle; logging the resolved `Session.id` instead loses nothing an operator needs. The `sessionKey` proof, when present, correlates exactly as `auth/authenticate`'s Security & Privacy already describes: within the one session it was bound to, and nowhere else.

### Retention

Consumers SHOULD implement refresh-token rotation and retain enough of the consumed token's identity (a hash, not the plaintext) to detect a replay of it after rotation — the mechanism 0.1 already describes. **The absolute lifetime is not a retention question**: `Session.absoluteExpiresAt`, once set at authentication, is retained for exactly the session's own remaining life and is never itself extended, rotated, or renewed by anything this task does.

### Consent/purpose

**Token theft is bounded exactly as in 0.1**, with one addition: where a session bound a `sessionKey`, a stolen `refreshToken` alone is no longer sufficient — the thief also needs the session's private `sessionKey`, which per `auth/authenticate`'s own design is generated fresh per login and held non-extractably where the platform allows it. **This is a real narrowing of the theft surface, not a complete defence**: a thief who compromises the browser context itself (not merely exfiltrates the token, e.g. malicious code running alongside the session) can still use the resident, non-extractable key to sign — the non-extractable property defends against key *export*, not against a compromised runtime invoking it. Consumers SHOULD still treat a replay of a consumed (rotated-away) refresh token as a theft signal and revoke the session, exactly as 0.1 recommends, whether or not a session key is bound.

**A stolen session key alone refreshes nothing.** Per Authorization, the `sessionKey` proof is a binding check layered on the bearer token, never a substitute for it: a party holding only the session key and not the `refreshToken` cannot complete this exchange. Combined with `auth/authenticate` Conformance item 13 (never accepted as an `assertionMethod` attestation), a session key compromised on its own — without the refresh token alongside it — grants an attacker nothing this task, or the approval surfaces the session key is barred from, will honor.

**The absolute lifetime is what finally bounds a fully compromised session.** Token theft and session-key theft are both detectable-but-not-immediate risks; `auth/revoke-session` is the deliberate response once theft is suspected, but a theft that goes unnoticed is otherwise bounded only by how long the session's tokens keep being accepted. `Session.absoluteExpiresAt`, once a consumer adopts it, is the backstop that makes "unnoticed forever" impossible: whatever a stolen `refreshToken` and a stolen `sessionKey` together can do, they cannot do it past the ceiling the legitimate login itself set, and after that instant only a fresh `auth/authenticate` — which asks the subject's own key to prove control all over again — can continue the session's line at all.
