---
slug: auth/oob/grant
version: "0.1"
title: "Auth — OOB Grant"
summary: "A member's signed decision on one out-of-band request: let this starter key act as me at this origin until notAfter, having seen this context — or decline. An attestation signed by the member's VTA for assertionMethod after user verification, carried only inside auth/oob/respond."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, grant, session-key, user-verification, vta, wallet]
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
    The proof is the grant. It is an attestation in the sense of SPEC §4.7.3: the identified DID's assertionMethod key attests that the named starter key may act as it, and the service relies on that attestation to create a session. An unsigned grant authorizes nothing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A grant arms a session in the member's name, and an issue time bounds how long a captured grant could be carried to the service; it is accepted only inside the request's decision window.
sideEffects:
  level: none
  rationale: >-
    The document does nothing on its own. It is evidence carried inside auth/oob/respond, and the effects — recording the decision and, at redeem, creating the session — are those tasks'.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Names a request, two throwaway keys, an origin, a digest and a time. Nothing secret moves; the DID's private key stays in the VTA. The authority the grant confers is exercised through auth/oob/respond and auth/oob/redeem, which declare it.
retention:
  class: durable
  rationale: >-
    The service keeps the signed grant as the audit record of who let which key act for them, where, and after seeing what; the VTA keeps its id so it never signs the same grant twice.
errorCodes: []
related:
  - auth/oob/respond
  - auth/oob/prove
  - auth/oob/redeem
  - consent/decision
---

## Abstract

The grant is the heart of the key-grant sign-in. The member does not hand the browser a session; the member signs a statement naming the browser's own key, and only the holder of that key can turn the statement into a session ([`auth/oob/redeem`](../../redeem/0.1/spec.md)). The grant binds:

- the **request** (`requestId`);
- the **starter key** that may act (`sessionKey`) and the **lock** that delivered it (`approverKey`);
- the **origin** at which it may act;
- **what the member was shown** (`contextDigest`, the digest of the signed step 2);
- **how long** (`notAfter`).

The wallet builds it unsigned with the identified DID as `issuer`. The member's VTA signs it, for `assertionMethod`, only when the request carries a `consent/decision` from the device's user-verification key over the digest of the unsigned grant. A decline is the same document with `decision: decline`, and needs no user verification. It travels as the `grant` member of [`auth/oob/respond`](../../respond/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the wallet with the member's VTA):

1. **MUST** set `issuer` to the DID that proved membership (`identifiedAs` in step 2), `recipient` to the service DID, `sessionKey` and `origin` from step 2, `approverKey` to the lock key, and `contextDigest` to the SHA-256 multihash, in base58btc multibase (`z…`), of the [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) canonical form of the signed step 2 response document, proof included.
2. **MUST** sign with a key the issuer's DID document lists under `assertionMethod`, with `proofPurpose: assertionMethod`.
3. A signing VTA, for `decision: approve`, **MUST** require a `consent/decision` signed by the requesting device's user-verification key whose payload digest equals the digest of the unsigned grant; **MUST** refuse unless `issuer` is the principal of the vault entry used and `recipient` is the DID that entry targets; and **MUST NOT** sign the same grant `id` twice.
4. **MUST NOT** send the document on its own.

A conforming **consumer** (the service, executing `auth/oob/respond`):

1. **MUST NOT** act on a grant received on its own, and **SHOULD** answer one with `unsupportedType`.
2. **MUST** verify the embedded grant as a Trust Task document in its own right ([SPEC §7.2](/SPEC.md#72-consumer-requirements)) — recipient is itself, fresh, `id` new — with its proof checked against the issuer's **`assertionMethod`** relationship.
3. **MUST** require `issuer` to equal the request's identified DID, `approverKey` the lock, `sessionKey` the starter key, `origin` the request's origin, and `contextDigest` the stored digest of the signed step 2, comparing the decoded multihash rather than the multibase text.

## Authorization

The grant is a delegation by the member: it is the authority, for one session, for `sessionKey` to act as the issuer at `origin` until `notAfter`. It is an assertion whose proof *is* the authorization, and only for that purpose; it confers nothing beyond the session the service creates at redeem, and that session cannot satisfy a step-up or any check that needs the member's own `assertionMethod` key. The service still checks membership at respond and again at redeem.

## Definitions

- **`decision`** — `approve` or `decline`.
- **`sessionKey`** — the starter key `K_b` the grant lets act as the issuer.
- **`approverKey`** — the lock key `K_a`.
- **`origin`** — the portal origin at which the session key may act.
- **`contextDigest`** — the digest of the signed step 2 response, binding the decision to what the member saw.
- **`notAfter`** — the latest instant the session key may act as the issuer, in integer epoch seconds.

## Request

The approving identity (`issuer`) addresses the service (`recipient`); the wallet carries the document inside `auth/oob/respond`. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member approves the sign-in

```json
{
  "id": "urn:uuid:8d4e5f60-7182-4c9d-8e0f-2a3b4c5d6e01",
  "type": "https://trusttasks.org/spec/auth/oob/grant/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:01:30Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
    "decision": "approve",
    "sessionKey": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "approverKey": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "origin": "https://members.community.example",
    "contextDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "notAfter": 1791655200
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-10-10T10:01:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

## Security & Privacy

### Data carried

A handle, two throwaway public keys, an origin, a digest, a time and a decision. Nothing secret and nothing about the member beyond the DID that is already the issuer.

### Correlation

The grant links the member's DID to one starter key and one origin, for one session, at the community the member chose. The identity's identifier scope is `any`; the service has identifier scope `public` because the VTA's vault entry targets the community's published DID.

### Retention

The service keeps the signed grant as the audit record of the sign-in. The VTA keeps the grant `id` and the user-verification decision in its own audit log, and refuses to sign that `id` again.

### Consent/purpose

The grant records one decision for one request, bound to the context the member was shown. It **MUST NOT** be reused as evidence of consent to anything else. Whether a given deployment requires user verification for a grant is the signing VTA's policy; this specification describes the evidence, not the policy.
