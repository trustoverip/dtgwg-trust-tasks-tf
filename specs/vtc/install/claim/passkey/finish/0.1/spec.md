---
slug: vtc/install/claim/passkey/finish
version: "0.1"
title: VTC Install Claim — Passkey — Finish
summary: Complete first-administrator enrolment with a passkey — submit the WebAuthn registration, derive the administrator's did:key from the credential's own Ed25519 or P-256 public key, and receive a setup-session token.
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
    meaning: The WebAuthn registration failed verification — its client data is not a `webauthn.create` over this registration's challenge and the community's origin, its authenticator data is not for the community's RP ID, its attestation statement does not verify, or its credential public key is malformed.
    retryable: false
  - code: vtc/install/claim/passkey/finish:unsupportedAlgorithm
    meaning: "The registered credential's public key is neither EdDSA over Ed25519 (COSE alg -8, OKP crv 6) nor ES256 over P-256 (COSE alg -7, EC2 crv 1), so no did:key this task defines can name it. Nothing was consumed: register a passkey with one of the two (start offers only those) and finish again."
    retryable: false
related:
  - vtc/install/claim/passkey/start
  - vtc/install/claim/finish
  - vtc/admin/bootstrap
---

## Abstract

The **VTC Install Claim — Passkey — Finish** Trust Task completes first-administrator enrolment by passkey. The installer submits the `installToken`, the `registrationId` from [`vtc/install/claim/passkey/start`](../../start/0.1/spec.md) and the `webauthnResponse`. The community derives the administrator's DID from the passkey credential's own public key — the `did:key` of an Ed25519 or P-256 key — and returns it with a short-lived `setupSessionToken`, which [`vtc/admin/bootstrap`](../../../../../admin/bootstrap/0.1/spec.md) consumes to write the administrator entry.

### Changes from vtc/install/claim/finish 0.2

This is the ceremony of [`vtc/install/claim/finish`](../../../finish/0.2/spec.md) `0.2`, under a slug of its own so that it is not superseded by the wallet ceremony of `vtc/install/claim/finish` `0.3`. [`passkey/start`](../../start/0.1/spec.md) says why the two ceremonies cannot share a version. `vtc/install/claim/finish` `0.2` is retired in favour of this task.

The payload members are unchanged. Two rules moved:

- **The key may be Ed25519 or P-256.** `0.2` derived the administrator's `did:key` from an Ed25519 passkey alone, and almost no platform authenticator creates one. This task also accepts ES256 and names that key by its own P-256 `did:key` (`zDn…`), so the response `adminDid` pattern admits both prefixes. Any other algorithm is refused with the new code `unsupportedAlgorithm`. The installer's authenticator holds a key from outside the community, and a key held outside it may be P-256 when it is named by its own `did:key`.
- **The response requires a proof.** `0.2` declared `exposure.discloses: none` and an optional proof for a response that carries a bearer setup-session token; the token is secret material, and a response carrying it is signed, as `vtc/install/claim/finish` `0.3`'s is.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** supplies all three members, `webauthnResponse` exactly as `navigator.credentials.create()` returned it, encoded as JSON.

A conforming **consumer**, in this order:

