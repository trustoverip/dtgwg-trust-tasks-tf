---
slug: auth/challenge
version: "0.1"
title: Auth — Challenge
summary: A party requests a one-time nonce from an authentication service that they will sign to prove control of their VID.
status: draft
targetFrameworkVersion: "0.1"
category: authentication
keywords:
  - auth
  - authentication
  - challenge
  - nonce
  - did
  - login
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject
    requirement: REQUIRED
  - role: Auth service
    requirement: REQUIRED
proofRequirement:
  requirement: OPTIONAL
  rationale: A challenge request is a public, write-once nudge — no evidentiary value attaches to it. The auth service returns its result regardless of who asked, because nothing of value is granted until the subsequent authenticate document, which IS proof-required.
errorCodes:
  - code: auth/challenge:subject_not_recognized
    meaning: The producer named a `subject` that the auth service does not know how to authenticate (e.g. an unregistered DID, or a VID scheme outside the issuer's trust framework).
    retryable: false
  - code: auth/challenge:rate_limited
    meaning: The producer (by source identifier — IP, DID, or both) has exceeded the issuer's challenge-issuance budget. The producer SHOULD back off; details.retryAfter MAY carry a seconds-until-retry hint.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        retryAfter: { type: integer, minimum: 0 }
related:
  - auth/authenticate
  - auth/refresh
  - auth/passkey/login/start
---

## Abstract

The **Auth — Challenge** Trust Task asks an *auth service* to issue a fresh nonce that a *subject* will sign to prove control of their *VID*. The subject embeds the returned `challenge` value into a subsequent [`auth/authenticate/0.1`](../../authenticate/0.1/spec.md) document; the framework `proof` on that document, signed by the subject's VID, IS the authentication.

This task only buys a nonce. No credentials are minted, no sessions are recorded as authenticated, and no claims are made about the producer. The auth service MAY rate-limit, log, or refuse — but the response carries no privilege.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the subject, or the subject's user agent) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/challenge/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Populate `payload.subject` with the VID the producer intends to authenticate as **when known**. Omitting `subject` is permitted for first-contact flows where the subject is not yet selected.
3. **MAY** include a `payload.purpose` hint (e.g. `"login"`, `"step-up"`, `"sign-out"`) so the consumer can scope the nonce to the producer's declared intent.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements). The proof field MAY be absent; if present it is verified per [SPEC.md §4.7](../../../../SPEC.md#47-proof) but its absence is not an error.
2. Generate a `challenge` value with at least 128 bits of cryptographically random entropy.
3. Bind the challenge server-side to:
   - The `subject` named in the request, when present. A successful subsequent authenticate MUST be rejected if its `issuer` does not equal this `subject`.
   - The `sessionId` returned. The same `(sessionId, challenge)` pair MUST NOT be issued twice.
   - The `expiresAt` returned. The consumer MUST refuse an authenticate carrying a challenge whose binding has expired.
4. Return a `#response` document carrying the issued challenge, sessionId, and expiry. Issuers SHOULD pick `expiresAt` between 30 s and 5 min in the future — long enough for slow network paths, short enough to bound replay risk.
5. Refuse with `auth/challenge:subject_not_recognized` when the named subject is outside the issuer's trust framework (unregistered DID, unsupported VID scheme).
6. Apply rate limiting on its own policy axes (source IP, source DID, both) and refuse with `auth/challenge:rate_limited` when exceeded.

A consumer **MAY** issue a *subject-agnostic* challenge when `payload.subject` is omitted. In that case the binding is established by the subject named in the proof on the authenticate document — the consumer trusts whichever VID signs the authenticate, subject to any later policy gates (e.g. ACL admission).

## Definitions

* **Subject.** The party whose VID will be authenticated; identified by `payload.subject` (when present) and by the `issuer` of the subsequent `auth/authenticate` document.
* **Auth service.** The party that issues challenges and verifies authenticate documents; identified by `recipient`.
* **VID.** *Verifiable Identifier* — a DID, did:webvh URL, or any other producer-identifying scheme the consumer's trust framework accepts. Per the framework spec, the choice of VID scheme is a consumer policy concern.
* **Challenge.** The base64url-encoded random value the subject embeds in their authenticate document.
* **Session identifier.** The opaque correlation handle returned alongside the challenge; echoed unchanged into the authenticate document.

## Payload

`payload.subject` — optional; the VID the producer intends to authenticate as. When present, the issuer SHOULD bind the challenge to this subject and reject authenticate documents whose `issuer` differs.

`payload.purpose` — optional free-text intent the producer declares (e.g. `"login"`, `"step-up"`, `"sign-out"`). Ecosystems define their own vocabulary; the framework imposes no syntax.

`payload.ext` — optional extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member). Every immediate child key MUST be reverse-DNS namespaced.

