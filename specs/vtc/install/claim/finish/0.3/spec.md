---
slug: vtc/install/claim/finish
version: "0.3"
title: VTC Install Claim — Finish
summary: Complete first-administrator enrolment under a DID the founder already controls — signed by that DID against its live document, carrying the founder's step-up approver's enrolment statement — and receive a setup-session token.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, install, bootstrap, wallet, approver, step-up]
parties:
  - role: founder (the DID the install token names)
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
  - role: founder's step-up approver
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the founder's proof of control of the DID the install token names, made outside any WebAuthn ceremony and verified against a verification method in that DID's live document. Without it the token alone would make whoever held it the first administrator under somebody else's DID. The embedded statement's proof is the approver's proof of possession, which binds the founder's step-up factor. The response carries the setup-session token, a bearer secret, so it is signed by the community too: the founder can then tell a genuine token from one an intermediary substituted.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Claim completion binds an installation to its owner once. A replayed completion re-binds an installation that may have been transferred, and the claim is what every later authorisation in the community is rooted in.
sideEffects:
  level: mutating
  rationale: "Binds the founder's step-up approver, pending bootstrap, and mints a short-lived setup-session token; the administrator entry itself is written by vtc/admin/bootstrap."
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: "The response carries the setup-session token, a short-lived bearer credential that vtc/admin/bootstrap consumes to write the first administrator. The request carries the approver DID, an optional label and the approver's statement."
retention:
  class: durable
  rationale: "The claim — which DID, which approver, on which install token — is the audit record every later authorisation in the community is rooted in; the approver binding is kept until revoked."
errorCodes:
  - code: vtc/install/claim/finish:invalidToken
    meaning: The install token the claim was opened under has expired or been consumed since claim/start.
    retryable: false
  - code: vtc/install/claim/finish:registrationMismatch
    meaning: The claimId does not name an open claim, or the claim has expired.
    retryable: false
  - code: vtc/install/claim/finish:subjectMismatch
    meaning: The document's issuer is not the DID the install token names, or its proof was made by a key that is not a verification method of that DID's own document (a delegated key, for instance).
    retryable: false
  - code: vtc/install/claim/finish:didUnresolvable
    meaning: The DID the install token names could not be resolved to its live document, so the proof cannot be checked against it. Nothing was consumed.
    retryable: true
  - code: vtc/install/claim/finish:statementInvalid
    meaning: "`statement` is not a valid auth/step-up/approver/attest/0.1 document for this claim: its proof does not verify, its issuer is not `approverDid`, its `recipient` or `audience` is not this community, its `purpose` is not `enrol`, its `subject` is not the founder's DID, its `challenge` is not the claim's, its `boundTo` is not `claimId`, it was issued outside the claim's lifetime, or its `id` was already spent."
    retryable: false
  - code: vtc/install/claim/finish:approverNotDistinct
    meaning: "`approverDid` is the founder's DID or appears as a verification method in the founder's DID document. A step-up factor must be a key the founder does not already hold for signing."
    retryable: false
related:
  - vtc/install/claim/start
  - vtc/admin/bootstrap
  - auth/step-up/approver/attest
  - auth/step-up/approver/list
  - auth/step-up/approver/revoke
---

## Abstract

The **VTC Install Claim — Finish** Trust Task completes first-administrator enrolment when the founder claims the community under a DID they already control. The founder signs the document **as that DID** — through their wallet — carrying the `claimId` from [`claim/start/0.3`](../../start/0.3/spec.md), the DID of their [step-up approver](../../../../../auth/step-up/approver/attest/0.1/spec.md) and the approver's enrolment statement over the claim's challenge. The community verifies the founder's proof against the **live** DID document, verifies the statement, binds the approver to the founder as their step-up factor (`enrolledVia: install`), and returns the founder's DID with a short-lived `setupSessionToken`, which [`vtc/admin/bootstrap`](../../../../admin/bootstrap/0.1/spec.md) consumes to write the administrator entry.

The founder thus leaves the claim as an administrator who can already answer the community's operation-bound step-ups — with an approver rather than a passkey — and the community never derives an identity for them.

### Changes from 0.2

0.2 derived the administrator's `did:key` from a passkey the founder registered, and said that a version letting the installer supply a DID they already control "would restore the distinction" between controlling an authenticator and controlling a DID, "and would then need a fourth piece of evidence again". That fourth piece is the founder's own proof, against the DID's live document; the approver's statement takes the place of the WebAuthn attestation as the step-up factor.

- **The document is signed**, by the DID the install token names (`subjectMismatch`, `didUnresolvable`).
- `webauthnResponse` is replaced by `approverDid`, an optional `label` and `statement`; `bindingInvalid` by `statementInvalid` and `approverNotDistinct`.
- `installToken` is no longer carried: the claim opened by `claimId` is bound to it. `registrationId` is renamed `claimId`.
- `adminDid` in the response is the founder's DID, not one derived by the community.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the founder):

1. **MUST** set `issuer` to the `adminDid` claim/start returned and sign the document with a verification method of that DID's current document.
2. **MUST** carry in `statement` the approver's signed [`attest/0.1`](../../../../../auth/step-up/approver/attest/0.1/spec.md) document exactly as produced, with `purpose: enrol`, `subject` its own DID, `audience` and `recipient` the start's `audience`, `challenge` the start's `challenge` and `boundTo` the `claimId`.

A conforming **consumer** (the community), in this order:

