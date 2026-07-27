---
slug: credential-exchange/request
version: "0.1"
title: Credential Exchange — Request
summary: Holder to issuer — an OID4VCI Credential Request carrying the key-binding proof that ties the credential to be issued to a key the holder controls.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - issuance
  - oid4vci
  - request
  - key-binding
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Holder
    requirement: REQUIRED
    member: issuer
  - role: Issuer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: An authcrypted transport already proves which DID sent the request. Note that the OID4VCI key-binding proof inside the body is a *different* thing and is always required by OID4VCI itself — it binds the credential to a holder key, where a document proof would only re-attest the sender. Conflating the two is the mistake this rationale exists to prevent.
sideEffects:
  level: none
  rationale: Asking for a credential mutates nothing at the issuer. Any state the issuer keeps to track the request is its own bookkeeping, not an effect of this task.
exposure:
  discloses: none
  actsAsSubject: true
  rationale: The holder is asking for a credential about itself, and the embedded key-binding proof names a key it controls — so the sender is the subject of what follows. No claims are disclosed by the request itself.
errorCodes:
  - code: credential-exchange/request:invalidProof
    meaning: The key-binding proof is missing, malformed, or does not verify.
    retryable: false
  - code: credential-exchange/request:unknownOffer
    meaning: The request does not correspond to an offer this issuer made.
    retryable: false
related:
  - credential-exchange/offer
  - credential-exchange/issue
---

## Abstract

The **Credential Exchange — Request** Trust Task is the holder's answer to an [offer](../../offer/0.1/): a request for the credential, carrying the proof that binds it to a key the holder holds.

The body is an [OID4VCI](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html) Credential Request, carried verbatim and not re-specified here.

## Two bindings, not one

This task involves two distinct authentications, and treating them as one is the recurring implementation error:

1. **Envelope authentication** proves *who sent this message*. On an authcrypted transport it is intrinsic.
2. **The key-binding proof** inside the OID4VCI request proves *which key the credential should be bound to*. The issuer mints against that key — the `cnf` for SD-JWT-VC, the `credentialSubject.id` for a W3C Data-Integrity credential.

They can legitimately differ. A relayer may carry the envelope for a holder that is not the sender, which is exactly the air-gap onboarding case. **A consumer MUST bind the credential to the key in the proof, never to the envelope sender** — doing the latter issues a credential the intended holder cannot use, and, if the relayer is hostile, issues it to the relayer.

## Conformance

Producer: send the OID4VCI Credential Request under `credential_request`, unmodified, with its key-binding proof. Reply on the offer's thread.

Consumer: verify the key-binding proof before minting anything, and reject with `invalidProof` if it does not verify — an unverified proof means the credential would be bound to a key nobody has demonstrated control of. Answer on-thread with [`issue`](../../issue/0.1/).

## Security & Privacy

`exposure.actsAsSubject` is true: the requester is asking about itself. The request discloses no claims — it names a credential configuration and a key, not attributes.

An issuer MUST NOT treat a verified key-binding proof as authorization. It proves control of a key; whether that party is *entitled* to the credential is a policy question this task does not answer.
