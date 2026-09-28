---
slug: auth/authenticate
version: "0.2"
title: Auth — Authenticate
summary: A subject presents a previously-issued challenge in a proof-bearing document; the proof verifies the subject's VID, and the auth service responds with a session + tokens. 0.2 lets the subject bind a did:key session key so later calls need no further wallet prompt.
status: draft
targetFrameworkVersion: "0.5.0"
category: authentication
keywords:
  - auth
  - authentication
  - did
  - login
  - challenge-response
  - jwt
  - session-key
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
  requirement: REQUIRED
  rationale: The framework `proof` is the authentication. Without a verified proof binding the document to the subject's VID, the auth service has no basis to issue a session. Because `sessionKey` sits inside `payload`, this same proof is what entitles the subject to bind that key — a party cannot register a session key without signing for it with the key `auth/authenticate` already trusts.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An authentication request that cannot be placed in time is a credential presentation with no expiry, which is precisely the form a captured request is replayed in. Here the acceptance window is the primary defence, not a secondary one.
sideEffects:
  level: mutating
  rationale: "Establishes an authenticated session and issues tokens; the session is revocable state. 0.2 adds one more piece of revocable state to the same session: the optional session-key binding, which ends exactly when the session does."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/authenticate:challengeNotFound
    meaning: The `sessionId` does not refer to any challenge the auth service issued, or the challenge was already consumed.
    retryable: false
  - code: auth/authenticate:challengeExpired
    meaning: The challenge's expiresAt is in the past.
    retryable: true
  - code: auth/authenticate:challengeMismatch
    meaning: The presented `challenge` value does not equal the one the auth service bound to `sessionId`.
    retryable: false
  - code: auth/authenticate:subjectMismatch
    meaning: The `issuer` of the authenticate document does not equal the `subject` the challenge was bound to.
    retryable: false
  - code: auth/authenticate:scopeDenied
    meaning: One or more requested scopes were refused by the consumer's authorization policy. `details.refused` MAY enumerate the denied scopes.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        refused:
          type: array
          items: { type: string }
  - code: auth/authenticate:sessionKeyUnsupported
    meaning: The consumer does not support the key type or DID method of the requested `sessionKey` (for example, a `did:key` encoding a curve the consumer's verifier does not implement) and refuses the request rather than silently authenticating without the binding. `details.requested` MAY echo the offending `sessionKey`.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        requested:
          type: string
related:
  - auth/challenge
  - auth/refresh
  - auth/revoke-session
  - auth/whoami
  - auth/passkey/login/finish
  - auth/step-up/approve-response
---

## Abstract

The **Auth — Authenticate** Trust Task is the second half of a challenge-response authentication. The subject signs a document carrying the challenge they received from [`auth/challenge/0.1`](../../challenge/0.1/spec.md); the framework `proof` on that document, verified against the subject's VID, IS the authentication. The auth service replies with a *Session* and *TokenBundle*.

This task does NOT mint tokens itself — it requests them. The auth service applies its own authorization policy (ACL, allowed scopes, AAL ceiling) before responding.

**New in 0.2: an optional session key.** A wallet-backed subject signs this document with a key its holder has to approve interactively (a browser extension prompt, a hardware signer). Requiring that same interaction for every subsequent call defeats the point of having a session at all. 0.2 lets the producer name a `did:key` — freshly generated for this login, held non-extractably where the platform allows it — in `payload.sessionKey`. The auth service binds it to the session it is about to create, and from then on accepts a `proof` made by that key, with `proofPurpose: authentication`, as speaking for the subject *for that session*: bounded by the session's `expiresAt` and `acr`, and ending the moment the session does. This is the same shape the passkey login path already has (a browser-bound key stands in for the subject once), generalized to the wallet-login path where none previously existed.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the subject) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/authenticate/0.2`, with itself as `issuer` and the auth service as `recipient`.
2. Echo `payload.challenge` and `payload.sessionId` verbatim from the `auth/challenge` response.
3. Include a `proof` member per [SPEC.md §4.7](/SPEC.md#47-proof). The proof's `verificationMethod` MUST resolve via the issuer's DID document.
4. **MAY** request specific `payload.scope` capabilities. The producer MUST be prepared for the consumer to issue a token bundle with a narrower `scope`.
5. **MAY** include `payload.sessionKey`, a `did:key` VID freshly generated for this login. A producer that includes one **MUST** hold the corresponding private key and **SHOULD** keep it non-extractable — for example, generated as a WebCrypto `CryptoKey` with `extractable: false` — so that a compromise of the producer's storage cannot exfiltrate the key, only use it while resident.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`. The proof being absent or invalid is a hard failure.
2. Look up the server-side binding for `payload.sessionId`. If no binding exists, respond with `auth/authenticate:challengeNotFound`.
3. Compare the binding's stored challenge to `payload.challenge` using a constant-time comparator. Mismatch → `auth/authenticate:challengeMismatch`.
4. Reject expired bindings with `auth/authenticate:challengeExpired`.
5. When the challenge was issued with a bound `subject`, verify the document's `issuer` equals that subject. Mismatch → `auth/authenticate:subjectMismatch`.
6. Consume the challenge. The same `(sessionId, challenge)` pair MUST NOT be honored a second time, even on identical authenticate documents.
7. Apply the consumer's authorization policy. Refused scopes → `auth/authenticate:scopeDenied` with `details.refused`.
8. Issue a `#response` document carrying a freshly-created `Session` (with `amr` containing at least `"did"` and `acr` defaulting to `"aal1"`) and a `TokenBundle`.
9. When `payload.sessionKey` is present, **MAY** refuse a key type or DID method it does not support with `auth/authenticate:sessionKeyUnsupported`, rather than silently authenticating without the binding. Otherwise it **MUST** bind `sessionKey` to the `Session` it is creating in the same response (step 8) — the binding is consumer-internal state keyed by the session, not a claim resolved through the subject's own DID document.
10. From that point on, **MUST** accept a `proof` made by the bound `sessionKey`, with `proofPurpose: authentication`, as attributable to the session's `subject` — but **only** for documents scoped to that session, and only for as long as three things all remain true: the session has not expired (`Session.expiresAt`), the session has not been revoked (`auth/revoke-session`, or a logout that has the same effect), and the task being performed does not itself demand a step up past the session's current `Session.acr`. A document that fails any of those three is refused exactly as it would be if signed by no key at all — the session-key proof does not itself satisfy a step-up.
11. **MUST NOT** accept a `sessionKey` proof anywhere a specification requires an `assertionMethod` attestation — in particular [`auth/step-up/approve-response`](../../step-up/approve-response/0.5/spec.md), [`task-consent/decision`](../../../task-consent/decision/0.1/spec.md) and [`confirm/response`](../../../confirm/response/0.1/spec.md). Those specifications require `proof.proofPurpose: assertionMethod` by a key listed under the signer's `assertionMethod` DID document relationship; a session key is never such a key (see Security & Privacy).
12. **MUST NOT** allow a `sessionKey` proof to extend or refresh the session past what the subject's own login granted — see the `auth/refresh` interaction rule in Security & Privacy.

## Definitions

* **Subject.** The party authenticating; identified by `issuer` and verified via `proof`.
* **Auth service.** The party verifying the proof and issuing the session; identified by `recipient`.
* **Session.** The logical authentication context the consumer creates on success. Schema: [`_shared/0.2/session.schema.json#Session`](../../_shared/0.2/session.schema.json).
* **Session key.** An optional `did:key` VID the producer generates for one login and asks the consumer to bind to the resulting session (`payload.sessionKey` / `Session.sessionKey`). Its private half never leaves the producer's device; its authority never exceeds the session it is bound to. Not a `Session` on its own, and not a general-purpose signing identity — see Security & Privacy.
* **TokenBundle.** The access + optional refresh tokens. Schema: [`_shared/0.1/tokens.schema.json#TokenBundle`](../../_shared/0.1/tokens.schema.json).
* **VID.** *Verifiable Identifier* — DID, did:webvh URL, or any other scheme accepted by the consumer's trust framework.

## Request

The subject sends this document to the auth service (`recipient`) to complete a challenge-response login; the top-level schema in [`payload.schema.json`](payload.schema.json) describes it.

`payload.challenge` (REQUIRED) — verbatim echo of the challenge value returned by the prior `auth/challenge` response.

`payload.sessionId` (REQUIRED) — verbatim echo of the sessionId from that response.

`payload.scope` (optional) — capability tags the subject is requesting; consumer-defined vocabulary.

`payload.sessionKey` (optional) — a `did:key` VID to bind to the session this document creates. **New in 0.2.** A `did:key` was chosen over a bare multikey for two reasons: it is the wire form the passkey login path already uses for browser-bound keys (`auth/_shared` conventions, `auth/passkey/*`), so a client library that already knows how to mint and hold a `did:key` for one login flow needs no second code path for the other; and it is independently a valid `verificationMethod` value with no further wrapping, whereas a bare multikey string is not a VID and would need a method wrapper before it could appear in a `proof` at all. See Security & Privacy for what the binding does and does not authorize.

`payload.ext` (optional) — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

### A subject completes a login

```json
{
  "id": "1a2b3c4d-5e6f-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:30Z",
  "payload": {
    "challenge": "ZGN3RvOXh0c3JydWxsbmJzcmVxdHJjQVZjbA",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T10:00:30Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

### Requesting narrower scopes

```json
{
  "id": "2b3c4d5e-6f78-9012-3456-7890abcdef12",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:30Z",
  "payload": {
    "challenge": "ZGN3RvOXh0c3JydWxsbmJzcmVxdHJjQVZjbA",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "scope": ["context:project-alpha", "acl:read"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T10:00:30Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

### A wallet login binds a session key

Alice's wallet extension generates a fresh, non-extractable `did:key` for this login and asks the auth service to bind it, so the console's later calls in this session can be signed by the browser without a second extension prompt.

```json
{
  "id": "9a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:30Z",
  "payload": {
    "challenge": "ZGN3RvOXh0c3JydWxsbmJzcmVxdHJjQVZjbA",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "sessionKey": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T10:00:30Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/authenticate/0.2#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ session, tokens }`. The `session.amr` MUST include `"did"` (the authentication factor that completed). Consumers issuing a fresh session set `session.acr` to `"aal1"` unless the consumer combined this exchange with additional factors at issuance time (e.g. a co-located passkey assertion), in which case higher AAL classes are appropriate. When the request carried a `sessionKey` the consumer bound, `session.sessionKey` echoes it back — see [`_shared/0.2/session.schema.json`](../../_shared/0.2/session.schema.json).

Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### Successful authentication with a bound session key

```json
{
  "id": "3c4d5e6f-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.2#response",
  "threadId": "9a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:00:31Z",
  "payload": {
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T10:15:31Z",
      "amr": ["did"],
      "acr": "aal1",
      "sessionKey": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"
    },
    "tokens": {
      "accessToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9…",
      "refreshToken": "rt_8f2c1d4e9a7b3056",
      "tokenType": "Bearer",
      "expiresIn": 900,
      "refreshExpiresIn": 86400,
      "scope": ["context:project-alpha", "acl:read"]
    }
  }
}
```

### Successful authentication without a session key

```json
{
  "id": "5d6e7f80-9123-4567-8901-abcdef123456",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.2#response",
  "threadId": "1a2b3c4d-5e6f-7890-1234-567890abcdef",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:00:31Z",
  "payload": {
    "session": {
      "id": "fa6e4d90-4a5a-4ac3-8e8e-3b9d1b8b8c0c",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T10:15:31Z",
      "amr": ["did"],
      "acr": "aal1"
    },
    "tokens": {
      "accessToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9…",
      "refreshToken": "rt_1a2b3c4d5e6f7080",
      "tokenType": "Bearer",
      "expiresIn": 900,
      "refreshExpiresIn": 86400
    }
  }
}
```

### Challenge already consumed

```json
{
  "id": "4d5e6f78-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "1a2b3c4d-5e6f-7890-1234-567890abcdef",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:00:31Z",
  "payload": {
    "code": "auth/authenticate:challengeNotFound",
    "message": "The sessionId ec5d3c89… does not correspond to an active challenge."
  }
}
```

## Security & Privacy

### Data carried

The framework `proof` carries the entire trust burden. Consumers MUST NOT take any other field as authentication evidence — not the `issuer` claim alone, not the transport-layer identity, and not any header outside the document. The proof's `verificationMethod` MUST be resolvable through the issuer's published DID document at verification time; consumers SHOULD cache resolved DID documents with a TTL that respects the issuer's `nextUpdate` hint if present.

`payload.sessionKey` carries only a `did:key` — public key material, not a secret — so it is safe to log or echo back; what it grants is the *binding*, described below. The TokenBundle returned in the response document is, by contrast, bearer-grade material. Consumers operating over transports that don't already provide confidentiality (raw HTTPS POST is fine; broadcast queues are not) MUST encrypt the response to the producer.

The optional `ext` extension is part of the signed surface; producers MUST NOT place secret material in `ext`.

### Correlation

**Binding skew.** If the consumer issued a subject-agnostic challenge, the `issuer` of the authenticate document is whichever VID the producer chose. The consumer's authorization policy decides whether that VID is recognized — `subjectMismatch` is reserved for the bound-subject case.

**Session keys correlate within a session, by design, and nowhere else.** A `sessionKey` is generated fresh for one login and bound to the one `Session` it creates, so a consumer can (and, for its own audit, SHOULD) treat every document signed by that key as the same request-chain as the login that minted it — that linkage is the whole feature. It MUST NOT be treated as a stable cross-session identifier: a producer that generates a new `sessionKey` per login (as [Conformance](#conformance) item 5 describes) gives an observer nothing to join a subject's sessions on beyond `Session.subject` itself, which every login already discloses.

### Retention

**Replay across challenges.** A successful authenticate consumes its challenge; consumers MUST persist the consumption marker for at least `challenge.expiresAt` so a replay arriving late can't slip through after the binding row would otherwise be cleaned up.

**A session-key binding is retained for exactly the session's own lifetime.** The consumer MUST delete or invalidate the binding no later than it deletes the `Session` itself — on `Session.expiresAt`, on `auth/revoke-session`, or on a logout that has the same effect — and MUST NOT honor a `sessionKey` proof against a session it has already invalidated, even inside what would otherwise still be a cached validity window. There is no independent retention question for the key beyond the session's own: unlike a `refreshToken`, which a consumer typically retains past its own use for rotation-replay detection, a `sessionKey` binding carries no forensic value once the session it names is gone.

### Consent/purpose

**Scope downgrade.** The consumer MUST treat `payload.scope` as a *request*, not a grant. Returning a `TokenBundle.scope` narrower than the request is valid and SHOULD NOT trigger an error. Returning broader scope than requested is a policy decision the consumer documents in its trust framework.

**A session key's purpose is narrow, and a consumer MUST NOT honor it for anything wider.** Binding a `sessionKey` lets the producer sign *ordinary calls in the session it was bound to* without re-prompting the subject's own key; it authorizes nothing else, and a party who steals the key inherits exactly the session's `scope`, `acr` and remaining lifetime — no more:

* **It cannot step up.** [Conformance](#conformance) item 11 forbids a consumer from accepting a `sessionKey` proof anywhere a specification requires an `assertionMethod` attestation — [`auth/step-up/approve-response`](../../step-up/approve-response/0.5/spec.md), [`task-consent/decision`](../../../task-consent/decision/0.1/spec.md) and [`confirm/response`](../../../confirm/response/0.1/spec.md) name three. Those specifications require `proof.proofPurpose: assertionMethod` by a `verificationMethod` listed under the signer's *own* `assertionMethod` DID document relationship (auth/step-up/approve-response/0.5's Security & Privacy: "an approve-response is an attestation... not an operational message"). A `sessionKey` is neither: its `proof.proofPurpose` for ordinary use is `authentication`, and — because it is never written into the subject's DID document at all — there is no relationship it could be listed under. Below is a document a relying party MUST refuse for exactly that reason.

  ```json
  {
    "id": "approve-resp-aaaa-1111-bbbb-2222cccc3333",
    "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.5",
    "issuer": "did:web:alice.example",
    "recipient": "did:web:vta.example",
    "issuedAt": "2026-05-23T14:00:30Z",
    "payload": {
      "subject": "did:web:alice.example",
      "sessionId": "9c2e1f7a-6b3d-4c8e-9a1b-2d3e4f5a6b7c",
      "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
      "decision": "approved",
      "grantedAcr": "aal2"
    },
    "proof": {
      "type": "DataIntegrityProof",
      "cryptosuite": "eddsa-jcs-2022",
      "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "created": "2026-05-23T14:00:30Z",
      "proofPurpose": "assertionMethod",
      "proofValue": "z3kg…"
    }
  }
  ```

  This is Alice's console, still holding her live `auth/authenticate/0.2` session key, attempting to self-approve a step-up for that same session. Setting `proofPurpose: assertionMethod` on the wire changes nothing: `did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH` is not listed under any `assertionMethod` relationship of `did:web:alice.example`'s DID document — it is not in that document at all — so the relying party's resolution step fails and it MUST refuse with `proofInvalid`, exactly as it would refuse any other key that isn't Alice's own. A stolen session key therefore cannot approve its own step-up; escalating past the session's `acr` still costs the subject's real key or a delegated approver's.

* **It cannot extend or refresh the session past what the login granted.** [Conformance](#conformance) item 12 states the same rule the other direction. `auth/refresh` is the one task that *does* legitimately move `Session.expiresAt` forward, but it is authorized by possession of the `TokenBundle.refreshToken` — a bearer secret returned in this task's own response and never derivable from the `sessionKey` — not by a `proof` at all (`auth/refresh`'s own `proofRequirement` is `OPTIONAL`, and where a consumer's policy does require one for audit, that proof MUST resolve through the subject's own DID document exactly as [Conformance](#conformance) item 3 requires here, not through the session-key binding). A `sessionKey` proof is therefore never sufficient authority to call `auth/refresh`, whether or not the consumer requires a proof there at all: the two credentials answer different questions — the refresh token says *this session may continue*, the session key says *this request is the session's subject* — and neither substitutes for the other.
