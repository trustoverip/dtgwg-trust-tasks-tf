---
slug: auth/authenticate
version: "0.3"
title: Auth — Authenticate
summary: A subject presents a challenge in a proof-bearing document and gets back a session + tokens. 0.3 lets a delegate (e.g. a holder's VTA) authenticate a principal on its behalf, alongside the 0.2 session-key binding.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - authentication
  - did
  - login
  - challenge-response
  - jwt
  - session-key
  - delegation
  - proxy-login
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject or delegate
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
  - role: Principal
    requirement: OPTIONAL
    identifierScope: any
proofRequirement:
  requirement: REQUIRED
  rationale: The framework `proof` authenticates whoever signs — the subject itself in the ordinary case, or a delegate acting for a named `principal` in the proxied case. Either way, without a verified proof binding the document to the signer's VID the auth service has no basis to issue a session, and because `sessionKey` and `delegationEvidence` both sit inside `payload`, this same proof is what entitles the signer to assert either.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An authentication request that cannot be placed in time is a credential presentation with no expiry, which is precisely the form a captured request is replayed in. Here the acceptance window is the primary defence, not a secondary one — and a replayed proxied authenticate would open a fresh session for the principal at a moment neither the principal nor the delegate chose.
sideEffects:
  level: mutating
  rationale: "Establishes an authenticated session and issues tokens; the session is revocable state. The proxied form adds no new kind of revocable state beyond what 0.2 already introduced for the session-key binding — the session-actor distinction is recorded alongside the same session and ends exactly when it does."
consequences:
  - "A proxied authenticate opens a session in the principal's name that the principal did not themselves sign into — every subsequent document on that session is attributable to `principal`, not to the delegate that opened it."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: "The proxied form (`payload.principal` present and unequal to `issuer`) exercises the principal's own authority to open an authenticated session in their name, on the strength of `delegationEvidence` rather than the principal's own proof — precisely the case `actsAsSubject` exists to flag. The ordinary form exercises only the signer's own authority, identically to auth/authenticate/0.2; the class is declared `true` because the specification permits both and a consumer implementing it MUST treat every request as capable of the proxied case until `payload.principal` says otherwise. Inbound, `delegationEvidence` and `principal` together identify which VID is being asserted and on what basis — identifiers, not secret material."
subjectPath: /principal
retention:
  class: durable
  rationale: "The session this task creates, and — for a proxied request — the fact that `issuer` rather than `subject` performed it, are audit-relevant for the session's whole lifetime and MUST be retained at least that long, on the same terms as auth/authenticate/0.2's session-key binding."
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
    meaning: The verified acting party — `payload.principal` for a proxied request, otherwise the document's `issuer` — does not equal the `subject` the challenge was bound to.
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
  - code: auth/authenticate:delegationEvidenceRequired
    meaning: "`payload.principal` is present and differs from the document's `issuer`, but `payload.delegationEvidence` is absent. A proxied authenticate MUST carry both."
    retryable: false
  - code: auth/authenticate:delegationNotRecognized
    meaning: "`payload.delegationEvidence` was present but did not establish, under the consumer's own authorization policy, that `issuer` is entitled to authenticate as `principal` — an unrecognized `kind`, a reference the consumer cannot resolve, a credential that fails verification, or one that verifies but names a different principal or has expired or been revoked."
    retryable: false
related:
  - auth/challenge
  - auth/refresh
  - auth/revoke-session
  - auth/whoami
  - auth/passkey/login/finish
  - auth/step-up/approve-response
  - vault/proxy-login
---

## Abstract

The **Auth — Authenticate** Trust Task is the second half of a challenge-response authentication. A signer presents the challenge it received from [`auth/challenge/0.1`](../../challenge/0.1/spec.md); the framework `proof` on that document, verified against the signer's VID, IS the authentication. The auth service replies with a *Session* and *TokenBundle*.

This task does NOT mint tokens itself — it requests them. The auth service applies its own authorization policy (ACL, allowed scopes, AAL ceiling) before responding.

**New in 0.3: a proxied form.** In the ordinary case the signer (`issuer`) authenticates as itself, exactly as [`auth/authenticate/0.2`](../0.2/spec.md). Some deployments need a second case: a holder's VTA — or another delegate the holder has entrusted with its own signing key — completing a login *for* a persona or principal DID it does not itself control, most commonly the VTA-proxied SIOPv2 login pattern where the VTA mints and signs the authentication on the principal's behalf. 0.3 names this explicitly: `payload.principal` carries the VID being authenticated as, `issuer` remains the signer that actually holds the proof key, and `payload.delegationEvidence` carries whatever the consumer's own policy requires to accept that the two are linked. The resulting `Session.subject` is `principal`; `Session.actor` records the delegate that opened it, so nothing downstream mistakes one for the other. 0.2's `sessionKey` binding is unchanged and orthogonal to this — it is available whether or not the login is proxied.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/authenticate/0.3`, with itself (the signer — the subject, or its delegate) as `issuer` and the auth service as `recipient`.
2. Echo `payload.challenge` and `payload.sessionId` verbatim from the `auth/challenge` response.
3. Include a `proof` member per [SPEC.md §4.7](/SPEC.md#47-proof). The proof's `verificationMethod` MUST resolve via the issuer's DID document, and MUST be the signer's own key — never the principal's, in the proxied case.
4. **MAY** request specific `payload.scope` capabilities. The producer MUST be prepared for the consumer to issue a token bundle with a narrower `scope`.
5. **MAY** include `payload.sessionKey`, a `did:key` VID freshly generated for this login, exactly as in [`auth/authenticate/0.2`](../0.2/spec.md#conformance) item 5. Behaves identically whether or not this document is a proxied authenticate.
6. **MAY** include `payload.principal` to authenticate as a party other than itself. When `payload.principal` is present and differs from `issuer`, the producer **MUST** also include `payload.delegationEvidence`, populated with whatever the auth service's Authorization policy requires — see [Authorization](#authorization). Omitting `principal`, or setting it equal to `issuer`, is the ordinary form and requires no `delegationEvidence`.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`. The proof being absent or invalid is a hard failure. The proof attributes the document to `issuer` only — never, by itself, to `principal`.
2. Look up the server-side binding for `payload.sessionId`. If no binding exists, respond with `auth/authenticate:challengeNotFound`.
3. Compare the binding's stored challenge to `payload.challenge` using a constant-time comparator. Mismatch → `auth/authenticate:challengeMismatch`.
4. Reject expired bindings with `auth/authenticate:challengeExpired`.
5. Determine the **acting party**: `payload.principal` when present, otherwise `issuer`. When the challenge was issued with a bound `subject`, verify the acting party equals that subject. Mismatch → `auth/authenticate:subjectMismatch`.
6. Consume the challenge. The same `(sessionId, challenge)` pair MUST NOT be honored a second time, even on identical authenticate documents.
7. When `payload.principal` is present and differs from `issuer` (a **proxied authenticate**):
   1. Refuse with `auth/authenticate:delegationEvidenceRequired` when `payload.delegationEvidence` is absent.
   2. Evaluate `payload.delegationEvidence` under the consumer's own authorization policy (see [Authorization](#authorization)) to decide whether `issuer` is entitled to authenticate as `principal`. The consumer **MUST** verify the evidence independently — a `kind` and a claim are not by themselves sufficient, exactly as `issuer`'s own claim of identity is not accepted without the `proof` that verifies it. Refuse with `auth/authenticate:delegationNotRecognized` when the evidence does not establish the entitlement.
   3. On success, **MUST** set the created `Session.subject` to `principal` and `Session.actor` to `issuer` — never the reverse, and never omit `actor` for a session created this way.
8. When `payload.principal` is absent, or equals `issuer` (the **ordinary** case), proceed exactly as [`auth/authenticate/0.2`](../0.2/spec.md#conformance) items 7–12, with `Session.subject = issuer` and no `Session.actor`.
9. Apply the consumer's authorization policy. Refused scopes → `auth/authenticate:scopeDenied` with `details.refused`.
10. Issue a `#response` document carrying a freshly-created `Session` (with `amr` containing at least `"did"` and `acr` defaulting to `"aal1"`) and a `TokenBundle`.
11. When `payload.sessionKey` is present, **MAY** refuse a key type or DID method it does not support with `auth/authenticate:sessionKeyUnsupported`, rather than silently authenticating without the binding. Otherwise it **MUST** bind `sessionKey` to the `Session` it is creating in the same response — consumer-internal state keyed by the session, not a claim resolved through either the signer's or the principal's own DID document. Behaves identically whether or not the login is proxied: the `sessionKey`, once bound, speaks for `Session.subject` (`principal` when proxied), never for `Session.actor`.
12. From that point on, **MUST** accept a `proof` made by the bound `sessionKey`, with `proofPurpose: authentication`, as attributable to `Session.subject` — but **only** for documents scoped to that session, and only for as long as three things all remain true: the session has not expired (`Session.expiresAt`), the session has not been revoked (`auth/revoke-session`, or a logout that has the same effect), and the task being performed does not itself demand a step up past the session's current `Session.acr`. A document that fails any of those three is refused exactly as it would be if signed by no key at all — the session-key proof does not itself satisfy a step-up, and it never carries the authority `delegationEvidence` established for the session's own creation.
13. **MUST NOT** accept a `sessionKey` proof anywhere a specification requires an `assertionMethod` attestation — in particular [`auth/step-up/approve-response`](../../step-up/approve-response/0.5/spec.md), [`task-consent/decision`](../../../task-consent/decision/0.1/spec.md) and [`confirm/response`](../../../confirm/response/0.1/spec.md). Those specifications require `proof.proofPurpose: assertionMethod` by a key listed under the signer's `assertionMethod` DID document relationship; a session key is never such a key (see Security & Privacy).
14. **MUST NOT** allow a `sessionKey` proof, or a session created by a proxied authenticate, to extend or refresh the session past what the original login granted — see the `auth/refresh` interaction rule in Security & Privacy.

## Authorization

*Per [SPEC.md §7.3 item 15](/SPEC.md#73-specification-requirements), required because this specification's `sideEffects.level` is `mutating` and its `exposure.actsAsSubject` is `true`.*

The **ordinary** form (`payload.principal` absent, or equal to `issuer`) presupposes nothing beyond identity: the `proof` establishes that `issuer` controls the VID it claims, and that control **is** the authorization to open a session as itself. No further evidence is asked, exactly as in `auth/authenticate/0.2`.

The **proxied** form's authority is `payload.delegationEvidence`, evaluated under the consumer's own policy — this specification names the evidence class the task presupposes without prescribing its shape, per item 15. Typical instances: a VTA-context credential the auth service already trusts for that principal (the pattern [`vault/proxy-login`](../../../vault/proxy-login/0.2/spec.md) uses to let a maintainer act at a third party on a holder's behalf, generalized here to the auth service accepting the same kind of evidence directly); a mandate credential naming `issuer` as an authorized delegate of `principal`; or, for a consumer with no independent registry to check, a `reference` to a binding it recorded itself when the delegation was established out of band. Whichever shape a deployment adopts, the consumer **MUST** verify it independently of the claim carried alongside it — `payload.delegationEvidence.kind` says what to check, never that the check passed.

Do not confuse this with identity or proof validation ([SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements)): the `proof` establishes only that `issuer` is who it claims to be and that the document is unaltered. It says nothing about whether `issuer` may act for `principal` — that is exactly the question `delegationEvidence` answers, and a consumer that skips evaluating it and honors `principal` on the strength of the proof alone has confused *authenticated* with *authorized* for a different party's identity, the confused-deputy shape [SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements) names directly.

The final decision always rests with the consumer's own policy and trust framework; this specification states only what evidence the task presupposes, not that any particular deployment must require it or in what strength.

## Definitions

* **Signer.** The party whose key produced the `proof`; identified by `issuer`. In the ordinary case the signer is also the subject; in the proxied case the signer is the delegate.
* **Subject / Principal.** The party being authenticated as. Identified by `issuer` in the ordinary case, or by `payload.principal` in the proxied case.
* **Delegate.** In the proxied case, the signer acting on the principal's behalf — typically the principal's VTA. Identified by `issuer`; recorded on the resulting session as `Session.actor`.
* **Delegation evidence.** `payload.delegationEvidence`, whatever the auth service's own policy requires as evidence that the delegate may act for the principal. Opaque to the framework; see [Authorization](#authorization).
* **Auth service.** The party verifying the proof (and, when proxied, the delegation evidence) and issuing the session; identified by `recipient`.
* **Session.** The logical authentication context the consumer creates on success. Schema: [`_shared/0.3/session.schema.json#Session`](../../_shared/0.3/session.schema.json).
* **Session key.** An optional `did:key` VID the producer generates for one login and asks the consumer to bind to the resulting session (`payload.sessionKey` / `Session.sessionKey`). Its private half never leaves the producer's device; its authority never exceeds the session it is bound to, and it always speaks for `Session.subject`, never for `Session.actor`. See Security & Privacy.
* **TokenBundle.** The access + optional refresh tokens. Schema: [`_shared/0.1/tokens.schema.json#TokenBundle`](../../_shared/0.1/tokens.schema.json).
* **VID.** *Verifiable Identifier* — DID, did:webvh URL, or any other scheme accepted by the consumer's trust framework.

## Request

The signer sends this document to the auth service (`recipient`) to complete a challenge-response login; the top-level schema in [`payload.schema.json`](payload.schema.json) describes it.

`payload.challenge` (REQUIRED) — verbatim echo of the challenge value returned by the prior `auth/challenge` response.

`payload.sessionId` (REQUIRED) — verbatim echo of the sessionId from that response.

`payload.scope` (optional) — capability tags being requested; consumer-defined vocabulary.

`payload.principal` (optional) — the VID being authenticated as, when it differs from `issuer`. **New in 0.3.** Marks a proxied authenticate; requires `payload.delegationEvidence`.

`payload.delegationEvidence` (optional; REQUIRED alongside a `principal` unequal to `issuer`) — the evidence the auth service's policy evaluates to accept the delegation. **New in 0.3.** See Authorization.

`payload.sessionKey` (optional) — a `did:key` VID to bind to the session this document creates, unchanged from 0.2. See Security & Privacy for what the binding does and does not authorize.

`payload.ext` (optional) — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

### A subject completes an ordinary login

```json
{
  "id": "1a2b3c4d-5e6f-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.3",
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

### A VTA authenticates a principal on its behalf (valid proxied login)

Alice's VTA (`did:web:vta.alice.example`) completed the SIOPv2 exchange with its own key and now presents the resulting authenticate document naming Alice's persona as `principal`, together with the VTA-context credential the auth service already trusts for that binding.

```json
{
  "id": "9a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.3",
  "issuer": "did:web:vta.alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:30Z",
  "payload": {
    "challenge": "ZGN3RvOXh0c3JydWxsbmJzcmVxdHJjQVZjbA",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "principal": "did:web:alice.example",
    "delegationEvidence": {
      "kind": "vtaContext",
      "reference": "vta-context:personal/console"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:vta.alice.example#key-1",
    "created": "2026-05-23T10:00:30Z",
    "proofPurpose": "authentication",
    "proofValue": "z9pq…"
  }
}
```

### Proxied login rejected — no delegation evidence (invalid)

A document naming `principal` without `delegationEvidence` is schema-valid (both members are individually optional) but a conforming consumer **MUST** refuse it — it cannot distinguish "the signer meant to authenticate as someone else, with no evidence" from a producer bug, and neither is a basis for a session:

```json
{
  "id": "bad-proxy-no-evidence-0001",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.3",
  "issuer": "did:web:vta.alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:30Z",
  "payload": {
    "challenge": "ZGN3RvOXh0c3JydWxsbmJzcmVxdHJjQVZjbA",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "principal": "did:web:alice.example"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:vta.alice.example#key-1",
    "created": "2026-05-23T10:00:30Z",
    "proofPurpose": "authentication",
    "proofValue": "z9pq…"
  }
}
```

The auth service refuses with `auth/authenticate:delegationEvidenceRequired`.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/authenticate/0.3#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ session, tokens }`. The `session.amr` MUST include `"did"` (the authentication factor that completed). Consumers issuing a fresh session set `session.acr` to `"aal1"` unless the consumer combined this exchange with additional factors at issuance time, in which case higher AAL classes are appropriate. `session.subject` is `principal` for a proxied request, `issuer` otherwise; `session.actor` is present, and equal to `issuer`, only for a proxied request. When the request carried a `sessionKey` the consumer bound, `session.sessionKey` echoes it back — see [`_shared/0.3/session.schema.json`](../../_shared/0.3/session.schema.json).

Failures use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### Successful proxied authentication

Response to the VTA-proxied example above. `session.subject` names Alice's persona; `session.actor` names the VTA that actually signed.

```json
{
  "id": "3c4d5e6f-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.3#response",
  "threadId": "9a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:vta.alice.example",
  "issuedAt": "2026-05-23T10:00:31Z",
  "payload": {
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "actor": "did:web:vta.alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T10:15:31Z",
      "absoluteExpiresAt": "2026-05-24T10:00:31Z",
      "amr": ["did"],
      "acr": "aal1"
    },
    "tokens": {
      "accessToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9…",
      "refreshToken": "rt_8f2c1d4e9a7b3056",
      "tokenType": "Bearer",
      "expiresIn": 900,
      "refreshExpiresIn": 86400
    }
  }
}
```

### Successful ordinary authentication with a bound session key

```json
{
  "id": "5d6e7f80-9123-4567-8901-abcdef123456",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.3#response",
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
      "absoluteExpiresAt": "2026-05-24T10:00:31Z",
      "amr": ["did"],
      "acr": "aal1",
      "sessionKey": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"
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

### Proxied login refused — unrecognized delegation evidence

```json
{
  "id": "4d5e6f78-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "9a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:vta.alice.example",
  "issuedAt": "2026-05-23T10:00:31Z",
  "payload": {
    "code": "auth/authenticate:delegationNotRecognized",
    "message": "did:web:vta.alice.example is not a recognized delegate for did:web:alice.example."
  }
}
```

## Changes from 0.2

`0.3` is a backwards-compatible `MINOR` increment ([SPEC §5.2](/SPEC.md#52-compatibility-rules)): every valid `0.2` payload is a valid `0.3` payload, and every `0.2` document (which never carries `principal` or `delegationEvidence`) is processed identically.

- **Adds the proxied form** — `payload.principal` and `payload.delegationEvidence`, and the `Session.actor` member on the shared `_shared/0.3` Session schema that records who actually signed a proxied login.
- **Broadens `subjectMismatch`'s meaning** to compare against the acting party (`principal` when proxied, otherwise `issuer`) rather than always `issuer`; the code itself is unchanged.
- **Adds two error codes**: `delegationEvidenceRequired` and `delegationNotRecognized`.
- **Adds the `## Authorization` section** [SPEC §7.3 item 15](/SPEC.md#73-specification-requirements) requires now that `exposure.actsAsSubject` is declared `true`.
- 0.2's session-key binding (`payload.sessionKey`, `Session.sessionKey`) is carried forward unchanged.

## Security & Privacy

### Data carried

The framework `proof` carries the entire trust burden for *identity*: it establishes that `issuer` controls the key it claims, and consumers MUST NOT take any other field — not a bare `issuer` claim, not transport-layer identity, not a header — as identity evidence. For a proxied request, `payload.delegationEvidence` carries the separate trust burden of *entitlement*: that the identified `issuer` may act for `payload.principal`. The two are independent and a consumer MUST evaluate both; a valid proof says nothing about entitlement, and (per Authorization) no shape of `delegationEvidence` is itself trusted without independent verification.

`payload.principal` and `payload.delegationEvidence.kind`/`reference` are identifiers and consumer-defined labels — not secret material, and safe to log. `payload.delegationEvidence.credential`, when populated inline, MAY itself carry a `proof` and other structured claims; producers MUST NOT use it to smuggle secret material the consumer has no business receiving, and consumers MUST log at most the fields their own policy needs to attribute the decision, not the whole credential verbatim, if that credential can itself carry the principal's personal data. `payload.sessionKey` carries only public key material, exactly as in 0.2. The TokenBundle returned in the response is bearer-grade material regardless of which form of authenticate produced it; consumers operating over transports that don't already provide confidentiality MUST encrypt the response to the producer.

### Correlation

**`Session.actor` exists so a proxied session correlates on the true acting party, not just the principal.** Every session this task creates names its `subject`; a proxied session additionally names its `actor`. Without that second field, every audit line, revocation, and downstream authorization decision would have to either drop the delegate entirely (indistinguishable from the principal logging in directly — the same asymmetry [`vault/proxy-login`](../../../vault/proxy-login/0.2/spec.md)'s Security & Privacy names at the third party) or overload `subject` with two meanings. Consumers SHOULD treat `actor` as the correlation key for "how many principals has this delegate authenticated," which is exactly the audit reach a compromised or misbehaving delegate needs to be caught by.

**Session keys correlate within a session, by design, and nowhere else** — unchanged from 0.2, and unaffected by whether the session is proxied: `Session.sessionKey` always speaks for `Session.subject`, and a producer generating a fresh key per login gives an observer nothing to join sessions on beyond `Session.subject`/`Session.actor` themselves, which every login already discloses.

### Retention

**Replay across challenges** — unchanged from 0.2: a successful authenticate consumes its challenge, and consumers MUST persist the consumption marker for at least `challenge.expiresAt`.

**A session's `actor`, like its `sessionKey` binding, is retained for exactly the session's own lifetime** for ordinary purposes, but its audit trail is durable: consumers MUST retain the fact that a given `actor` authenticated a given `subject` at least as long as they retain any other authority-affecting audit record, on the same terms `auth/revoke-session`'s Retention states for the analogous case. There is no independent retention question for `actor` beyond that — it is a fact about how a session came to exist, not itself a credential that can be separately revoked.

### Consent/purpose

**Scope downgrade** — unchanged from 0.2: `payload.scope` is a request, and a narrower `TokenBundle.scope` is not an error.

**Delegation abuse is the risk this version exists to bound, and the mitigation is structural rather than advisory.** A delegate holding a live signing key can request `principal` to be *any* VID it names — nothing in the proof stops it, because the proof only ever attributes the document to the delegate's own key. The entire defence is that `delegationEvidence` MUST be independently verified (Authorization) before the consumer honors `principal` at all: a delegate presenting evidence for one principal gains a session for exactly that principal, never a blank cheque to name others, and a consumer that skips the check because a request "looks like the usual VTA traffic" reintroduces the exact confused-deputy failure item 15 exists to force into the open. Below is a request a relying auth service MUST refuse for exactly that reason — the same well-formed VTA key from the valid example above, now naming a principal the VTA presents no evidence for:

```json
{
  "id": "bad-proxy-unentitled-0001",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.3",
  "issuer": "did:web:vta.alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:05:00Z",
  "payload": {
    "challenge": "dGhpcyBpcyBhIGRpZmZlcmVudCBjaGFsbGVuZ2U",
    "sessionId": "b2c8f4e1-9a3d-4f7c-8b1e-6d5a4c3b2a19",
    "principal": "did:web:carol.example",
    "delegationEvidence": {
      "kind": "vtaContext",
      "reference": "vta-context:personal/console"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:vta.alice.example#key-1",
    "created": "2026-05-23T10:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z9pq…"
  }
}
```

`vta-context:personal/console` is a reference the auth service resolves — and, resolved, it names Alice's context, not Carol's. The reference is well-formed and independently checkable; it simply does not establish the entitlement claimed. The auth service MUST refuse with `auth/authenticate:delegationNotRecognized`, exactly as it would a reference to a context that does not exist at all: `delegationEvidence` that resolves to the wrong principal is not weaker evidence, it is evidence of a different, unrequested fact.

**A stolen session key is bounded exactly as in 0.2, whether or not the session is proxied.** [Conformance](#conformance) item 13 forbids a consumer from accepting a `sessionKey` proof anywhere a specification requires an `assertionMethod` attestation, so a stolen session key still cannot self-approve its own step-up (0.2's Security & Privacy example applies unchanged — substitute `Session.subject` for "Alice" and the refusal is identical whether that subject authenticated directly or was authenticated for by a delegate).

**It cannot extend or refresh the session past what the login granted, and neither can the delegation.** [Conformance](#conformance) item 14 states the same rule 0.2 item 12 stated, extended to name the proxied case explicitly: `auth/refresh` is authorized by possession of `TokenBundle.refreshToken`, never by a `sessionKey` proof and never by re-presenting `delegationEvidence` — a delegate that authenticated a principal once does not thereby gain standing authority to refresh that principal's session indefinitely; each refresh is bounded by [`auth/refresh/0.2`](../../refresh/0.2/spec.md)'s own rules, including the absolute session lifetime it enforces.