1. **MUST** verify the install token → `invalidToken`.
2. **MUST** match `registrationId` to an open registration for that token → `registrationMismatch`.
3. **MUST** verify the registration as a WebAuthn relying party verifies a new credential ([WebAuthn Level 3 §7.1](https://www.w3.org/TR/webauthn-3/#sctn-registering-a-new-credential)) → `bindingInvalid`: `clientDataJSON.type` is `webauthn.create`, its `challenge` is the one [`start`](../../start/0.1/spec.md) issued for this registration and its `origin` is the community's; the authenticator data's RP ID hash is the community's RP ID and its user-present flag is set; and the attestation statement verifies under its format. **This registration is the proof of control:** the credential public key it carries is the key the administrator's DID will name, bound by the registration to this registration's challenge. No other evidence of control over the DID is taken or needed.
4. **MUST** read the credential public key from the attested credential data and refuse with `unsupportedAlgorithm`, consuming nothing, unless it is one of:
   - COSE `alg` -8 (EdDSA), `kty` 1 (OKP), `crv` 6 (Ed25519), with a 32-byte `x`;
   - COSE `alg` -7 (ES256), `kty` 2 (EC2), `crv` 1 (P-256), with 32-byte `x` and `y` naming a point on the curve.

   A key of a listed algorithm that is malformed — the wrong length, or a P-256 point not on the curve — is `bindingInvalid`.
5. **MUST** derive the administrator's DID from that key alone, as `did:key:` followed by the base58btc multibase (prefix `z`) encoding of the multicodec-prefixed public key:
   - **Ed25519:** the varint of multicodec `0xed01` (`ed25519-pub`; bytes `0xed 0x01`) followed by the 32-byte `x` — a DID beginning `did:key:z6Mk`;
   - **P-256:** the varint of multicodec `0x1200` (`p256-pub`; bytes `0x80 0x24`) followed by the 33-byte SEC1 **compressed** point — `0x02` when `y` is even, `0x03` when odd, then `x` — a DID beginning `did:key:zDn`.

   It **MUST NOT** take the DID, or anything it is derived from, from the install token or from the request outside the attested credential data.
6. **MUST** consume the install token, keep the credential as the administrator's passkey, mint a setup-session token (aud=vtc-install-session, 5-minute TTL) that names the derived DID, and sign the response (`proof` is REQUIRED on it).

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorization evidence this task presupposes is three things together, none of which is sufficient alone: a valid **install token**, a matching **`registrationId`**, and a verifying **WebAuthn registration**.

The combination is the point. The token establishes that this installation was expected, the registration match that this is the claim it was expected for, and the registration that this ceremony's authenticator produced the key the administrator's DID is derived from. Each failure has its own code (`invalidToken`, `registrationMismatch`, `bindingInvalid`, and `unsupportedAlgorithm` for a key no `did:key` here can name) so an operator can tell which assumption broke.

There is no fourth piece of evidence binding a DID, and the install token names none, because the DID is derived from the registered key: a separate signature by that key would be the key asserting control of itself twice. A founder who claims under a DID they already control proves control of it separately, with [`vtc/install/claim/start`](../../../start/0.3/spec.md) and [`finish`](../../../finish/0.3/spec.md) `0.3`.

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
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-0",
    "created": "2026-10-01T09:01:06Z",
    "proofPurpose": "authentication",
    "proofValue": "z3Fq…"
  }
}
```

### The claim is complete, with a P-256 passkey

A platform authenticator chose ES256; the administrator's DID is the P-256 key's own `did:key`.

```json
{
  "id": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d422",
  "type": "https://trusttasks.org/spec/vtc/install/claim/passkey/finish/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:key:z6MkfqzP7fJmYmN4KZrC8k1c7gTCh5Yy2nNw9Sz4o8bqvEhJ",
  "issuedAt": "2026-10-01T09:01:06Z",
  "threadId": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d42f",
  "payload": {
    "adminDid": "did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169",
    "setupSessionToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJ2dGMtaW5zdGFsbC1zZXNzaW9uIn0.c2lnbmF0dXJl"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-0",
    "created": "2026-10-01T09:01:06Z",
    "proofPurpose": "authentication",
    "proofValue": "z2Kd…"
  }
}
```

## Security & Privacy

**Proof of control.** The WebAuthn registration is the proof of control: it carries the credential public key, bound to this registration's challenge, and the administrator's DID is derived from that key alone — so the DID names the authenticator's key and no other. Where the attestation statement is signed (`packed` self-attestation by the credential key, or an attestation key), that signature covers the challenge through `clientDataHash`. Under the `none` format, which most platform passkeys use, nothing signs the challenge at registration: the binding rests on the client data the browser produced for the community's origin, and possession of the private key is first exercised when the administrator authenticates with the passkey. A deployment that wants possession proven before the administrator entry exists can require a signed attestation format, or an assertion before [`vtc/admin/bootstrap`](../../../../../admin/bootstrap/0.1/spec.md). The setup-session token is short-lived and single-audience, limiting the window in which bootstrap can run.

**What this does not prove.** The administrator's DID is *assigned by* this ceremony, not *asserted by* the installer. Nothing here demonstrates control of any pre-existing identity. A deployment that needs an operator to claim administration under a DID they already hold uses [`vtc/install/claim/finish`](../../../finish/0.3/spec.md) `0.3`, which proves ownership of a verification method in that DID's live document.

### Data carried

The request carries the install token, the registration handle and the WebAuthn attestation, which includes the passkey's public key and authenticator data. The response carries the derived DID — an Ed25519 or P-256 `did:key` — and the setup-session token, a short-lived bearer credential for one bootstrap.

### Correlation

The derived DID is new and specific to this passkey, so it links nothing beyond this community unless the installer reuses the passkey elsewhere. The authenticator data may identify the authenticator's model where the authenticator attests it.

### Retention

The passkey is kept as the administrator's credential until it is revoked; the claim's audit record is durable. The setup-session token lives five minutes.

### Consent/purpose

The claim exists to make the installer this community's first administrator. The setup-session token **MUST NOT** be accepted for anything but the bootstrap it was minted for.
