---
slug: vtc/install/claim/start
version: "0.3"
title: VTC Install Claim — Start
summary: Begin claiming a fresh community under a DID the founder already controls — exchange the install token and its claim code for a challenge the founder's step-up approver will sign.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, install, bootstrap, wallet, approver, step-up]
parties:
  - role: installer
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: The single-use install token and its separately delivered claim code are the gate, and start changes nothing. Proof of control of the founder's DID is taken at claim/finish, where it is verified against that DID's live document.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Placing the request in time bounds how long a captured start stays replayable, independent of the install token's own expiry.
sideEffects:
  level: none
  rationale: "Opens a single-use claim ceremony with a fresh challenge; nothing is bound and the install token is not consumed until claim/finish."
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: "The request carries the install token and claim code, which together authorise claiming the community. The response names the DID the token was issued for, a challenge, the community's DID and an expiry."
retention:
  class: exchange
  rationale: "The claim ceremony lives only until claim/finish or its expiry."
errorCodes:
  - code: vtc/install/claim/start:invalidToken
    meaning: The install token is missing, malformed, expired or already consumed, or the claim code does not match it. Deliberately one code for all of these, so an unauthenticated start is no oracle for which half was wrong.
    retryable: false
  - code: vtc/install/claim/start:tokenNamesNoDid
    meaning: The install token was issued for a passkey claim and names no administrator DID. Claim it with vtc/install/claim/start 0.2, or have the operator issue a token naming the founder's DID.
    retryable: false
related:
  - vtc/install/claim/finish
  - vtc/admin/bootstrap
  - auth/step-up/approver/attest
---

## Abstract

The **VTC Install Claim — Start** Trust Task begins first-administrator enrolment on a freshly installed community that has no administrator yet. In 0.3 the founder claims the community under a **DID they already control** — typically a persona their VTA wallet holds — with a [step-up approver](../../../../../auth/step-up/approver/attest/0.1/spec.md) as their step-up factor, instead of registering a passkey from which a `did:key` is derived.

The installer presents the single-use install token and the claim code printed by `vtc setup`. The community returns a `claimId`, the `adminDid` the token was issued for, a `challenge` and its own DID as `audience`. The founder's approver signs an enrolment statement over them, and the founder signs [`vtc/install/claim/finish/0.3`](../../finish/0.3/spec.md) as that DID.

### Changes from 0.2

0.2 noted that a future version **MAY** let an installer claim admin under a DID they already control, as "a genuinely separate proof — a different key, held elsewhere … proof of ownership over a verification method present in the DID's **active** DID document, signed outside the WebAuthn ceremony", and left it absent rather than approximated. This is that version.

- The response returns a `challenge` for the founder's step-up approver, not WebAuthn creation options.
- The install token **MUST** name the founder's DID (`tokenNamesNoDid` otherwise); a passkey install keeps using 0.2.
- `installToken` and `claimSecret` are renamed `token` and `claimCode`, the names the step-up approver redemption uses, and `claimCode` is **required**: binding a step-up factor rests on two channels.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** supplies `token` and `claimCode`.

A conforming **consumer**:

1. **MUST** verify the install token and the claim code against its stored hash, and refuse with `invalidToken` on any failure, with the same code, message and timing for each.
2. **MUST** refuse with `tokenNamesNoDid` a valid token that names no administrator DID.
3. **MUST** generate a fresh `claimId` and a `challenge` of at least 128 bits, bound to the token's jti, the DID it names and an expiry (RECOMMENDED: 5 minutes), and return them with `adminDid` and its own DID as `audience`.
4. **MUST** leave the token unconsumed: an abandoned claim may be restarted until the token expires.
5. **SHOULD** rate-limit this task per source, and **SHOULD** invalidate a token after a small number of wrong claim codes (RECOMMENDED: 5).

## Request

The installer (`issuer`, typically the founder's console) sends the request to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet founder begins the claim

```json
{
  "id": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8c01",
  "type": "https://trusttasks.org/spec/vtc/install/claim/start/0.3",
  "issuer": "did:webvh:QmAliceScid4:wallet.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-01T09:00:00Z",
  "threadId": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8cff",
  "payload": {
    "token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJ2dGMtaW5zdGFsbCJ9.c2lnbmF0dXJl",
    "claimCode": "R7WN-4KQ2-XHPM"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. Refusals use `trust-task-error`.

### The claim is open

```json
{
  "id": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8c02",
  "type": "https://trusttasks.org/spec/vtc/install/claim/start/0.3#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-01T09:00:01Z",
  "threadId": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8cff",
  "payload": {
    "claimId": "clm_1f2e3d4c5b6a79880a1b2c3d4e5f6a7b",
    "adminDid": "did:webvh:QmAliceScid4:wallet.example:alice",
    "challenge": "Q2xhaW1DaGFsbGVuZ2VOb25jZTQ1Njc4OTA",
    "audience": "did:webvh:QmVtcScid7:acme-vtc.example",
    "expiresAt": "2026-10-01T09:05:01Z"
  }
}
```

## Security & Privacy

**Token-gated cold start.** Before an administrator exists there is no key to sign a framework proof, so the single-use, audience-scoped install token, with its claim code on a second channel, is the authentication. Control of the founder's DID is proven at finish, against that DID's live document, and possession of the approver key by its statement.

**`adminDid` is the operator's choice, not the claimant's.** The DID comes from the install token the operator minted, not from the request, so a party holding a leaked token cannot claim the community under a DID of their own: they would also need that DID's key at finish.

### Data carried

The request carries the install token and claim code. The response carries a claim id, the DID the token names, a challenge, the community's DID and an expiry. The claim code is compared with its stored hash and never logged.

### Correlation

The response links the community to the founder's DID, which the operator already chose to do when minting the token. The claim id and challenge are single-use.

### Retention

The claim ceremony is kept until claim/finish or its expiry; wrong-code counts live with the token.

### Consent/purpose

The token and code are used to open this one claim and for nothing else.
