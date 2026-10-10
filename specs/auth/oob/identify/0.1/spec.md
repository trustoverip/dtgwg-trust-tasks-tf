---
slug: auth/oob/identify
version: "0.1"
title: "Auth — OOB Identify"
summary: "A member's DID states that it holds the lock on one out-of-band request and has typed the number shown on the starter's screen; signed by the member's VTA for authentication and carried only inside auth/oob/prove."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, membership, vta, wallet]
parties:
  - role: Approving identity
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the whole of what this document contributes: it is the member DID's signature, for `authentication`, over the request, the lock and the typed number. Without it the service has no membership proof and no number proof.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The document is accepted only inside the request's decision window, and an issue time lets the service refuse one signed for an earlier request or held back and replayed.
sideEffects:
  level: none
  rationale: >-
    The document does nothing on its own. It is evidence carried inside auth/oob/prove, and the effect — recording the identified DID — is that task's.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Reveals which DID the member chose for this sign-in, to the service the member is signing in to, together with a handle, a throwaway key and two digits. It grants no authority.
retention:
  class: durable
  rationale: >-
    The service audits every proof, including failed ones, with the DID, because a proof is also the moment a member was shown where a request came from.
errorCodes: []
related:
  - auth/oob/prove
  - auth/oob/grant
---

## Abstract

`identify` is the member's statement **"I am this DID, I hold this lock, and I can see the screen."** The wallet builds it unsigned with the chosen DID as `issuer`, and the member's VTA signs it with `vault/sign-trust-task` for `authentication`. It grants no authority on its own and is never sent by itself: it travels as the `identify` member of [`auth/oob/prove`](../../prove/0.1/spec.md), signed on the outside by the lock key.

Its payload is closed — three members, `additionalProperties: false`, no `ext` — because a VTA signs it without user verification. A VTA validates it against this schema and the exact type URI before signing, so a compromised wallet cannot use it to obtain a signature over anything else.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the wallet with the member's VTA):

1. **MUST** set `issuer` to the DID the member chose, `recipient` to the service DID, `requestId` to the request, `approverKey` to its lock key, and `enteredNumber` to the digits the member typed.
2. **MUST** sign with a key the issuer's DID document lists under `authentication`, with `proofPurpose: authentication`.
3. A signing VTA **MUST** validate the document against this schema with the exact type URI before signing, and **MUST** refuse unless `issuer` is the principal of the vault entry used and `recipient` is the DID that entry targets.
4. **MUST NOT** send the document on its own.

A conforming **consumer** (the service, executing `auth/oob/prove`):

1. **MUST NOT** act on an `identify` received on its own, and **SHOULD** answer one with `unsupportedType`.
2. **MUST** check that the issuer string is an active member **before** resolving any DID.
3. **MUST** verify the embedded document as a Trust Task document in its own right ([SPEC §7.2](/SPEC.md#72-consumer-requirements)): recipient is itself, it is fresh, its `id` is new, and its proof verifies against a key the issuer's DID document lists under `authentication`. Any failure is `auth/oob:notAuthorized` on the prove.
4. **MUST** require `requestId` to name the request and `approverKey` to equal the lock and the prove's `issuer`.

## Authorization

Membership of the community is what this document lets the service check, and only after the issuer string is found in the service's own member list. The proof establishes control of the DID; the membership check establishes standing; the typed number establishes that the holder can see the starter's screen. None of them is authority to sign anyone in — that requires a user-verified [`auth/oob/grant`](../../grant/0.1/spec.md).

## Definitions

- **`requestId`** — the request this proof is for.
- **`approverKey`** — the lock key `K_a` from the claim.
- **`enteredNumber`** — the two digits the member typed.

## Request

The approving identity (`issuer`) addresses the service (`recipient`); the wallet carries the document inside `auth/oob/prove`. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member's identify, signed by their VTA

```json
{
  "id": "urn:uuid:6b2c3d4e-5f60-4a7b-8c9d-0e1f2a3b4c01",
  "type": "https://trusttasks.org/spec/auth/oob/identify/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:01:00Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
    "approverKey": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "enteredNumber": "47"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-10-10T10:01:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Security & Privacy

### Data carried

The member's chosen DID, the handle, the lock key and two digits. Nothing secret; the DID's private key stays in the VTA.

### Correlation

This is the step at which the member's DID is revealed, and only the DID the member chose, to the community they are signing in to. Their other identities are never sent. The identity's identifier scope is `any` because it is whichever DID holds membership at the service; the service has identifier scope `public` because the VTA's vault entry targets the community's published DID.

### Retention

The service audits each identify, including failed and peeking proofs, with the DID and the request. The lock key and number mean nothing once the request ends.

### Consent/purpose

The document proves membership and sight of the screen for one request. A service **MUST NOT** treat it as consent to anything else; consent to the sign-in is the grant.