1. **MUST** look up the claim by `claimId` → `registrationMismatch`; and its install token → `invalidToken` if it has expired or been consumed since start.
2. **MUST** require `issuer` to equal the DID the install token names → `subjectMismatch`.
3. **MUST** resolve that DID afresh — not from a cache populated before this request — and verify the proof against a verification method of the **live** document, under the relationship its `proofPurpose` names ([SPEC §4.7.3](/SPEC.md#473-proof-purpose-and-verification-relationship)). Resolution failure → `didUnresolvable`; a verification method absent from the live document, or a proof that does not verify → the framework `proofInvalid`; a key that is not a verification method of the DID's own document → `subjectMismatch`. Material supplied in the request is never the document checked against.
4. **MUST** verify `statement` as `attest/0.1` requires of its consumer, over the object exactly as received, with `issuer` equal to `approverDid`, `purpose` `enrol`, `subject` the founder's DID, `audience` and `recipient` its own DID, `challenge` the claim's and `boundTo` equal to `claimId` → `statementInvalid`.
5. **MUST** refuse with `approverNotDistinct` when `approverDid` is the founder's DID or appears in the live document as a verification method.
6. **MUST** consume the claim, bind the approver to the founder's DID with `enrolledVia: install`, and mint a setup-session token (aud=vtc-install-session, 5-minute TTL) that names the founder's DID, atomically. The binding **MUST** take effect only when vtc/admin/bootstrap consumes that token and writes the administrator entry, and **MUST** be discarded if the token expires unused, so an abandoned claim leaves no factor behind.
7. **MUST** record an audit event naming the founder's DID, the approver DID and the install token's jti.

A co-administrator named at install claims the same way, with a token naming their DID.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorization evidence this task presupposes is three things together, none sufficient alone: an **open claim**, opened with the install token and its claim code; the **founder's proof** of control of the DID the token names, against its live document; and the **approver's statement**, proving possession of the factor being bound.

The token is the anchor: it shows the operator, who holds the host, expected this installation to be claimed by this DID. The founder's proof shows the claimant controls that DID — the distinction 0.2 noted was missing when the DID was derived from the attested key. The statement shows the step-up factor is held, and is bound on the token's anchor rather than on the founder's signing key, so the founder's first factor does not rest on the key it is meant to add to.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Request

The founder (`issuer`) sends the request to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet founder completes the claim with their browser plugin's approver

```json
{
  "id": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8c11",
  "type": "https://trusttasks.org/spec/vtc/install/claim/finish/0.3",
  "issuer": "did:webvh:QmAliceScid4:wallet.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-01T09:01:05Z",
  "threadId": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8c1f",
  "payload": {
    "claimId": "clm_1f2e3d4c5b6a79880a1b2c3d4e5f6a7b",
    "approverDid": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "label": "Browser plugin — work laptop",
    "statement": {
      "id": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8c1a",
      "type": "https://trusttasks.org/spec/auth/step-up/approver/attest/0.1",
      "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
      "issuedAt": "2026-10-01T09:01:00Z",
      "payload": {
        "purpose": "enrol",
        "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
        "audience": "did:webvh:QmVtcScid7:acme-vtc.example",
        "challenge": "Q2xhaW1DaGFsbGVuZ2VOb25jZTQ1Njc4OTA",
        "boundTo": "clm_1f2e3d4c5b6a79880a1b2c3d4e5f6a7b"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "created": "2026-10-01T09:01:00Z",
        "proofPurpose": "authentication",
        "proofValue": "z4mD…"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid4:wallet.example:alice#key-1",
    "created": "2026-10-01T09:01:05Z",
    "proofPurpose": "authentication",
    "proofValue": "z5Lg…"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`, as in 0.2. Refusals use `trust-task-error`.

### The claim is complete

```json
{
  "id": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8c12",
  "type": "https://trusttasks.org/spec/vtc/install/claim/finish/0.3#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-01T09:01:06Z",
  "threadId": "urn:uuid:d4a8b1c5-6e9f-4a01-c23d-4e5f6a7b8c1f",
  "payload": {
    "adminDid": "did:webvh:QmAliceScid4:wallet.example:alice",
    "setupSessionToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJ2dGMtaW5zdGFsbC1zZXNzaW9uIn0.c2lnbmF0dXJl"
  }
}
```

## Security & Privacy

**Live resolution.** The proof is checked against the DID's document as it stands now, resolved for this request. A cached or request-supplied document would let a key the founder had already rotated out — or never held — claim the community.

**The founder's first factor rests on the install token, not on their key.** Binding a step-up approver on the strength of the founder's own signature would let whoever stole that key mint a factor for it. Here the anchor is the operator's token and claim code; the founder's signature only attributes the claim, and the statement only proves possession.

**An abandoned claim leaves nothing.** The approver binding takes effect with the administrator entry, so a claim whose bootstrap never runs leaves neither an administrator nor a factor.

**Free text.** `label` is bounded at 64 characters, authored by the founder and untrusted; it is retained with the binding.

### Data carried

The request carries the claim id, an approver DID, an optional label and the approver's statement naming the founder, the community, the claim challenge and the claim id. The response carries the founder's DID and the setup-session token, a short-lived bearer credential for one bootstrap.

### Correlation

The claim links the community to the founder's DID and to their approver DID, which the founder chose to do by claiming. The approver DID **SHOULD** be one generated for this community alone.

### Retention

The approver binding is kept until revoked; the claim's audit record is durable. The setup-session token lives five minutes.

### Consent/purpose

The claim exists to make the founder this community's first administrator and to bind their step-up factor. The approver **MUST NOT** be accepted for anything but the founder's step-ups at this community, and the setup-session token for anything but the bootstrap it was minted for.