The full JSON Schema is in [`payload.schema.json`](payload.schema.json).

## Examples

### A subject requests a challenge to log in

```json
{
  "id": "9c1f4a2d-5e3b-4b2f-9bb9-7c1d2e3f4a5b",
  "type": "https://trusttasks.org/spec/auth/challenge/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "purpose": "login"
  }
}
```

### Subject-agnostic challenge

A flow where the subject hasn't been chosen yet (e.g. a hardware token will select which key signs):

```json
{
  "id": "a02b8e7c-0fb4-4c12-a8e1-22df14b0d8d2",
  "type": "https://trusttasks.org/spec/auth/challenge/0.1",
  "issuer": "did:web:client.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:00Z",
  "payload": {
    "purpose": "login"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/challenge/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ challenge, sessionId, expiresAt }`. The subject MUST treat all three as opaque (no field-level structure is implied) and echo `challenge` + `sessionId` verbatim into the authenticate document.

Failures use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful challenge

Response to the first request example:

```json
{
  "id": "f1a2b3c4-d5e6-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/challenge/0.1#response",
  "threadId": "9c1f4a2d-5e3b-4b2f-9bb9-7c1d2e3f4a5b",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:00:00Z",
  "payload": {
    "challenge": "ZGN3RvOXh0c3JydWxsbmJzcmVxdHJjQVZjbA",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "expiresAt": "2026-05-23T10:02:00Z"
  }
}
```

### Subject not recognized

```json
{
  "id": "b1c2d3e4-f5a6-7890-1234-567890fedcba",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "9c1f4a2d-5e3b-4b2f-9bb9-7c1d2e3f4a5b",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:00:00Z",
  "payload": {
    "code": "auth/challenge:subject_not_recognized",
    "message": "VID did:web:alice.example is not registered with this auth service."
  }
}
```

## Security & Privacy

The challenge document itself carries no evidentiary value — it does NOT prove anything about the producer or the auth service. The real cryptographic gate is the proof on the subsequent authenticate document. Treat the challenge response as ephemeral, not as audit material.

**Replay.** A challenge is single-use. Consumers MUST mark it as consumed (or expired) the moment the matching authenticate succeeds; a second authenticate against the same `(sessionId, challenge)` MUST fail.

**Entropy.** The 128-bit minimum is a floor, not a target. Consumers SHOULD use 192–256 bits of entropy in the `challenge` value to leave headroom against future cryptanalysis.

**Enumeration.** Returning `subject_not_recognized` distinguishes registered VIDs from unregistered ones. Consumers operating in environments where membership is sensitive (private communities, regulated rosters) MAY substitute a generic rate-limited error to avoid leaking the registered set.

**Rate limiting.** Unauthenticated challenge issuance is the cheapest attack surface in the auth family. Consumers SHOULD apply per-IP AND per-subject limits; a single-axis limiter is bypassable by either rotating IPs or rotating attempted subjects.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member)) is part of the producer's signed surface when a proof is included; producers MUST NOT place data in `ext` they would not be comfortable signing.
