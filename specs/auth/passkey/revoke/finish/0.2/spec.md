---
slug: auth/passkey/revoke/finish
version: "0.2"
title: Auth — Passkey Revoke (finish)
summary: The producer — the credential's owner, or an administrator acting for them — submits the user-verification assertion that completes a passkey revocation; on success the credential, of either purpose, is unbound permanently.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - revoke
  - credential-management
  - step-up
  - fido2
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "This leg destroys an authentication capability. The proof identifies who is acting and the assertion proves they are present; requiring both means neither a stolen key alone nor a captured assertion alone is sufficient."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Revocation removes an authenticator irreversibly. Replayed after the holder has enrolled a replacement it strips the replacement too, and a consumer with no window to date the document in cannot refuse it as stale."
sideEffects:
  level: destructive
  rationale: "The credential is unbound. A revoked passkey cannot be restored — it must be enrolled afresh, through an invite for a step-up credential."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: "The response names the revoked credential, its subject and purpose, and how many credentials of that purpose the subject has left."
errorCodes:
  - code: auth/passkey/revoke/finish:revocationNotFound
    meaning: "No pending revocation with this id, or it was started by a different producer."
    retryable: false
  - code: auth/passkey/revoke/finish:revocationExpired
    meaning: "The revocationId outlived its window. Start a new ceremony."
    retryable: true
  - code: auth/passkey/revoke/finish:userVerificationFailed
    meaning: "The assertion did not verify, did not match the challenge bound at start, was not from a credential chosen at start, or did not carry the UV flag. Deliberately one code for all of these."
    retryable: true
  - code: auth/passkey/revoke/finish:lastCredential
    meaning: "Re-checked at commit time, the session credential is now the subject's last, because another revocation completed in between. Never returned for a step-up credential. `details.remaining` MAY carry the count."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        remaining: { type: integer, minimum: 0 }
  - code: auth/passkey/revoke/finish:notAuthorized
    meaning: "Re-checked at commit time, the producer no longer holds the administrator standing over the target subject that the start relied on."
    retryable: false
related:
  - auth/passkey/revoke/start
  - auth/passkey/list
  - auth/passkey/enroll/invite
  - auth/revoke-session
---

## Abstract

The second leg of removing a passkey. The producer echoes the `revocationId` from [`auth/passkey/revoke/start/0.2`](../../start/0.2/spec.md) and the assertion from `navigator.credentials.get` over its `uvOptions`. The auth service unbinds the credential named at start.

## Changes from 0.1

