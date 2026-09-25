---
slug: auth/passkey/enroll/redeem/start
version: "0.1"
title: Auth — Passkey Enroll (redeem, start)
summary: The invitee presents an enrollment invite's token and its separately delivered claim code, and the auth service begins the registration the invite authorises, with a user-verification ceremony over the subject's existing credentials of the same purpose.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - enrollment
  - invite
  - redeem
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Invitee
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: "The invitee holds no key the auth service trusts yet — that is why there is an invite — and typically redeems from a browser that holds none. The authorisation is the invite token together with the claim code; a proof, if present, adds nothing the service relies on."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Placing the request in time bounds how long a captured redemption request stays replayable, independent of the invite's own expiry."
sideEffects:
  level: none
  rationale: "Begins a WebAuthn registration ceremony and returns options. Nothing is bound, and the invite is not consumed, until the matching finish."
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: "The request carries the invite token and claim code, which authorise the binding. The response names the subject the credential will be bound to, and the creation options."
retention:
  class: exchange
  rationale: "The ceremony state lives only until the finish or its expiry; the auth service records failed attempts against the invite for rate limiting."
errorCodes:
  - code: auth/passkey/enroll/redeem/start:inviteInvalid
    meaning: "The token names no redeemable invite, or the claim code is wrong, or the invite has expired or been consumed. Deliberately one code for all of these, so that redemption is no oracle for which half was wrong."
    retryable: false
  - code: auth/passkey/enroll/redeem/start:tooManyAttempts
    meaning: "Too many wrong claim codes were presented for this invite; it has been invalidated. Ask the administrator for a new one."
    retryable: false
related:
  - auth/passkey/enroll/invite
  - auth/passkey/enroll/redeem/finish
---

## Abstract

The first leg of redeeming an [`auth/passkey/enroll/invite`](../../../invite/0.2/spec.md). The invitee — typically in a browser, on the auth service's own origin, having opened the invite URL — sends the token from the URL and the claim code they were given separately. The auth service answers with WebAuthn creation options for the invite's subject, and, when the subject already holds credentials of the invite's purpose, request options over those too.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorisation evidence is **an unexpired, unconsumed invite whose token hash matches `token`, together with the matching claim code**. Nothing about the sender's identity is relied on.

A conforming **consumer** **MUST**:

1. Look the invite up by a hash of `token`. Refuse with `inviteInvalid` when none matches, when it has expired or been consumed, or when `claimCode` does not verify against the stored hash — the same code, message and timing for each.
2. Count wrong claim codes per invite, and on reaching its limit (RECOMMENDED: 5) invalidate the invite and refuse with `tooManyAttempts`. Rate-limit this task per source as well.
3. Generate a fresh `enrollmentId` bound to the invite, the subject, the registration challenge and, where present, the user-verification challenge, with an expiry (RECOMMENDED 5 minutes).
4. Return creation options whose `excludeCredentials` covers the subject's existing credentials of the invite's purpose, and — exactly when the subject has any — `uvOptions` over them with `userVerification: "required"`.
5. Leave the invite unconsumed: a ceremony the invitee abandons may be retried until the invite expires.

## Payload

`payload.token` (REQUIRED) — from the invite URL. `payload.claimCode` (REQUIRED) — delivered separately. `payload.ext` — extension slot.

## Examples

```json
{
  "id": "urn:uuid:5d1e7c2a-3b4f-4e8a-9c61-0f2a7b3c9d11",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/redeem/start/0.1",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-25T10:20:00Z",
  "payload": {
    "token": "inv_8f2c1d4e9a7b30568f2c1d4e9a7b3056a1b2c3d4",
    "claimCode": "7KQ4-MX2P-9TDA"
  }
}
```

## Response

`{ enrollmentId, subject, purpose, deviceLabel?, options, uvOptions?, expiresAt }`, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). The invitee's surface **SHOULD** show `subject` and `purpose` before it creates the credential, so a human can notice an invite for somebody else.

## Security & Privacy

**No oracle.** A single refusal for every way a redemption can be wrong keeps an attacker holding one channel from learning whether they have the right token.

**Origin.** The ceremony binds to the auth service's relying-party id; an invitee who is shown the URL on another origin is being phished, and the attestation will not verify at finish.

**Retention.** The claim code is compared with its stored hash and never logged. Failed attempts are recorded against the invite only as a count.

### Data carried

The request carries the invite token and the claim code. The response names the subject, the purpose, the inviter's suggested label and the WebAuthn options, which include the subject's existing credential ids of the purpose when there are any.

### Correlation

Anyone holding both halves of an invite learns whose invite it is. Nothing else about the subject is disclosed, and a wrong code learns nothing — not even whether the token was right.

### Retention

The ceremony state is kept until the finish or its expiry. Failed attempts are counted against the invite for its lifetime only.

### Consent/purpose

The token and code are used to begin this one registration and for nothing else.
