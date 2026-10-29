---
slug: vtc/install/claim/passkey/finish
version: "0.1"
title: VTC Install Claim — Passkey — Finish
summary: Complete first-administrator enrolment with a passkey — submit the WebAuthn attestation, derive the administrator's did:key from the attested key, and receive a setup-session token.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, install, bootstrap, passkey, webauthn]
parties:
  - role: installer
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: >-
    The request's gate is the install token plus the passkey attestation in the payload; no administrator key exists yet to carry a framework proof, and WebAuthn never exposes the passkey's private key to sign one with. The response carries the setup-session token, a bearer secret, so it is signed by the community: the installer can then tell a genuine token from one an intermediary substituted (SPEC §7.3 item 8).
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Claim completion binds an installation to its owner once. A replayed completion re-binds an installation that may have been transferred, and the claim is what every later authorisation in the community is rooted in.
sideEffects:
  level: mutating
  rationale: "Consumes the install token, registers the passkey, derives the administrator's DID from it and mints a short-lived setup-session token; the administrator entry itself is written by vtc/admin/bootstrap."
exposure:
  discloses: secret
  ingests: secret
  actsAsSubject: false
  rationale: "The response carries the setup-session token, a short-lived bearer credential that vtc/admin/bootstrap consumes to write the first administrator. The request carries the install token, which authorises the claim, and the WebAuthn attestation."
retention:
  class: durable
  rationale: "The claim — which passkey, on which install token — is the audit record every later authorisation in the community is rooted in, and the passkey is kept as the administrator's credential."
errorCodes:
  - code: vtc/install/claim/passkey/finish:invalidToken
    meaning: The install token is missing, malformed, expired, or already consumed.
    retryable: false
  - code: vtc/install/claim/passkey/finish:registrationMismatch
    meaning: The registrationId does not match an open registration for this token.
    retryable: false
  - code: vtc/install/claim/passkey/finish:bindingInvalid
    meaning: The WebAuthn attestation failed verification.
    retryable: false
related:
  - vtc/install/claim/passkey/start
  - vtc/install/claim/finish
  - vtc/admin/bootstrap
---

## Abstract

The **VTC Install Claim — Passkey — Finish** Trust Task completes first-administrator enrolment by passkey. The installer submits the `installToken`, the `registrationId` from [`vtc/install/claim/passkey/start`](../../start/0.1/spec.md) and the `webauthnResponse`. The community derives the administrator's DID from the passkey's Ed25519 key and returns it with a short-lived `setupSessionToken`, which [`vtc/admin/bootstrap`](../../../../../admin/bootstrap/0.1/spec.md) consumes to write the administrator entry.

### Changes from vtc/install/claim/finish 0.2

This is the ceremony of [`vtc/install/claim/finish`](../../../finish/0.2/spec.md) `0.2`, under a slug of its own so that it is not superseded by the wallet ceremony of `vtc/install/claim/finish` `0.3`. [`passkey/start`](../../start/0.1/spec.md) says why the two ceremonies cannot share a version. `vtc/install/claim/finish` `0.2` is retired in favour of this task.

The request and response payloads are unchanged. One declaration moved: the **response** now requires a `proof`. `0.2` declared `exposure.discloses: none` and an optional proof for a response that carries a bearer setup-session token; the token is secret material, and a response carrying it is signed, as `vtc/install/claim/finish` `0.3`'s is.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** supplies all three members.

A conforming **consumer**, in this order:

1. **MUST** verify the install token → `invalidToken`.
2. **MUST** match `registrationId` to an open registration for that token → `registrationMismatch`.
3. **MUST** verify the WebAuthn attestation against that registration's challenge → `bindingInvalid`.
4. **MUST** consume the install token, derive the administrator's DID from the attested Ed25519 key as a `did:key`, and mint a setup-session token (aud=vtc-install-session, 5-minute TTL) that names it.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorization evidence this task presupposes is three things together, none of which is sufficient alone: a valid **install token**, a matching **`registrationId`**, and a verifying **WebAuthn attestation**.

The combination is the point. The token establishes that this installation was expected, the registration match that this is the claim it was expected for, and the attestation that the authenticator is genuine and holds the key the administrator's DID will be derived from. Each failure has its own code (`invalidToken`, `registrationMismatch`, `bindingInvalid`) so an operator can tell which assumption broke.

There is no fourth piece of evidence binding a DID, because the DID is derived from the attested key: a separate signature by that key would be the key asserting control of itself twice. A founder who claims under a DID they already control proves control of it separately, with [`vtc/install/claim/start`](../../../start/0.3/spec.md) and [`finish`](../../../finish/0.3/spec.md) `0.3`.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Request

The installer (`issuer`) sends the request to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An installer completes the passkey claim

```json
{
  "id": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d411",
  "type": "https://trusttasks.org/spec/vtc/install/claim/passkey/finish/0.1",
  "issuer": "did:key:z6MkfqzP7fJmYmN4KZrC8k1c7gTCh5Yy2nNw9Sz4o8bqvEhJ",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-01T09:01:05Z",
  "threadId": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d41f",
  "payload": {
    "installToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJ2dGMtaW5zdGFsbCJ9.c2lnbmF0dXJl",
    "registrationId": "3f2a9c1e-5b7d-4e8f-a0b1-c2d3e4f5a6b7",
    "webauthnResponse": {
      "id": "a1b2c3d4e5f6",
      "rawId": "a1b2c3d4e5f6",
      "type": "public-key",
      "response": {
        "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uY3JlYXRlIn0",
        "attestationObject": "o2NmbXRkbm9uZWdhdHRTdG10oGhhdXRoRGF0YQ"
      }
    }
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. Refusals use `trust-task-error`.

### The claim is complete

```json
{
  "id": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d412",
  "type": "https://trusttasks.org/spec/vtc/install/claim/passkey/finish/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:key:z6MkfqzP7fJmYmN4KZrC8k1c7gTCh5Yy2nNw9Sz4o8bqvEhJ",
  "issuedAt": "2026-10-01T09:01:06Z",
  "threadId": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d41f",
  "payload": {
    "adminDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "setupSessionToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJ2dGMtaW5zdGFsbC1zZXNzaW9uIn0.c2lnbmF0dXJl"
  }
}
```

## Security & Privacy

**Proof of control.** The passkey attestation proves the installer holds the authenticator, and the administrator's DID is derived from the very key that attestation covers — so the DID cannot be claimed without holding the authenticator. The setup-session token is short-lived and single-audience, limiting the window in which bootstrap can run.

**What this does not prove.** The administrator's DID is *assigned by* this ceremony, not *asserted by* the installer. Nothing here demonstrates control of any pre-existing identity. A deployment that needs an operator to claim administration under a DID they already hold uses [`vtc/install/claim/finish`](../../../finish/0.3/spec.md) `0.3`, which proves ownership of a verification method in that DID's live document.

### Data carried

The request carries the install token, the registration handle and the WebAuthn attestation, which includes the passkey's public key and authenticator data. The response carries the derived DID and the setup-session token, a short-lived bearer credential for one bootstrap.

### Correlation

The derived DID is new and specific to this passkey, so it links nothing beyond this community unless the installer reuses the passkey elsewhere. The authenticator data may identify the authenticator's model where the authenticator attests it.

### Retention

The passkey is kept as the administrator's credential until it is revoked; the claim's audit record is durable. The setup-session token lives five minutes.

### Consent/purpose

The claim exists to make the installer this community's first administrator. The setup-session token **MUST NOT** be accepted for anything but the bootstrap it was minted for.
