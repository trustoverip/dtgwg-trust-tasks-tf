---
slug: vtc/install/claim/passkey/start
version: "0.1"
title: VTC Install Claim — Passkey — Start
summary: Begin claiming a fresh community with a passkey — exchange the install token for a WebAuthn registration challenge, from whose key the first administrator's did:key will be derived.
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
  requirement: OPTIONAL
  rationale: The single-use install token (an EdDSA-signed JWT in the payload), with its claim code where one was issued, is the gate; the community has no administrator yet to sign a framework proof, and the installer holds no key until the passkey exists.
sideEffects:
  level: none
  rationale: "Issues a single-use registration challenge; nothing is bound and the install token is not consumed until vtc/install/claim/passkey/finish."
exposure:
  discloses: none
  ingests: secret
  actsAsSubject: false
  rationale: "The request carries the install token and, where one was issued, its claim code, which together authorise claiming the community. The response is a registration handle and WebAuthn creation options, which disclose nothing beyond the community's own relying-party identity."
retention:
  class: exchange
  rationale: "The registration challenge lives only until vtc/install/claim/passkey/finish or the install token's expiry."
errorCodes:
  - code: vtc/install/claim/passkey/start:invalidToken
    meaning: The install token is missing, malformed, expired, or already consumed, or the claim code it was issued with is missing or does not match.
    retryable: false
related:
  - vtc/install/claim/passkey/finish
  - vtc/install/claim/start
  - vtc/admin/bootstrap
---

## Abstract

The **VTC Install Claim — Passkey — Start** Trust Task begins first-administrator enrolment on a freshly installed community that has no administrator yet, by **registering a passkey**. The installer presents the single-use `installToken` printed by `vtc setup` (and its `claimSecret`, when the token was issued with one); the community returns a `registrationId` and WebAuthn creation `options`. The installer creates a passkey, then completes at [`vtc/install/claim/passkey/finish`](../../finish/0.1/spec.md), where the administrator's `did:key` is derived from the passkey's key.

### Changes from vtc/install/claim/start 0.2

This is the ceremony of [`vtc/install/claim/start`](../../../start/0.2/spec.md) `0.2`, unchanged on the wire. It moved to a slug of its own because `vtc/install/claim/start` `0.3` is a **different ceremony** — the founder claims under a DID they already control, with a step-up approver as their factor — and the two cannot share one version:

- **They answer different requests with different evidence.** The passkey ceremony derives the administrator's DID from the attested key, so there is no DID to prove control of; the wallet ceremony requires the token to name a DID and proves control of it. Neither is a stronger reading of the other, and a deployment may offer either.
- **Their proof requirements differ, and that requirement is per version.** `vtc/install/claim/finish` `0.3` declares `proof` **REQUIRED** — the founder's signature as their DID is the proof of control — and every conforming consumer enforces that from the front matter. The passkey ceremony cannot carry one: the installer holds no key the document could be signed with until the passkey exists, and WebAuthn never exposes it. One version serving both would have to declare `proof` OPTIONAL and leave the wallet ceremony's requirement to prose, which no consumer pipeline enforces.

A version of a slug is superseded as soon as a later version of the same slug is published, so while the two ceremonies shared `vtc/install/claim/start`, `0.3` superseded `0.2` and the passkey install was scheduled for withdrawal with it. Under its own slug each ceremony versions on its own. `vtc/install/claim/start` `0.2` is retired in favour of this task.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** supplies `installToken`, and `claimSecret` when the operator was given one.

A conforming **consumer**:

1. **MUST** verify the install token, and refuse with `invalidToken` when it is missing, malformed, expired or consumed.
2. **MUST**, when the token was issued with a claim code, require `claimSecret` and verify it against the stored hash **before** issuing any registration challenge, refusing with `invalidToken` when it is absent or does not match.
3. **MUST** return registration options bound to the token's jti, with `registrationId` the jti.
4. **MUST** leave the token unconsumed: an abandoned registration may be restarted until the token expires.

## Request

The installer (`issuer`, typically the community's install page in a browser) sends the request to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An installer begins a passkey claim

```json
{
  "id": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d401",
  "type": "https://trusttasks.org/spec/vtc/install/claim/passkey/start/0.1",
  "issuer": "did:key:z6MkfqzP7fJmYmN4KZrC8k1c7gTCh5Yy2nNw9Sz4o8bqvEhJ",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-01T09:00:00Z",
  "threadId": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d4ff",
  "payload": {
    "installToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJ2dGMtaW5zdGFsbCJ9.c2lnbmF0dXJl",
    "claimSecret": "R7WN-4KQ2-XHPM"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. Refusals use `trust-task-error`.

### The registration is open

```json
{
  "id": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d402",
  "type": "https://trusttasks.org/spec/vtc/install/claim/passkey/start/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:key:z6MkfqzP7fJmYmN4KZrC8k1c7gTCh5Yy2nNw9Sz4o8bqvEhJ",
  "issuedAt": "2026-10-01T09:00:01Z",
  "threadId": "urn:uuid:7c1e2d3f-4a5b-4c6d-8e7f-90a1b2c3d4ff",
  "payload": {
    "registrationId": "3f2a9c1e-5b7d-4e8f-a0b1-c2d3e4f5a6b7",
    "options": {
      "publicKey": {
        "rp": { "id": "acme-vtc.example", "name": "Acme community" },
        "challenge": "Q2xhaW1SZWdpc3RyYXRpb25DaGFsbGVuZ2U",
        "pubKeyCredParams": [{ "type": "public-key", "alg": -8 }]
      }
    }
  }
}
```

## Security & Privacy

**Token-gated cold start.** Before an administrator exists there is no key to sign a framework proof, so the single-use, audience-scoped install token — with its claim code on a second channel, where one was issued — is the authentication. The installer proves control of the passkey at [`finish`](../../finish/0.1/spec.md), and the administrator's DID is derived from the key that attestation covers.

### Data carried

The request carries the install token and, optionally, the claim code. The response carries a registration handle and WebAuthn creation options: a challenge, the community's relying-party identity and the accepted algorithm. The claim code is compared with its stored hash and never logged.

### Correlation

The `registrationId` is the install token's jti, already known to whoever holds the token, and is single-use. Nothing in the exchange names the installer.

### Retention

The registration challenge is kept until finish or the token's expiry. The install token's state — issued or consumed — is the community's install record.

### Consent/purpose

The token and code are used to open this one registration and for nothing else.