- **The producer need not own the credential.** An administrator who started the revocation for another subject finishes it; the assertion is theirs.
- **The assertion must come from a credential chosen at start** — the producer's session credentials, or, for an owner revoking a step-up credential, their step-up credentials.
- **Purpose-aware last-credential guard**: re-checked for session credentials only.
- The administrator's standing is re-checked at commit time (`notAuthorized`).
- The response adds `subject` and `purpose`, and `remaining` counts the subject's credentials **of that purpose** and may be zero for a step-up credential.
- The framework target moves to 0.6.0. Released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/revoke/finish/0.2`, with itself as `issuer` and the auth service as `recipient`, carrying a verified `proof`.
2. Echo `payload.revocationId` verbatim from the start response, and populate `payload.uvCredential` with the assertion.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorisation evidence is **the producer's verified proof, the revocation bound to that producer at start, and a user-verified assertion from one of the credentials chosen at start** — plus, for an administrator acting for another subject, their standing over that subject, re-read at commit time.

A conforming **consumer** **MUST**, in this order:

1. Verify the `proof` and identify the producer's VID.
2. Resolve `revocationId`. Unknown, already consumed, or bound to a different producer → `revocationNotFound`; expired → `revocationExpired`.
3. Verify the assertion: a credential among those bound at start, signature valid against its stored key, `clientDataJSON.challenge` equal to the challenge bound at start, origin and `rpId` as expected, the signature counter increased, and the **UV flag set**. Any failure → `userVerificationFailed`.
4. Under the same per-subject serialization as start: for an administrator acting for another subject, re-check their standing (`notAuthorized`); for a session credential, re-check the last-credential guard (`lastCredential`).
5. Unbind the credential recorded against the `revocationId`, consume the handle, and record an audit event naming the producer, the subject, the credential id and its purpose.
6. Return `{ credentialId, subject, purpose, revokedAt, remaining }`.

A conforming consumer **MUST NOT** accept a target credential from this payload, **MUST NOT** accept an assertion whose UV flag is clear even if the signature verifies, and **MUST NOT** allow a `revocationId` to be redeemed twice.

### Pending ceremonies of a revoked step-up credential

A consumer **MUST** treat any step-up it issued that has not yet been answered, and that the revoked credential could have answered, as unanswerable by it from the moment of revocation. A step-up credential revoked because it was stolen must not still satisfy a step-up requested a minute earlier.

### Sessions established by the revoked credential

As in [0.1](../0.1/spec.md#sessions-established-by-the-revoked-credential), for session credentials. A step-up credential establishes no session, so there are none to end.

## Definitions

* **Producer**, **target subject**, **purpose**: as in [revoke/start 0.2](../../start/0.2/spec.md#definitions).
* **UV flag.** The `UV` bit in WebAuthn authenticator data, asserting that the authenticator verified the user during this ceremony. Distinct from `UP` (user *presence*).

## Payload

`payload.revocationId` — REQUIRED, echoed from start.

`payload.uvCredential` — REQUIRED, the assertion.

`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Examples

### The administrator completes the revocation

```json
{
  "id": "urn:uuid:3b0c9a4e-8f6d-4c2b-a1e7-5d9f0b2c6e21",
  "type": "https://trusttasks.org/spec/auth/passkey/revoke/finish/0.2",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-25T12:00:20Z",
  "payload": {
    "revocationId": "rev_4c1d7e9a0b3f5268",
    "uvCredential": {
      "id": "ZGFuYS1zZXNzaW9uLWtleQ",
      "rawId": "ZGFuYS1zZXNzaW9uLWtleQ",
      "type": "public-key",
      "response": {
        "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0In0",
        "authenticatorData": "SZYN5YgOjGh0NBcPZHZgW4_krrmihjLHmVzzuoMdl2MFAAAAAQ",
        "signature": "MEUCIQDx"
      }
    }
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/revoke/finish/0.2#response`.

```json
{
  "id": "urn:uuid:3b0c9a4e-8f6d-4c2b-a1e7-5d9f0b2c6e22",
  "type": "https://trusttasks.org/spec/auth/passkey/revoke/finish/0.2#response",
  "threadId": "urn:uuid:3b0c9a4e-8f6d-4c2b-a1e7-5d9f0b2c6e21",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-25T12:00:21Z",
  "payload": {
    "credentialId": "c3RlcHVwLWNyZWQtY2Fyb2w",
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "purpose": "stepUp",
    "revokedAt": "2026-09-25T12:00:21Z",
    "remaining": 0
  }
}
```

## Security & Privacy

**Why the target is bound at start**, **enumeration** and **ceremony expiry**: as in [0.1](../0.1/spec.md#security--privacy). The set of credentials that may verify is bound at start too, so a finish cannot switch to an assertion from a credential the start did not offer.

**A step-up credential revoked to none.** The subject then has no gesture a bound step-up can ask of them, and every act that requires one is refused until an administrator invites them to enrol another. That is the intended failure: closed.

**Notice.** A consumer **SHOULD** tell the subject when an administrator revoked one of their credentials, over a channel it already has with them.

### Data carried

The request carries the revocation handle and an assertion. The response names the revoked credential, its subject and purpose, and a count.

### Correlation

As at start: the audit record links the producer, the subject and the credential.

### Retention

The credential's public key and metadata are deleted; the audit record of the revocation is durable.

### Consent/purpose

The assertion authorises this one revocation and is spent by it.

The optional `ext` extension is part of the producer's signed surface.
